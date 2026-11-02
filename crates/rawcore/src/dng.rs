// SPDX-License-Identifier: GPL-3.0-only
//! DNG (Adobe's open specification, version 1.7): finds the raw image, decodes its strips or
//! tiles (uncompressed or lossless JPEG), and reads the levels, areas and colour tags the
//! rest of the pipeline needs. Written from the specification (docs/raw/dng.md).

use rayon::prelude::*;

use crate::RawError;
use crate::cfa::Mosaic;
use crate::color::ColorTags;
use crate::tiff::{Ifd, Tiff};

pub mod tag {
    pub const NEW_SUBFILE_TYPE: u16 = 254;
    pub const WIDTH: u16 = 256;
    pub const HEIGHT: u16 = 257;
    pub const BITS_PER_SAMPLE: u16 = 258;
    pub const COMPRESSION: u16 = 259;
    pub const PHOTOMETRIC: u16 = 262;
    pub const STRIP_OFFSETS: u16 = 273;
    pub const SAMPLES_PER_PIXEL: u16 = 277;
    pub const ROWS_PER_STRIP: u16 = 278;
    pub const STRIP_BYTE_COUNTS: u16 = 279;
    pub const TILE_WIDTH: u16 = 322;
    pub const TILE_LENGTH: u16 = 323;
    pub const TILE_OFFSETS: u16 = 324;
    pub const TILE_BYTE_COUNTS: u16 = 325;
    pub const SUB_IFDS: u16 = 330;
    pub const CFA_REPEAT_PATTERN_DIM: u16 = 33421;
    pub const CFA_PATTERN: u16 = 33422;
    pub const DNG_VERSION: u16 = 50706;
    pub const LINEARIZATION_TABLE: u16 = 50712;
    pub const BLACK_LEVEL_REPEAT_DIM: u16 = 50713;
    pub const BLACK_LEVEL: u16 = 50714;
    pub const WHITE_LEVEL: u16 = 50717;
    pub const DEFAULT_CROP_ORIGIN: u16 = 50719;
    pub const DEFAULT_CROP_SIZE: u16 = 50720;
    pub const ACTIVE_AREA: u16 = 50829;
}

/// A raw image larger than this is refused before decoding.
const MAX_PIXELS: u64 = 250_000_000;

/// What a DNG holds that Fotollection develops.
#[derive(Debug, Clone)]
pub struct Dng {
    pub mosaic: Mosaic,
    pub color: ColorTags,
}

/// Whether `data` is a DNG (a TIFF whose first directory has `DNGVersion`).
pub fn is_dng(data: &[u8]) -> bool {
    Tiff::new(data)
        .and_then(|t| t.first_ifd().and_then(|o| t.ifd(o)))
        .is_ok_and(|ifd| ifd.get(tag::DNG_VERSION).is_some())
}

fn one(tiff: &Tiff<'_>, ifd: &Ifd, t: u16) -> Option<u32> {
    ifd.get(t)
        .and_then(|e| tiff.uints(e).ok())
        .and_then(|v| v.first().copied())
}

fn list(tiff: &Tiff<'_>, ifd: &Ifd, t: u16) -> Vec<u32> {
    ifd.get(t)
        .and_then(|e| tiff.uints(e).ok())
        .unwrap_or_default()
}

fn reals(tiff: &Tiff<'_>, ifd: &Ifd, t: u16) -> Vec<f64> {
    ifd.get(t)
        .and_then(|e| tiff.reals(e).ok())
        .unwrap_or_default()
}

/// The full-resolution raw image: `NewSubFileType` 0 with a colour filter array, in the
/// first directory or one of its sub-directories.
fn raw_ifd(tiff: &Tiff<'_>, first: &Ifd) -> Result<Ifd, RawError> {
    let is_raw = |ifd: &Ifd| {
        one(tiff, ifd, tag::NEW_SUBFILE_TYPE).unwrap_or(0) == 0
            && one(tiff, ifd, tag::PHOTOMETRIC) == Some(32803)
    };
    if is_raw(first) {
        return Ok(first.clone());
    }
    for offset in list(tiff, first, tag::SUB_IFDS) {
        let ifd = tiff.ifd(offset)?;
        if is_raw(&ifd) {
            return Ok(ifd);
        }
    }
    Err(RawError::Unsupported(
        "no colour-filter raw image (a linear DNG)",
    ))
}

