// SPDX-License-Identifier: GPL-3.0-only
//! Best-shot suggestions (FR-041): measure each photo once, group near-identical shots, and
//! rank each group with reasons. Everything runs offline on the bundled models (ADR-0018).

pub mod cache;
pub mod faces;
pub mod group;
pub mod metrics;
pub mod models;
pub mod score;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::image::RgbImage;
use crate::library::{PhotoId, PhotoRecord};
use metrics::{ANALYSIS_EDGE, Gray, Metrics, dhash, measure};
use score::GroupResult;

/// Bumped whenever a measure changes, so cached analyses are recomputed.
pub const ANALYSIS_VERSION: u32 = 1;

#[derive(Debug, Clone, Error)]
pub enum CullingError {
    #[error("bundled model does not match its checksum")]
    ModelChecksum,
    #[error("model failed: {0}")]
    Model(String),
    #[error("could not resize the photo for analysis")]
    Resize,
    #[error("a model cache lock was poisoned")]
    Poisoned,
}

impl CullingError {
    /// Stable error code for the UI (FR-020).
    pub fn code(&self) -> &'static str {
        match self {
            Self::ModelChecksum | Self::Model(_) | Self::Poisoned => "culling.model_failed",
            Self::Resize => "culling.analysis_failed",
        }
    }
}

/// One face, with its box as fractions of the image so shots of different sizes compare.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceSummary {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Mean eye aspect ratio of both eyes; `None` when the face was too small or turned away.
    pub ear: Option<f32>,
    /// Tenengrad energy around the eyes (or the upper face without landmarks).
    pub sharpness: f32,
}

impl FaceSummary {
    /// Probably the same person in another shot of the group: the boxes overlap enough.
    pub fn overlaps(&self, other: &Self) -> bool {
        let ix = (self.x + self.w).min(other.x + other.w) - self.x.max(other.x);
        let iy = (self.y + self.h).min(other.y + other.h) - self.y.max(other.y);
        if ix <= 0.0 || iy <= 0.0 {
            return false;
        }
        let inter = ix * iy;
        inter / (self.w * self.h + other.w * other.h - inter) > 0.3
    }
}

/// Everything best-shot suggestions need to know about one photo; cached per file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAnalysis {
    pub version: u32,
    pub metrics: Metrics,
    pub hash: u64,
    pub faces: Vec<FaceSummary>,
}

impl PhotoAnalysis {
    /// Values an analysis can actually produce; anything else (a damaged or tampered cache
    /// file) is recomputed instead of trusted.
    pub fn is_plausible(&self) -> bool {
        let m = &self.metrics;
        let finite = [
            m.sharpness,
            m.subject_sharpness,
            m.laplacian_variance,
            m.coherence,
            m.highlights,
            m.shadows,
            m.exposure_ev,
            m.contrast,
            m.noise,
        ]
        .iter()
        .all(|v| v.is_finite());
        let faces_ok = self.faces.len() <= faces::MAX_FACES
            && self.faces.iter().all(|f| {
                [f.x, f.y].iter().all(|v| (-1.0..=2.0).contains(v))
                    && [f.w, f.h].iter().all(|v| *v > 0.0 && *v <= 2.0)
                    && f.sharpness.is_finite()
                    && f.ear.is_none_or(|e| (0.0..=2.0).contains(&e))
            });
        finite && faces_ok
    }
}

/// One group of similar shots with its ranking, for the Groups view (FR-041).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    /// The group's first photo in capture order; identifies the group in commands.
    pub id: PhotoId,
    pub result: GroupResult,
}

/// Groups and ranks every analysed photo of the library. Photos without an analysis are
/// left out.
pub fn suggest<S: std::hash::BuildHasher>(
    records: &[PhotoRecord],
    analyses: &HashMap<PhotoId, PhotoAnalysis, S>,
) -> Vec<Group> {
    let shots = records
        .iter()
        .filter_map(|r| Some(group::Shot::from_record(r, analyses.get(&r.id)?.hash)))
        .collect();
    group::group(shots)
        .into_iter()
        .filter_map(|ids| {
            let members: Vec<_> = ids
                .iter()
                .filter_map(|id| Some((*id, analyses.get(id)?)))
                .collect();
            Some(Group {
                id: *ids.first()?,
                result: score::rank(&members),
            })
        })
        .collect()
}

