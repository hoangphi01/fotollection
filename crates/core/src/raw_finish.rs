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
