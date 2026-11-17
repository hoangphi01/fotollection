// SPDX-License-Identifier: GPL-3.0-only
//! Near-duplicates (FR-015, FR-141 S5): the same picture saved twice, received again through
//! a chat, or given another colour (black and white, a filter), never one capture (those are
//! S3's stacks). Three steps, measured on the owner's 104 labelled pairs (2026-11-17):
//!
//! 1. Candidates: a 64-bit difference hash (dHash) of each upright photo's 9 × 8 grey
//!    reduction, pairs within `max_distance` bits. Two hashes at most 7 bits apart share one
//!    of their eight bytes, so an index on (byte, value) finds every such pair.
//! 2. Proof: the two pictures on one grid of `grid` cells, the second's grey fitted to the
//!    first's by a line (a filter's brightness and contrast), then the largest mean residual
//!    over blocks of 8 × 8 cells. A re-save or a recoloured copy stays low everywhere; two
//!    screens of one app (two bank transfers) or two shots differ strongly somewhere. The
//!    hash alone called 25 of 104 such pairs duplicates wrongly; the proof none.
//! 3. Kind: a copy whose saturation or mean colour moved is a colour version, kept by default;
//!    the rest are re-saves.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::stacks::{Groups, Stack};
use crate::codec::JpegCodec;
use crate::image::RgbImage;

/// The hash's width and height; one column more for the differences.
const W: usize = 9;
const H: usize = 8;
/// The pigeonhole index is exact up to this distance.
pub const MAX_EXACT_DISTANCE: u32 = 7;

/// The difference hash of an upright image: bit `y * 8 + x` is set when cell `(x, y)` of its
/// 9 × 8 grey reduction is brighter than cell `(x + 1, y)`.
pub fn dhash(image: &RgbImage) -> u64 {
    let (w, h) = (image.width as usize, image.height as usize);
    if w == 0 || h == 0 || image.data.len() < w * h * 3 {
        return 0;
    }
    let mut cells = [0f32; W * H];
    for (cy, row) in cells.chunks_mut(W).enumerate() {
        let (y0, y1) = (cy * h / H, ((cy + 1) * h / H).max(cy * h / H + 1));
        for (cx, cell) in row.iter_mut().enumerate() {
            let (x0, x1) = (cx * w / W, ((cx + 1) * w / W).max(cx * w / W + 1));
            let mut sum = 0u64;
            for y in y0..y1.min(h) {
                for x in x0..x1.min(w) {
                    let p = &image.data[(y * w + x) * 3..(y * w + x) * 3 + 3];
                    sum += u64::from(p[0]) * 299 + u64::from(p[1]) * 587 + u64::from(p[2]) * 114;
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = ((y1.min(h) - y0) * (x1.min(w) - x0)).max(1) as f32;
            #[allow(clippy::cast_precision_loss)]
            let mean = sum as f32 / n;
            *cell = mean;
        }
    }
    let mut hash = 0u64;
    for y in 0..H {
        for x in 0..W - 1 {
            if cells[y * W + x] > cells[y * W + x + 1] {
                hash |= 1 << (y * (W - 1) + x);
            }
        }
    }
    hash
}

/// A photo's hash from the cheapest picture it has: the JPEG thumbnail in its EXIF, else the
/// file decoded for the screen; turned upright first. `None` when neither decodes (a HEIC
/// where the system has no decoder).
pub fn hash_file(path: &Path, orientation: u8, codec: &dyn JpegCodec) -> Option<u64> {
    let image = crate::metadata::embedded_thumbnail(path)
        .and_then(|t| codec.decode(&t).ok())
        .map(|d| d.image)
        .or_else(|| {
            let bytes = std::fs::read(path).ok()?;
            crate::codec::decode_for_screen(codec, &bytes, 512)
                .ok()
                .map(|d| d.image)
        })?;
    Some(dhash(&crate::image::apply_orientation(image, orientation)))
}

/// Every pair within `max_distance` bits (at most [`MAX_EXACT_DISTANCE`]), lower index first,
/// sorted, leaving out pairs already stacked as one capture.
pub fn candidate_pairs(
    hashes: &[Option<u64>],
    max_distance: u32,
    same_capture: &[Stack],
) -> Vec<(usize, usize)> {
    let max_distance = max_distance.min(MAX_EXACT_DISTANCE);
    let mut stack_of: HashMap<usize, usize> = HashMap::new();
    for (s, st) in same_capture.iter().enumerate() {
        for &m in &st.members {
            stack_of.insert(m as usize, s);
        }
    }
    let mut index: HashMap<(usize, u8), Vec<usize>> = HashMap::new();
    for (i, h) in hashes.iter().enumerate() {
        if let Some(h) = h {
            for (b, byte) in h.to_le_bytes().into_iter().enumerate() {
                index.entry((b, byte)).or_default().push(i);
            }
        }
    }
    let mut pairs: HashSet<(usize, usize)> = HashSet::new();
    for bucket in index.values() {
        for (k, &a) in bucket.iter().enumerate() {
            for &b in &bucket[k + 1..] {
                let (Some(ha), Some(hb)) = (hashes[a], hashes[b]) else {
                    continue;
                };
                let together = stack_of
                    .get(&a)
                    .is_some_and(|s| stack_of.get(&b) == Some(s));
                if !together && (ha ^ hb).count_ones() <= max_distance {
                    pairs.insert((a.min(b), a.max(b)));
                }
            }
        }
    }
    let mut pairs: Vec<_> = pairs.into_iter().collect();
    pairs.sort_unstable();
    pairs
}

/// How a duplicate differs from the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DupKind {
    /// The same picture saved again: one of them can go.
    Resave,
    /// The same picture in another colour (black and white, a filter): both kept by default.
    ColourVersion,
}

/// The proof's thresholds, as data.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct DupRules {
    pub max_distance: u32,
    /// Cells on the long side of the proof's grid.
    pub grid: usize,
    /// The largest mean residual of an 8 × 8 block of cells, in grey levels.
    pub max_block: f32,
    /// A colour version's change in mean saturation, or in a channel's mean, in levels.
    pub colour_saturation: f32,
    pub colour_mean: f32,
}

