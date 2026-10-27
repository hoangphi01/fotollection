// SPDX-License-Identifier: GPL-3.0-only
//! The tone of a developed photo. The colour model gives scene-linear sRGB; a print needs a
//! tone curve. The default curve is the camera's own: the brightness of the develop is
//! matched to the brightness of the camera's embedded JPEG, and that one curve scales the
//! three channels together, so hue and saturation stay those of the colour model rather
//! than being bent channel by channel. Without an embedded image, an automatic exposure and
//! a gentle S curve take its place.

use rayon::prelude::*;

/// Scene-linear sRGB, row by row.
#[derive(Debug, Clone)]
pub struct Linear {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<f32>,
}

/// An 8-bit sRGB picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

/// The camera's own rendering of the shot (its embedded JPEG), decoded by the caller.
#[derive(Debug, Clone, Copy)]
pub struct Reference<'a> {
    pub width: usize,
    pub height: usize,
    pub rgb: &'a [u8],
}

const BINS: usize = 4096;
/// Linear luminance mapped by the curve: up to two stops over white, for highlights.
const RANGE: f32 = 4.0;
const MAX_SAMPLES: usize = 1 << 20;

fn luminance(p: &[f32]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

fn srgb_to_linear(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Steps of the linear → sRGB table: fine enough that no 8-bit level is skipped in shadows.
const ENCODE_STEPS: usize = 1 << 16;

/// Linear (0–1) to 8-bit sRGB, one value per step.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn encode_table() -> Vec<u8> {
    (0..=ENCODE_STEPS)
        .map(|i| (linear_to_srgb(i as f32 / ENCODE_STEPS as f32) * 255.0 + 0.5) as u8)
        .collect()
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn bin(y: f32) -> usize {
    ((y / RANGE).clamp(0.0, 1.0) * (BINS - 1) as f32) as usize
}

#[allow(clippy::cast_precision_loss)]
fn cdf(hist: &[u64]) -> Vec<f64> {
    let total = hist.iter().sum::<u64>().max(1) as f64;
    let mut sum = 0u64;
    hist.iter()
        .map(|&n| {
            sum += n;
            sum as f64 / total
        })
        .collect()
}

/// A tone curve: output linear luminance for each input bin.
pub struct Curve(Vec<f32>);

/// The camera's look: the lens shading it corrected, its tone curve, and a 3×3 matrix on
/// display-linear sRGB for its colour rendering (saturation and hue shifts), with rows
/// summing to one so grey stays grey.
pub struct Look {
    pub shading: Shading,
    pub curve: Curve,
    pub matrix: [[f32; 3]; 3],
}

/// A radial brightness gain `exp(a·r² + b·r⁴)`, `r` from the centre (0) to a corner (1):
/// what the camera did to correct its lens's vignetting in its own JPEG.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Shading {
    pub a: f32,
    pub b: f32,
}

/// Steps of the gain table over r² from 0 (centre) to 1 (corner).
const SHADING_STEPS: usize = 1024;

impl Shading {
    /// `r²` of pixel `(x, y)`, normalised by the half diagonal so a corner is 1.
    #[allow(clippy::cast_precision_loss)]
    fn r2(x: usize, y: usize, width: usize, height: usize) -> f32 {
        let (dx, dy) = (
            (x as f32 + 0.5) / width as f32 - 0.5,
            (y as f32 + 0.5) / height as f32 - 0.5,
        );
        let aspect = width as f32 / height.max(1) as f32;
        (dx * dx * aspect * aspect + dy * dy) / (0.25 * (aspect * aspect + 1.0))
    }

    fn at_r2(self, r2: f32) -> f32 {
        (self.a * r2 + self.b * r2 * r2).exp().clamp(0.25, 4.0)
    }

    fn gain(self, x: usize, y: usize, width: usize, height: usize) -> f32 {
        if self.a == 0.0 && self.b == 0.0 {
            return 1.0;
        }
        self.at_r2(Self::r2(x, y, width, height))
    }

    /// The gain for every step of r², so the per-pixel cost is a lookup.
    #[allow(clippy::cast_precision_loss)]
    fn table(self) -> Vec<f32> {
        (0..=SHADING_STEPS)
            .map(|i| self.at_r2(i as f32 / SHADING_STEPS as f32))
            .collect()
    }
}

/// The grid of block means both pictures are compared on: coarse enough to forgive small
/// shifts between the develop and the camera's JPEG (lens correction, a different crop).
const GRID: (usize, usize) = (96, 64);

fn block_means(
    width: usize,
    height: usize,
    at: impl Fn(usize, usize) -> [f32; 3],
) -> Vec<[f32; 3]> {
    let (gw, gh) = GRID;
    let mut out = Vec::with_capacity(gw * gh);
    for by in 0..gh {
        for bx in 0..gw {
            let (x0, x1) = (
                bx * width / gw,
                ((bx + 1) * width / gw).max(bx * width / gw + 1),
            );
            let (y0, y1) = (
                by * height / gh,
                ((by + 1) * height / gh).max(by * height / gh + 1),
            );
            let mut sum = [0f32; 3];
            let mut n = 0f32;
            // A few samples per block are enough for a mean.
            let (sx, sy) = (((x1 - x0) / 8).max(1), ((y1 - y0) / 8).max(1));
            for y in (y0..y1.min(height)).step_by(sy) {
                for x in (x0..x1.min(width)).step_by(sx) {
                    let p = at(x, y);
                    for c in 0..3 {
                        sum[c] += p[c];
                    }
                    n += 1.0;
                }
            }
            out.push(sum.map(|v| v / n.max(1.0)));
        }
    }
    out
}

fn solve3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let inv = crate::color::inverse(&a)?;
    Some(crate::color::apply(&inv, b))
}

