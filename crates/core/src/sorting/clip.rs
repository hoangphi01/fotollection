// SPDX-License-Identifier: GPL-3.0-only
//! CLIP's input and output in Rust (FR-141 S6): a photo turned into the image tower's input
//! as CLIP's own processor makes it (the short side to `size` by bicubic, a square cut, each
//! channel scaled by its mean and spread), three cuts along the long side averaged into one
//! embedding, and the zero-shot reading of it against every prompt's text embedding, computed
//! once at build time, each label's share summed. The model
//! itself runs in tract (ADR-0018): TinyCLIP-39M (MIT), chosen by measurement (ADR-0056), its
//! weights stored as int8 that tract folds back to f32 at load. The shell compiles the model
//! in and hands its bytes over; the labels' text side, computed once, is compiled in here.

use std::io::Cursor;
use std::sync::{Arc, OnceLock};

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use serde::Deserialize;
use tract_onnx::prelude::*;

use crate::image::RgbImage;

/// The bundled model's BLAKE3, checked at load (`assets/models/MODELS.md`).
pub const MODEL_BLAKE3: &str = "f036302b86db489db3837c693dd46aea77ec946dd63e75808f5e77f2fe497dbc";
/// The model's file in the repository, for the shell's `include_bytes!` and the tests.
pub const MODEL_FILE: &str = "tinyclip-39m.w8.onnx";
/// The embedding's length.
pub const DIM: usize = 512;

#[derive(Debug, thiserror::Error)]
pub enum ClipError {
    #[error("the CLIP model does not match its checksum")]
    Checksum,
    #[error("the CLIP model could not run: {0}")]
    Model(String),
}

impl ClipError {
    /// Stable machine-readable code (SECURITY.md §8), for the `ERR-` code on screen.
    pub const fn code(&self) -> &'static str {
        "sorting.clip_failed"
    }
}

/// The labels' text embeddings and the model's input, as the export wrote them.
#[derive(Deserialize)]
pub struct Labels {
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub size: u32,
    pub logit_scale: f32,
    /// Each label's prompts, one unit text embedding each.
    pub labels: std::collections::BTreeMap<String, Vec<Vec<f32>>>,
}

/// The compiled-in labels of the bundled model.
pub fn labels() -> &'static Labels {
    static LABELS: OnceLock<Labels> = OnceLock::new();
    LABELS.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/models/tinyclip-39m.labels.json"))
            .unwrap_or_else(|e| unreachable!("the labels file is checked by its test: {e}"))
    })
}

/// The image tower, loaded once; `embed` is safe from several threads.
pub struct Embedder {
    plan: Arc<TypedRunnableModel>,
}

impl Embedder {
    /// Checks the bytes against `blake3`, then loads and optimizes the model for one photo.
    pub fn load(bytes: &[u8], blake3: &str) -> Result<Self, ClipError> {
        if blake3::hash(bytes).to_hex().as_str() != blake3 {
            return Err(ClipError::Checksum);
        }
        let s = labels().size as usize;
        let err = |e: TractError| ClipError::Model(e.to_string());
        // The file records shapes for a batch of any size; the input fixes it to one.
        let plan = tract_onnx::onnx()
            .with_ignore_value_info(true)
            .with_ignore_output_shapes(true)
            .model_for_read(&mut Cursor::new(bytes))
            .and_then(|m| m.with_input_fact(0, f32::fact([1, 3, s, s]).into()))
            .and_then(tract_onnx::prelude::InferenceModelExt::into_optimized)
            .and_then(TypedModel::into_runnable)
            .map_err(err)?;
        Ok(Self { plan })
    }

