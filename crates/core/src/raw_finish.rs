// SPDX-License-Identifier: GPL-3.0-only
//! Finishing a developed camera RAW (ADR-0029). A developer such as rawler gives the
//! sensor's colours in sRGB, but without the camera's exposure, tone curve or lens shading,
//! so the picture looks dark and flat beside the camera's own JPEG.
//!
//! Every RAW file carries the camera's own rendering of the same shot (the embedded JPEG).
//! Matching each channel's distribution to it gives three curves that bring the develop to
//! the camera's look, at full resolution and 16-bit precision, with no colour table from
//! anyone else. A file without an embedded image gets an automatic exposure and a gentle
//! S curve instead. This is our own code, so it stays when our own decoder replaces rawler.

use rayon::prelude::*;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::image::RgbImage;

/// The 16-bit input is looked up in this many steps (16 levels each).
const BINS: usize = 4096;
const SHIFT: u32 = 4;
/// At most this many pixels are sampled from the develop to build its histograms.
const MAX_SAMPLES: usize = 1 << 20;
/// A reference smaller than this on its short side says too little about the look.
const MIN_REFERENCE_EDGE: u32 = 120;

/// One lookup table per channel: a 16-bit value `v` becomes `lut[c][v >> 4]`.
pub struct Curves([Vec<u8>; 3]);

fn histograms_u16(rgb: &[u16]) -> [Vec<u64>; 3] {
    let pixels = rgb.len() / 3;
    let step = (pixels / MAX_SAMPLES).max(1);
    let mut hist = [vec![0u64; BINS], vec![0u64; BINS], vec![0u64; BINS]];
    for px in rgb.as_chunks::<3>().0.iter().step_by(step) {
        for (c, h) in hist.iter_mut().enumerate() {
            h[usize::from(px[c] >> SHIFT)] += 1;
        }
    }
    hist
}

fn histograms_u8(rgb: &[u8]) -> [Vec<u64>; 3] {
    let mut hist = [vec![0u64; 256], vec![0u64; 256], vec![0u64; 256]];
    for px in rgb.as_chunks::<3>().0 {
        for (c, h) in hist.iter_mut().enumerate() {
            h[usize::from(px[c])] += 1;
        }
    }
    hist
}

/// Cumulative share of the pixels at or below each bin, 0–1.
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

/// Curves that give `developed` (16-bit RGB) the distribution of `reference` (the camera's
/// embedded JPEG), channel by channel. `None` when the reference is too small to trust.
pub fn match_reference(developed: &[u16], reference: &RgbImage) -> Option<Curves> {
    if reference.width.min(reference.height) < MIN_REFERENCE_EDGE || developed.len() < 3 {
        return None;
    }
    let source = histograms_u16(developed);
    let target = histograms_u8(&reference.data);
    let curves = std::array::from_fn(|c| {
        let (from, to) = (cdf(&source[c]), cdf(&target[c]));
        let mut level = 0usize;
        from.iter()
            .map(|&share| {
                // The darkest reference level that holds at least the same share: monotonic.
                while level < 255 && to[level] < share {
                    level += 1;
                }
                u8::try_from(level).unwrap_or(u8::MAX)
            })
            .collect()
    });
    Some(Curves(curves))
}

/// For a file with no embedded image: exposure set so the brightest 0.5 % reaches 95 %,
/// then a gentle S curve, the same for the three channels.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn default_curves(developed: &[u16]) -> Curves {
    let hist = histograms_u16(developed);
    let luma: Vec<u64> = (0..BINS)
        .map(|b| hist[0][b] + 2 * hist[1][b] + hist[2][b])
        .collect();
    let share = cdf(&luma);
    let white = share
        .iter()
        .position(|&s| s >= 0.995)
        .unwrap_or(BINS - 1)
        .max(1);
    let gain = 0.95 * (BINS - 1) as f64 / white as f64;
    let curve: Vec<u8> = (0..BINS)
        .map(|b| {
            let x = (b as f64 / (BINS - 1) as f64 * gain).min(1.0);
            let s = x * x * (3.0 - 2.0 * x);
            ((0.7 * x + 0.3 * s) * 255.0).round().clamp(0.0, 255.0) as u8
        })
        .collect();
    Curves([curve.clone(), curve.clone(), curve])
}

