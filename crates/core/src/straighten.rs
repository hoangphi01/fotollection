// SPDX-License-Identifier: GPL-3.0-only
//! Straightening (FR-006): turning the photo by up to ±15° so a horizon is level. The
//! crop lives in the *straightened* frame, the photo turned about its centre on a canvas of
//! the same size. It is kept as large as possible without showing an empty corner, and only
//! the cropped pixels are resampled, never the whole photo.

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use rayon::prelude::*;

use crate::image::RgbImage;
use crate::plan::{CropFrame, CropRect};

/// Largest straightening angle, in degrees either way.
pub const MAX_DEGREES: f32 = 15.0;
/// Below this, a photo counts as not straightened and takes the lossless path.
const EPSILON_DEGREES: f32 = 0.01;

/// An angle from the user or a file, kept finite and within ±15°.
pub fn sanitize(degrees: f32) -> f32 {
    if degrees.is_finite() {
        degrees.clamp(-MAX_DEGREES, MAX_DEGREES)
    } else {
        0.0
    }
}

/// True when `degrees` turns the photo at all.
pub fn is_straightened(degrees: f32) -> bool {
    sanitize(degrees).abs() >= EPSILON_DEGREES
}

/// Whether the `w` × `h` rectangle centred at (`cx`, `cy`) in the straightened frame lies
/// inside the photo (all coordinates relative to the photo's centre).
#[allow(clippy::too_many_arguments)]
fn fits(cx: f64, cy: f64, w: f64, h: f64, half_w: f64, half_h: f64, cos: f64, sin: f64) -> bool {
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .iter()
        .all(|(sx, sy)| {
            let (qx, qy) = (cx + sx * w / 2.0, cy + sy * h / 2.0);
            // Back into the source photo: turn by −θ.
            let (px, py) = (cos * qx + sin * qy, -sin * qx + cos * qy);
            px.abs() <= half_w + 1e-6 && py.abs() <= half_h + 1e-6
        })
}