    /// The photo's unit embedding, from its upright picture: the mean of three square cuts,
    /// the centre and both ends of the long side, so people and trees at a photo's edges count
    /// (owner, 2026-11-18; measured in OPTIMIZATION §5). A square photo is read once.
    pub fn embed(&self, image: &RgbImage) -> Result<Vec<f32>, ClipError> {
        let l = labels();
        let square = image.width == image.height;
        let cuts: &[f32] = if square { &[0.5] } else { &[0.5, 0.0, 1.0] };
        let mut sum = vec![0f32; DIM];
        for &at in cuts {
            let px = pixels_at(image, l.size, l.mean, l.std, at)
                .ok_or_else(|| ClipError::Model("empty picture".into()))?;
            for (s, v) in sum.iter_mut().zip(self.one(px)?) {
                *s += v;
            }
        }
        unit(sum)
    }

    /// One cut's unit embedding.
    fn one(&self, px: Vec<f32>) -> Result<Vec<f32>, ClipError> {
        let s = labels().size as usize;
        let input = tract_ndarray::Array4::from_shape_vec((1, 3, s, s), px)
            .map_err(|e| ClipError::Model(e.to_string()))?;
        let out = self
            .plan
            .run(tvec!(Tensor::from(input).into()))
            .map_err(|e| ClipError::Model(e.to_string()))?;
        let e: Vec<f32> = out
            .first()
            .ok_or_else(|| ClipError::Model("no output".into()))?
            .to_plain_array_view::<f32>()
            .map_err(|e| ClipError::Model(e.to_string()))?
            .iter()
            .copied()
            .collect();
        unit(e)
    }
}

/// `e` scaled to length one, or an error for a vector of the wrong length or none.
fn unit(mut e: Vec<f32>) -> Result<Vec<f32>, ClipError> {
    let norm = e.iter().map(|v| v * v).sum::<f32>().sqrt();
    if e.len() != DIM || !norm.is_finite() || norm == 0.0 {
        return Err(ClipError::Model("a bad embedding".into()));
    }
    for v in &mut e {
        *v /= norm;
    }
    Ok(e)
}

/// The subject labels' keys in order, a subject's index into them being what the library keeps.
pub fn subject_keys() -> &'static [&'static str] {
    static KEYS: OnceLock<Vec<&'static str>> = OnceLock::new();
    KEYS.get_or_init(|| {
        labels()
            .labels
            .keys()
            .filter(|k| !k.starts_with("src-"))
            .map(String::as_str)
            .collect()
    })
}

