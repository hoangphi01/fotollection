// SPDX-License-Identifier: GPL-3.0-only
//! Light output sharpening after a downscale (FR-053; the start of FR-029): an unsharp
//! mask over a 3 × 3 binomial blur, `out = in + amount · (in − blur)`. Downscaling softens
//! fine texture; this restores it without halos at the small amounts used.

use rayon::prelude::*;

use crate::image::RgbImage;

/// Sharpens `image` in place by `amount` (0 = unchanged; around 0.3–0.5 for screens).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn unsharp(image: &mut RgbImage, amount: f32) {
    let (w, h) = (image.width as usize, image.height as usize);
    if w < 3 || h < 3 || !(amount.is_finite() && amount > 0.0) {
        return;
    }
    let src = image.data.clone();
    let px = |x: usize, y: usize, c: usize| f32::from(src[(y * w + x) * 3 + c]);
    image
        .data
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let (y0, y2) = (y.saturating_sub(1), (y + 1).min(h - 1));
            for x in 0..w {
                let (x0, x2) = (x.saturating_sub(1), (x + 1).min(w - 1));
                for c in 0..3 {
                    // 1-2-1 weights in both directions, total 16.
                    let blur = (px(x0, y0, c)
                        + 2.0 * px(x, y0, c)
                        + px(x2, y0, c)
                        + 2.0 * (px(x0, y, c) + 2.0 * px(x, y, c) + px(x2, y, c))
                        + px(x0, y2, c)
                        + 2.0 * px(x, y2, c)
                        + px(x2, y2, c))
                        / 16.0;
                    let v = px(x, y, c);
                    row[x * 3 + c] = (v + amount * (v - blur)).round().clamp(0.0, 255.0) as u8;
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_053_flat_areas_stay_flat_and_edges_gain_contrast() {
        let mut flat = RgbImage::new(8, 8);
        flat.data.fill(100);
        unsharp(&mut flat, 0.5);
        assert!(flat.data.iter().all(|&v| v == 100));

        // A vertical edge: dark left half, light right half.
        let mut edge = RgbImage::new(8, 8);
        for y in 0..8 {
            for x in 4..8 {
                edge.data[(y * 8 + x) * 3..(y * 8 + x) * 3 + 3].fill(200);
            }
        }
        for y in 0..8 {
            for x in 0..4 {
                edge.data[(y * 8 + x) * 3..(y * 8 + x) * 3 + 3].fill(50);
            }
        }
        unsharp(&mut edge, 0.5);
        let at = |x: usize| edge.data[(4 * 8 + x) * 3];
        assert!(at(3) < 50 && at(4) > 200, "edge {} {}", at(3), at(4));
        assert_eq!(at(0), 50);
    }
}
