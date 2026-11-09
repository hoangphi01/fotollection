// SPDX-License-Identifier: GPL-3.0-only
//! Dehaze (FR-115): the veil of haze taken out of a photo, or added, by the dark channel
//! prior (He, Sun and Tang, 2009). In a clear photo most patches hold a pixel dark in one
//! channel; haze lifts that dark channel towards the air's colour, so how far it is lifted
//! tells how much veil lies over each place.
//!
//! The veil is estimated on a copy at most [`ESTIMATE_EDGE`] across, smoothed by a guided
//! filter whose coefficients are then laid over the full photo (He's fast guided filter,
//! 2015), so a 1600 px preview and a 24 MP export get the same result (`fr_115_*`). Every
//! radius is a fraction of the photo's size. Runs before the look's table, so the GPU
//! preview, which draws the table over a base that already has it, keeps Rust's order.

use rayon::prelude::*;

use crate::image::RgbImage;

/// The long edge of the copy the veil is estimated on.
pub const ESTIMATE_EDGE: u32 = 512;
/// The dark channel's patch, as a fraction of the copy's long edge (He's 15 px at 600).
const PATCH: f32 = 0.012;
/// The guided filter's radius, a multiple of the patch, and its smoothing.
const GUIDE: f32 = 4.0;
const EPS: f32 = 1e-3;
/// The share of the veil taken out at +100: a little haze kept so depth still reads.
const OMEGA: f32 = 0.95;
/// The thinnest veil divided by, so the far distance does not turn to noise.
const T_MIN: f32 = 0.1;
/// The share of the brightest dark-channel pixels that give the air's colour.
const AIR_SHARE: f32 = 0.001;
/// At −100, the share of the air's colour laid over the photo.
const HAZE_MAX: f32 = 0.6;

/// Dehaze `image` by `amount` (−100…100): + takes haze out, − adds it, 0 leaves every byte.
pub fn apply(image: &mut RgbImage, amount: i8) {
    if amount == 0 || image.width == 0 || image.height == 0 {
        return;
    }
    let (px, w, h) = shrink(image, ESTIMATE_EDGE);
    let Some(veil) = Veil::estimate(&px, w, h) else {
        return;
    };
    let strength = f32::from(amount.unsigned_abs().min(100)) / 100.0;
    if amount > 0 {
        // Takes the veil out: J = (I − A) / max(t, t₀) + A, t moved from 1 (none) towards
        // the estimate by the amount.
        let air = veil.air;
        veil.over(image, |p, t| {
            let t = (1.0 - strength * (1.0 - t)).max(T_MIN);
            [0, 1, 2].map(|c| to_byte((p[c] - air[c]) / t + air[c]))
        });
    } else {
        // Lays a veil on: a milky grey as bright as the air or brighter, as a real haze
        // (the air of a clear photo is only its brightest place, often a coloured one);
        // even a clear place gets half of it, the far distance all.
        let milk = luma(veil.air).max(0.8);
        veil.over(image, |p, t| {
            let k = HAZE_MAX * strength * (0.5 + 0.5 * (1.0 - t));
            [0, 1, 2].map(|c| to_byte(p[c] * (1.0 - k) + milk * k))
        });
    }
}

/// What the estimate gives: the air's colour and, on the small copy's grid, the guided
/// filter's coefficients of the transmission against the light of each pixel.
struct Veil {
    air: [f32; 3],
    width: usize,
    height: usize,
    /// `t ≈ a · luma + b`, as the full photo's own luma gives it.
    a: Vec<f32>,
    b: Vec<f32>,
}

fn luma(p: [f32; 3]) -> f32 {
    0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]
}

/// A byte as 0–1, read from a table.
static UNIT: std::sync::LazyLock<[f32; 256]> = std::sync::LazyLock::new(|| {
    std::array::from_fn(|i| f32::from(u8::try_from(i).unwrap_or(255)) / 255.0)
});

#[cfg(test)]
fn pixels(image: &RgbImage) -> Vec<[f32; 3]> {
    image
        .data
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| p.map(|v| UNIT[usize::from(v)]))
        .collect()
}

