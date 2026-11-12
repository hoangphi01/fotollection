// SPDX-License-Identifier: GPL-3.0-only
//! A look's effects (FR-079 *Tools*, FR-118, v0.14.1 session 13): the vignette round the
//! cropped photo, then film grain sized in millimetres on the print, both drawn after the
//! look's table, as Lightroom draws them after its colour.
//!
//! Methods read first (owner, 2026-11-09; RESULT §40): `darktable`'s grain (noise on lightness,
//! weaker in the shadows and highlights) and `RawTherapee`'s vignetting filter (on the crop,
//! roundness from a rounded rectangle to a circle), never their code. The grain differs from
//! both, and from Lightroom's, on purpose: theirs follows the image's pixels, so a smaller
//! export has smaller grain; here it follows the print, so a 10 × 15 and a 20 × 30 show the
//! same grain in millimetres.
//!
//! - **Grain:** value noise on a lattice whose cells are the grain's size in millimetres,
//!   three octaves weighted by the roughness, added to the three channels alike (so colours
//!   keep their hue) and weighted by `(4L(1 − L))^0.7`, strongest in the midtones, as film's
//!   grain shows. A cell smaller than a pixel averages away as it does on paper seen at that
//!   size. The same photo gets the same grain at every export: the lattice is hashed from a
//!   seed and the cell's index, whatever the threads or the tiles.
//! - **Vignette:** a mask from the frame's centre, `smoothstep` round a midpoint with a
//!   feather, its shape from a rounded rectangle to a circle; darkening multiplies in linear
//!   light (up to two stops at −100), keeping some of the highlights, lightening blends
//!   towards white.
//!
//! [`mask_and_noise`] gives the GPU preview (`gpu-look.ts`) the same mask and noise, so it
//! draws in Rust's order with Rust's numbers (ADR-0039).

use rayon::prelude::*;

use crate::image::RgbImage;

/// The grain's cell at size 0 and at 100, in millimetres on the print. An estimate until it
/// is calibrated against Lightroom's exports (`preset_bench`, FR-118).
const GRAIN_MM: (f32, f32) = (0.02, 0.12);
/// The grain's strength at 100, in lightness (0–1) per unit of noise.
const GRAIN_STRENGTH: f32 = 0.12;
/// The darkening at −100, in stops.
const VIGNETTE_STOPS: f32 = 2.0;
const OCTAVES: u32 = 3;

/// What a look's effects draw on one output, everything resolved: strengths scaled by the
/// look's amount, the grain's cell in this output's pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effects {
    /// Grain's strength, 0–1.
    pub grain: f32,
    /// The grain's cell in pixels of this output.
    pub grain_px: f32,
    /// 0 for even grain, 1 for rough.
    pub roughness: f32,
    pub seed: u32,
    /// −1 (dark corners) to 1 (light ones).
    pub vignette: f32,
    /// Where the vignette's fall starts, 0 near the centre to 1 near the corners.
    pub midpoint: f32,
    /// −1 a rounded rectangle, 0 an ellipse on the frame, 1 a circle.
    pub roundness: f32,
    /// 0 a hard edge to 1 a soft one.
    pub feather: f32,
    /// How much of the highlights a darkening keeps, 0–1.
    pub highlights: f32,
}

impl Effects {
    /// The effects of a look on a print at `px_per_mm`, `None` when they draw nothing.
    #[must_use]
    pub fn of(
        adjust: &crate::looks::Adjust,
        look_amount: u8,
        px_per_mm: f32,
        seed: u32,
    ) -> Option<Self> {
        let scale = f32::from(look_amount.min(100)) / 100.0;
        let grain = f32::from(adjust.grain.amount.min(100)) / 100.0 * scale;
        let vignette = f32::from(adjust.vignette.clamp(-100, 100)) / 100.0 * scale;
        if grain <= 0.0 && vignette == 0.0 {
            return None;
        }
        let shape = adjust.vignette_shape;
        let size = f32::from(adjust.grain.size.min(100)) / 100.0;
        Some(Self {
            grain,
            grain_px: (GRAIN_MM.0 + (GRAIN_MM.1 - GRAIN_MM.0) * size) * px_per_mm.max(0.01),
            roughness: f32::from(adjust.grain.roughness.min(100)) / 100.0,
            seed,
            vignette,
            midpoint: f32::from(shape.midpoint.min(100)) / 100.0,
            roundness: f32::from(shape.roundness.clamp(-100, 100)) / 100.0,
            feather: f32::from(shape.feather.min(100)) / 100.0,
            highlights: f32::from(shape.highlights.min(100)) / 100.0,
        })
    }
}

