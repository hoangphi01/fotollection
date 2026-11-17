// SPDX-License-Identifier: GPL-3.0-only
//! CLIP's input and output in Rust (FR-141 S6): a photo turned into the image tower's input
//! as CLIP's own processor makes it (the short side to `size` by bicubic, the centre cut
//! square, each channel scaled by its mean and spread), and the zero-shot reading of an
//! embedding against the labels' text embeddings, computed once at build time. The model
//! itself runs in tract (ADR-0018); which one is measured first (`examples/clip_bench.rs`).

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::image::RgbImage;

/// The model's input: `3 × size × size` floats, channel-major, as CLIP's processor makes it.
pub fn pixels(image: &RgbImage, size: u32, mean: [f32; 3], std: [f32; 3]) -> Option<Vec<f32>> {
    let (w, h) = (image.width, image.height);
    if w == 0 || h == 0 || size == 0 {
        return None;
    }
    // The short side to `size`, the long side in proportion (rounded as CLIP's processor).
    let (rw, rh) = if w <= h {
        (
            u64::from(size),
            ((u64::from(h) * u64::from(size) + u64::from(w) / 2) / u64::from(w)).max(1),
        )
    } else {
        (
            ((u64::from(w) * u64::from(size) + u64::from(h) / 2) / u64::from(h)).max(1),
            u64::from(size),
        )
    };
    let (rw, rh) = (u32::try_from(rw).ok()?, u32::try_from(rh).ok()?);
    let src = ImageRef::new(w, h, &image.data, PixelType::U8x3).ok()?;
    let mut dst = Image::new(rw, rh, PixelType::U8x3);
    Resizer::new()
        .resize(
            &src,
            &mut dst,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::CatmullRom)),
        )
        .ok()?;
    let data = dst.into_vec();
    let (x0, y0) = ((rw - size) / 2, (rh - size) / 2);
    let n = (size * size) as usize;
    let mut out = vec![0f32; 3 * n];
    for y in 0..size {
        for x in 0..size {
            let i = (((y0 + y) * rw + x0 + x) * 3) as usize;
            let o = (y * size + x) as usize;
            for c in 0..3 {
                out[c * n + o] = (f32::from(data[i + c]) / 255.0 - mean[c]) / std[c];
            }
        }
    }
    Some(out)
}

/// The labels' probabilities for one image embedding: cosine with each label's (unit) text
/// embedding, times `scale`, through a softmax.
pub fn zero_shot(embedding: &[f32], labels: &[Vec<f32>], scale: f32) -> Vec<f32> {
    let norm = embedding
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt()
        .max(f32::EPSILON);
    let logits: Vec<f32> = labels
        .iter()
        .map(|t| scale * t.iter().zip(embedding).map(|(a, b)| a * b).sum::<f32>() / norm)
        .collect();
    let top = logits.iter().copied().fold(f32::MIN, f32::max);
    let exp: Vec<f32> = logits.iter().map(|l| (l - top).exp()).collect();
    let sum: f32 = exp.iter().sum();
    exp.into_iter().map(|e| e / sum).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_141_clip_input_is_the_centre_square_normalised() {
        // A 400 × 200 picture: left half black, right half white, a red square in the centre.
        let mut img = RgbImage::new(400, 200);
        for y in 0..200u32 {
            for x in 0..400u32 {
                let i = ((y * 400 + x) * 3) as usize;
                let v = if (180..220).contains(&x) && (80..120).contains(&y) {
                    [255, 0, 0]
                } else if x >= 200 {
                    [255, 255, 255]
                } else {
                    [0, 0, 0]
                };
                img.data[i..i + 3].copy_from_slice(&v);
            }
        }
        let (mean, std) = ([0.5; 3], [0.25; 3]);
        let px = pixels(&img, 20, mean, std).unwrap_or_default();
        assert_eq!(px.len(), 3 * 20 * 20);
        let at = |c: usize, x: usize, y: usize| px[c * 400 + y * 20 + x];
        // The cut keeps the middle 200 of 400 columns: the left edge black, the right white.
        assert!((at(0, 0, 0) - (-2.0)).abs() < 0.05, "{}", at(0, 0, 0));
        assert!((at(1, 19, 0) - 2.0).abs() < 0.05, "{}", at(1, 19, 0));
        // The red square sits in the centre: red channel high, green low.
        assert!(at(0, 10, 10) > 1.5 && at(1, 10, 10) < -1.5);
        let p = zero_shot(&[1.0, 0.0], &[vec![1.0, 0.0], vec![0.0, 1.0]], 100.0);
        assert!(p[0] > 0.99 && (p.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }
}