/// The photo made at most `edge` across by the mean of square blocks, in 0–1, on rayon:
/// one read of the photo whatever its size.
fn shrink(image: &RgbImage, edge: u32) -> (Vec<[f32; 3]>, usize, usize) {
    let (fw, fh) = (image.width as usize, image.height as usize);
    let k = fw.max(fh).div_ceil(edge as usize).max(1);
    let (w, h) = (fw.div_ceil(k), fh.div_ceil(k));
    let unit = &*UNIT;
    let px = (0..h)
        .into_par_iter()
        .flat_map_iter(|by| {
            let rows = by * k..((by + 1) * k).min(fh);
            (0..w).map(move |bx| {
                let cols = bx * k..((bx + 1) * k).min(fw);
                let mut sum = [0.0f32; 3];
                for y in rows.clone() {
                    let line = &image.data[(y * fw + cols.start) * 3..(y * fw + cols.end) * 3];
                    for p in line.as_chunks::<3>().0 {
                        for c in 0..3 {
                            sum[c] += unit[usize::from(p[c])];
                        }
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                let n = (rows.len() * cols.len()) as f32;
                sum.map(|v| v / n)
            })
        })
        .collect();
    (px, w, h)
}

impl Veil {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn estimate(px: &[[f32; 3]], w: usize, h: usize) -> Option<Self> {
        if px.is_empty() {
            return None;
        }
        let r = ((w.max(h) as f32 * PATCH).round() as usize).max(1);
        let dark = min_filter(
            &px.iter()
                .map(|p| p[0].min(p[1]).min(p[2]))
                .collect::<Vec<_>>(),
            w,
            h,
            r,
        );
        // The air: the mean colour of the pixels whose dark channel is highest.
        let n = ((px.len() as f32 * AIR_SHARE).ceil() as usize).clamp(1, px.len());
        let mut order: Vec<usize> = (0..px.len()).collect();
        order.select_nth_unstable_by(n - 1, |&i, &j| dark[j].total_cmp(&dark[i]));
        let mut air = [0.0f32; 3];
        for &i in &order[..n] {
            for c in 0..3 {
                air[c] += px[i][c];
            }
        }
        let air = air.map(|v| (v / n as f32).max(0.05));
        // The transmission before smoothing: 1 − ω · the dark channel of the photo over its air.
        let normalised: Vec<f32> = px
            .iter()
            .map(|p| (p[0] / air[0]).min(p[1] / air[1]).min(p[2] / air[2]))
            .collect();
        let t: Vec<f32> = min_filter(&normalised, w, h, r)
            .iter()
            .map(|d| 1.0 - OMEGA * d)
            .collect();
        // The guided filter, guided by the light, keeps the veil's edges on the photo's.
        let guide: Vec<f32> = px.iter().map(|&p| luma(p)).collect();
        let rg = ((r as f32 * GUIDE).round() as usize).max(1);
        let mean_i = box_mean(&guide, w, h, rg);
        let mean_t = box_mean(&t, w, h, rg);
        let light_veil = box_mean(
            &guide.iter().zip(&t).map(|(i, t)| i * t).collect::<Vec<_>>(),
            w,
            h,
            rg,
        );
        let light_light = box_mean(&guide.iter().map(|i| i * i).collect::<Vec<_>>(), w, h, rg);
        let (a, b): (Vec<f32>, Vec<f32>) = (0..px.len())
            .map(|k| {
                let var = light_light[k] - mean_i[k] * mean_i[k];
                let cov = light_veil[k] - mean_i[k] * mean_t[k];
                let a = cov / (var + EPS);
                (a, mean_t[k] - a * mean_i[k])
            })
            .unzip();
        Some(Self {
            air,
            width: w,
            height: h,
            a: box_mean(&a, w, h, rg),
            b: box_mean(&b, w, h, rg),
        })
    }

    /// Where a full-size row or column `i` of `full` falls on the small grid of `small`:
    /// the two samples around it and the weight of the second.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn between(i: usize, full: usize, small: usize) -> (usize, usize, f32) {
        let s =
            ((i as f32 + 0.5) * small as f32 / full as f32 - 0.5).clamp(0.0, (small - 1) as f32);
        let i0 = s.floor() as usize;
        (i0, (i0 + 1).min(small - 1), s - i0 as f32)
    }

    /// Each pixel of `image` through `f(pixel in 0–1, its transmission)`, on rayon: the
    /// coefficients read between the small grid's samples, a row at a time.
    fn over(&self, image: &mut RgbImage, f: impl Fn([f32; 3], f32) -> [u8; 3] + Sync) {
        let (fw, fh) = (image.width as usize, image.height as usize);
        let columns: Vec<_> = (0..fw).map(|x| Self::between(x, fw, self.width)).collect();
        let unit = &*UNIT;
        image
            .data
            .par_chunks_mut(fw * 3)
            .enumerate()
            .for_each(|(y, line)| {
                let (y0, y1, fy) = Self::between(y, fh, self.height);
                let row = |v: &[f32]| -> Vec<f32> {
                    (0..self.width)
                        .map(|x| v[y0 * self.width + x] * (1.0 - fy) + v[y1 * self.width + x] * fy)
                        .collect()
                };
                let (a, b) = (row(&self.a), row(&self.b));
                for (px, &(x0, x1, fx)) in line.as_chunks_mut::<3>().0.iter_mut().zip(&columns) {
                    let p = px.map(|v| unit[usize::from(v)]);
                    let t = (a[x0] + (a[x1] - a[x0]) * fx) * luma(p) + b[x0] + (b[x1] - b[x0]) * fx;
                    *px = f(p, t.clamp(0.0, 1.0));
                }
            });
    }
}

fn to_byte(v: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let b = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    b
}

/// `pass` on each row of `v`, then on each column, in parallel on rayon.
fn separable(v: &[f32], w: usize, h: usize, pass: impl Fn(&[f32]) -> Vec<f32> + Sync) -> Vec<f32> {
    let rows: Vec<f32> = v.par_chunks(w).flat_map_iter(&pass).collect();
    let rows = &rows;
    let transposed: Vec<f32> = (0..w)
        .into_par_iter()
        .flat_map_iter(|x| (0..h).map(move |y| rows[y * w + x]))
        .collect();
    let columns: Vec<f32> = transposed.par_chunks(h).flat_map_iter(&pass).collect();
    let columns = &columns;
    (0..h)
        .into_par_iter()
        .flat_map_iter(|y| (0..w).map(move |x| columns[x * h + y]))
        .collect()
}

/// The minimum over a square of radius `r`: a running minimum with a queue, O(n) whatever `r`.
fn min_filter(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    separable(v, w, h, |line| running_min(line, r))
}

fn running_min(line: &[f32], r: usize) -> Vec<f32> {
    let n = line.len();
    let mut out = Vec::with_capacity(n);
    let mut queue = std::collections::VecDeque::with_capacity(2 * r + 1);
    let mut next = 0;
    for i in 0..n {
        let end = (i + r).min(n - 1);
        while next <= end {
            while queue.back().is_some_and(|&j: &usize| line[j] >= line[next]) {
                queue.pop_back();
            }
            queue.push_back(next);
            next += 1;
        }
        while queue.front().is_some_and(|&j| j + r < i) {
            queue.pop_front();
        }
        out.push(queue.front().map_or(line[i], |&j| line[j]));
    }
    out
}

/// The mean over a square of radius `r`, cut at the edges.
fn box_mean(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    separable(v, w, h, |line| running_mean(line, r))
}

#[allow(clippy::cast_precision_loss)]
fn running_mean(line: &[f32], r: usize) -> Vec<f32> {
    let n = line.len();
    let mut prefix = Vec::with_capacity(n + 1);
    prefix.push(0.0f64);
    for &x in line {
        prefix.push(prefix[prefix.len() - 1] + f64::from(x));
    }
    (0..n)
        .map(|i| {
            let (lo, hi) = (i.saturating_sub(r), (i + r + 1).min(n));
            #[allow(clippy::cast_possible_truncation)]
            let m = ((prefix[hi] - prefix[lo]) / (hi - lo) as f64) as f32;
            m
        })
        .collect()
}

/// A clear scene for the tests and `measure_dehaze`: saturated tiles with shadows, as the
/// prior expects of a clear photo, and the depth of each place (0 near, 1 far, rising up
/// the frame), at any size, the same scene.
#[doc(hidden)]
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn clear_scene(width: u32, height: u32) -> (RgbImage, Vec<f32>) {
    const COLOURS: [[f32; 3]; 6] = [
        [0.75, 0.2, 0.1],
        [0.15, 0.55, 0.2],
        [0.1, 0.25, 0.7],
        [0.8, 0.65, 0.1],
        [0.5, 0.1, 0.55],
        [0.1, 0.5, 0.6],
    ];
    let mut data = Vec::with_capacity(width as usize * height as usize * 3);
    let mut depth = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        for x in 0..width {
            let (u, v) = (x as f32 / width as f32, y as f32 / height as f32);
            // The top is sky, far enough that its veil is all air.
            if v < SKY {
                data.extend([0.55, 0.7, 0.92].map(to_byte));
                depth.push(4.0);
                continue;
            }
            let tile = ((u * 8.0) as usize + (v * 6.0) as usize * 3) % COLOURS.len();
            // A soft shade across each tile, so each patch holds a darker part.
            let shade = 0.55 + 0.45 * ((u * 8.0).fract() * std::f32::consts::PI).sin();
            data.extend(COLOURS[tile].map(|c| to_byte(c * shade)));
            depth.push(1.0 - (v - SKY) / (1.0 - SKY));
        }
    }
    (
        RgbImage {
            width,
            height,
            data,
        },
        depth,
    )
}