/// Applies the curves to the whole develop, in parallel: 16-bit RGB in, 8-bit RGB out.
pub fn apply(developed: &[u16], width: u32, height: u32, curves: &Curves) -> RgbImage {
    let mut data = vec![0u8; developed.len()];
    data.par_chunks_mut(3 * 4096)
        .zip(developed.par_chunks(3 * 4096))
        .for_each(|(out, src)| {
            for (o, s) in out
                .as_chunks_mut::<3>()
                .0
                .iter_mut()
                .zip(src.as_chunks::<3>().0)
            {
                for c in 0..3 {
                    o[c] = curves.0[c][usize::from(s[c] >> SHIFT)];
                }
            }
        });
    RgbImage {
        width,
        height,
        data,
    }
}

/// How a camera RAW photo is finished (ADR-0040, FR-072). Absent from a photo's edits means
/// [`RawLook::Neutral`]; a project from before v0.11.0 opens with [`RawLook::Camera`], so its
/// prints keep their colours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RawLook {
    /// The sensor's colours through the camera's matrix, the lens's shading corrected, the
    /// camera's exposure, a gentle curve.
    #[default]
    Neutral,
    /// Fitted to the camera's own JPEG inside the file: its contrast and colours.
    Camera,
}

/// One 8-bit sRGB pixel in CIELAB (D65); also `highlights_bench`'s (FR-112).
pub fn lab(p: [u8; 3]) -> [f32; 3] {
    let lin = |v: u8| {
        let c = f32::from(v) / 255.0;
        if c <= 0.040_45 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = p.map(lin);
    let xyz = [
        0.4124 * r + 0.3576 * g + 0.1805 * b,
        0.2126 * r + 0.7152 * g + 0.0722 * b,
        0.0193 * r + 0.1192 * g + 0.9505 * b,
    ];
    let f = |t: f32| {
        if t > 0.008_856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(xyz[0] / 0.950_47), f(xyz[1]), f(xyz[2] / 1.088_83));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIEDE2000 between two CIELAB colours (Sharma, Wu and Dalal's formulation).
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn de2000(a: [f32; 3], b: [f32; 3]) -> f32 {
    use std::f32::consts::PI;
    let (l1, a1, b1) = (a[0], a[1], a[2]);
    let (l2, a2, b2) = (b[0], b[1], b[2]);
    let c_mean = f32::midpoint(a1.hypot(b1), a2.hypot(b2));
    let g = 0.5 * (1.0 - (c_mean.powi(7) / (c_mean.powi(7) + 25f32.powi(7))).sqrt());
    let (a1p, a2p) = (a1 * (1.0 + g), a2 * (1.0 + g));
    let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
    let hue = |b: f32, a: f32| {
        let h = b.atan2(a);
        if h < 0.0 { h + 2.0 * PI } else { h }
    };
    let (h1p, h2p) = (hue(b1, a1p), hue(b2, a2p));
    let dl = l2 - l1;
    let dc = c2p - c1p;
    let mut dh = h2p - h1p;
    if c1p * c2p == 0.0 {
        dh = 0.0;
    } else if dh > PI {
        dh -= 2.0 * PI;
    } else if dh < -PI {
        dh += 2.0 * PI;
    }
    let dhh = 2.0 * (c1p * c2p).sqrt() * (dh / 2.0).sin();
    let lm = f32::midpoint(l1, l2);
    let cm = f32::midpoint(c1p, c2p);
    let hm = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= PI {
        f32::midpoint(h1p, h2p)
    } else if h1p + h2p < 2.0 * PI {
        0.5 * (h1p + h2p + 2.0 * PI)
    } else {
        0.5 * (h1p + h2p - 2.0 * PI)
    };
    let t = 1.0 - 0.17 * (hm - PI / 6.0).cos()
        + 0.24 * (2.0 * hm).cos()
        + 0.32 * (3.0 * hm + PI / 30.0).cos()
        - 0.20 * (4.0 * hm - 63.0 * PI / 180.0).cos();
    let d_theta = (30.0 * PI / 180.0) * (-(((hm * 180.0 / PI) - 275.0) / 25.0).powi(2)).exp();
    let rc = 2.0 * (cm.powi(7) / (cm.powi(7) + 25f32.powi(7))).sqrt();
    let sl = 1.0 + 0.015 * (lm - 50.0).powi(2) / (20.0 + (lm - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cm;
    let sh = 1.0 + 0.015 * cm * t;
    let rt = -(2.0 * d_theta).sin() * rc;
    ((dl / sl).powi(2) + (dc / sc).powi(2) + (dhh / sh).powi(2) + rt * (dc / sc) * (dhh / sh))
        .max(0.0)
        .sqrt()
}

/// ΔE2000 between two pictures of one size, pixel by pixel: the mean, the 95th percentile and
/// the mean in the shadows, mid-tones and highlights of `reference` (CIELAB L < 33, < 66,
/// above), for measurements against a reference render (`preset_bench`, FR-089).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeStats {
    pub mean: f32,
    pub p95: f32,
    pub zones: [f32; 3],
}

/// [`DeStats`] of `ours` against `reference`; `None` when their sizes differ.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn de2000_stats(ours: &RgbImage, reference: &RgbImage) -> Option<DeStats> {
    if (ours.width, ours.height) != (reference.width, reference.height) {
        return None;
    }
    let pairs: Vec<(f32, usize)> = ours
        .data
        .as_chunks::<3>()
        .0
        .par_iter()
        .zip(reference.data.as_chunks::<3>().0)
        .map(|(a, b)| {
            let lb = lab(*b);
            let zone = if lb[0] < 33.0 {
                0
            } else if lb[0] < 66.0 {
                1
            } else {
                2
            };
            (de2000(lab(*a), lb), zone)
        })
        .collect();
    let mut all: Vec<f32> = pairs.iter().map(|p| p.0).collect();
    let mean = all.iter().map(|&v| f64::from(v)).sum::<f64>() / all.len().max(1) as f64;
    all.sort_by(f32::total_cmp);
    let p95 = all.get(all.len() * 95 / 100).copied().unwrap_or(0.0);
    let mut zones = [0f32; 3];
    for (z, zone) in zones.iter_mut().enumerate() {
        let (sum, n) = pairs
            .iter()
            .filter(|p| p.1 == z)
            .fold((0f64, 0usize), |(s, n), p| (s + f64::from(p.0), n + 1));
        *zone = if n == 0 { 0.0 } else { (sum / n as f64) as f32 };
    }
    Some(DeStats {
        mean: mean as f32,
        p95,
        zones,
    })
}

/// Mean ΔE2000 between two pictures of one size, pixel by pixel (for measurements such as
/// `measure_raw_tiles`); `None` when their sizes differ.
pub fn mean_de2000(a: &RgbImage, b: &RgbImage) -> Option<f32> {
    ((a.width, a.height) == (b.width, b.height))
        .then(|| error_against(a.width as usize, a.height as usize, &a.data, b))
}

/// Mean ΔE2000, pixel by pixel, between a small rendering and the camera's JPEG averaged
/// down to the same size: per pixel, not per block, since a fit can match block averages and
/// still miss the colours in them (RESULT §27).
#[allow(clippy::cast_precision_loss)]
fn error_against(width: usize, height: usize, rgb: &[u8], reference: &RgbImage) -> f32 {
    let (rw, rh) = (reference.width as usize, reference.height as usize);
    let total: f64 = (0..height)
        .into_par_iter()
        .map(|y| {
            let (y0, y1) = (
                y * rh / height,
                ((y + 1) * rh / height).max(y * rh / height + 1).min(rh),
            );
            let mut sum = 0f64;
            for x in 0..width {
                let (x0, x1) = (
                    x * rw / width,
                    ((x + 1) * rw / width).max(x * rw / width + 1).min(rw),
                );
                let mut acc = [0u32; 3];
                let mut n = 0u32;
                for ry in y0..y1 {
                    for rx in x0..x1 {
                        let j = (ry * rw + rx) * 3;
                        for (c, sum) in acc.iter_mut().enumerate() {
                            *sum += u32::from(reference.data[j + c]);
                        }
                        n += 1;
                    }
                }
                let theirs = acc.map(|v| u8::try_from(v / n.max(1)).unwrap_or(u8::MAX));
                let i = (y * width + x) * 3;
                sum += f64::from(de2000(lab([rgb[i], rgb[i + 1], rgb[i + 2]]), lab(theirs)));
            }
            sum
        })
        .sum();
    #[allow(clippy::cast_possible_truncation)]
    let mean = (total / (width * height).max(1) as f64) as f32;
    mean
}

/// Scene-linear sRGB to 16-bit sRGB-encoded values, as the three-curve fit reads them.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn encode16(rgb: &[f32]) -> Vec<u16> {
    const STEPS: usize = 1 << 16;
    let table: Vec<u16> = (0..=STEPS)
        .map(|i| {
            let c = i as f32 / STEPS as f32;
            let e = if c <= 0.003_130_8 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            (e * 65535.0).round() as u16
        })
        .collect();
    rgb.par_iter()
        .map(|&v| table[(v.clamp(0.0, 1.0) * STEPS as f32) as usize])
        .collect()
}

/// A box-averaged copy of `linear`, at most `max` pixels across, for comparing fits cheaply.
fn shrink(linear: &fotollection_rawcore::Linear, max: usize) -> fotollection_rawcore::Linear {
    let f = linear.width.div_ceil(max).max(1);
    let (w, h) = (linear.width / f, linear.height / f);
    let mut rgb = vec![0f32; w * h * 3];
    rgb.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let mut sum = [0f32; 3];
            for dy in 0..f {
                for dx in 0..f {
                    let i = ((y * f + dy) * linear.width + x * f + dx) * 3;
                    for (c, v) in sum.iter_mut().enumerate() {
                        *v += linear.rgb[i + c];
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = (f * f) as f32;
            for c in 0..3 {
                row[x * 3 + c] = sum[c] / n;
            }
        }
    });
    fotollection_rawcore::Linear {
        width: w,
        height: h,
        rgb,
    }
}

fn picture(p: fotollection_rawcore::Picture) -> Option<RgbImage> {
    RgbImage::from_raw(
        u32::try_from(p.width).ok()?,
        u32::try_from(p.height).ok()?,
        p.rgb,
    )
}

/// A finished RAW, with the camera fit taken and both fits' errors against the camera's
/// JPEG (`[curve and matrix, three curves]`), for the measurements (RESULT §27).
pub struct Finished {
    pub image: RgbImage,
    pub fit: Option<CameraFit>,
    pub scores: [Option<f32>; 2],
}

/// Which camera fit a develop takes (RESULT §27).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraFit {
    /// One tone curve, a colour matrix and the lens shading (`rawcore::match_look`).
    CurveAndMatrix,
    /// One curve per channel (`match_reference`).
    ThreeCurves,
}

/// The largest copy a fit is made on. The preview and the export fit the same copy, so what
/// prints is what was shown, and a slider's step re-finishes only this copy (FR-088).
pub const FIT_EDGE: usize = 2048;

/// The copy the two camera fits are scored on: at 480 px they tied on close calls the
/// full-size measurement told apart (the A7 III, RESULT §27).
const SCORE_EDGE: usize = 1024;

/// How a develop is finished, fitted once.
pub enum Fit {
    /// A curve on luminance, a colour matrix and the lens shading: neutral, or the camera's
    /// look by one curve and a matrix.
    Look(fotollection_rawcore::Look),
    /// The camera's look by one curve per channel.
    Curves(Curves),
    /// No usable JPEG: an automatic exposure and a gentle curve.
    Auto(fotollection_rawcore::tone::Curve),
    /// Already toned by the system's codec: encoded as it is.
    AsIs,
}

/// Fits `small` (at most [`FIT_EDGE`] across) as `look` asks. `toned` is a develop the
/// system's codec already finished (its neutral is its own rendering).
pub fn fit(
    small: &fotollection_rawcore::Linear,
    reference: Option<&RgbImage>,
    look: RawLook,
    toned: bool,
) -> (Fit, Option<CameraFit>, [Option<f32>; 2]) {
    let auto = || {
        if toned {
            Fit::AsIs
        } else {
            Fit::Auto(fotollection_rawcore::default_curve(small))
        }
    };
    let Some(reference) = reference.filter(|r| r.width.min(r.height) >= MIN_REFERENCE_EDGE) else {
        return (auto(), None, [None, None]);
    };
    let r = fotollection_rawcore::Reference {
        width: reference.width as usize,
        height: reference.height as usize,
        rgb: &reference.data,
    };
    match look {
        RawLook::Neutral if toned => (Fit::AsIs, None, [None, None]),
        RawLook::Neutral => match fotollection_rawcore::neutral_look(small, &r) {
            Some(n) => (Fit::Look(n), None, [None, None]),
            None => (auto(), None, [None, None]),
        },
        RawLook::Camera => {
            let tiny = shrink(small, SCORE_EDGE);
            let fitted = if toned {
                None
            } else {
                fotollection_rawcore::match_look(small, &r)
            };
            let curves = match_reference(&encode16(&small.rgb), reference);
            let score_a = fitted.as_ref().map(|l| {
                let p = fotollection_rawcore::finish_look(&tiny, l);
                error_against(p.width, p.height, &p.rgb, reference)
            });
            let score_b = curves.as_ref().and_then(|c| {
                let (tw, th) = (
                    u32::try_from(tiny.width).ok()?,
                    u32::try_from(tiny.height).ok()?,
                );
                let p = apply(&encode16(&tiny.rgb), tw, th, c);
                Some(error_against(tiny.width, tiny.height, &p.data, reference))
            });
            let scores = [score_a, score_b];
            match (fitted, curves, score_a, score_b) {
                (_, Some(c), Some(a), Some(b)) if b < a => {
                    (Fit::Curves(c), Some(CameraFit::ThreeCurves), scores)
                }
                (Some(l), ..) => (Fit::Look(l), Some(CameraFit::CurveAndMatrix), scores),
                (None, Some(c), ..) => (Fit::Curves(c), Some(CameraFit::ThreeCurves), scores),
                _ => (auto(), None, scores),
            }
        }
    }
}

/// Renders a (possibly adjusted) linear develop with a fit, 8-bit sRGB.
pub fn render(linear: &fotollection_rawcore::Linear, fit: &Fit) -> Option<RgbImage> {
    let (w, h) = (
        u32::try_from(linear.width).ok()?,
        u32::try_from(linear.height).ok()?,
    );
    match fit {
        Fit::Look(l) => picture(fotollection_rawcore::finish_look(linear, l)),
        Fit::Curves(c) => Some(apply(&encode16(&linear.rgb), w, h, c)),
        Fit::Auto(c) => picture(fotollection_rawcore::finish(linear, c)),
        Fit::AsIs => RgbImage::from_raw(
            w,
            h,
            encode16(&linear.rgb)
                .iter()
                .map(|&v| u8::try_from(v >> 8).unwrap_or(u8::MAX))
                .collect(),
        ),
    }
}

/// Finishes a whole develop as `look` asks with `adjust` made in linear light first (FR-088),
/// the fit made on a copy at most [`FIT_EDGE`] across, as the preview's.
pub fn finish_linear(
    mut linear: fotollection_rawcore::Linear,
    reference: Option<&RgbImage>,
    look: RawLook,
    adjust: &fotollection_rawcore::RawAdjust,
    toned: bool,
) -> Option<Finished> {
    let small = shrink(&linear, FIT_EDGE);
    let (fitted, kind, scores) = fit(&small, reference, look, toned);
    drop(small);
    fotollection_rawcore::adjust(&mut linear, adjust);
    Some(Finished {
        image: render(&linear, &fitted)?,
        fit: kind,
        scores,
    })
}

/// A RAW ready for the preview (FR-088): its develop at most [`FIT_EDGE`] across, the
/// camera's JPEG, and the fits made so far, so a slider's step only re-finishes this copy.
pub struct Prepared {
    linear: fotollection_rawcore::Linear,
    reference: Option<RgbImage>,
    toned: bool,
    fits: std::sync::Mutex<Vec<(RawLook, std::sync::Arc<Fit>)>>,
}

impl Prepared {
    pub fn new(
        full: &fotollection_rawcore::Linear,
        reference: Option<RgbImage>,
        toned: bool,
    ) -> Self {
        Self {
            linear: shrink(full, FIT_EDGE),
            reference,
            toned,
            fits: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// About what it holds in memory: its linear copy and the camera's JPEG (FR-091).
    pub fn bytes(&self) -> usize {
        self.linear.rgb.len() * 4 + self.reference.as_ref().map_or(0, |r| r.data.len())
    }

    /// The preview's copy finished as `look` asks with `adjust`.
    pub fn finish(
        &self,
        look: RawLook,
        adjust: &fotollection_rawcore::RawAdjust,
    ) -> Option<RgbImage> {
        let cached = self.fits.lock().ok().and_then(|f| {
            f.iter()
                .find(|(l, _)| *l == look)
                .map(|(_, f)| std::sync::Arc::clone(f))
        });
        let fitted = if let Some(f) = cached {
            f
        } else {
            let f =
                std::sync::Arc::new(fit(&self.linear, self.reference.as_ref(), look, self.toned).0);
            if let Ok(mut fits) = self.fits.lock() {
                fits.push((look, std::sync::Arc::clone(&f)));
            }
            f
        };
        let mut linear = self.linear.clone();
        fotollection_rawcore::adjust(&mut linear, adjust);
        render(&linear, &fitted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dark, flat develop and a bright reference of the same gradient.
    fn gradient(n: usize) -> (Vec<u16>, RgbImage) {
        let developed: Vec<u16> = (0..n)
            .flat_map(|i| {
                let v = u16::try_from(i * 20_000 / n).unwrap_or(0);
                [v, v, v]
            })
            .collect();
        let reference = RgbImage {
            width: u32::try_from(n).unwrap_or(0),
            height: 1,
            data: (0..n)
                .flat_map(|i| {
                    let v = u8::try_from(i * 255 / n).unwrap_or(0);
                    [v, v, v]
                })
                .collect(),
        };
        (developed, reference)
    }

    #[test]
    fn fr_016_matching_brings_a_dark_develop_to_the_reference_look() {
        let (developed, mut reference) = gradient(4000);
        reference.height = 200;
        reference.data = reference.data.repeat(200);
        let curves = match_reference(&developed, &reference);
        let Some(curves) = curves else {
            panic!("a large reference is used");
        };
        let out = apply(&developed, 4000, 1, &curves);
        // The darkest stays dark, the brightest of the develop (20,000 of 65,535) becomes white.
        assert!(out.data[0] < 5);
        assert!(out.data[out.data.len() - 1] > 250);
        // Monotonic: no level comes out darker than the one before it.
        let px = out.data.as_chunks::<3>().0;
        assert!(px.iter().zip(px.iter().skip(1)).all(|(a, b)| b[0] >= a[0]));
    }

    #[test]
    fn fr_016_a_tiny_reference_is_not_trusted_and_the_default_curve_exposes() {
        let (developed, reference) = gradient(4000);
        assert!(match_reference(&developed, &reference).is_none());
        let out = apply(&developed, 4000, 1, &default_curves(&developed));
        assert!(
            out.data[out.data.len() - 1] > 230,
            "the brightest is exposed up"
        );
        assert!(out.data[0] < 5);
    }
}