/// The camera's look from its embedded JPEG: the lens shading it corrected, its tone curve,
/// then the colour matrix that best maps our tone-mapped colours to the JPEG's.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn match_look(linear: &Linear, reference: &Reference<'_>) -> Option<Look> {
    let lut: Vec<f32> = (0..=255u8).map(srgb_to_linear).collect();
    let theirs = block_means(reference.width, reference.height, |x, y| {
        let i = (y * reference.width + x) * 3;
        let p = &reference.rgb[i..i + 3];
        [
            lut[usize::from(p[0])],
            lut[usize::from(p[1])],
            lut[usize::from(p[2])],
        ]
    });
    let ours_with = |shading: Shading, curve: &Curve| {
        block_means(linear.width, linear.height, |x, y| {
            let i = (y * linear.width + x) * 3;
            let p = &linear.rgb[i..i + 3];
            let g = shading.gain(x, y, linear.width, linear.height);
            let yv = (luminance(p) * g).max(1e-6);
            let k = curve.0[bin(yv)] / yv * g;
            [p[0] * k, p[1] * k, p[2] * k]
        })
    };
    // Shading and curve depend on each other: three rounds, each fitting what is left.
    let mut shading = Shading::default();
    let mut curve = match_reference_shaded(linear, reference, shading)?;
    let (gw, gh) = GRID;
    for _ in 0..3 {
        let ours = ours_with(shading, &curve);
        let (mut ata, mut atb, mut used) = ([[0f64; 3]; 3], [0f64; 3], 0usize);
        for (k, (o, t)) in ours.iter().zip(&theirs).enumerate() {
            let (yo, yt) = (luminance(o), luminance(t));
            if !(0.02..0.85).contains(&yo) || !(0.02..0.85).contains(&yt) {
                continue;
            }
            let (bx, by) = (k % gw, k / gw);
            let (dx, dy) = (
                (bx as f64 + 0.5) / gw as f64 - 0.5,
                (by as f64 + 0.5) / gh as f64 - 0.5,
            );
            let aspect = linear.width as f64 / linear.height.max(1) as f64;
            let r2 = (dx * dx * aspect * aspect + dy * dy) / (0.25 * (aspect * aspect + 1.0));
            let v = [1.0, r2, r2 * r2];
            let l = (f64::from(yt) / f64::from(yo)).ln();
            for i in 0..3 {
                for j in 0..3 {
                    ata[i][j] += v[i] * v[j];
                }
                atb[i] += v[i] * l;
            }
            used += 1;
        }
        if used < 300 {
            break;
        }
        let Some([_, a, b]) = solve3(ata, atb) else {
            break;
        };
        // The display-linear ratio is about the scene gain raised to the curve's slope; a
        // display gamma near 1/2.2 makes the scene correction a little larger. Half of what is
        // left each round keeps it stable.
        shading.a += (a as f32) * 0.9;
        shading.b += (b as f32) * 0.9;
        shading.a = shading.a.clamp(-1.5, 1.5);
        shading.b = shading.b.clamp(-1.5, 1.5);
        curve = match_reference_shaded(linear, reference, shading)?;
    }
    let matrix = look_matrix(&ours_with(shading, &curve), &theirs);
    Some(Look {
        shading,
        curve,
        matrix,
    })
}