/// A 32-bit hash of a lattice point (a bit mixer: avalanche without a table).
fn hash(seed: u32, i: i32, j: i32, octave: u32) -> u32 {
    #[allow(clippy::cast_sign_loss)]
    let mut h = seed
        ^ (i as u32).wrapping_mul(0x8da6_b343)
        ^ (j as u32).wrapping_mul(0xd816_3841)
        ^ octave.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^ (h >> 16)
}

/// A lattice value, uniform with unit variance.
#[allow(clippy::cast_precision_loss)]
fn lattice(seed: u32, i: i32, j: i32, octave: u32) -> f32 {
    let u = (hash(seed, i, j, octave) >> 8) as f32 / 16_777_216.0;
    (u * 2.0 - 1.0) * 1.732_050_8
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// One octave of the grain's lattice along a row: its cell, its weight, and for each column the
/// cell it falls in and where in it (the same on every row, so computed once an image).
struct Octave {
    cell: f32,
    weight: f32,
    i: Vec<i32>,
    w: Vec<f32>,
    /// The lattice's values, hashed once into a tile at most `TILE` cells a side (a power of
    /// two) and read by index: about 6 hashes a pixel became one lookup.
    tile: Vec<f32>,
    mask: (i32, i32),
}

/// The largest side of an octave's tile, in cells: its pattern repeats after this many cells,
/// about 6 cm of the print at the coarsest grain, too fine and too far apart for an eye.
const TILE: u32 = 1024;

impl Octave {
    fn at(&self, i: i32, j: i32) -> f32 {
        #[allow(clippy::cast_sign_loss)]
        let k = ((j & self.mask.1) * (self.mask.0 + 1) + (i & self.mask.0)) as usize;
        self.tile[k]
    }
}

/// The grain's octaves for an output `width` × `height`. Octaves whose cell is under a pixel
/// average to white noise on each pixel, so they are one octave, their weights summed in
/// quadrature: one tile and one lookup a pixel for all of them.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn octaves(fx: &Effects, width: u32, height: u32) -> Vec<Octave> {
    let gain = 0.25 + 0.5 * fx.roughness;
    let norm = (0..OCTAVES)
        .map(|k| gain.powi(2 * i32::try_from(k).unwrap_or(0)))
        .sum::<f32>()
        .sqrt();
    let mut wanted: Vec<(u32, f32, f32)> = Vec::new();
    let mut fine = (OCTAVES, 0.0f32);
    for k in 0..OCTAVES {
        let cell = fx.grain_px / (1u32 << k) as f32;
        // A cell under a pixel averages out over it, as grain finer than the eye does.
        let weight = gain.powi(i32::try_from(k).unwrap_or(0)) * cell.min(1.0) / norm;
        if cell >= 1.0 {
            wanted.push((k, cell, weight));
        } else {
            fine = (fine.0.min(k), fine.1 + weight * weight);
        }
    }
    if fine.1 > 0.0 {
        wanted.push((fine.0, 0.5, fine.1.sqrt()));
    }
    wanted
        .into_iter()
        .filter(|&(_, _, weight)| weight >= 1e-4)
        .map(|(k, cell, weight)| {
            let (i, w) = (0..width)
                .map(|x| {
                    let q = (x as f32 + 0.5) / cell;
                    let i = q.floor();
                    (i as i32, smooth(q - i))
                })
                .unzip();
            // Cells across and down (pixels, for a cell under one), a power of two each, at
            // most TILE: a small preview hashes only what it reads.
            #[allow(clippy::cast_sign_loss)]
            let side = |n: u32| {
                let cells = (n as f32 / cell.max(1.0)) as u32 + 2;
                cells.next_power_of_two().min(TILE)
            };
            let (tw, th) = (side(width), side(height));
            #[allow(clippy::cast_possible_wrap)]
            let tile = (0..th as i32)
                .into_par_iter()
                .flat_map_iter(|j| (0..tw as i32).map(move |i| lattice(fx.seed, i, j, k)))
                .collect();
            #[allow(clippy::cast_possible_wrap)]
            let mask = (tw as i32 - 1, th as i32 - 1);
            Octave {
                cell,
                weight,
                i,
                w,
                tile,
                mask,
            }
        })
        .collect()
}

