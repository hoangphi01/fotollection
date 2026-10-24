// SPDX-License-Identifier: GPL-3.0-only
//! A small visual signature per photo and two similarity measures, for grouping shots of one
//! moment (FR-041). Unlike a dHash, flat areas such as sky carry no weight, a re-framed shot
//! (moved, zoomed a little, or turned from landscape to portrait) still matches, and
//! exposure differences do not matter.

use serde::{Deserialize, Serialize};

use crate::image::RgbImage;

/// Long edge of the luma signature.
const SIGNATURE_EDGE: usize = 32;
/// Shifts tried, in signature pixels (±3 of 32 ≈ ±10 % of the frame).
const MAX_SHIFT: i32 = 3;
/// Zoom factors tried between the two shots.
const SCALES: [f32; 3] = [1.0, 0.85, 1.0 / 0.85];
/// Matches must overlap at least this share of the first photo.
const MIN_OVERLAP: f32 = 0.5;
const HUE_BINS: usize = 12;
const SAT_BINS: usize = 3;
/// Hue × saturation bins, plus one bin for greys.
const COLOR_BINS: usize = HUE_BINS * SAT_BINS + 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub width: u8,
    pub height: u8,
    /// Box-averaged luma, row-major.
    pub luma: Vec<u8>,
    /// Hue/saturation histogram, each bin a share of the image × 255.
    pub color: Vec<u8>,
}

impl Signature {
    /// Values a signature can actually have (cache files are checked, never trusted).
    pub fn is_plausible(&self) -> bool {
        let (w, h) = (usize::from(self.width), usize::from(self.height));
        w > 0
            && h > 0
            && w.max(h) <= SIGNATURE_EDGE
            && self.luma.len() == w * h
            && self.color.len() == COLOR_BINS
    }

    fn at(&self, x: f32, y: f32) -> Option<f32> {
        let (w, h) = (f32::from(self.width), f32::from(self.height));
        if x < 0.0 || y < 0.0 || x > w - 1.0 || y > h - 1.0 {
            return None;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (x0, y0) = (x as usize, y as usize);
        let stride = usize::from(self.width);
        let x1 = (x0 + 1).min(stride - 1);
        let y1 = (y0 + 1).min(usize::from(self.height) - 1);
        #[allow(clippy::cast_precision_loss)]
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let p = |xx: usize, yy: usize| f32::from(self.luma[yy * stride + xx]);
        let top = p(x0, y0) * (1.0 - fx) + p(x1, y0) * fx;
        let bottom = p(x0, y1) * (1.0 - fx) + p(x1, y1) * fx;
        Some(top * (1.0 - fy) + bottom * fy)
    }
}

/// The signature of an (already reduced) upright photo.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn signature(image: &RgbImage) -> Signature {
    let (iw, ih) = (image.width as usize, image.height as usize);
    if iw == 0 || ih == 0 {
        return Signature::default();
    }
    let scale = SIGNATURE_EDGE as f32 / iw.max(ih) as f32;
    let w = ((iw as f32 * scale).round() as usize).clamp(1, SIGNATURE_EDGE);
    let h = ((ih as f32 * scale).round() as usize).clamp(1, SIGNATURE_EDGE);
    let mut sums = vec![0f32; w * h];
    let mut counts = vec![0u32; w * h];
    let mut color = [0u32; COLOR_BINS];
    for (y, row) in image.data.chunks_exact(iw * 3).enumerate() {
        let sy = (y * h / ih).min(h - 1);
        for (x, &[r, g, b]) in row.as_chunks::<3>().0.iter().enumerate() {
            let sx = (x * w / iw).min(w - 1);
            let (r, g, b) = (f32::from(r), f32::from(g), f32::from(b));
            sums[sy * w + sx] += 0.299 * r + 0.587 * g + 0.114 * b;
            counts[sy * w + sx] += 1;
            color[color_bin(r, g, b)] += 1;
        }
    }
    let total = (iw * ih) as f32;
    Signature {
        width: w as u8,
        height: h as u8,
        luma: sums
            .iter()
            .zip(&counts)
            .map(|(s, &n)| (s / n.max(1) as f32).round().clamp(0.0, 255.0) as u8)
            .collect(),
        color: color
            .iter()
            .map(|&n| (n as f32 / total * 255.0).round() as u8)
            .collect(),
    }
}

/// Hue (12 bins) × saturation (3 bins); dark, bright-white and grey pixels share one bin.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn color_bin(r: f32, g: f32, b: f32) -> usize {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    if max < 24.0 || chroma < 0.12 * max {
        return COLOR_BINS - 1;
    }
    let hue = if (max - r).abs() < f32::EPSILON {
        ((g - b) / chroma).rem_euclid(6.0)
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / chroma + 2.0
    } else {
        (r - g) / chroma + 4.0
    };
    let h = ((hue / 6.0 * HUE_BINS as f32) as usize).min(HUE_BINS - 1);
    let s = ((chroma / max - 0.12) / 0.88 * SAT_BINS as f32) as usize;
    h * SAT_BINS + s.min(SAT_BINS - 1)
}