/// The subjects an embedding shows, most likely first: a softmax over every prompt of the
/// subject labels (the `src-` labels, about where a file came from, apart), each label's
/// prompts' shares summed, so a photo of autumn trees is near "trees", not a blur of scenery.
pub fn subjects(embedding: &[f32]) -> Vec<(&'static str, f32)> {
    let l = labels();
    let mut owner = Vec::new();
    let mut vecs = Vec::new();
    for (k, prompts) in l.labels.iter().filter(|(k, _)| !k.starts_with("src-")) {
        for v in prompts {
            owner.push(k.as_str());
            vecs.push(v.clone());
        }
    }
    let mut out: Vec<(&'static str, f32)> = Vec::new();
    for (k, p) in owner
        .into_iter()
        .zip(zero_shot(embedding, &vecs, l.logit_scale))
    {
        match out.last_mut() {
            Some(last) if last.0 == k => last.1 += p,
            _ => out.push((k, p)),
        }
    }
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

/// The subject a photo is named by: its likeliest at `min_p` or more; else, when buildings and
/// scenery together reach it (an arcade, a door, a street split between the two), the larger
/// of them (owner, 2026-11-19; on the owner's labels *Other* 57 → 40 of 759 photos, 90.2 % and
/// 92.2 % right, OPTIMIZATION §5).
pub fn top_subject(best: &[(&'static str, f32)], min_p: f32) -> Option<&'static str> {
    let (k, p) = *best.first()?;
    if p >= min_p {
        return Some(k);
    }
    let of = |key: &str| best.iter().find(|x| x.0 == key).map_or(0.0, |x| x.1);
    let (scenery, building) = (of("scenery"), of("building"));
    (scenery + building >= min_p).then_some(if building > scenery {
        "building"
    } else {
        "scenery"
    })
}

/// The model's input: `3 × size × size` floats, channel-major, as CLIP's processor makes it
/// (its centre cut).
pub fn pixels(image: &RgbImage, size: u32, mean: [f32; 3], std: [f32; 3]) -> Option<Vec<f32>> {
    pixels_at(image, size, mean, std, 0.5)
}

/// The same input cut `at` along the long side: 0 its start, 0.5 its centre, 1 its end.
pub fn pixels_at(
    image: &RgbImage,
    size: u32,
    mean: [f32; 3],
    std: [f32; 3],
    at: f32,
) -> Option<Vec<f32>> {
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
    // Whole pixels: the slack times `at`, floored as CLIP's centre cut (`at` is 0, 0.5 or 1).
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let cut = |slack: u32| ((slack as f32) * at.clamp(0.0, 1.0)).floor() as u32;
    let (x0, y0) = (cut(rw - size), cut(rh - size));
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
        // The end cuts: the left end black, the right end white, the red square at their
        // inner edges only.
        let a = pixels_at(&img, 20, mean, std, 0.0).unwrap_or_default();
        let b = pixels_at(&img, 20, mean, std, 1.0).unwrap_or_default();
        let most = |px: &[f32], v: f32| {
            px.iter().filter(|x| (*x - v).abs() < 0.05).count() * 10 > px.len() * 9
        };
        assert!(most(&a, -2.0) && most(&b, 2.0));
        let p = zero_shot(&[1.0, 0.0], &[vec![1.0, 0.0], vec![0.0, 1.0]], 100.0);
        assert!(p[0] > 0.99 && (p.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }

    fn model() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/models/tinyclip-39m.w8.onnx"
        );
        std::fs::read(path).unwrap_or_default()
    }

    #[test]
    fn fr_141_the_bundled_clip_model_loads_and_matches_its_checksum() {
        let l = labels();
        assert_eq!(l.size, 224);
        assert!(
            l.labels.contains_key("cat") && l.labels.values().flatten().all(|v| v.len() == DIM)
        );
        let bytes = model();
        assert!(matches!(
            Embedder::load(&bytes[..bytes.len() / 2], MODEL_BLAKE3),
            Err(ClipError::Checksum)
        ));
        let m = Embedder::load(&bytes, MODEL_BLAKE3).unwrap_or_else(|e| unreachable!("{e}"));
        let mut img = RgbImage::new(300, 200);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = u8::try_from(i % 251).unwrap_or(0);
        }
        let e = m.embed(&img).unwrap_or_default();
        let norm = e.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(e.len() == DIM && (norm - 1.0).abs() < 1e-4);
        let s = subjects(&e);
        assert!(
            s.iter().all(|(k, _)| !k.starts_with("src-"))
                && (s.iter().map(|x| x.1).sum::<f32>() - 1.0).abs() < 1e-4
        );
    }

    #[test]
    fn fr_141_clip_reads_a_subject_by_its_own_text() {
        // The text embedding of "cat" read back as an image embedding names the cat.
        let cat = labels().labels["cat"][0].clone();
        assert_eq!(subjects(&cat)[0].0, "cat");
        // Autumn trees, one prompt among scenery's many, still name scenery.
        let trees = labels().labels["scenery"][6].clone();
        assert_eq!(subjects(&trees)[0].0, "scenery");
        // Buildings and scenery together name the larger; a split elsewhere names nothing.
        let split = [("people", 0.3), ("building", 0.26), ("scenery", 0.24)];
        assert_eq!(top_subject(&split, 0.45), Some("building"));
        assert_eq!(top_subject(&[("people", 0.3), ("food", 0.3)], 0.45), None);
        assert_eq!(top_subject(&[("cat", 0.5)], 0.45), Some("cat"));
        // The subjects added on 2026-11-19: a tram and a church read as themselves.
        let tram = labels().labels["vehicle"][0].clone();
        assert_eq!(subjects(&tram)[0].0, "vehicle");
        let church = labels().labels["building"][1].clone();
        assert_eq!(subjects(&church)[0].0, "building");
    }
}