pub fn parse(data: &[u8]) -> Result<Dng, RawError> {
    let tiff = Tiff::new(data)?;
    let first = tiff.ifd(tiff.first_ifd()?)?;
    let ifd = raw_ifd(&tiff, &first)?;
    let width = one(&tiff, &ifd, tag::WIDTH).ok_or(RawError::Corrupt("no width"))? as usize;
    let height = one(&tiff, &ifd, tag::HEIGHT).ok_or(RawError::Corrupt("no height"))? as usize;
    let pixels = (width as u64) * (height as u64);
    if pixels == 0 || pixels > MAX_PIXELS {
        return Err(RawError::TooLarge(pixels));
    }
    if one(&tiff, &ifd, tag::SAMPLES_PER_PIXEL).unwrap_or(1) != 1 {
        return Err(RawError::Unsupported("more than one sample per pixel"));
    }
    if list(&tiff, &ifd, tag::CFA_REPEAT_PATTERN_DIM) != [2, 2] {
        return Err(RawError::Unsupported("a colour filter other than 2×2"));
    }
    let pattern: [u8; 4] = ifd
        .get(tag::CFA_PATTERN)
        .and_then(|e| tiff.raw(e).ok())
        .and_then(|b| b.try_into().ok())
        .ok_or(RawError::Corrupt("no CFA pattern"))?;
    // Bayer only: one red, one blue, and the two greens on a diagonal (the demosaic assumes it).
    let mut sorted = pattern;
    sorted.sort_unstable();
    let diagonal = (pattern[0] == 1 && pattern[3] == 1) || (pattern[1] == 1 && pattern[2] == 1);
    if sorted != [0, 1, 1, 2] || !diagonal {
        return Err(RawError::Unsupported(
            "a colour filter other than Bayer RGB",
        ));
    }
    let bits = one(&tiff, &ifd, tag::BITS_PER_SAMPLE).unwrap_or(16);
    if !(8..=16).contains(&bits) {
        return Err(RawError::Unsupported("bits per sample"));
    }
    let mut data_u16 = match one(&tiff, &ifd, tag::COMPRESSION).unwrap_or(1) {
        1 => uncompressed(&tiff, &ifd, width, height, bits)?,
        7 => lossless_jpeg(&tiff, &ifd, width, height)?,
        _ => return Err(RawError::Unsupported("this DNG compression")),
    };
    // Linearization table: raw value → linear value.
    let table = list(&tiff, &ifd, tag::LINEARIZATION_TABLE);
    if !table.is_empty() {
        let last = table.len() - 1;
        data_u16.par_iter_mut().for_each(|v| {
            let t = table[usize::from(*v).min(last)];
            *v = u16::try_from(t).unwrap_or(u16::MAX);
        });
    }
    #[allow(clippy::cast_possible_truncation)]
    let black = {
        let dim = list(&tiff, &ifd, tag::BLACK_LEVEL_REPEAT_DIM);
        let levels = reals(&tiff, &ifd, tag::BLACK_LEVEL);
        // Rows × columns of the repeat (1 × 1 when absent), the values row by row; a 2×2
        // pattern position takes the value its row and column fall on.
        let (rows, cols) = match dim.as_slice() {
            [r, c] if (1..=2).contains(r) && (1..=2).contains(c) => (*r as usize, *c as usize),
            _ => (1, 1),
        };
        let mut black = [0f32; 4];
        if levels.len() >= rows * cols {
            for (i, b) in black.iter_mut().enumerate() {
                *b = levels[(i / 2 % rows) * cols + i % 2 % cols] as f32;
            }
        } else if let Some(l) = levels.first() {
            black = [*l as f32; 4];
        }
        black
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let white = reals(&tiff, &ifd, tag::WHITE_LEVEL)
        .first()
        .map_or(((1u32 << bits) - 1) as f32, |w| *w as f32);
    let active = match list(&tiff, &ifd, tag::ACTIVE_AREA).as_slice() {
        [top, left, bottom, right] => [
            *top as usize,
            *left as usize,
            *bottom as usize,
            *right as usize,
        ],
        _ => [0, 0, height, width],
    };
    // DefaultCropOrigin/Size are relative to the active area.
    let origin = reals(&tiff, &ifd, tag::DEFAULT_CROP_ORIGIN);
    let size = reals(&tiff, &ifd, tag::DEFAULT_CROP_SIZE);
    let (aw, ah) = (
        active[3].saturating_sub(active[1]),
        active[2].saturating_sub(active[0]),
    );
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let crop = match (origin.as_slice(), size.as_slice()) {
        ([x, y], [w, h]) => [*x as usize, *y as usize, *w as usize, *h as usize],
        _ => [0, 0, aw, ah],
    };
    let mosaic = Mosaic::new(width, height, data_u16, pattern, black, white, active, crop)?;
    let color = ColorTags::read(&tiff, &first, &ifd);
    Ok(Dng { mosaic, color })
}

fn chunks(
    tiff: &Tiff<'_>,
    ifd: &Ifd,
    offsets: u16,
    counts: u16,
) -> Result<Vec<(usize, usize)>, RawError> {
    let o = list(tiff, ifd, offsets);
    let c = list(tiff, ifd, counts);
    if o.is_empty() || o.len() != c.len() {
        return Err(RawError::Corrupt("strip or tile offsets"));
    }
    Ok(o.into_iter()
        .zip(c)
        .map(|(a, b)| (a as usize, b as usize))
        .collect())
}

/// Uncompressed strips: 8 or 16 bits as stored, other depths packed most significant bit
/// first, each row starting on a byte.
fn uncompressed(
    tiff: &Tiff<'_>,
    ifd: &Ifd,
    width: usize,
    height: usize,
    bits: u32,
) -> Result<Vec<u16>, RawError> {
    let rows_per_strip = one(tiff, ifd, tag::ROWS_PER_STRIP)
        .map_or(height, |r| r as usize)
        .max(1);
    let row_bytes = (width * bits as usize).div_ceil(8);
    let mut out = vec![0u16; width * height];
    for (s, (offset, _)) in chunks(tiff, ifd, tag::STRIP_OFFSETS, tag::STRIP_BYTE_COUNTS)?
        .into_iter()
        .enumerate()
    {
        let y0 = s * rows_per_strip;
        let rows = rows_per_strip.min(height.saturating_sub(y0));
        let src = tiff.bytes.slice(offset, row_bytes * rows)?;
        for r in 0..rows {
            let row = &src[r * row_bytes..(r + 1) * row_bytes];
            let dst = &mut out[(y0 + r) * width..(y0 + r + 1) * width];
            unpack_row(row, dst, bits, tiff.bytes.order);
        }
    }
    Ok(out)
}

fn unpack_row(row: &[u8], dst: &mut [u16], bits: u32, order: crate::tiff::Order) {
    match bits {
        8 => dst
            .iter_mut()
            .zip(row)
            .for_each(|(d, &b)| *d = u16::from(b)),
        16 => {
            for (d, b) in dst.iter_mut().zip(row.as_chunks::<2>().0) {
                *d = match order {
                    crate::tiff::Order::Little => u16::from_le_bytes([b[0], b[1]]),
                    crate::tiff::Order::Big => u16::from_be_bytes([b[0], b[1]]),
                };
            }
        }
        _ => {
            let (mut acc, mut have, mut i) = (0u32, 0u32, 0usize);
            for d in dst.iter_mut() {
                while have < bits {
                    acc = (acc << 8) | u32::from(row.get(i).copied().unwrap_or(0));
                    i += 1;
                    have += 8;
                }
                have -= bits;
                #[allow(clippy::cast_possible_truncation)]
                {
                    *d = ((acc >> have) & ((1 << bits) - 1)) as u16;
                }
            }
        }
    }
}

/// Lossless JPEG strips or tiles; each one's frame covers `tile_w × tile_h` samples (a
/// frame of W×C samples per row is W·C pixels of the tile, C components side by side).
fn lossless_jpeg(
    tiff: &Tiff<'_>,
    ifd: &Ifd,
    width: usize,
    height: usize,
) -> Result<Vec<u16>, RawError> {
    let (tile_w, tile_h, parts) = if ifd.get(tag::TILE_OFFSETS).is_some() {
        let tw = one(tiff, ifd, tag::TILE_WIDTH).ok_or(RawError::Corrupt("tile width"))? as usize;
        let th = one(tiff, ifd, tag::TILE_LENGTH).ok_or(RawError::Corrupt("tile length"))? as usize;
        (
            tw,
            th,
            chunks(tiff, ifd, tag::TILE_OFFSETS, tag::TILE_BYTE_COUNTS)?,
        )
    } else {
        let rows = one(tiff, ifd, tag::ROWS_PER_STRIP).map_or(height, |r| r as usize);
        (
            width,
            rows,
            chunks(tiff, ifd, tag::STRIP_OFFSETS, tag::STRIP_BYTE_COUNTS)?,
        )
    };
    if tile_w == 0 || tile_h == 0 {
        return Err(RawError::Corrupt("tile size"));
    }
    let across = width.div_ceil(tile_w);
    let decoded: Vec<(usize, crate::ljpeg::Frame)> = parts
        .par_iter()
        .enumerate()
        .map(|(i, &(offset, len))| {
            let bytes = tiff.bytes.slice(offset, len)?;
            Ok((i, crate::ljpeg::decode(bytes)?))
        })
        .collect::<Result<_, RawError>>()?;
    let mut out = vec![0u16; width * height];
    for (i, frame) in decoded {
        let (tx, ty) = ((i % across) * tile_w, (i / across) * tile_h);
        let frame_w = frame.width * frame.components;
        for r in 0..tile_h.min(frame.height) {
            let y = ty + r;
            if y >= height {
                break;
            }
            let n = tile_w.min(frame_w).min(width.saturating_sub(tx));
            let src = &frame.samples[r * frame_w..r * frame_w + n];
            out[y * width + tx..y * width + tx + n].copy_from_slice(src);
        }
    }
    Ok(out)
}

/// The largest image the camera or converter embedded (a JPEG in one strip), for the tone
/// reference and thumbnails.
pub fn embedded_jpeg(data: &[u8]) -> Option<&[u8]> {
    let tiff = Tiff::new(data).ok()?;
    let first = tiff.ifd(tiff.first_ifd().ok()?).ok()?;
    let mut candidates = vec![first.clone()];
    for offset in list(&tiff, &first, tag::SUB_IFDS) {
        if let Ok(ifd) = tiff.ifd(offset) {
            candidates.push(ifd);
        }
    }
    candidates
        .iter()
        .filter(|ifd| one(&tiff, ifd, tag::NEW_SUBFILE_TYPE) == Some(1))
        .filter(|ifd| matches!(one(&tiff, ifd, tag::COMPRESSION), Some(6 | 7)))
        .filter_map(|ifd| {
            let w = one(&tiff, ifd, tag::WIDTH)?;
            let (offset, len) = *chunks(&tiff, ifd, tag::STRIP_OFFSETS, tag::STRIP_BYTE_COUNTS)
                .ok()?
                .first()?;
            let bytes = tiff.bytes.slice(offset, len).ok()?;
            (bytes.get(..2) == Some(&[0xFF, 0xD8])).then_some((w, bytes))
        })
        .max_by_key(|(w, _)| *w)
        .map(|(_, b)| b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A little-endian, uncompressed 16-bit DNG of `w × h` with `extra` entries
    /// (tag, TIFF type, count, value bytes) beside the ones every raw image needs.
    fn synthetic(w: u16, h: u16, data: &[u16], extra: &[(u16, u16, u32, Vec<u8>)]) -> Vec<u8> {
        let short = |v: &[u16]| v.iter().flat_map(|s| s.to_le_bytes()).collect::<Vec<u8>>();
        let long = |v: u32| v.to_le_bytes().to_vec();
        let pixels: Vec<u8> = short(data);
        let mut entries: Vec<(u16, u16, u32, Vec<u8>)> = vec![
            (tag::NEW_SUBFILE_TYPE, 4, 1, long(0)),
            (tag::WIDTH, 3, 1, short(&[w])),
            (tag::HEIGHT, 3, 1, short(&[h])),
            (tag::BITS_PER_SAMPLE, 3, 1, short(&[16])),
            (tag::COMPRESSION, 3, 1, short(&[1])),
            (tag::PHOTOMETRIC, 3, 1, short(&[32803])),
            (tag::STRIP_OFFSETS, 4, 1, long(0)),
            (tag::SAMPLES_PER_PIXEL, 3, 1, short(&[1])),
            (tag::ROWS_PER_STRIP, 3, 1, short(&[h])),
            (
                tag::STRIP_BYTE_COUNTS,
                4,
                1,
                long(u32::try_from(pixels.len()).unwrap_or(0)),
            ),
            (tag::CFA_REPEAT_PATTERN_DIM, 3, 2, short(&[2, 2])),
            (tag::CFA_PATTERN, 1, 4, vec![0, 1, 1, 2]),
            (tag::DNG_VERSION, 1, 4, vec![1, 6, 0, 0]),
        ];
        entries.extend_from_slice(extra);
        entries.sort_by_key(|e| e.0);
        let ifd_len = 2 + 12 * entries.len() + 4;
        let mut values = Vec::new();
        let mut ifd = u16::try_from(entries.len())
            .unwrap_or(0)
            .to_le_bytes()
            .to_vec();
        let pixels_at = 8 + ifd_len + entries.iter().map(|e| e.3.len().max(4)).sum::<usize>();
        for (t, kind, count, bytes) in &entries {
            ifd.extend_from_slice(&t.to_le_bytes());
            ifd.extend_from_slice(&kind.to_le_bytes());
            ifd.extend_from_slice(&count.to_le_bytes());
            let bytes = if *t == tag::STRIP_OFFSETS {
                long(u32::try_from(pixels_at).unwrap_or(0))
            } else {
                bytes.clone()
            };
            if bytes.len() <= 4 {
                let mut inline = bytes;
                inline.resize(4, 0);
                ifd.extend_from_slice(&inline);
            } else {
                let at = 8 + ifd_len + values.len();
                ifd.extend_from_slice(&u32::try_from(at).unwrap_or(0).to_le_bytes());
                values.extend_from_slice(&bytes);
            }
        }
        ifd.extend_from_slice(&0u32.to_le_bytes());
        let mut file = b"II*\0".to_vec();
        file.extend_from_slice(&8u32.to_le_bytes());
        file.extend(ifd);
        file.extend(values);
        file.resize(pixels_at, 0);
        file.extend(pixels);
        file
    }

    fn shorts(v: &[u16]) -> Vec<u8> {
        v.iter().flat_map(|s| s.to_le_bytes()).collect()
    }

    /// DNG 1.7, `BlackLevel` and `CFAPattern`: both patterns start at the top-left corner of
    /// the `ActiveArea`, not of the stored image. An active area at an odd row and column
    /// would otherwise swap red with blue and each site's black level with its neighbour's.
    #[test]
    fn patterns_start_at_the_active_area() -> Result<(), RawError> {
        let (w, h) = (8u16, 6u16);
        let (top, left) = (1usize, 1usize);
        let black = [10u16, 20, 30, 40];
        // Each pixel holds exactly its own black level, so every level reads 0.
        let mut data = vec![999u16; usize::from(w) * usize::from(h)];
        for y in top..usize::from(h) - 1 {
            for x in left..usize::from(w) - 1 {
                data[y * usize::from(w) + x] = black[((y - top) & 1) * 2 + ((x - left) & 1)];
            }
        }
        let file = synthetic(
            w,
            h,
            &data,
            &[
                (tag::BLACK_LEVEL_REPEAT_DIM, 3, 2, shorts(&[2, 2])),
                (tag::BLACK_LEVEL, 3, 4, shorts(&black)),
                (tag::WHITE_LEVEL, 3, 1, shorts(&[1000])),
                (tag::ACTIVE_AREA, 3, 4, shorts(&[1, 1, h - 1, w - 1])),
            ],
        );
        let m = parse(&file)?.mosaic;
        assert_eq!(m.active, [1, 1, 5, 7]);
        for y in top..m.active[2] {
            for x in left..m.active[3] {
                assert_eq!(
                    m.site(x, y),
                    [0, 1, 1, 2][((y - top) & 1) * 2 + ((x - left) & 1)]
                );
                assert!(
                    m.level(x, y).abs() < 1e-6,
                    "({x}, {y}) reads {}",
                    m.level(x, y)
                );
            }
        }
        Ok(())
    }

    /// A black level repeated per column (`BlackLevelRepeatDim` 1 × 2) applies to every row.
    #[test]
    fn a_black_level_per_column() -> Result<(), RawError> {
        let (w, h) = (4u16, 4u16);
        let data: Vec<u16> = (0..16).map(|i| if i % 2 == 0 { 64 } else { 80 }).collect();
        let file = synthetic(
            w,
            h,
            &data,
            &[
                (tag::BLACK_LEVEL_REPEAT_DIM, 3, 2, shorts(&[1, 2])),
                (tag::BLACK_LEVEL, 3, 2, shorts(&[64, 80])),
                (tag::WHITE_LEVEL, 3, 1, shorts(&[1000])),
            ],
        );
        let m = parse(&file)?.mosaic;
        for y in 0..4 {
            for x in 0..4 {
                assert!(
                    m.level(x, y).abs() < 1e-6,
                    "({x}, {y}) reads {}",
                    m.level(x, y)
                );
            }
        }
        Ok(())
    }
}
