// SPDX-License-Identifier: GPL-3.0-only
//! Faces and eyes for best-shot suggestions (FR-041, ADR-0018): YuNet finds faces, FaceMesh
//! places 478 landmarks on each, and the eye aspect ratio (EAR) of the eyelid landmarks tells
//! open from closed. Landmarks are used for nothing else and never stored.

use tract_onnx::prelude::Tensor;

use super::metrics::{Gray, region_sharpness};
use super::models::{FACEMESH_SIDE, YUNET_LONG_EDGE, facemesh, yunet};
use super::{CullingError, FaceSummary, reduce};
use crate::image::RgbImage;

/// YuNet detections below this score are ignored.
const MIN_SCORE: f32 = 0.6;
const NMS_IOU: f32 = 0.3;
/// Faces smaller than this (analysis pixels) skip the eye check.
const MIN_FACE_PX: f32 = 64.0;
/// At most this many faces get landmarks, largest first, to bound the time per photo.
pub const MAX_FACES: usize = 8;
/// FaceMesh crop side relative to the face box.
const CROP_SCALE: f32 = 1.5;
const MIN_FACE_FLAG: f32 = 0.5;

/// Eyelid landmarks (FaceMesh indices): outer corner, upper ×2, inner corner, lower ×2,
/// ordered p1…p6 for EAR = (|p2−p6| + |p3−p5|) / (2 |p1−p4|).
const RIGHT_EYE: [usize; 6] = [33, 160, 158, 133, 153, 144];
const LEFT_EYE: [usize; 6] = [263, 387, 385, 362, 380, 373];

#[derive(Debug, Clone, Copy)]
struct Detection {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    score: f32,
    /// Right eye, left eye, nose tip, mouth corners (the person's right and left).
    keypoints: [(f32, f32); 5],
}

fn iou(a: &Detection, b: &Detection) -> f32 {
    let ix = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let iy = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if ix <= 0.0 || iy <= 0.0 {
        return 0.0;
    }
    let inter = ix * iy;
    inter / (a.w * a.h + b.w * b.h - inter)
}

/// Bilinear sample with black outside the image.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn sample(image: &RgbImage, x: f32, y: f32) -> [f32; 3] {
    let (w, h) = (image.width as f32, image.height as f32);
    if x < -0.5 || y < -0.5 || x > w - 0.5 || y > h - 0.5 {
        return [0.0; 3];
    }
    let (x, y) = (x.clamp(0.0, w - 1.0), y.clamp(0.0, h - 1.0));
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = (
        (x0 + 1).min(image.width as usize - 1),
        (y0 + 1).min(image.height as usize - 1),
    );
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let stride = image.width as usize;
    let px = |xx: usize, yy: usize, c: usize| f32::from(image.data[(yy * stride + xx) * 3 + c]);
    std::array::from_fn(|c| {
        let top = px(x0, y0, c) * (1.0 - fx) + px(x1, y0, c) * fx;
        let bottom = px(x0, y1, c) * (1.0 - fx) + px(x1, y1, c) * fx;
        top * (1.0 - fy) + bottom * fy
    })
}