/// One row of grain noise: `out[x]` for every x of row `y`, about unit variance at a cell of
/// a pixel or more, fainter for cells smaller than a pixel. A row's lattice values are fetched
/// once per cell, not per pixel.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn noise_row(octaves: &[Octave], y: u32, out: &mut [f32]) {
    out.fill(0.0);
    for o in octaves {
        // A cell under a pixel: what averages over the pixel is white noise, one hash a pixel
        // instead of four a cell (the lattice's corners change at nearly every pixel).
        if o.cell < 1.0 {
            #[allow(clippy::cast_possible_wrap)]
            let j = y as i32;
            for (x, v) in out.iter_mut().enumerate() {
                #[allow(clippy::cast_possible_wrap)]
                let i = x as i32;
                *v += o.at(i, j) * o.weight;
            }
            continue;
        }
        let qy = (y as f32 + 0.5) / o.cell;
        let j = qy.floor();
        let fy = smooth(qy - j);
        let j = j as i32;
        let mut cached = i32::MIN;
        let (mut top, mut top1, mut bottom, mut bottom1) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for ((v, &i), &wx) in out.iter_mut().zip(&o.i).zip(&o.w) {
            if i != cached {
                cached = i;
                top = o.at(i, j);
                top1 = o.at(i + 1, j);
                bottom = o.at(i, j + 1);
                bottom1 = o.at(i + 1, j + 1);
            }
            let a = top + (top1 - top) * wx;
            let b = bottom + (bottom1 - bottom) * wx;
            *v += (a + (b - a) * fy) * o.weight;
        }
    }
}

/// Steps of the vignette's mask table, over `s = |u|^p + |v|^p`.
const MASK_STEPS: usize = 2048;

/// The vignette's mask on a `width` × `height` frame, 0 at the centre to 1 in the corners: the
/// columns' `|u|^p` once an image, the rows' `|v|^p` once a row, the mask from a table of
/// their sum, so no power is taken per pixel.
struct Shape {
    p: f32,
    half_h: f32,
    centre_h: f32,
    u: Vec<f32>,
    top: f32,
    table: Vec<f32>,
}

