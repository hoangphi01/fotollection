// SPDX-License-Identifier: GPL-3.0-only
//! Executes a `RenderPlan`: crop/resize, then draw the stamp inside the safe area.

use std::sync::LazyLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, VariableFont, point};
use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::image::RgbImage;
use crate::plan::{RenderPlan, StampSpec};
use crate::settings::Corner;

static INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

static FONT: LazyLock<Option<FontRef<'static>>> = LazyLock::new(|| {
    let mut font = FontRef::try_from_slice(INTER).ok()?;
    font.set_variation(b"wght", 600.0);
    Some(font)
});

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("resize failed: {0}")]
    Resize(String),
    #[error("crop rectangle is outside the image")]
    BadCrop,
}

/// Crops (and, for previews, downsizes) `source` per the plan, then draws the stamp.
pub fn render(source: &RgbImage, plan: &RenderPlan) -> Result<RgbImage, RenderError> {
    let mut out = crop_and_resize(source, plan)?;
    if let Some(stamp) = &plan.stamp {
        draw_stamp(&mut out, stamp);
    }
    Ok(out)
}

fn crop_and_resize(source: &RgbImage, plan: &RenderPlan) -> Result<RgbImage, RenderError> {
    let c = plan.crop;
    if c.x + c.width > source.width || c.y + c.height > source.height {
        return Err(RenderError::BadCrop);
    }
    if (plan.out_width, plan.out_height) == (c.width, c.height) {
        // Full-resolution export: copy rows, no resampling (FR-008).
        let mut out = Vec::with_capacity(c.width as usize * c.height as usize * 3);
        let row_bytes = source.width as usize * 3;
        for y in c.y..c.y + c.height {
            let start = y as usize * row_bytes + c.x as usize * 3;
            out.extend_from_slice(&source.data[start..start + c.width as usize * 3]);
        }
        return RgbImage::from_raw(c.width, c.height, out).ok_or(RenderError::BadCrop);
    }
    let src = fast_image_resize::images::ImageRef::new(
        source.width,
        source.height,
        &source.data,
        PixelType::U8x3,
    )
    .map_err(|e| RenderError::Resize(e.to_string()))?;
    let mut dst = Image::new(plan.out_width, plan.out_height, PixelType::U8x3);
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
        .crop(
            f64::from(c.x),
            f64::from(c.y),
            f64::from(c.width),
            f64::from(c.height),
        );
    Resizer::new()
        .resize(&src, &mut dst, &options)
        .map_err(|e| RenderError::Resize(e.to_string()))?;
    RgbImage::from_raw(plan.out_width, plan.out_height, dst.into_vec()).ok_or(RenderError::BadCrop)
}

/// Downsizes so the long edge is at most `long_edge` (no-op if already smaller).
pub fn fit_long_edge(source: &RgbImage, long_edge: u32) -> Result<RgbImage, RenderError> {
    if source.long_edge() <= long_edge {
        return Ok(source.clone());
    }
    let scale = f64::from(long_edge) / f64::from(source.long_edge());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let dim = |v: u32| ((f64::from(v) * scale).round() as u32).max(1);
    let plan = RenderPlan {
        print: crate::print_size::PrintSize::ALL[1],
        crop: crate::plan::CropRect {
            x: 0,
            y: 0,
            width: source.width,
            height: source.height,
        },
        out_width: dim(source.width),
        out_height: dim(source.height),
        stamp: None,
    };
    crop_and_resize(source, &plan)
}

/// Pixel bounds of laid-out stamp text: `(x, y, width, height)` in output pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StampBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Where the stamp text goes for an image of `width × height` (FR-004, FR-028).
#[allow(clippy::cast_precision_loss)]
pub fn stamp_box(width: u32, height: u32, spec: &StampSpec) -> Option<StampBox> {
    let font = FONT.as_ref()?;
    let scaled = font.as_scaled(PxScale::from(spec.font_px));
    let mut text_width = 0.0;
    let mut prev = None;
    for c in spec.text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            text_width += scaled.kern(p, id);
        }
        text_width += scaled.h_advance(id);
        prev = Some(id);
    }
    let text_height = scaled.ascent() - scaled.descent();
    let (w, h, m) = (width as f32, height as f32, spec.margin_px);
    let x = match spec.corner {
        Corner::BottomRight | Corner::TopRight => w - m - text_width,
        Corner::BottomLeft | Corner::TopLeft => m,
    };
    let y = match spec.corner {
        Corner::BottomRight | Corner::BottomLeft => h - m - text_height,
        Corner::TopRight | Corner::TopLeft => m,
    };
    Some(StampBox {
        x,
        y,
        width: text_width,
        height: text_height,
    })
}