/// Runs YuNet on the image reduced to a 512 px long edge and padded to multiples of 32
/// (BGR, 0–255, NCHW).
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn detect(image: &RgbImage) -> Result<Vec<Detection>, CullingError> {
    let reduced = reduce(image, YUNET_LONG_EDGE as u32)?;
    let scale = reduced.width as f32 / image.width as f32;
    let (w, h) = (reduced.width as usize, reduced.height as usize);
    let (mut width, height) = (w.div_ceil(32) * 32, h.div_ceil(32) * 32);
    // tract runs YuNet faster on a padded square than on a tall input (11.0 against 12.8 ms
    // on REF-MAC), and rotating instead would turn the faces sideways.
    width = width.max(height);
    let plane = width * height;
    let mut input = vec![0f32; 3 * plane];
    for (y, row) in reduced.data.chunks_exact(w * 3).enumerate() {
        for (x, &[r, g, b]) in row.as_chunks::<3>().0.iter().enumerate() {
            let i = y * width + x;
            input[i] = f32::from(b);
            input[plane + i] = f32::from(g);
            input[2 * plane + i] = f32::from(r);
        }
    }
    let tensor = Tensor::from_shape(&[1, 3, height, width], &input)
        .map_err(|e| CullingError::Model(e.to_string()))?;
    let model = yunet(width, height)?;
    let outputs = model.run(tensor)?;
    let get = |name: &str| {
        outputs
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.as_slice())
    };

    let mut found = Vec::new();
    for stride in [8usize, 16, 32] {
        let (Some(cls), Some(obj), Some(bbox), Some(kps)) = (
            get(&format!("cls_{stride}")),
            get(&format!("obj_{stride}")),
            get(&format!("bbox_{stride}")),
            get(&format!("kps_{stride}")),
        ) else {
            return Err(CullingError::Model(format!(
                "YuNet output for stride {stride} missing"
            )));
        };
        let (columns, lines) = (width / stride, height / stride);
        let s = stride as f32;
        for i in 0..columns * lines {
            let score = (cls[i].clamp(0.0, 1.0) * obj[i].clamp(0.0, 1.0)).sqrt();
            if score < MIN_SCORE {
                continue;
            }
            let (c, r) = ((i % columns) as f32, (i / columns) as f32);
            let b = &bbox[i * 4..i * 4 + 4];
            let (cx, cy) = ((c + b[0]) * s, (r + b[1]) * s);
            let (w, h) = (b[2].exp() * s, b[3].exp() * s);
            let k = &kps[i * 10..i * 10 + 10];
            found.push(Detection {
                x: (cx - w / 2.0) / scale,
                y: (cy - h / 2.0) / scale,
                w: w / scale,
                h: h / scale,
                score,
                keypoints: std::array::from_fn(|n| {
                    ((c + k[2 * n]) * s / scale, (r + k[2 * n + 1]) * s / scale)
                }),
            });
        }
    }
    found.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Detection> = Vec::new();
    for d in found {
        if kept.iter().all(|k| iou(k, &d) < NMS_IOU) {
            kept.push(d);
        }
    }
    Ok(kept)
}

fn ear(points: &[(f32, f32)], eye: [usize; 6]) -> f32 {
    let p = |i: usize| points[eye[i]];
    let dist = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
    let width = dist(p(0), p(3));
    if width <= f32::EPSILON {
        return 0.0;
    }
    (dist(p(1), p(5)) + dist(p(2), p(4))) / (2.0 * width)
}

struct Eyes {
    ear: f32,
    /// Bounding box of both eyes in image pixels.
    region: (f32, f32, f32, f32),
}