/// The crop in the straightened frame of a `width` × `height` photo, at ratio `rw` : `rh`:
/// the largest such rectangle that shows no empty corner, scaled by the frame's zoom and
/// moved toward its centre as far as it stays inside.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn crop_rect(
    width: u32,
    height: u32,
    rw: u64,
    rh: u64,
    frame: Option<CropFrame>,
    degrees: f32,
) -> CropRect {
    let (w_img, h_img) = (f64::from(width), f64::from(height));
    let theta = f64::from(sanitize(degrees)).to_radians();
    let (cos, sin) = (theta.cos(), theta.sin());
    #[allow(clippy::cast_precision_loss)]
    let r = rw.max(1) as f64 / rh.max(1) as f64;
    let (c, s) = (cos.abs(), sin.abs());
    let w_max = (w_img / (c + s / r)).min(h_img / (s + c / r));
    let f = frame.map_or(
        CropFrame {
            cx: 0.5,
            cy: 0.5,
            zoom: 1.0,
        },
        CropFrame::sanitized,
    );
    let w = (w_max / f64::from(f.zoom)).max(1.0);
    let h = (w / r).max(1.0);
    let (half_w, half_h) = (w_img / 2.0, h_img / 2.0);
    let want = (
        f64::from(f.cx) * w_img - half_w,
        f64::from(f.cy) * h_img - half_h,
    );
    // The centre itself always fits (the rectangle is no larger than the inscribed one), so
    // look for the furthest point toward the wanted centre that still does.
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    if fits(want.0, want.1, w, h, half_w, half_h, cos, sin) {
        lo = 1.0;
    } else {
        for _ in 0..30 {
            let mid = f64::midpoint(lo, hi);
            if fits(want.0 * mid, want.1 * mid, w, h, half_w, half_h, cos, sin) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
    }
    let (cx, cy) = (want.0 * lo + half_w, want.1 * lo + half_h);
    CropRect {
        x: (cx - w / 2.0).round().max(0.0) as u32,
        y: (cy - h / 2.0).round().max(0.0) as u32,
        width: w.round() as u32,
        height: h.round() as u32,
    }
}

/// Catmull-Rom weights for a fractional offset `t` in 0…1.
fn weights(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Bicubic sample of `image` at (`x`, `y`) (pixel centres at integer + 0.5 are at integer
/// coordinates here), edges clamped.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn sample(image: &RgbImage, x: f32, y: f32) -> [u8; 3] {
    let (w, h) = (image.width as i32, image.height as i32);
    let (x0, y0) = (x.floor(), y.floor());
    let (wx, wy) = (weights(x - x0), weights(y - y0));
    let (x0, y0) = (x0 as i32, y0 as i32);
    let mut acc = [0f32; 3];
    for (j, wyj) in wy.iter().enumerate() {
        let yy = (y0 - 1 + j as i32).clamp(0, h - 1) as usize;
        let row = &image.data[yy * image.width as usize * 3..];
        for (i, wxi) in wx.iter().enumerate() {
            let xx = (x0 - 1 + i as i32).clamp(0, w - 1) as usize;
            let k = wxi * wyj;
            for (a, v) in acc.iter_mut().zip(&row[xx * 3..xx * 3 + 3]) {
                *a += k * f32::from(*v);
            }
        }
    }
    acc.map(|v| v.round().clamp(0.0, 255.0) as u8)
}

/// Renders the straightened `crop` of `source` at `out_width` × `out_height`. Strong
/// reductions are sampled at twice the output size first and finished with Lanczos3, so
/// fine detail does not alias.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn render(
    source: &RgbImage,
    crop: CropRect,
    degrees: f32,
    out_width: u32,
    out_height: u32,
) -> Option<RgbImage> {
    let theta = sanitize(degrees).to_radians();
    let (cos, sin) = (theta.cos(), theta.sin());
    let (half_w, half_h) = (source.width as f32 / 2.0, source.height as f32 / 2.0);
    let (sw, sh) = (
        out_width
            .saturating_mul(2)
            .min(crop.width)
            .max(out_width)
            .max(1),
        out_height
            .saturating_mul(2)
            .min(crop.height)
            .max(out_height)
            .max(1),
    );
    let (step_x, step_y) = (
        crop.width as f32 / sw as f32,
        crop.height as f32 / sh as f32,
    );
    let mut data = vec![0u8; sw as usize * sh as usize * 3];
    data.par_chunks_mut(sw as usize * 3)
        .enumerate()
        .for_each(|(v, row)| {
            let qy = crop.y as f32 + (v as f32 + 0.5) * step_y - half_h;
            for (u, px) in row.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                let qx = crop.x as f32 + (u as f32 + 0.5) * step_x - half_w;
                // Turn back by −θ into the source photo; pixel centres sit at +0.5.
                let x = cos * qx + sin * qy + half_w - 0.5;
                let y = -sin * qx + cos * qy + half_h - 0.5;
                px.copy_from_slice(&sample(source, x, y));
            }
        });
    let sampled = RgbImage::from_raw(sw, sh, data)?;
    if (sw, sh) == (out_width, out_height) {
        return Some(sampled);
    }
    let src = ImageRef::new(sw, sh, &sampled.data, PixelType::U8x3).ok()?;
    let mut dst = Image::new(out_width, out_height, PixelType::U8x3);
    Resizer::new()
        .resize(
            &src,
            &mut dst,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3)),
        )
        .ok()?;
    RgbImage::from_raw(out_width, out_height, dst.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_006_level_photos_keep_the_plain_crop() {
        let plain = crate::plan::crop_rect(6000, 4000, 3, 2, None);
        assert_eq!(crop_rect(6000, 4000, 3, 2, None, 0.0), plain);
        assert!(!is_straightened(0.0) && is_straightened(2.0));
        assert_eq!(sanitize(40.0), 15.0);
        assert_eq!(sanitize(f32::NAN), 0.0);
    }

    #[test]
    fn fr_006_straightened_crop_never_shows_an_empty_corner() {
        for degrees in [-15.0, -7.5, -1.0, 1.0, 4.0, 15.0] {
            for (cx, cy, zoom) in [
                (0.5, 0.5, 1.0),
                (0.0, 0.0, 1.5),
                (1.0, 1.0, 3.0),
                (0.2, 0.9, 1.0),
            ] {
                let r = crop_rect(6000, 4000, 3, 2, Some(CropFrame { cx, cy, zoom }), degrees);
                let theta = f64::from(degrees).to_radians();
                let (w, h) = (f64::from(r.width), f64::from(r.height));
                let centre = (
                    f64::from(r.x) + w / 2.0 - 3000.0,
                    f64::from(r.y) + h / 2.0 - 2000.0,
                );
                // Rounding to whole pixels may cross the edge by a pixel at most.
                let inside = fits(
                    centre.0,
                    centre.1,
                    w - 2.0,
                    h - 2.0,
                    3000.0,
                    2000.0,
                    theta.cos(),
                    theta.sin(),
                );
                assert!(inside, "{degrees}° {cx},{cy},{zoom}: {r:?}");
                assert!((w / h - 1.5).abs() < 0.01, "ratio kept: {r:?}");
            }
        }
        // Turning costs area: the level crop of a 3:2 photo is all of it, a 10° one less.
        let level = crop_rect(6000, 4000, 3, 2, None, 0.0);
        let turned = crop_rect(6000, 4000, 3, 2, None, 10.0);
        assert!(turned.width < level.width && turned.width > level.width * 2 / 3);
    }

    #[test]
    fn fr_006_rendering_turns_the_picture() {
        // A vertical white line on black: after straightening by 10°, the white pixels of the
        // top row and the bottom row sit at different columns.
        let mut src = RgbImage::new(400, 300);
        for y in 0..300 {
            let i = (y * 400 + 200) * 3;
            src.data[i..i + 6].copy_from_slice(&[255; 6]);
        }
        let crop = crop_rect(400, 300, 4, 3, None, 10.0);
        let out = render(&src, crop, 10.0, crop.width, crop.height)
            .unwrap_or_else(|| RgbImage::new(1, 1));
        let brightest = |y: u32| {
            (0..out.width)
                .max_by_key(|&x| out.data[((y * out.width + x) * 3) as usize])
                .unwrap_or(0)
        };
        let (top, bottom) = (brightest(2), brightest(out.height - 3));
        assert!(top.abs_diff(bottom) > 20, "top {top}, bottom {bottom}");
        // At 0° the same path gives back the plain crop pixel for pixel.
        let level = render(
            &src,
            CropRect {
                x: 0,
                y: 0,
                width: 400,
                height: 300,
            },
            0.0,
            400,
            300,
        );
        assert_eq!(level.map(|i| i.data), Some(src.data));
    }
}
