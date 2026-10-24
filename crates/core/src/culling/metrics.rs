// SPDX-License-Identifier: GPL-3.0-only
//! Per-photo quality measures for best-shot suggestions (FR-041). One pass over a
//! greyscale copy of about 1024 px on the long edge. The numbers are only compared within a
//! group of near-identical shots, so they need to be consistent, not absolute.

use serde::{Deserialize, Serialize};

use crate::image::RgbImage;

/// Long edge of the greyscale image the metrics are computed on.
pub const ANALYSIS_EDGE: u32 = 1024;
const TILES_X: usize = 16;
const TILES_Y: usize = 12;

/// Luma, 0–255, row-major.
#[derive(Debug, Clone)]
pub struct Gray {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl Gray {
    /// Rec. 601 luma of an RGB image.
    pub fn from_rgb(image: &RgbImage) -> Self {
        let data = image
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .map(|&[r, g, b]| 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b))
            .collect();
        Self {
            width: image.width as usize,
            height: image.height as usize,
            data,
        }
    }

    fn at(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    /// Median of the per-tile Tenengrad energy: overall sharpness.
    pub sharpness: f32,
    /// 90th percentile of the tiles: the sharpest part, usually the subject.
    pub subject_sharpness: f32,
    /// Variance of the Laplacian over the whole image.
    pub laplacian_variance: f32,
    /// Structure-tensor coherence weighted by gradient energy, 0–1. Higher means edges point
    /// in one direction, as motion blur makes them.
    pub coherence: f32,
    /// Share of pixels at 250 or above.
    pub highlights: f32,
    /// Share of pixels at 5 or below.
    pub shadows: f32,
    /// log2(median luma / 118): 0 is well exposed, ±1 is a stop off.
    pub exposure_ev: f32,
    /// Standard deviation of luma.
    pub contrast: f32,
    /// Immerkær noise estimate (σ in luma units) over the flatter half of the image.
    pub noise: f32,
}

fn sobel(g: &Gray, x: usize, y: usize) -> (f32, f32) {
    let p = |dx: isize, dy: isize| g.at(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
    let gx = p(1, -1) + 2.0 * p(1, 0) + p(1, 1) - p(-1, -1) - 2.0 * p(-1, 0) - p(-1, 1);
    let gy = p(-1, 1) + 2.0 * p(0, 1) + p(1, 1) - p(-1, -1) - 2.0 * p(0, -1) - p(1, -1);
    (gx, gy)
}

fn percentile(values: &mut [f32], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let i = ((values.len() - 1) as f32 * p).round() as usize;
    values[i]
}

/// Gradient magnitudes are binned at this resolution for the noise threshold (Sobel
/// magnitudes on 8-bit luma stay below 1,443).
const MAGNITUDE_BINS: usize = 1448;

/// Index of the `p` quantile in a histogram holding `total` values.
fn histogram_quantile(histogram: &[u32], total: u64, p: f64) -> usize {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let target = ((total as f64 - 1.0) * p).round() as u64;
    let mut seen = 0u64;
    for (i, &n) in histogram.iter().enumerate() {
        seen += u64::from(n);
        if seen > target {
            return i;
        }
    }
    histogram.len() - 1
}

/// All metrics in one pass over the pixels; percentiles come from histograms, not sorts
/// (NFR-014).
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn measure(g: &Gray) -> Metrics {
    let (w, h) = (g.width, g.height);
    if w < 3 || h < 3 {
        return Metrics::default();
    }
    let tile_of_x: Vec<usize> = (0..w).map(|x| (x * TILES_X / w).min(TILES_X - 1)).collect();
    let mut tile_energy = [0f64; TILES_X * TILES_Y];
    let mut tile_count = [0u32; TILES_X * TILES_Y];
    let mut tile_j = [[0f64; 3]; TILES_X * TILES_Y];
    let (mut lap_sum, mut lap_sq) = (0f64, 0f64);
    // Per gradient-magnitude bin: how many pixels, and the sum of their Immerkær responses.
    let mut magnitude_count = vec![0u32; MAGNITUDE_BINS];
    let mut magnitude_noise = vec![0f64; MAGNITUDE_BINS];
    for y in 1..h - 1 {
        let ty = (y * TILES_Y / h).min(TILES_Y - 1);
        let up = &g.data[(y - 1) * w..y * w];
        let row = &g.data[y * w..(y + 1) * w];
        let down = &g.data[(y + 1) * w..(y + 2) * w];
        let rows = up
            .array_windows::<3>()
            .zip(row.array_windows::<3>())
            .zip(down.array_windows::<3>());
        for (((&[a, b, c], &[d, e, f]), &[gg, hh, ii]), &tx) in rows.zip(&tile_of_x[1..]) {
            // a b c
            // d e f
            // g h i
            let gx = c + 2.0 * f + ii - a - 2.0 * d - gg;
            let gy = gg + 2.0 * hh + ii - a - 2.0 * b - c;
            let t = ty * TILES_X + tx;
            let (xx, yy, xy) = (gx * gx, gy * gy, gx * gy);
            tile_energy[t] += f64::from(xx + yy);
            tile_count[t] += 1;
            tile_j[t][0] += f64::from(xx);
            tile_j[t][1] += f64::from(yy);
            tile_j[t][2] += f64::from(xy);
            let lap = f64::from(b + d + f + hh - 4.0 * e);
            lap_sum += lap;
            lap_sq += lap * lap;
            // Immerkær: the 3×3 kernel [1 −2 1; −2 4 −2; 1 −2 1] cancels image structure.
            let n = a + c + gg + ii - 2.0 * (b + d + f + hh) + 4.0 * e;
            let bin = ((xx + yy).sqrt() as usize).min(MAGNITUDE_BINS - 1);
            magnitude_count[bin] += 1;
            magnitude_noise[bin] += f64::from(n.abs());
        }
    }
    let count = ((w - 2) * (h - 2)) as f64;
    let laplacian_variance = (lap_sq / count - (lap_sum / count).powi(2)) as f32;

    let mut tiles: Vec<f32> = tile_energy
        .iter()
        .zip(&tile_count)
        .map(|(e, &n)| {
            if n > 0 {
                (e / f64::from(n)) as f32
            } else {
                0.0
            }
        })
        .collect();
    let sharpness = percentile(&mut tiles, 0.5);
    let subject_sharpness = percentile(&mut tiles, 0.9);

    // Coherence per tile, weighted by how much edge energy the tile has.
    let (mut coh_sum, mut weight) = (0f64, 0f64);
    for [jxx, jyy, jxy] in &tile_j {
        let trace = jxx + jyy;
        if trace > 0.0 {
            let coherence = ((jxx - jyy).powi(2) + 4.0 * jxy * jxy).sqrt() / trace;
            coh_sum += coherence * trace;
            weight += trace;
        }
    }
    let coherence = if weight > 0.0 {
        (coh_sum / weight) as f32
    } else {
        0.0
    };

    // Noise only in the flatter half of the image: edges and texture would read as noise, and
    // the sharper frame of a burst would always look noisier.
    let edge_bin = histogram_quantile(&magnitude_count, count as u64, 0.5);
    let flat: u64 = magnitude_count[..=edge_bin]
        .iter()
        .map(|&n| u64::from(n))
        .sum();
    let flat_sum: f64 = magnitude_noise[..=edge_bin].iter().sum();
    let noise = if flat == 0 {
        0.0
    } else {
        ((std::f64::consts::FRAC_PI_2).sqrt() * flat_sum / (6.0 * flat as f64)) as f32
    };

    let (highlights, shadows, exposure_ev, contrast) = tones(g);

    Metrics {
        sharpness,
        subject_sharpness,
        laplacian_variance,
        coherence,
        highlights,
        shadows,
        exposure_ev,
        contrast,
        noise,
    }
}

/// Highlight and shadow shares, exposure in EV from the median, and RMS contrast.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn tones(g: &Gray) -> (f32, f32, f32, f32) {
    let mut luma = [0u32; 256];
    let (mut sum, mut sum_sq) = (0f64, 0f64);
    for &v in &g.data {
        luma[(v as usize).min(255)] += 1;
        sum += f64::from(v);
        sum_sq += f64::from(v * v);
    }
    let total = g.data.len() as f64;
    let share = |range: std::ops::Range<usize>| {
        (luma[range].iter().map(|&n| u64::from(n)).sum::<u64>() as f64 / total) as f32
    };
    let mean = sum / total;
    let contrast = (sum_sq / total - mean * mean).max(0.0).sqrt() as f32;
    let median = (histogram_quantile(&luma, g.data.len() as u64, 0.5) as f32).max(1.0);
    (
        share(250..256),
        share(0..6),
        (median / 118.0).log2(),
        contrast,
    )
}

/// Mean Tenengrad energy inside a rectangle (face or eye regions).
#[allow(clippy::cast_precision_loss)]
pub fn region_sharpness(g: &Gray, x0: usize, y0: usize, x1: usize, y1: usize) -> f32 {
    let (x0, y0) = (x0.max(1), y0.max(1));
    let (x1, y1) = (x1.min(g.width - 1), y1.min(g.height - 1));
    if x1 <= x0 || y1 <= y0 {
        return 0.0;
    }
    let mut sum = 0f64;
    for y in y0..y1 {
        for x in x0..x1 {
            let (gx, gy) = sobel(g, x, y);
            sum += f64::from(gx * gx + gy * gy);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    let mean = (sum / ((x1 - x0) * (y1 - y0)) as f64) as f32;
    mean
}

/// 64-bit difference hash of a 9×8 greyscale reduction, for grouping near-identical shots.
pub fn dhash(g: &Gray) -> u64 {
    let (w, h) = (g.width.max(1), g.height.max(1));
    // Box-average into 9×8 cells.
    let mut cells = [0f32; 72];
    for (i, cell) in cells.iter_mut().enumerate() {
        let (cx, cy) = (i % 9, i / 9);
        let (x0, x1) = (cx * w / 9, ((cx + 1) * w / 9).max(cx * w / 9 + 1).min(w));
        let (y0, y1) = (cy * h / 8, ((cy + 1) * h / 8).max(cy * h / 8 + 1).min(h));
        let mut sum = 0.0;
        let mut n = 0.0;
        for y in y0..y1 {
            for x in x0..x1 {
                sum += g.at(x, y);
                n += 1.0;
            }
        }
        *cell = if n > 0.0 { sum / n } else { 0.0 };
    }
    let mut hash = 0u64;
    for y in 0..8 {
        for x in 0..8 {
            hash = (hash << 1) | u64::from(cells[y * 9 + x] > cells[y * 9 + x + 1]);
        }
    }
    hash
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A deterministic test scene: soft gradients plus crisp squares and lines.
    pub fn scene(w: usize, h: usize, seed: u32) -> Gray {
        let mut data = vec![0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                #[allow(clippy::cast_precision_loss)]
                let base = 60.0 + 80.0 * (x as f32 / w as f32) + 30.0 * (y as f32 / h as f32);
                #[allow(clippy::cast_precision_loss)]
                let square = ((x / 24 + y / 24 + seed as usize) % 2) as f32 * 70.0;
                let line = if (x + y) % 37 < 2 { 50.0 } else { 0.0 };
                data[y * w + x] = (base + square + line).min(255.0);
            }
        }
        Gray {
            width: w,
            height: h,
            data,
        }
    }

    /// Separable box blur, horizontal and/or vertical (vertical-only = motion blur).
    pub fn blur(g: &Gray, radius: usize, horizontal: bool, vertical: bool) -> Gray {
        let mut out = g.clone();
        if horizontal {
            let src = out.clone();
            for y in 0..g.height {
                for x in 0..g.width {
                    let (a, b) = (x.saturating_sub(radius), (x + radius).min(g.width - 1));
                    #[allow(clippy::cast_precision_loss)]
                    let mean = (a..=b).map(|i| src.at(i, y)).sum::<f32>() / (b - a + 1) as f32;
                    out.data[y * g.width + x] = mean;
                }
            }
        }
        if vertical {
            let src = out.clone();
            for y in 0..g.height {
                for x in 0..g.width {
                    let (a, b) = (y.saturating_sub(radius), (y + radius).min(g.height - 1));
                    #[allow(clippy::cast_precision_loss)]
                    let mean = (a..=b).map(|i| src.at(x, i)).sum::<f32>() / (b - a + 1) as f32;
                    out.data[y * g.width + x] = mean;
                }
            }
        }
        out
    }

    #[test]
    fn fr_041_blur_lowers_sharpness() {
        let sharp = measure(&scene(320, 240, 0));
        let soft = measure(&blur(&scene(320, 240, 0), 3, true, true));
        assert!(
            sharp.sharpness > 2.0 * soft.sharpness,
            "{sharp:?} vs {soft:?}"
        );
        assert!(sharp.laplacian_variance > 2.0 * soft.laplacian_variance);
    }

    #[test]
    fn fr_041_motion_blur_raises_coherence() {
        let still = measure(&scene(320, 240, 0));
        let shaken = measure(&blur(&scene(320, 240, 0), 5, false, true));
        assert!(
            shaken.coherence > still.coherence + 0.1,
            "{still:?} vs {shaken:?}"
        );
    }

    #[test]
    fn fr_041_exposure_and_clipping() {
        let normal = measure(&scene(200, 150, 0));
        let mut bright = scene(200, 150, 0);
        bright
            .data
            .iter_mut()
            .for_each(|v| *v = (*v * 2.2).min(255.0));
        let bright = measure(&bright);
        let mut dark = scene(200, 150, 0);
        dark.data.iter_mut().for_each(|v| *v *= 0.15);
        let dark = measure(&dark);
        assert!(bright.highlights > normal.highlights + 0.2);
        assert!(
            bright.exposure_ev > 0.5 && dark.exposure_ev < -1.5,
            "{bright:?} {dark:?}"
        );
        assert!(normal.exposure_ev.abs() < 0.6, "{normal:?}");
    }

    #[test]
    fn fr_041_noise_is_measured_apart_from_edges() {
        let clean = measure(&scene(240, 180, 0));
        let mut noisy = scene(240, 180, 0);
        let mut state = 12345u32;
        for v in &mut noisy.data {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            #[allow(clippy::cast_precision_loss)]
            let n = ((state >> 16) % 41) as f32 - 20.0;
            *v = (*v + n).clamp(0.0, 255.0);
        }
        let noisy = measure(&noisy);
        assert!(noisy.noise > clean.noise + 3.0, "{clean:?} vs {noisy:?}");
    }

    #[test]
    fn fr_041_dhash_matches_similar_and_separates_different() {
        let a = dhash(&scene(320, 240, 0));
        let a_soft = dhash(&blur(&scene(320, 240, 0), 2, true, true));
        // The same scene seen from the other side: a different picture.
        let mut mirrored = scene(320, 240, 0);
        mirrored
            .data
            .as_chunks_mut::<320>()
            .0
            .iter_mut()
            .for_each(|row| row.reverse());
        let b = dhash(&mirrored);
        let limit = crate::culling::group::MAX_HASH_DISTANCE;
        assert!((a ^ a_soft).count_ones() <= limit);
        assert!((a ^ b).count_ones() > limit, "{}", (a ^ b).count_ones());
    }
}