impl Shape {
    #[allow(clippy::cast_precision_loss)]
    fn new(fx: &Effects, width: u32, height: u32) -> Self {
        let (w, h) = (width as f32, height as f32);
        let short = w.min(h);
        // Roundness: towards a circle on the short side (+), towards a rounded rectangle (−).
        let r = fx.roundness.max(0.0);
        let half = (
            f32::midpoint(w, (short - w) * r),
            f32::midpoint(h, (short - h) * r),
        );
        let p = 2.0 + 8.0 * (-fx.roundness).max(0.0);
        let at = 0.3 + 0.9 * fx.midpoint;
        let soft = 0.05 + 0.95 * fx.feather;
        let (from, to) = (at - soft / 2.0, at + soft / 2.0);
        let u: Vec<f32> = (0..width)
            .map(|x| ((x as f32 + 0.5 - w / 2.0) / half.0).abs().powf(p))
            .collect();
        let edge = |n: f32, half: f32| ((n / 2.0) / half).abs().powf(p);
        let top = edge(w, half.0) + edge(h, half.1);
        let table = (0..=MASK_STEPS)
            .map(|k| {
                let d = (top * k as f32 / MASK_STEPS as f32).powf(1.0 / p);
                smooth(((d - from) / (to - from)).clamp(0.0, 1.0))
            })
            .collect();
        Self {
            p,
            half_h: half.1,
            centre_h: h / 2.0,
            u,
            top,
            table,
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn row(&self, y: u32, out: &mut [f32]) {
        let v = ((y as f32 + 0.5 - self.centre_h) / self.half_h)
            .abs()
            .powf(self.p);
        let scale = MASK_STEPS as f32 / self.top.max(1e-6);
        for (m, &u) in out.iter_mut().zip(&self.u) {
            let at = ((u + v) * scale).min(MASK_STEPS as f32);
            let k = at as usize;
            let next = self.table[(k + 1).min(MASK_STEPS)];
            *m = self.table[k] + (next - self.table[k]) * (at - k as f32);
        }
    }
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// The grain's weight by lightness, strongest in the midtones (darktable's mid-tones bias,
/// AV1's `f(Y)`).
fn grain_weight(l: f32) -> f32 {
    (4.0 * l * (1.0 - l)).max(0.0).powf(0.7)
}

/// One pixel's colour, 0–1, through the effects at mask `m` and noise `n`: the formula the
/// GPU preview repeats.
#[must_use]
pub fn shade(rgb: [f32; 3], fx: &Effects, m: f32, n: f32) -> [f32; 3] {
    let mut c = rgb;
    if fx.vignette != 0.0 && m > 0.0 {
        let t = fx.vignette * m;
        let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let keep = fx.highlights * smooth(((lum - 0.6) / 0.4).clamp(0.0, 1.0));
        for v in &mut c {
            let lin = srgb_to_linear(*v);
            let shaded = if t < 0.0 {
                lin * (VIGNETTE_STOPS * t).exp2()
            } else {
                lin + (1.0 - lin) * t
            };
            *v = linear_to_srgb(shaded + (lin - shaded) * keep);
        }
    }
    if fx.grain > 0.0 {
        let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let delta = fx.grain * GRAIN_STRENGTH * grain_weight(l) * n;
        for v in &mut c {
            *v += delta;
        }
    }
    c.map(|v| v.clamp(0.0, 1.0))
}

/// Draws the effects on `image` (sRGB, 8 bits a channel), rows in parallel: [`shade`]'s
/// formula with its powers in tables (sRGB to linear by the byte, linear to sRGB in 4096 steps,
/// the grain's weight in 1024), within a level of it.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn apply(image: &mut RgbImage, fx: &Effects) {
    const ENC_STEPS: usize = 4096;
    const WEIGHT_STEPS: usize = 1024;
    let (width, height) = (image.width, image.height);
    if width == 0 || height == 0 {
        return;
    }
    let vignette = fx.vignette != 0.0;
    let grain = fx.grain > 0.0;
    let shape = vignette.then(|| Shape::new(fx, width, height));
    let octaves = if grain {
        octaves(fx, width, height)
    } else {
        Vec::new()
    };
    let lin: Vec<f32> = (0..=255u8)
        .map(|v| srgb_to_linear(f32::from(v) / 255.0))
        .collect();
    let enc: Vec<f32> = (0..=ENC_STEPS)
        .map(|k| linear_to_srgb(k as f32 / ENC_STEPS as f32))
        .collect();
    // The darkening by the mask's value, so no power is taken per pixel.
    let dark: Vec<f32> = (0..=WEIGHT_STEPS)
        .map(|k| (VIGNETTE_STOPS * fx.vignette * k as f32 / WEIGHT_STEPS as f32).exp2())
        .collect();
    // Without highlights kept, a vignette's byte depends on the byte and the mask alone: one
    // table of 256 mask levels × 256 values, a lookup a channel.
    let shaded: Option<Vec<u8>> = (vignette && fx.highlights <= 0.0).then(|| {
        (0..256usize)
            .flat_map(|level| {
                let t = fx.vignette * level as f32 / 255.0;
                let factor = (VIGNETTE_STOPS * t).exp2();
                let lin = &lin;
                let enc = &enc;
                (0..256usize).map(move |b| {
                    let l = lin[b];
                    let out = if t < 0.0 {
                        l * factor
                    } else {
                        l + (1.0 - l) * t
                    };
                    (enc[(out.clamp(0.0, 1.0) * ENC_STEPS as f32).round() as usize] * 255.0).round()
                        as u8
                })
            })
            .collect()
    });
    // The grain's step by lightness byte, in levels (×255).
    let weight: Vec<f32> = (0..=255u8)
        .map(|l| 255.0 * fx.grain * GRAIN_STRENGTH * grain_weight(f32::from(l) / 255.0))
        .collect();
    image
        .data
        .par_chunks_mut(width as usize * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let mut mask = vec![0.0f32; width as usize];
            let mut noise = vec![0.0f32; width as usize];
            if let Some(shape) = &shape {
                shape.row(y as u32, &mut mask);
            }
            if grain {
                noise_row(&octaves, y as u32, &mut noise);
            }
            for ((px, &m), &n) in row.as_chunks_mut::<3>().0.iter_mut().zip(&mask).zip(&noise) {
                if let Some(table) = &shaded {
                    let row = (m * 255.0).round() as usize * 256;
                    for v in px.iter_mut() {
                        *v = table[row + usize::from(*v)];
                    }
                }
                // A vignette that keeps highlights: the full formula, in floats.
                if shaded.is_none() && m > 0.0 && vignette {
                    let t = fx.vignette * m;
                    let c = [px[0], px[1], px[2]].map(|v| f32::from(v) / 255.0);
                    let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                    let keep = fx.highlights * smooth(((lum - 0.6) / 0.4).clamp(0.0, 1.0));
                    let dark = dark[(m * WEIGHT_STEPS as f32).round() as usize];
                    for v in px.iter_mut() {
                        let l = lin[usize::from(*v)];
                        let shaded = if t < 0.0 { l * dark } else { l + (1.0 - l) * t };
                        let out = (shaded + (l - shaded) * keep).clamp(0.0, 1.0);
                        *v = (enc[(out * ENC_STEPS as f32).round() as usize] * 255.0).round() as u8;
                    }
                }
                // Grain on the bytes: the lightness in integers, its weight already ×255, the
                // same step added to the three channels.
                if n != 0.0 {
                    let l = (54 * u32::from(px[0])
                        + 183 * u32::from(px[1])
                        + 19 * u32::from(px[2])
                        + 128)
                        >> 8;
                    let step = (weight[l as usize] * n).round() as i16;
                    for v in px.iter_mut() {
                        *v = (i16::from(*v) + step).clamp(0, 255) as u8;
                    }
                }
            }
        });
}

/// The neutral texel of [`gpu_map`]: no vignette, no grain.
pub const NEUTRAL: [u8; 3] = [128, 128, 0];

/// The effects of `fx` on a `width` × `height` output as the GPU preview reads them, their
/// strengths already in (`gpu-look.ts`): red the vignette's `t = vignette × mask` as
/// `128 + 127 t`, green the grain's lightness step per unit of weight as `128 + 512 d`, blue
/// the highlights a darkening keeps. With it the shader repeats [`shade`] with no other input,
/// so a drag of any colour slider keeps the photo's grain and vignette.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn gpu_map(fx: &Effects, width: u32, height: u32) -> RgbImage {
    let (width, height) = (width.max(1), height.max(1));
    let shape = Shape::new(fx, width, height);
    let mut image = crate::render::filled(width, height, NEUTRAL);
    let keep = (fx.highlights * 255.0).round() as u8;
    let octaves = octaves(fx, width, height);
    image
        .data
        .par_chunks_mut(width as usize * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let mut mask = vec![0.0f32; width as usize];
            let mut noise = vec![0.0f32; width as usize];
            if fx.vignette != 0.0 {
                shape.row(y as u32, &mut mask);
            }
            if fx.grain > 0.0 {
                noise_row(&octaves, y as u32, &mut noise);
            }
            for (x, px) in row.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                let t = fx.vignette * mask[x];
                let d = fx.grain * GRAIN_STRENGTH * noise[x];
                px[0] = (128.0 + 127.0 * t).round().clamp(0.0, 255.0) as u8;
                px[1] = (128.0 + 512.0 * d).round().clamp(0.0, 255.0) as u8;
                px[2] = keep;
            }
        });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx() -> Effects {
        Effects {
            grain: 0.6,
            grain_px: 3.0,
            roughness: 0.5,
            seed: 7,
            vignette: 0.0,
            midpoint: 0.5,
            roundness: 0.0,
            feather: 0.5,
            highlights: 0.0,
        }
    }

    fn grey(width: u32, height: u32, v: u8) -> RgbImage {
        crate::render::filled(width, height, [v, v, v])
    }

    fn grain_at(px_per_mm: f32, size: u8) -> Effects {
        let adjust = crate::looks::Adjust {
            grain: crate::looks::Grain {
                amount: 80,
                size,
                roughness: 0,
            },
            ..crate::looks::Adjust::default()
        };
        let Some(fx) = Effects::of(&adjust, 100, px_per_mm, 11) else {
            panic!("grain draws");
        };
        fx
    }

    /// The lag, in pixels, where a row's grain stops resembling itself (autocorrelation 0.5).
    #[allow(clippy::cast_precision_loss)]
    fn grain_length(image: &RgbImage) -> f32 {
        let w = image.width as usize;
        let rows: Vec<Vec<f32>> = (0..image.height as usize)
            .step_by(7)
            .map(|y| {
                (0..w)
                    .map(|x| f32::from(image.data[(y * w + x) * 3]))
                    .collect()
            })
            .collect();
        let corr = |lag: usize| {
            let (mut num, mut den) = (0.0f64, 0.0f64);
            for r in &rows {
                let mean = r.iter().sum::<f32>() / r.len() as f32;
                for x in 0..w - lag {
                    num += f64::from((r[x] - mean) * (r[x + lag] - mean));
                    den += f64::from((r[x] - mean) * (r[x] - mean));
                }
            }
            num / den
        };
        let mut lag = 1;
        while lag < w / 4 && corr(lag) > 0.5 {
            lag += 1;
        }
        // Between the two lags that cross 0.5.
        let (a, b) = (corr(lag - 1), corr(lag));
        #[allow(clippy::cast_possible_truncation)]
        let frac = ((a - 0.5) / (a - b).max(1e-9)) as f32;
        (lag - 1) as f32 + frac
    }

    /// FR-079, session 13: grain is the same size in millimetres whatever the print's pixels a
    /// millimetre: a 10 × 15 and a 20 × 30 from the same photo show the same grain.
    #[test]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn fr_079_grain_is_the_same_millimetres_on_every_print_size() {
        let mut lengths = Vec::new();
        for px_per_mm in [40.0f32, 80.0] {
            let side = (25.0 * px_per_mm) as u32;
            let mut image = grey(side, side, 128);
            apply(&mut image, &grain_at(px_per_mm, 100));
            lengths.push(grain_length(&image) / px_per_mm);
        }
        let (a, b) = (lengths[0], lengths[1]);
        assert!((a - b).abs() / a.max(b) < 0.1, "{a} mm against {b} mm");
        // And a coarser setting gives coarser grain.
        let mut fine = grey(1000, 1000, 128);
        apply(&mut fine, &grain_at(40.0, 0));
        assert!(grain_length(&fine) / 40.0 < a * 0.5, "size moves the grain");
    }

    /// FR-079, session 13: the same photo gets the same grain at every export, whatever the
    /// number of threads.
    #[test]
    fn fr_079_grain_is_deterministic() {
        let fx = grain_at(30.0, 40);
        let mut a = grey(700, 500, 120);
        let mut b = a.clone();
        apply(&mut a, &fx);
        let pool =
            crate::pool::contained(Some(1), "grain-test").unwrap_or_else(|e| panic!("a pool: {e}"));
        pool.install(|| apply(&mut b, &fx));
        assert_eq!(a.data, b.data);
        let mut other = grey(700, 500, 120);
        apply(&mut other, &Effects { seed: 12, ..fx });
        assert_ne!(a.data, other.data, "another photo, another grain");
    }

    /// FR-079, session 13: grain is strongest in the midtones, spares pure black and white,
    /// and moves the three channels alike, so a colour keeps its hue.
    #[test]
    fn fr_079_grain_spares_the_extremes() {
        let fx = grain_at(30.0, 40);
        for v in [0u8, 255] {
            let mut image = grey(300, 200, v);
            apply(&mut image, &fx);
            assert!(image.data.iter().all(|&p| p == v), "{v} untouched");
        }
        let spread = |v: u8| {
            let mut image = grey(400, 300, v);
            apply(&mut image, &fx);
            let changed = image.data.iter().filter(|&&p| p != v).count();
            #[allow(clippy::cast_precision_loss)]
            let share = changed as f64 / image.data.len() as f64;
            share
        };
        assert!(spread(128) > spread(20) && spread(128) > spread(235));
        let mut colour = crate::render::filled(300, 200, [180, 110, 60]);
        apply(&mut colour, &fx);
        for px in colour.data.as_chunks::<3>().0 {
            let (r, g, b) = (i16::from(px[0]), i16::from(px[1]), i16::from(px[2]));
            assert!((r - g - 70).abs() <= 1 && (g - b - 50).abs() <= 1, "{px:?}");
        }
    }

    /// FR-079, session 13: the vignette sits on the cropped frame: darker corners than the
    /// centre at −100, the same in every corner, its edge moving with the midpoint, nothing
    /// at all at 0.
    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn fr_079_vignette_on_the_cropped_frame() {
        let vignette = |amount: i8, midpoint: u8| {
            let adjust = crate::looks::Adjust {
                vignette: amount,
                vignette_shape: crate::looks::VignetteShape {
                    midpoint,
                    ..crate::looks::VignetteShape::default()
                },
                ..crate::looks::Adjust::default()
            };
            Effects::of(&adjust, 100, 20.0, 1)
        };
        assert!(
            vignette(0, 50).is_none(),
            "nothing to draw at 0: the print is unchanged"
        );
        let Some(fx) = vignette(-100, 50) else {
            panic!("a vignette");
        };
        let mut image = grey(600, 400, 160);
        apply(&mut image, &fx);
        let at = |x: u32, y: u32| image.data[((y * 600 + x) * 3) as usize];
        assert_eq!(at(300, 200), 160, "the centre keeps its light");
        assert!(at(2, 2) < 100, "a corner two stops darker: {}", at(2, 2));
        assert_eq!(at(2, 2), at(597, 397));
        assert_eq!(at(2, 397), at(597, 2));
        let Some(wide) = vignette(-100, 100) else {
            panic!("a vignette");
        };
        let mut late = grey(600, 400, 160);
        apply(&mut late, &wide);
        let edge = |img: &RgbImage| img.data[((200 * 600 + 40) * 3) as usize];
        assert!(
            edge(&late) > edge(&image),
            "a later midpoint keeps more of the frame"
        );
    }

    /// The GPU's map, decoded as `gpu-look.ts` decodes it, gives the colours `shade` gives,
    /// within a level (ADR-0039: one formula, one set of numbers).
    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn fr_079_the_gpu_map_draws_what_rust_draws() {
        let adjust = crate::looks::Adjust {
            vignette: -60,
            grain: crate::looks::Grain {
                amount: 70,
                size: 50,
                roughness: 50,
            },
            vignette_shape: crate::looks::VignetteShape {
                highlights: 40,
                ..crate::looks::VignetteShape::default()
            },
            ..crate::looks::Adjust::default()
        };
        let Some(fx) = Effects::of(&adjust, 100, 25.0, 3) else {
            panic!("effects");
        };
        let (w, h) = (320u32, 200u32);
        let map = gpu_map(&fx, w, h);
        let shape = Shape::new(&fx, w, h);
        let oct = octaves(&fx, w, h);
        let (mut mask, mut noise) = (vec![0.0f32; w as usize], vec![0.0f32; w as usize]);
        for y in (0..h).step_by(13) {
            shape.row(y, &mut mask);
            noise_row(&oct, y, &mut noise);
            for x in (0..w).step_by(11) {
                let px = &map.data[((y * w + x) * 3) as usize..][..3];
                let t = (f32::from(px[0]) - 128.0) / 127.0;
                let d = (f32::from(px[1]) - 128.0) / 512.0;
                let keep = f32::from(px[2]) / 255.0;
                for base in [[0.2f32, 0.5, 0.7], [0.9, 0.85, 0.8], [0.4, 0.4, 0.4]] {
                    let rust = shade(base, &fx, mask[x as usize], noise[x as usize]);
                    // The shader's steps with the decoded numbers.
                    let mut c = base;
                    let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                    let k = keep * smooth(((lum - 0.6) / 0.4).clamp(0.0, 1.0));
                    for v in &mut c {
                        let l = srgb_to_linear(*v);
                        let s = if t < 0.0 {
                            l * (2.0 * t).exp2()
                        } else {
                            l + (1.0 - l) * t
                        };
                        *v = linear_to_srgb(s + (l - s) * k);
                    }
                    let l2 = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                    let gpu = c.map(|v| (v + d * grain_weight(l2)).clamp(0.0, 1.0));
                    for (a, b) in rust.iter().zip(gpu) {
                        assert!((a - b).abs() * 255.0 <= 1.0, "{rust:?} against {gpu:?}");
                    }
                }
            }
        }
    }

    /// `apply`, with its tables and its bytes, draws what `shade` (the GPU's formula) draws,
    /// within two levels.
    #[test]
    fn apply_draws_what_shade_draws() {
        for highlights in [0u8, 60] {
            let adjust = crate::looks::Adjust {
                vignette: -70,
                grain: crate::looks::Grain {
                    amount: 80,
                    size: 60,
                    roughness: 50,
                },
                vignette_shape: crate::looks::VignetteShape {
                    highlights,
                    ..crate::looks::VignetteShape::default()
                },
                ..crate::looks::Adjust::default()
            };
            let Some(fx) = Effects::of(&adjust, 100, 30.0, 5) else {
                panic!("effects");
            };
            let (w, h) = (240u32, 160u32);
            let mut image = crate::render::filled(w, h, [0, 0, 0]);
            for (k, px) in image.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let v = (k * 37 % 256) as u8;
                px.copy_from_slice(&[v, v.wrapping_add(40), v.wrapping_mul(3)]);
            }
            let before = image.clone();
            apply(&mut image, &fx);
            let shape = Shape::new(&fx, w, h);
            let oct = octaves(&fx, w, h);
            let (mut mask, mut noise) = (vec![0.0f32; w as usize], vec![0.0f32; w as usize]);
            for y in (0..h).step_by(7) {
                shape.row(y, &mut mask);
                noise_row(&oct, y, &mut noise);
                for x in (0..w).step_by(5) {
                    let at = ((y * w + x) * 3) as usize;
                    let rgb = [0, 1, 2].map(|c| f32::from(before.data[at + c]) / 255.0);
                    let want = shade(rgb, &fx, mask[x as usize], noise[x as usize]);
                    for (c, w) in want.iter().enumerate() {
                        let got = f32::from(image.data[at + c]);
                        assert!(
                            (got - w * 255.0).abs() <= 2.0,
                            "{got} against {}",
                            w * 255.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn noise_has_about_unit_spread_and_no_bias() {
        let mut row = vec![0.0f32; 4000];
        let (mut sum, mut sq, mut n) = (0.0f64, 0.0f64, 0.0f64);
        let oct = octaves(&fx(), 4000, 300);
        for y in 0..40 {
            noise_row(&oct, y * 7, &mut row);
            for v in &row {
                sum += f64::from(*v);
                sq += f64::from(*v) * f64::from(*v);
                n += 1.0;
            }
        }
        let mean = sum / n;
        let sd = (sq / n - mean * mean).sqrt();
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((0.4..1.2).contains(&sd), "sd {sd}");
    }
}
