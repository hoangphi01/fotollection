// SPDX-License-Identifier: GPL-3.0-only
//! Fotollection's own camera RAW decoder (ADR-0030): pure Rust, no `unsafe`, written from
//! public specifications (DNG 1.7, TIFF 6.0, ITU T.81, the Malvar–He–Cutler paper) and our
//! own notes on formats that have none (`docs/raw/`). No code from rawler, rawloader, dcraw,
//! `LibRaw` or `RawTherapee` is read into it.
//!
//! RAW files are untrusted input: every offset, count and size is checked against the bytes
//! there, and a bad file is an error, never a panic.

pub mod cfa;
pub mod color;
pub mod demosaic;
pub mod directional;
pub mod dng;
pub mod falsecolour;
pub mod ljpeg;
pub mod prepare;
pub mod tiff;
pub mod tone;

use rayon::prelude::*;
use thiserror::Error;

pub use tone::{
    Linear, Look, Picture, RawAdjust, Reference, adjust, default_curve, finish, finish_look,
    match_look, match_reference, neutral_look,
};

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum RawError {
    #[error("the file is cut short or points outside itself")]
    Truncated,
    #[error("this file is not a supported RAW: {0}")]
    Unsupported(&'static str),
    #[error("the file is damaged: {0}")]
    Corrupt(&'static str),
    #[error("the photo is too large ({0} pixels)")]
    TooLarge(u64),
}

/// Whether this decoder reads the file (DNG for now; ARW and RW2 follow, ADR-0030).
pub fn supports(data: &[u8]) -> bool {
    dng::is_dng(data)
}

/// Whether a TIFF-based RAW (CR2, NEF, ARW, DNG, ORF, RW2…) points at image data past its
/// own end: some decoders read on past the end and make up the missing part instead of
/// failing (rawler's CR2 draws diagonal stripes, LESSONS 2026-11-05). Every strip and tile
/// of the main directories and their sub-directories must end inside the file. A file that
/// is not TIFF says nothing here (`false`); at most [`MAX_DIRECTORIES`] are looked at.
pub fn cut_short(data: &[u8]) -> bool {
    const STRIP_OFFSETS: u16 = 273;
    const STRIP_BYTE_COUNTS: u16 = 279;
    const TILE_OFFSETS: u16 = 324;
    const TILE_BYTE_COUNTS: u16 = 325;
    const SUB_IFDS: u16 = 330;
    const PANASONIC_RAW_OFFSET: u16 = 280;
    let Ok(tiff) = tiff::Tiff::new(data) else {
        return false;
    };
    let Ok(first) = tiff.first_ifd() else {
        return true;
    };
    let mut queue = vec![first];
    let mut seen = Vec::new();
    while let Some(offset) = queue.pop() {
        if offset == 0 || seen.contains(&offset) || seen.len() >= MAX_DIRECTORIES {
            continue;
        }
        seen.push(offset);
        let Ok(ifd) = tiff.ifd(offset) else {
            // The main directory must be there; a damaged later one is not ours to judge.
            return seen.len() == 1;
        };
        queue.push(ifd.next);
        if let Some(e) = ifd.get(SUB_IFDS) {
            queue.extend(tiff.uints(e).unwrap_or_default());
        }
        for (offsets, counts) in [
            (STRIP_OFFSETS, STRIP_BYTE_COUNTS),
            (TILE_OFFSETS, TILE_BYTE_COUNTS),
        ] {
            let (Some(o), Some(c)) = (ifd.get(offsets), ifd.get(counts)) else {
                continue;
            };
            let mut o = tiff.uints(o).unwrap_or_default();
            let c = tiff.uints(c).unwrap_or_default();
            // Panasonic writes a placeholder strip and the data's offset in a tag of its own.
            if o == [u32::MAX] {
                let raw = ifd
                    .get(PANASONIC_RAW_OFFSET)
                    .and_then(|e| tiff.uints(e).ok());
                o = raw.unwrap_or_default();
            }
            if o.iter()
                .zip(&c)
                .any(|(&o, &c)| u64::from(o) + u64::from(c) > data.len() as u64)
            {
                return true;
            }
        }
    }
    false
}

/// Directories [`cut_short`] looks at, at most (a real RAW has a handful).
const MAX_DIRECTORIES: usize = 32;

/// The developed size: the camera's default crop.
pub fn dimensions(data: &[u8]) -> Result<(usize, usize), RawError> {
    Ok(dng::parse(data)?.mosaic.size())
}

/// Develops a DNG to scene-linear sRGB (demosaic, white balance, the colour model);
/// `finish` gives it its tone.
pub fn develop(data: &[u8]) -> Result<Linear, RawError> {
    let dng = dng::parse(data)?;
    develop_mosaic(&dng.mosaic, &dng.color)
}

/// Where any channel nears its sensor's clip (its white-balance gain), the pixel moves toward
/// a neutral at its brightest channel: the other channels' values there say how bright it
/// was, but not its colour, which a clipped channel would make magenta or cyan (FR-088).
pub fn highlights_to_white(rgb: &mut [f32], clip: [f32; 3]) {
    const FROM: f32 = 0.85;
    rgb.par_chunks_exact_mut(3).for_each(|p| {
        let t = (0..3)
            .map(|c| ((p[c] / clip[c] - FROM) / (1.0 - FROM)).clamp(0.0, 1.0))
            .fold(0f32, f32::max);
        if t > 0.0 {
            let t = t * t * (3.0 - 2.0 * t);
            let m = p[0].max(p[1]).max(p[2]);
            for v in p.iter_mut() {
                *v += (m - *v) * t;
            }
        }
    });
}

/// Develops a sensor's mosaic, whatever file it came from, with its colour tags.
pub fn develop_mosaic(mosaic: &cfa::Mosaic, tags: &color::ColorTags) -> Result<Linear, RawError> {
    let rendering = color::rendering(tags).ok_or(RawError::Unsupported("no colour matrix"))?;
    // Faults of single sites first, while each still holds only what it measured; then the
    // directional demosaic and the false-colour median (RESULT §25).
    let mut plane = demosaic::balance(mosaic, rendering.gains);
    prepare::hot_pixels(&mut plane);
    prepare::green_balance(&mut plane);
    // Each channel keeps its own range after white balance (its gain over green's), so a
    // highlight past green's clip still holds red and blue (FR-088); what nears a channel's
    // clip goes to white instead of turning magenta or cyan.
    let headroom = rendering.gains.iter().copied().fold(1.0f32, f32::max);
    let mut camera = directional::directional(&plane, headroom);
    drop(plane);
    falsecolour::suppress(&mut camera);
    highlights_to_white(&mut camera.rgb, rendering.gains);
    let m = rendering.to_srgb;
    #[allow(clippy::cast_possible_truncation)]
    let m32 = m.map(|row| row.map(|v| v as f32));
    let mut rgb = camera.rgb;
    rgb.par_chunks_exact_mut(3).for_each(|p| {
        let (r, g, b) = (p[0], p[1], p[2]);
        for (i, row) in m32.iter().enumerate() {
            p[i] = (row[0] * r + row[1] * g + row[2] * b).max(0.0);
        }
    });
    Ok(Linear {
        width: camera.width,
        height: camera.height,
        rgb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The CC0 samples (`scripts/fetch-raw-samples.py`) when `FOTOLLECTION_RAW_SAMPLES` names
    /// their folder: each file this decoder reads develops at the size `dimensions` gives, and
    /// cut short at 256 places it fails cleanly, never panics. `examples/mutate.rs` goes further.
    /// A strip that ends past the file is caught, one that ends inside it is not, and a
    /// file that is not TIFF says nothing.
    #[test]
    fn fr_016_a_strip_past_the_end_is_cut_short() {
        // IFD0 at 8 with StripOffsets = 26 and StripByteCounts = n, then the data.
        let file = |count: u32, data: usize| {
            let mut t = b"II*\0".to_vec();
            t.extend_from_slice(&8u32.to_le_bytes());
            t.extend_from_slice(&2u16.to_le_bytes());
            for (tag, value) in [(273u16, 38u32), (279, count)] {
                t.extend_from_slice(&tag.to_le_bytes());
                t.extend_from_slice(&4u16.to_le_bytes());
                t.extend_from_slice(&1u32.to_le_bytes());
                t.extend_from_slice(&value.to_le_bytes());
            }
            t.extend_from_slice(&0u32.to_le_bytes());
            t.extend(std::iter::repeat_n(7u8, data));
            t
        };
        assert!(!cut_short(&file(100, 100)));
        assert!(cut_short(&file(100, 60)));
        assert!(!cut_short(b"not a tiff at all"));
    }

    /// A pixel whose green reached its clip while red and blue did not comes out neutral, at
    /// its brightest channel; one far from any clip keeps its colour.
    #[test]
    fn fr_088_a_clipped_channel_goes_to_white_not_to_a_colour() {
        let gains = [2.0, 1.0, 1.6];
        let mut rgb = vec![1.4, 1.0, 1.2, 0.3, 0.5, 0.2];
        highlights_to_white(&mut rgb, gains);
        assert!(
            (rgb[0] - 1.4).abs() < 1e-6
                && (rgb[1] - 1.4).abs() < 1e-6
                && (rgb[2] - 1.4).abs() < 1e-6,
            "{rgb:?}"
        );
        assert_eq!(&rgb[3..], &[0.3, 0.5, 0.2]);
    }

    /// FR-088: highlights −1 brings a ramp from middle grey to three stops over white down,
    /// still in order (detail kept, not flattened), and leaves middle grey and below alone;
    /// shadows +1 lifts the dark end, also in order.
    #[test]
    fn fr_088_highlights_and_shadows_keep_the_order_of_tones() {
        let ramp: Vec<f32> = [0.002, 0.02, 0.18, 0.5, 1.0, 2.0, 4.0, 8.0]
            .iter()
            .flat_map(|&v| [v, v, v])
            .collect();
        let run = |a: RawAdjust| {
            let mut l = Linear {
                width: 8,
                height: 1,
                rgb: ramp.clone(),
            };
            adjust(&mut l, &a);
            l.rgb
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| p[1])
                .collect::<Vec<_>>()
        };
        let down = run(RawAdjust {
            highlights: -1.0,
            ..RawAdjust::default()
        });
        assert!(down.windows(2).all(|w| w[0] < w[1]), "{down:?}");
        assert!(
            (down[2] - 0.18).abs() < 1e-4 && down[7] < 8.0 / 3.0,
            "{down:?}"
        );
        let up = run(RawAdjust {
            shadows: 1.0,
            ..RawAdjust::default()
        });
        assert!(
            up.windows(2).all(|w| w[0] < w[1]) && up[0] > 0.002 * 3.0,
            "{up:?}"
        );
    }

    #[test]
    fn samples_develop_at_their_size_and_truncation_never_panics()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(dir) = std::env::var_os("FOTOLLECTION_RAW_SAMPLES") else {
            return Ok(());
        };
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let bytes = std::fs::read(path)?;
            if !supports(&bytes) {
                continue;
            }
            let (w, h) = dimensions(&bytes)?;
            let linear = develop(&bytes)?;
            assert_eq!((linear.width, linear.height), (w, h));
            for i in 0..256 {
                let cut = bytes.len() * i / 256;
                let short = &bytes[..cut];
                let _ = dimensions(short);
                assert!(
                    develop(short).is_err() || cut > bytes.len() / 2,
                    "cut at {cut} still developed"
                );
            }
        }
        Ok(())
    }
}