/// The 3×3 matrix, rows summing to 1 so neutrals stay neutral, that best maps our block
/// colours to the JPEG's; the identity when too few blocks are usable.
#[allow(clippy::cast_possible_truncation)]
fn look_matrix(ours: &[[f32; 3]], theirs: &[[f32; 3]]) -> [[f32; 3]; 3] {
    // Normal equations, one output channel at a time, over blocks neither clipped nor black.
    let mut ata = [[0f64; 3]; 3];
    let mut atb = [[0f64; 3]; 3];
    let mut used = 0usize;
    for (o, t) in ours.iter().zip(theirs) {
        let fine = |p: &[f32; 3]| p.iter().all(|&v| v > 0.003 && v < 0.92);
        if !fine(o) || !fine(t) {
            continue;
        }
        used += 1;
        for i in 0..3 {
            for j in 0..3 {
                ata[i][j] += f64::from(o[i]) * f64::from(o[j]);
            }
            for c in 0..3 {
                atb[c][i] += f64::from(o[i]) * f64::from(t[c]);
            }
        }
    }
    let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut matrix = identity;
    if used >= 200 {
        for c in 0..3 {
            let row = solve3(ata, atb[c]);
            let sum: f64 = row.map_or(0.0, |r| r.iter().sum());
            match row {
                Some(row) if sum.abs() > 1e-6 => matrix[c] = row.map(|v| (v / sum) as f32),
                _ => return identity,
            }
        }
    }
    matrix
}

/// The curve that gives `linear` the brightness distribution of `reference`.
pub fn match_reference(linear: &Linear, reference: &Reference<'_>) -> Option<Curve> {
    match_reference_shaded(linear, reference, Shading::default())
}

#[allow(clippy::cast_precision_loss)]
fn match_reference_shaded(
    linear: &Linear,
    reference: &Reference<'_>,
    shading: Shading,
) -> Option<Curve> {
    if reference.width.min(reference.height) < 120
        || reference.rgb.len() < reference.width * reference.height * 3
    {
        return None;
    }
    let step = (linear.rgb.len() / 3 / MAX_SAMPLES).max(1);
    let mut ours = vec![0u64; BINS];
    for (i, p) in linear
        .rgb
        .as_chunks::<3>()
        .0
        .iter()
        .enumerate()
        .step_by(step)
    {
        let g = shading.gain(
            i % linear.width,
            i / linear.width,
            linear.width,
            linear.height,
        );
        ours[bin(luminance(p) * g)] += 1;
    }
    let lut: Vec<f32> = (0..=255u8).map(srgb_to_linear).collect();
    let mut theirs = vec![0u64; BINS];
    let rstep = (reference.width * reference.height / MAX_SAMPLES).max(1);
    for p in reference.rgb.as_chunks::<3>().0.iter().step_by(rstep) {
        let y = 0.2126 * lut[usize::from(p[0])]
            + 0.7152 * lut[usize::from(p[1])]
            + 0.0722 * lut[usize::from(p[2])];
        theirs[bin(y)] += 1;
    }
    let (from, to) = (cdf(&ours), cdf(&theirs));
    let mut j = 0usize;
    let curve = from
        .iter()
        .map(|&share| {
            while j < BINS - 1 && to[j] < share {
                j += 1;
            }
            j as f32 / (BINS - 1) as f32 * RANGE
        })
        .collect();
    Some(Curve(curve))
}