/// Pearson correlation of `a` against `b` sampled through `map`, over pixels both cover.
fn correlation(a: &Signature, b: &Signature, map: impl Fn(f32, f32) -> (f32, f32)) -> Option<f32> {
    let (mut n, mut sa, mut sb, mut saa, mut sbb, mut sab) = (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
    for y in 0..a.height {
        for x in 0..a.width {
            let (bx, by) = map(f32::from(x), f32::from(y));
            let Some(vb) = b.at(bx, by) else { continue };
            let va = f32::from(a.luma[usize::from(y) * usize::from(a.width) + usize::from(x)]);
            n += 1.0;
            sa += va;
            sb += vb;
            saa += va * va;
            sbb += vb * vb;
            sab += va * vb;
        }
    }
    let area = f32::from(a.width) * f32::from(a.height);
    if n < MIN_OVERLAP * area {
        return None;
    }
    let cov = sab / n - sa / n * sb / n;
    let var_a = saa / n - (sa / n).powi(2);
    let var_b = sbb / n - (sb / n).powi(2);
    // Two featureless frames (fog, a white wall) say nothing about each other.
    if var_a < 4.0 || var_b < 4.0 {
        return Some(0.0);
    }
    Some(cov / (var_a * var_b).sqrt())
}

/// How alike the two photos' layouts are, −1…1: the best correlation over small shifts and
/// zooms, centres aligned (so a landscape and a portrait shot of one view also compare).
pub fn structure(a: &Signature, b: &Signature) -> f32 {
    structure_zoomed(a, b, 1.0)
}

/// Like [`structure`] when `b` was shot `zoom` times closer than `a` (the ratio of the
/// focal lengths): the closer shot is compared with the matching centre of the wider one.
#[allow(clippy::cast_precision_loss)]
pub fn structure_zoomed(a: &Signature, b: &Signature, zoom: f32) -> f32 {
    if !a.is_plausible() || !b.is_plausible() || !zoom.is_finite() || zoom <= 0.0 {
        return 0.0;
    }
    // Walk the closer shot's pixels and sample the wider one, so the overlap is complete.
    let (a, b, zoom) = if zoom > 1.0 {
        (b, a, 1.0 / zoom)
    } else {
        (a, b, zoom)
    };
    let (ca, cb) = (
        (f32::from(a.width) - 1.0) / 2.0,
        (f32::from(a.height) - 1.0) / 2.0,
    );
    let (cx, cy) = (
        (f32::from(b.width) - 1.0) / 2.0,
        (f32::from(b.height) - 1.0) / 2.0,
    );
    let mut best = -1f32;
    for scale in SCALES.map(|s| s * zoom) {
        for dy in -MAX_SHIFT..=MAX_SHIFT {
            for dx in -MAX_SHIFT..=MAX_SHIFT {
                let map = |x: f32, y: f32| {
                    (
                        (x - ca) * scale + cx + dx as f32,
                        (y - cb) * scale + cy + dy as f32,
                    )
                };
                if let Some(c) = correlation(a, b, map) {
                    best = best.max(c);
                }
            }
        }
    }
    best
}

/// How alike the two photos' colours are, 0–1 (histogram intersection).
pub fn color(a: &Signature, b: &Signature) -> f32 {
    if !a.is_plausible() || !b.is_plausible() {
        return 0.0;
    }
    let shared: u32 = a
        .color
        .iter()
        .zip(&b.color)
        .map(|(x, y)| u32::from(*x.min(y)))
        .sum();
    let total: u32 = a.color.iter().map(|&v| u32::from(v)).sum::<u32>().max(1);
    #[allow(clippy::cast_precision_loss)]
    let share = shared as f32 / total as f32;
    share.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::culling::metrics::tests::scene;

    /// The 640 × 480 test scene, cropped to `w` × `h` at (`x`, `y`), scaled to `out_w` wide
    /// (nearest neighbour), brightness × `gain`.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn view(x: usize, y: usize, w: usize, h: usize, out_w: usize, gain: f32) -> RgbImage {
        let g = scene(640, 480, 0);
        let out_h = h * out_w / w;
        let mut data = Vec::with_capacity(out_w * out_h * 3);
        for oy in 0..out_h {
            for ox in 0..out_w {
                let v = g.data[(y + oy * h / out_h) * 640 + x + ox * w / out_w];
                let v = (v * gain).clamp(0.0, 255.0) as u8;
                data.extend([v, v.saturating_sub(20), v / 2]);
            }
        }
        RgbImage::from_raw(out_w as u32, out_h as u32, data).unwrap_or_else(|| RgbImage::new(1, 1))
    }

    #[test]
    fn fr_041_layout_survives_shifts_and_exposure() {
        let a = signature(&view(0, 0, 600, 450, 256, 1.0));
        let shifted = signature(&view(30, 20, 600, 450, 256, 1.0));
        let darker = signature(&view(0, 0, 600, 450, 256, 0.6));
        assert!(structure(&a, &shifted) > 0.9, "{}", structure(&a, &shifted));
        assert!(structure(&a, &darker) > 0.95, "{}", structure(&a, &darker));
        assert!(color(&a, &a) > 0.99);
    }

    #[test]
    fn fr_041_zoomed_reshoot_matches_with_the_focal_ratio() {
        let wide = signature(&view(0, 0, 640, 480, 256, 1.0));
        let close = signature(&view(160, 120, 320, 240, 256, 1.0)); // 2× closer, same centre
        let with_ratio = structure_zoomed(&wide, &close, 2.0);
        // The scene's 24 px checks alias in the wide shot's 20 px cells, so the match is not
        // perfect; it must still clear the grouping bound.
        assert!(
            with_ratio >= crate::culling::group::MIN_STRUCTURE,
            "{with_ratio}"
        );
        assert!(with_ratio > structure(&wide, &close) + 0.1);
        assert!((structure_zoomed(&close, &wide, 0.5) - with_ratio).abs() < 1e-6);
    }

    #[test]
    fn damaged_signatures_compare_as_unrelated() {
        let a = signature(&view(0, 0, 600, 450, 256, 1.0));
        let broken = Signature {
            luma: vec![1, 2, 3],
            ..a.clone()
        };
        assert!(!broken.is_plausible());
        assert!(structure(&a, &broken) <= 0.0);
        assert!(color(&a, &broken) <= 0.0);
    }
}