/// The share of a [`clear_scene`]'s height that is sky, at the top.
#[doc(hidden)]
pub const SKY: f32 = 0.15;

/// The rows of `image` below the sky of a [`clear_scene`]: the part whose truth a veil
/// leaves to be found again.
#[doc(hidden)]
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn below_sky(image: &RgbImage) -> RgbImage {
    let top = (image.height as f32 * SKY).ceil() as u32 + 1;
    let row = image.width as usize * 3;
    RgbImage {
        width: image.width,
        height: image.height - top,
        data: image.data[top as usize * row..].to_vec(),
    }
}

/// `scene` behind a veil of air `air`: I = J·t + A(1 − t), t = e^(−β·depth).
#[doc(hidden)]
#[must_use]
pub fn hazed(scene: &RgbImage, depth: &[f32], air: [f32; 3], beta: f32) -> RgbImage {
    let mut out = scene.clone();
    for (px, d) in out.data.as_chunks_mut::<3>().0.iter_mut().zip(depth) {
        let t = (-beta * d).exp();
        *px = [0, 1, 2].map(|c| to_byte(f32::from(px[c]) / 255.0 * t + air[c] * (1.0 - t)));
    }
    out
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)]
mod tests {
    use super::*;
    use crate::raw_finish::mean_de2000;