fn draw_stamp(image: &mut RgbImage, spec: &StampSpec) {
    let (Some(font), Some(bounds)) = (FONT.as_ref(), stamp_box(image.width, image.height, spec))
    else {
        return;
    };
    let shadow_offset = (spec.font_px * 0.06).max(1.0);
    let shadow = [0, 0, 0];
    draw_text(
        image,
        font,
        spec,
        bounds.x + shadow_offset,
        bounds.y + shadow_offset,
        shadow,
        0.55,
    );
    draw_text(image, font, spec, bounds.x, bounds.y, spec.color, 1.0);
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
fn draw_text(
    image: &mut RgbImage,
    font: &FontRef<'static>,
    spec: &StampSpec,
    left: f32,
    top: f32,
    color: [u8; 3],
    opacity: f32,
) {
    let scale = PxScale::from(spec.font_px);
    let scaled = font.as_scaled(scale);
    let baseline = top + scaled.ascent();
    let (w, h) = (image.width as i32, image.height as i32);
    let mut caret = left;
    let mut prev = None;
    for c in spec.text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            caret += scaled.kern(p, id);
        }
        let glyph = id.with_scale_and_position(scale, point(caret, baseline));
        caret += scaled.h_advance(id);
        prev = Some(id);
        let Some(outline) = font.outline_glyph(glyph) else {
            continue;
        };
        let b = outline.px_bounds();
        outline.draw(|gx, gy, coverage| {
            let (x, y) = (b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32);
            if x < 0 || y < 0 || x >= w || y >= h {
                return;
            }
            let alpha = (coverage * opacity).clamp(0.0, 1.0);
            let i = (y as usize * image.width as usize + x as usize) * 3;
            for (channel, target) in image.data[i..i + 3].iter_mut().zip(color) {
                let blended = f32::from(*channel) * (1.0 - alpha) + f32::from(target) * alpha;
                *channel = blended.round() as u8;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{SAFE_AREA_MM, plan};
    use crate::print_size::PrintSize;
    use crate::settings::RenderSettings;

    fn capture() -> Option<chrono::NaiveDateTime> {
        chrono::NaiveDate::from_ymd_opt(2026, 7, 14).and_then(|d| d.and_hms_opt(10, 0, 0))
    }

    #[test]
    fn fr_004_stamp_inside_safe_area_for_every_corner() {
        let print = PrintSize::ALL[1]; // 10x15
        for corner in [
            Corner::BottomRight,
            Corner::BottomLeft,
            Corner::TopRight,
            Corner::TopLeft,
        ] {
            let mut settings = RenderSettings::default();
            settings.stamp.corner = corner;
            let p = plan(6000, 4000, capture(), &settings, print, None);
            let spec = p.stamp.as_ref();
            let b = spec.and_then(|s| stamp_box(p.out_width, p.out_height, s));
            let Some(b) = b else {
                panic!("stamp expected for {corner:?}");
            };
            #[allow(clippy::cast_precision_loss)]
            let safe = SAFE_AREA_MM * p.out_width as f32 / 150.0;
            #[allow(clippy::cast_precision_loss)]
            let (w, h) = (p.out_width as f32, p.out_height as f32);
            assert!(b.x >= safe && b.y >= safe, "{corner:?}: {b:?}");
            assert!(
                b.x + b.width <= w - safe && b.y + b.height <= h - safe,
                "{corner:?}: {b:?}"
            );
        }
    }

    #[test]
    fn fr_004_stamp_changes_pixels_only_near_its_box() {
        let print = PrintSize::ALL[1];
        let settings = RenderSettings::default();
        let source = RgbImage::new(1500, 1000);
        let p = plan(1500, 1000, capture(), &settings, print, None);
        let out = render(&source, &p).unwrap_or_else(|_| RgbImage::new(0, 0));
        let changed = out.data.iter().filter(|&&v| v != 0).count();
        assert!(changed > 0, "stamp drew nothing");
        // Top-left quadrant must stay untouched for a bottom-right stamp.
        let untouched = (0..500).all(|y| (0..700).all(|x| out.data[(y * 1500 + x) * 3] == 0));
        assert!(untouched);
    }

    #[test]
    fn fr_008_full_resolution_crop_is_pixel_exact() {
        let mut source = RgbImage::new(30, 20);
        for (i, v) in source.data.iter_mut().enumerate() {
            *v = u8::try_from(i % 251).unwrap_or(0);
        }
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let p = plan(30, 20, None, &settings, PrintSize::ALL[6], None); // 15x15 square
        let out = render(&source, &p).unwrap_or_else(|_| RgbImage::new(0, 0));
        assert_eq!((out.width, out.height), (20, 20));
        // First output pixel is source pixel (5, 0).
        assert_eq!(out.data[..3], source.data[15..18]);
    }
}