/// FaceMesh on a crop rotated so the eye line is level. `None` when the model says the crop
/// holds no usable face.
#[allow(clippy::cast_precision_loss)]
fn eyes(image: &RgbImage, d: &Detection) -> Result<Option<Eyes>, CullingError> {
    let side = FACEMESH_SIDE;
    let ((rx, ry), (lx, ly)) = (d.keypoints[0], d.keypoints[1]);
    let angle = (ly - ry).atan2(lx - rx);
    let (sin, cos) = angle.sin_cos();
    let (cx, cy) = (d.x + d.w / 2.0, d.y + d.h / 2.0);
    let crop = CROP_SCALE * d.w.max(d.h);
    let unit = crop / side as f32;
    // Crop pixel → image pixel.
    let to_image = |u: f32, v: f32| {
        let (du, dv) = (
            (u - side as f32 / 2.0) * unit,
            (v - side as f32 / 2.0) * unit,
        );
        (cx + du * cos - dv * sin, cy + du * sin + dv * cos)
    };
    let mut input = Vec::with_capacity(side * side * 3);
    for v in 0..side {
        for u in 0..side {
            let (x, y) = to_image(u as f32 + 0.5, v as f32 + 0.5);
            input.extend(sample(image, x - 0.5, y - 0.5).map(|c| c / 255.0));
        }
    }
    let tensor = Tensor::from_shape(&[1, side, side, 3], &input)
        .map_err(|e| CullingError::Model(e.to_string()))?;
    let outputs = facemesh()?.run(tensor)?;
    let Some((_, landmarks)) = outputs.iter().find(|(_, v)| v.len() == 478 * 3) else {
        return Err(CullingError::Model("FaceMesh landmarks missing".into()));
    };
    // The face flag is a logit.
    let flag = outputs
        .iter()
        .find(|(n, v)| v.len() == 1 && n.ends_with("Identity_1"))
        .map_or(1.0, |(_, v)| 1.0 / (1.0 + (-v[0]).exp()));
    if flag < MIN_FACE_FLAG {
        return Ok(None);
    }
    let points: Vec<(f32, f32)> = landmarks
        .as_chunks::<3>()
        .0
        .iter()
        .map(|&[x, y, _]| (x, y))
        .collect();
    let ear = f32::midpoint(ear(&points, RIGHT_EYE), ear(&points, LEFT_EYE));
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &i in RIGHT_EYE.iter().chain(&LEFT_EYE) {
        let (x, y) = to_image(points[i].0, points[i].1);
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    // Lids and lashes around the corners carry the focus detail.
    let pad = 0.3 * (y1 - y0).max((x1 - x0) / 4.0);
    Ok(Some(Eyes {
        ear,
        region: (x0 - pad, y0 - pad, x1 + pad, y1 + pad),
    }))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sharpness_in(gray: &Gray, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> f32 {
    let px = |v: f32| v.max(0.0) as usize;
    region_sharpness(gray, px(x0), px(y0), px(x1.ceil()), px(y1.ceil()))
}

/// Faces in the analysis image, largest first, with eye openness where it can be measured.
#[allow(clippy::cast_precision_loss)]
pub fn faces(image: &RgbImage, gray: &Gray) -> Result<Vec<FaceSummary>, CullingError> {
    let mut detections = detect(image)?;
    detections.sort_by(|a, b| (b.w * b.h).total_cmp(&(a.w * a.h)));
    detections.truncate(MAX_FACES);
    let (w, h) = (image.width as f32, image.height as f32);
    detections
        .iter()
        .map(|d| {
            let eyes = if d.w.min(d.h) >= MIN_FACE_PX {
                eyes(image, d)?
            } else {
                None
            };
            // Without landmarks, the upper half of the face box holds the eyes.
            let region = eyes
                .as_ref()
                .map_or((d.x, d.y + 0.2 * d.h, d.x + d.w, d.y + 0.55 * d.h), |e| {
                    e.region
                });
            Ok(FaceSummary {
                x: d.x / w,
                y: d.y / h,
                w: d.w / w,
                h: d.h / h,
                ear: eyes.map(|e| e.ear),
                sharpness: sharpness_in(gray, region),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_041_no_faces_in_an_empty_or_patterned_image() {
        let mut image = RgbImage::new(800, 600);
        image.data.fill(128);
        let found = faces(&image, &Gray::from_rgb(&image)).map_err(|e| e.to_string());
        assert_eq!(found.as_deref().map(<[_]>::len), Ok(0), "{found:?}");

        let stripes = RgbImage::from_raw(
            640,
            480,
            (0..640 * 480 * 3)
                .map(|i| if (i / 3 % 640) / 20 % 2 == 0 { 30 } else { 220 })
                .collect(),
        );
        let stripes = stripes.unwrap_or_else(|| RgbImage::new(1, 1));
        let found = faces(&stripes, &Gray::from_rgb(&stripes)).map_err(|e| e.to_string());
        assert_eq!(found.as_deref().map(<[_]>::len), Ok(0), "{found:?}");
    }

    #[test]
    fn ear_of_open_and_closed_eye_shapes() {
        let mut points = vec![(0.0, 0.0); 478];
        // An eye 30 px wide, lids 10 px apart → EAR 1/3.
        for (i, p) in RIGHT_EYE.iter().zip([
            (0.0, 0.0),
            (10.0, -5.0),
            (20.0, -5.0),
            (30.0, 0.0),
            (20.0, 5.0),
            (10.0, 5.0),
        ]) {
            points[*i] = p;
        }
        assert!((ear(&points, RIGHT_EYE) - 1.0 / 3.0).abs() < 1e-5);
        for i in [1, 2, 4, 5] {
            points[RIGHT_EYE[i]].1 /= 10.0;
        }
        assert!(ear(&points, RIGHT_EYE) < 0.05);
    }
}