    const AIR: [f32; 3] = [0.85, 0.87, 0.9];

    /// FR-115: at 0 nothing changes, byte for byte.
    #[test]
    fn fr_115_dehaze_at_zero_changes_nothing() {
        let (scene, depth) = clear_scene(300, 200);
        let mut image = hazed(&scene, &depth, AIR, 1.2);
        let before = image.clone();
        apply(&mut image, 0);
        assert_eq!(image.data, before.data);
    }

    /// FR-115: a photo behind a known veil comes back closer to the clear scene, and more
    /// so at +100 than at +50 (the truth is known: the scene before its veil).
    #[test]
    fn fr_115_dehaze_takes_out_a_known_veil() -> Result<(), String> {
        let (scene, depth) = clear_scene(600, 400);
        let veiled = hazed(&scene, &depth, AIR, 1.2);
        let truth = below_sky(&scene);
        let start = mean_de2000(&below_sky(&veiled), &truth).ok_or("sizes")?;
        let mut half = veiled.clone();
        apply(&mut half, 50);
        let mut full = veiled.clone();
        apply(&mut full, 100);
        let half = mean_de2000(&below_sky(&half), &truth).ok_or("sizes")?;
        let full = mean_de2000(&below_sky(&full), &truth).ok_or("sizes")?;
        assert!(
            full < start * 0.5,
            "the veil mostly gone: {start:.2} → {full:.2}"
        );
        assert!(
            full < half && half < start,
            "{start:.2} → {half:.2} → {full:.2}"
        );
        Ok(())
    }

    /// FR-115: − adds a veil: the dark channel rises, as haze lifts it, and the contrast falls.
    #[test]
    fn fr_115_negative_dehaze_adds_haze() {
        let (scene, _) = clear_scene(300, 200);
        let mut image = scene.clone();
        apply(&mut image, -60);
        let stats = |i: &RgbImage| {
            let px = pixels(&below_sky(i));
            let n = px.len() as f32;
            let dark = px.iter().map(|p| p[0].min(p[1]).min(p[2])).sum::<f32>() / n;
            let mean = px.iter().map(|&p| luma(p)).sum::<f32>() / n;
            let spread = px.iter().map(|&p| (luma(p) - mean).powi(2)).sum::<f32>() / n;
            (dark, spread)
        };
        let ((dark0, spread0), (dark1, spread1)) = (stats(&scene), stats(&image));
        assert!(dark1 > dark0 + 0.1, "dark channel {dark0:.3} → {dark1:.3}");
        assert!(
            spread1 < spread0 * 0.75,
            "contrast {spread0:.4} → {spread1:.4}"
        );
    }

    /// FR-115: the preview and the export agree: the large photo's result, made small,
    /// matches the small photo's own.
    #[test]
    fn fr_115_dehaze_is_the_same_at_any_size() -> Result<(), String> {
        let (small, small_depth) = clear_scene(800, 533);
        let (large, large_depth) = clear_scene(3200, 2133);
        let mut small = hazed(&small, &small_depth, AIR, 1.2);
        let mut large = hazed(&large, &large_depth, AIR, 1.2);
        apply(&mut small, 80);
        apply(&mut large, 80);
        let large = crate::render::fit_long_edge(&large, 800).map_err(|e| e.to_string())?;
        let gap = mean_de2000(&small, &large).ok_or("sizes")?;
        assert!(gap < 1.5, "preview and export {gap:.2} apart");
        Ok(())
    }

    #[test]
    fn the_filters_match_their_definition() {
        let v = [5.0, 1.0, 4.0, 3.0, 9.0, 2.0];
        assert_eq!(running_min(&v, 1), vec![1.0, 1.0, 1.0, 3.0, 2.0, 2.0]);
        assert_eq!(running_mean(&[3.0, 6.0, 9.0], 1), vec![4.5, 6.0, 7.5]);
    }
}