/// Resizes to exactly `width` × `height` with a box filter: plain area averaging, faster
/// than the print path's Lanczos and free of its sharpening, which would bias the sharpness
/// measures.
fn resize(image: &RgbImage, width: u32, height: u32) -> Result<RgbImage, CullingError> {
    if (width, height) == (image.width, image.height) {
        return Ok(image.clone());
    }
    let src = ImageRef::new(image.width, image.height, &image.data, PixelType::U8x3)
        .map_err(|_| CullingError::Resize)?;
    let mut dst = Image::new(width, height, PixelType::U8x3);
    Resizer::new()
        .resize(
            &src,
            &mut dst,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Box)),
        )
        .map_err(|_| CullingError::Resize)?;
    RgbImage::from_raw(width, height, dst.into_vec()).ok_or(CullingError::Resize)
}

/// Halves both sides by averaging 2×2 blocks (an odd last row or column is dropped).
fn halve(image: &RgbImage) -> RgbImage {
    let (half_w, half_h) = (image.width / 2, image.height / 2);
    let (w, h) = (half_w as usize, half_h as usize);
    let stride = image.width as usize * 3;
    let mut out = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        let top = &image.data[2 * y * stride..][..w * 6];
        let bottom = &image.data[(2 * y + 1) * stride..][..w * 6];
        for (a, b) in top.as_chunks::<6>().0.iter().zip(bottom.as_chunks::<6>().0) {
            for c in 0..3 {
                let sum =
                    u16::from(a[c]) + u16::from(a[c + 3]) + u16::from(b[c]) + u16::from(b[c + 3]);
                #[allow(clippy::cast_possible_truncation)]
                out.push(((sum + 2) / 4) as u8);
            }
        }
    }
    RgbImage {
        width: half_w,
        height: half_h,
        data: out,
    }
}

/// Reduces so the long edge is at most `long_edge`: exact halvings first (cheap), then one
/// box resize for the rest.
pub(crate) fn reduce(image: &RgbImage, long_edge: u32) -> Result<RgbImage, CullingError> {
    let mut halved = None;
    while halved.as_ref().unwrap_or(image).long_edge() / 2 >= long_edge {
        halved = Some(halve(halved.as_ref().unwrap_or(image)));
    }
    let current = halved.as_ref().unwrap_or(image);
    let (w, h) = fitted(current, long_edge);
    match halved {
        Some(done) if (w, h) == (done.width, done.height) => Ok(done),
        _ => resize(current, w, h),
    }
}

/// Size with the long edge reduced to at most `long_edge`, keeping the aspect ratio.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn fitted(image: &RgbImage, long_edge: u32) -> (u32, u32) {
    if image.long_edge() <= long_edge {
        return (image.width, image.height);
    }
    let scale = f64::from(long_edge) / f64::from(image.long_edge());
    let dim = |v: u32| ((f64::from(v) * scale).round() as u32).max(1);
    (dim(image.width), dim(image.height))
}

/// Analyses one upright photo (any size; it is reduced to 1024 px first).
pub fn analyze(image: &RgbImage) -> Result<PhotoAnalysis, CullingError> {
    let small = reduce(image, ANALYSIS_EDGE)?;
    let gray = Gray::from_rgb(&small);
    Ok(PhotoAnalysis {
        version: ANALYSIS_VERSION,
        metrics: measure(&gray),
        hash: dhash(&gray),
        faces: faces::faces(&small, &gray)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_halves_then_resizes_to_the_long_edge() {
        let mut image = RgbImage::new(2048, 1536);
        for (i, v) in image.data.iter_mut().enumerate() {
            *v = u8::try_from(i % 251).unwrap_or(0);
        }
        let small = reduce(&image, 1024).ok();
        assert_eq!(
            small.as_ref().map(|s| (s.width, s.height)),
            Some((1024, 768))
        );
        let small = reduce(&image, 480).ok();
        assert_eq!(small.map(|s| (s.width, s.height)), Some((480, 360)));
        let odd = reduce(&RgbImage::new(1001, 3), 1024).ok();
        assert_eq!(odd.map(|s| (s.width, s.height)), Some((1001, 3)));
        // 2×2 averaging with rounding.
        let block = RgbImage::from_raw(2, 2, vec![0, 10, 255, 1, 10, 255, 2, 10, 255, 3, 11, 254]);
        let one = block.map(|b| halve(&b).data);
        assert_eq!(one, Some(vec![2, 10, 255]));
    }
}