/// Without a reference: the brightest 0.5 % reach white, then a gentle S curve.
#[allow(clippy::cast_precision_loss)]
pub fn default_curve(linear: &Linear) -> Curve {
    let step = (linear.rgb.len() / 3 / MAX_SAMPLES).max(1);
    let mut hist = vec![0u64; BINS];
    for p in linear.rgb.as_chunks::<3>().0.iter().step_by(step) {
        hist[bin(luminance(p))] += 1;
    }
    let share = cdf(&hist);
    let white_bin = share
        .iter()
        .position(|&s| s >= 0.995)
        .unwrap_or(BINS - 1)
        .max(1);
    let white = white_bin as f32 / (BINS - 1) as f32 * RANGE;
    let curve = (0..BINS)
        .map(|b| {
            let x = (b as f32 / (BINS - 1) as f32 * RANGE / white).min(1.0);
            // In sRGB-encoded terms, a mild S; back to linear for the table.
            let e = linear_to_srgb(x);
            let s = e * e * (3.0 - 2.0 * e);
            let out = 0.75 * e + 0.25 * s;
            if out <= 0.040_45 {
                out / 12.92
            } else {
                ((out + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    Curve(curve)
}

/// Applies `curve` to the brightness of each pixel, scaling its three channels together,
/// and encodes 8-bit sRGB. A channel pushed past white pulls the pixel toward white instead
/// of clipping alone, so bright colours do not shift hue.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn finish(linear: &Linear, curve: &Curve) -> Picture {
    finish_with(
        linear,
        Shading::default(),
        curve,
        &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    )
}

/// [`finish`] with the camera's look ([`match_look`]).
pub fn finish_look(linear: &Linear, look: &Look) -> Picture {
    finish_with(linear, look.shading, &look.curve, &look.matrix)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn finish_with(
    linear: &Linear,
    shading: Shading,
    curve: &Curve,
    matrix: &[[f32; 3]; 3],
) -> Picture {
    let mut rgb = vec![0u8; linear.rgb.len()];
    let w = linear.width.max(1);
    let table = shading.table();
    let flat = shading == Shading::default();
    let encode = encode_table();
    rgb.par_chunks_mut(3 * w)
        .zip(linear.rgb.par_chunks(3 * w))
        .enumerate()
        .for_each(|(row, (out, src))| {
            for (x, (o, p)) in out
                .as_chunks_mut::<3>()
                .0
                .iter_mut()
                .zip(src.as_chunks::<3>().0)
                .enumerate()
            {
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                let g = if flat {
                    1.0
                } else {
                    let r2 = Shading::r2(x, row, linear.width, linear.height).min(1.0);
                    table[(r2 * SHADING_STEPS as f32) as usize]
                };
                let y = (luminance(p) * g).max(1e-6);
                let target = curve.0[bin(y)];
                let k = target / y * g;
                let t = [p[0] * k, p[1] * k, p[2] * k];
                let mut c = [0, 1, 2].map(|i| {
                    (matrix[i][0] * t[0] + matrix[i][1] * t[1] + matrix[i][2] * t[2]).max(0.0)
                });
                let peak = c[0].max(c[1]).max(c[2]);
                if peak > 1.0 {
                    // Keep the luminance, move toward white until the brightest channel fits.
                    let t = target.min(1.0);
                    let s = ((1.0 - t) / (peak - t).max(1e-6)).clamp(0.0, 1.0);
                    c = c.map(|v| t + (v - t) * s);
                }
                for (dst, v) in o.iter_mut().zip(c) {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        *dst = encode[(v.clamp(0.0, 1.0) * ENCODE_STEPS as f32) as usize];
                    }
                }
            }
        });
    Picture {
        width: linear.width,
        height: linear.height,
        rgb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn grey_stays_grey_and_the_curve_follows_the_reference() {
        // A dark grey ramp, and a reference with the same ramp brighter.
        let n = 300 * 300;
        let linear = Linear {
            width: 300,
            height: 300,
            rgb: (0..n)
                .flat_map(|i| [i as f32 / n as f32 * 0.25; 3])
                .collect(),
        };
        let reference_rgb: Vec<u8> = (0..n)
            .flat_map(|i| {
                let v = (linear_to_srgb(i as f32 / n as f32) * 255.0) as u8;
                [v; 3]
            })
            .collect();
        let reference = Reference {
            width: 300,
            height: 300,
            rgb: &reference_rgb,
        };
        let Some(curve) = match_reference(&linear, &reference) else {
            panic!("a curve");
        };
        let out = finish(&linear, &curve);
        let last = &out.rgb[out.rgb.len() - 3..];
        assert!(last[0] > 240, "{last:?}");
        assert!(
            out.rgb
                .as_chunks::<3>()
                .0
                .iter()
                .all(|p| p[0] == p[1] && p[1] == p[2])
        );
    }
}