/// What the proof reads of one picture: its grey on the grid, its mean colour and saturation.
#[derive(Debug, Clone, PartialEq)]
pub struct Fingerprint {
    pub width: usize,
    pub height: usize,
    pub grey: Vec<f32>,
    pub mean: [f32; 3],
    pub saturation: f32,
}

/// The picture box-averaged onto `grid` cells on its long side.
pub fn fingerprint(image: &RgbImage, grid: usize) -> Option<Fingerprint> {
    let (w, h) = (image.width as usize, image.height as usize);
    if w == 0 || h == 0 || image.data.len() < w * h * 3 || grid == 0 {
        return None;
    }
    let (gw, gh) = if w >= h {
        (grid, (grid * h + w / 2) / w)
    } else {
        ((grid * w + h / 2) / h, grid)
    };
    let (gw, gh) = (gw.clamp(1, w), gh.clamp(1, h));
    let mut grey = vec![0f32; gw * gh];
    let (mut mean, mut saturation) = ([0f32; 3], 0f32);
    for gy in 0..gh {
        let (y0, y1) = (gy * h / gh, ((gy + 1) * h / gh).max(gy * h / gh + 1));
        for gx in 0..gw {
            let (x0, x1) = (gx * w / gw, ((gx + 1) * w / gw).max(gx * w / gw + 1));
            let mut sum = [0u64; 3];
            for y in y0..y1 {
                let row = &image.data[(y * w + x0) * 3..(y * w + x1) * 3];
                for p in row.as_chunks::<3>().0 {
                    for c in 0..3 {
                        sum[c] += u64::from(p[c]);
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = ((y1 - y0) * (x1 - x0)) as f32;
            #[allow(clippy::cast_precision_loss)]
            let rgb = sum.map(|v| v as f32 / n);
            grey[gy * gw + gx] = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
            for c in 0..3 {
                mean[c] += rgb[c];
            }
            saturation += rgb.iter().copied().fold(f32::MIN, f32::max)
                - rgb.iter().copied().fold(f32::MAX, f32::min);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let cells = (gw * gh) as f32;
    Some(Fingerprint {
        width: gw,
        height: gh,
        grey,
        mean: mean.map(|m| m / cells),
        saturation: saturation / cells,
    })
}

/// Whether two pictures are one (and how they differ), by the proof's rules.
pub fn same_picture(a: &Fingerprint, b: &Fingerprint, rules: &DupRules) -> Option<DupKind> {
    if (a.width, a.height) != (b.width, b.height) || a.grey.is_empty() {
        return None;
    }
    // b's grey fitted to a's by least squares: y = k x + c.
    #[allow(clippy::cast_precision_loss)]
    let n = a.grey.len() as f32;
    let (mx, my) = (
        b.grey.iter().sum::<f32>() / n,
        a.grey.iter().sum::<f32>() / n,
    );
    let (mut sxy, mut sxx) = (0f32, 0f32);
    for (x, y) in b.grey.iter().zip(&a.grey) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
    }
    let k = if sxx > 0.0 { sxy / sxx } else { 1.0 };
    let c = my - k * mx;
    let block = (a.width.max(a.height) / 16).max(2);
    let mut worst = 0f32;
    for by in (0..a.height).step_by(block) {
        for bx in (0..a.width).step_by(block) {
            let (mut sum, mut count) = (0f32, 0f32);
            for y in by..(by + block).min(a.height) {
                for x in bx..(bx + block).min(a.width) {
                    let i = y * a.width + x;
                    sum += (a.grey[i] - (k * b.grey[i] + c)).abs();
                    count += 1.0;
                }
            }
            worst = worst.max(sum / count);
        }
    }
    if worst > rules.max_block {
        return None;
    }
    let shift = (0..3)
        .map(|c| (a.mean[c] - b.mean[c]).abs())
        .fold(0f32, f32::max);
    let recoloured =
        (a.saturation - b.saturation).abs() > rules.colour_saturation || shift > rules.colour_mean;
    Some(if recoloured {
        DupKind::ColourVersion
    } else {
        DupKind::Resave
    })
}

/// A group of duplicates and the kinds that tie it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DupGroup {
    pub members: Vec<usize>,
    pub kinds: Vec<DupKind>,
}

/// The groups the proven pairs make, by their first member.
pub fn groups(n: usize, proven: &[(usize, usize, DupKind)]) -> Vec<DupGroup> {
    let mut g = Groups::new(n);
    for &(a, b, _) in proven {
        g.join(a, b, super::stacks::StackKind::Twins);
    }
    let mut kinds: HashMap<usize, Vec<DupKind>> = HashMap::new();
    for &(a, _, kind) in proven {
        let list = kinds.entry(g.root(a)).or_default();
        if !list.contains(&kind) {
            list.push(kind);
        }
    }
    g.sets()
        .into_iter()
        .map(|members| {
            let kinds = kinds.remove(&g.root(members[0])).unwrap_or_default();
            DupGroup { members, kinds }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test picture: a soft diagonal ramp with a bright square, at any size.
    fn picture(w: u32, h: u32, square_at: u32) -> RgbImage {
        let mut img = RgbImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let base = u8::try_from((x * 120 / w + y * 100 / h).min(255)).unwrap_or(0);
                let inside = (x * 8 / w, y * 8 / h) == (square_at, square_at);
                let v = if inside { 250 } else { base };
                let i = ((y * w + x) * 3) as usize;
                img.data[i..i + 3].copy_from_slice(&[v, v / 2, 255 - v]);
            }
        }
        img
    }

    #[test]
    fn fr_015_a_resaved_copy_is_near_and_another_picture_far() {
        let a = dhash(&picture(1200, 900, 2));
        let small = dhash(&picture(160, 120, 2));
        let other = dhash(&crate::image::apply_orientation(picture(1200, 900, 2), 3));
        assert!(
            (a ^ small).count_ones() <= 3,
            "{}",
            (a ^ small).count_ones()
        );
        assert!(
            (a ^ other).count_ones() >= 8,
            "{}",
            (a ^ other).count_ones()
        );
    }

    const RULES: DupRules = DupRules {
        max_distance: 7,
        grid: 128,
        max_block: 13.0,
        colour_saturation: 8.0,
        colour_mean: 5.0,
    };

    #[test]
    fn fr_015_the_pigeonhole_index_finds_every_close_pair() {
        let mut seed = 0x2545_F491_4F6C_DD1D_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut hashes: Vec<Option<u64>> = (0..1000).map(|_| Some(next())).collect();
        // Near copies of the first hundred, 0 to 7 bits off.
        for i in 0..100 {
            let flips = (0..(i % 8)).fold(0u64, |m, _| m | 1 << (next() % 64));
            hashes.push(hashes[i].map(|h| h ^ flips));
        }
        let mut brute = Vec::new();
        for a in 0..hashes.len() {
            for b in a + 1..hashes.len() {
                if let (Some(x), Some(y)) = (hashes[a], hashes[b])
                    && (x ^ y).count_ones() <= 7
                {
                    brute.push((a, b));
                }
            }
        }
        assert_eq!(candidate_pairs(&hashes, 7, &[]), brute);
        assert!(brute.len() >= 100);
    }

    #[test]
    fn fr_015_one_capture_is_no_duplicate() {
        let hashes = [Some(7u64), Some(7), Some(7)];
        let stack = Stack {
            kinds: vec![],
            members: vec![0, 1],
            cover: 0,
        };
        assert_eq!(candidate_pairs(&hashes, 0, &[stack]), vec![(0, 2), (1, 2)]);
    }

    #[test]
    fn fr_015_the_proof_tells_copies_colour_versions_and_two_screens_apart() {
        let print = |img: &RgbImage| fingerprint(img, RULES.grid).unwrap_or_else(|| unreachable!());
        let photo = picture(1200, 900, 2);
        let copy = picture(160, 120, 2);
        let mut grey = photo.clone();
        for p in grey.data.as_chunks_mut::<3>().0 {
            let y = u8::try_from(
                (u32::from(p[0]) * 299 + u32::from(p[1]) * 587 + u32::from(p[2]) * 114) / 1000,
            )
            .unwrap_or(0);
            p.copy_from_slice(&[y, y, y]);
        }
        // Another screen of the same app: one line of text differs.
        let mut screen = photo.clone();
        for y in 360..440 {
            for x in 300..700 {
                let i = (y * 1200 + x) * 3;
                screen.data[i..i + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        assert_eq!(
            same_picture(&print(&photo), &print(&copy), &RULES),
            Some(DupKind::Resave)
        );
        assert_eq!(
            same_picture(&print(&photo), &print(&grey), &RULES),
            Some(DupKind::ColourVersion)
        );
        assert_eq!(same_picture(&print(&photo), &print(&screen), &RULES), None);
        let g = groups(
            4,
            &[(0, 1, DupKind::Resave), (1, 3, DupKind::ColourVersion)],
        );
        assert_eq!(
            g,
            vec![DupGroup {
                members: vec![0, 1, 3],
                kinds: vec![DupKind::Resave, DupKind::ColourVersion]
            }]
        );
    }
}
