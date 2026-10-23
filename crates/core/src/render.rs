// SPDX-License-Identifier: GPL-3.0-only
//! Executes a `RenderPlan`: crop/resize, draw the stamp inside the safe area, then (when
//! enabled) compose the info frame around the photo (FR-040).

use std::sync::LazyLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, VariableFont, point};
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::frame::{FrameLayout, FrameLogo};
use crate::image::RgbImage;
use crate::logo::LogoImage;
use crate::plan::{FramePlan, RenderPlan, StampSpec};
use crate::settings::Corner;

static INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

fn inter(weight: f32) -> Option<FontRef<'static>> {
    let mut font = FontRef::try_from_slice(INTER).ok()?;
    font.set_variation(b"wght", weight);
    Some(font)
}

static SEMIBOLD: LazyLock<Option<FontRef<'static>>> = LazyLock::new(|| inter(600.0));
static REGULAR: LazyLock<Option<FontRef<'static>>> = LazyLock::new(|| inter(400.0));
static BLACK: LazyLock<Option<FontRef<'static>>> = LazyLock::new(|| inter(800.0));

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("resize failed: {0}")]
    Resize(String),
    #[error("crop rectangle is outside the image")]
    BadCrop,
}

/// Crops (and, for previews, downsizes) `source` per the plan, draws the stamp, and
/// composes the frame when the plan has one.
pub fn render(source: &RgbImage, plan: &RenderPlan) -> Result<RgbImage, RenderError> {
    let mut photo = crop_and_resize(source, plan)?;
    if let Some(stamp) = &plan.stamp {
        draw_stamp(&mut photo, stamp);
    }
    Ok(match &plan.frame {
        Some(frame) => compose_frame(&photo, frame),
        None => photo,
    })
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
    let src = ImageRef::new(source.width, source.height, &source.data, PixelType::U8x3)
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
        frame: None,
    };
    crop_and_resize(source, &plan)
}

/// Text advance width in pixels, including `tracking` (extra space per glyph, in em).
fn text_width(font: &FontRef<'static>, px: f32, text: &str, tracking: f32) -> f32 {
    let scaled = font.as_scaled(PxScale::from(px));
    let mut width = 0.0;
    let mut prev = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            width += scaled.kern(p, id) + tracking * px;
        }
        width += scaled.h_advance(id);
        prev = Some(id);
    }
    width
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
    let font = SEMIBOLD.as_ref()?;
    let scaled = font.as_scaled(PxScale::from(spec.font_px));
    let text_width = text_width(font, spec.font_px, &spec.text, 0.0);
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
    let (Some(font), Some(bounds)) = (
        SEMIBOLD.as_ref(),
        stamp_box(image.width, image.height, spec),
    ) else {
        return;
    };
    let shadow_offset = (spec.font_px * 0.06).max(1.0);
    let run = TextRun {
        font,
        text: &spec.text,
        px: spec.font_px,
        tracking: 0.0,
    };
    draw_text(
        image,
        &run,
        bounds.x + shadow_offset,
        bounds.y + shadow_offset,
        [0, 0, 0],
        0.55,
    );
    draw_text(image, &run, bounds.x, bounds.y, spec.color, 1.0);
}

struct TextRun<'a> {
    font: &'a FontRef<'static>,
    text: &'a str,
    px: f32,
    /// Extra space between glyphs, in em.
    tracking: f32,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
fn draw_text(
    image: &mut RgbImage,
    run: &TextRun<'_>,
    left: f32,
    top: f32,
    color: [u8; 3],
    opacity: f32,
) {
    let scale = PxScale::from(run.px);
    let scaled = run.font.as_scaled(scale);
    let baseline = top + scaled.ascent();
    let (w, h) = (image.width as i32, image.height as i32);
    let mut caret = left;
    let mut prev = None;
    for c in run.text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            caret += scaled.kern(p, id) + run.tracking * run.px;
        }
        let glyph = id.with_scale_and_position(scale, point(caret, baseline));
        caret += scaled.h_advance(id);
        prev = Some(id);
        let Some(outline) = run.font.outline_glyph(glyph) else {
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

/// Places the photo on a canvas at the print ratio and draws the info bar (FR-040).
fn compose_frame(photo: &RgbImage, frame: &FramePlan) -> RgbImage {
    let l = &frame.layout;
    let mut canvas = RgbImage::new(l.canvas_width, l.canvas_height);
    for px in canvas.data.as_chunks_mut::<3>().0 {
        *px = l.background;
    }
    blit(&mut canvas, photo, l.photo.x, l.photo.y);
    draw_frame_logo(&mut canvas, l, frame.logo.as_deref());
    for line in &l.lines {
        let font = if line.bold {
            SEMIBOLD.as_ref()
        } else {
            REGULAR.as_ref()
        };
        let Some(font) = font else { continue };
        let width = text_width(font, line.font_px, &line.text, 0.0);
        let run = TextRun {
            font,
            text: &line.text,
            px: line.font_px,
            tracking: 0.0,
        };
        draw_text(
            &mut canvas,
            &run,
            line.right_x - width,
            line.top_y,
            line.color,
            1.0,
        );
    }
    canvas
}

fn blit(canvas: &mut RgbImage, photo: &RgbImage, x: u32, y: u32) {
    let width = photo.width.min(canvas.width.saturating_sub(x)) as usize;
    let rows = photo.height.min(canvas.height.saturating_sub(y));
    for row in 0..rows {
        let src = row as usize * photo.width as usize * 3;
        let dst = ((y + row) as usize * canvas.width as usize + x as usize) * 3;
        canvas.data[dst..dst + width * 3].copy_from_slice(&photo.data[src..src + width * 3]);
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn draw_frame_logo(canvas: &mut RgbImage, l: &FrameLayout, image: Option<&LogoImage>) {
    match (&l.logo, image) {
        (FrameLogo::Text(brand), _) => {
            let Some(font) = BLACK.as_ref() else { return };
            let px = l.logo_height * 1.4;
            let scaled = font.as_scaled(PxScale::from(px));
            let mut px_fit = px;
            let width = text_width(font, px, brand, 0.06);
            if width > l.logo_max_width {
                px_fit = px * l.logo_max_width / width;
            }
            let top = l.logo_center_y - (scaled.ascent() - scaled.descent()) * (px_fit / px) / 2.0;
            let run = TextRun {
                font,
                text: brand,
                px: px_fit,
                tracking: 0.06,
            };
            draw_text(canvas, &run, l.logo_x, top, l.logo_color, 1.0);
        }
        (FrameLogo::Image(_), Some(logo)) => {
            let target_h = (l.logo_height * 1.5).round().max(1.0);
            let mut target_w = (logo.width as f32 * target_h / logo.height as f32).round();
            let mut target_h = target_h;
            if target_w > l.logo_max_width {
                target_h = (target_h * l.logo_max_width / target_w).round().max(1.0);
                target_w = l.logo_max_width.round();
            }
            let Some(scaled) = resize_rgba(logo, target_w as u32, target_h as u32) else {
                return;
            };
            let top = (l.logo_center_y - target_h / 2.0).max(0.0) as u32;
            blend_rgba(canvas, &scaled, l.logo_x.round() as u32, top);
        }
        _ => {}
    }
}

fn resize_rgba(logo: &LogoImage, width: u32, height: u32) -> Option<LogoImage> {
    let (width, height) = (width.max(1), height.max(1));
    let src = ImageRef::new(logo.width, logo.height, &logo.rgba, PixelType::U8x4).ok()?;
    let mut dst = Image::new(width, height, PixelType::U8x4);
    // Alpha is premultiplied internally by the resizer (`mul_div_alpha`), avoiding halos.
    Resizer::new()
        .resize(
            &src,
            &mut dst,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3)),
        )
        .ok()?;
    Some(LogoImage {
        width,
        height,
        rgba: dst.into_vec(),
    })
}

fn blend_rgba(canvas: &mut RgbImage, logo: &LogoImage, x: u32, y: u32) {
    for row in 0..logo.height.min(canvas.height.saturating_sub(y)) {
        for col in 0..logo.width.min(canvas.width.saturating_sub(x)) {
            let s = (row as usize * logo.width as usize + col as usize) * 4;
            let d = ((y + row) as usize * canvas.width as usize + (x + col) as usize) * 3;
            let alpha = u32::from(logo.rgba[s + 3]);
            for c in 0..3 {
                let src = u32::from(logo.rgba[s + c]);
                let dst = u32::from(canvas.data[d + c]);
                // INVARIANT: the weighted average of two u8 values fits in u8.
                canvas.data[d + c] =
                    u8::try_from((src * alpha + dst * (255 - alpha) + 127) / 255).unwrap_or(255);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{FrameFit, FrameSettings, build_text};
    use crate::library::ShotInfo;
    use crate::plan::{FrameRequest, SAFE_AREA_MM, plan};
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
            let p = plan(6000, 4000, capture(), &settings, print, None, None);
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
        let p = plan(1500, 1000, capture(), &settings, print, None, None);
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
        let p = plan(30, 20, None, &settings, PrintSize::ALL[6], None, None); // 15x15 square
        let out = render(&source, &p).unwrap_or_else(|_| RgbImage::new(0, 0));
        assert_eq!((out.width, out.height), (20, 20));
        // First output pixel is source pixel (5, 0).
        assert_eq!(out.data[..3], source.data[15..18]);
    }

    fn framed(fit: FrameFit, w: u32, h: u32, print: PrintSize) -> (RgbImage, RenderPlan) {
        let shot = ShotInfo {
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            iso: Some(200),
            ..ShotInfo::default()
        };
        let frame_settings = FrameSettings {
            enabled: true,
            fit,
            ..FrameSettings::default()
        };
        let request = FrameRequest {
            text: build_text(&shot, None, &frame_settings, capture(), None),
            settings: frame_settings,
            logo: None,
        };
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let source = RgbImage::new(w, h);
        let p = plan(w, h, capture(), &settings, print, None, Some(&request));
        let out = render(&source, &p).unwrap_or_else(|_| RgbImage::new(0, 0));
        (out, p)
    }

    #[test]
    fn fr_040_rendered_output_has_print_ratio() {
        for print in [PrintSize::ALL[1], PrintSize::ALL[4], PrintSize::ALL[6]] {
            for fit in [FrameFit::Crop, FrameFit::Fit] {
                for (w, h) in [(1200, 900), (900, 1200)] {
                    let (out, p) = framed(fit, w, h, print);
                    assert_eq!((out.width, out.height), p.output_size());
                    let ratio =
                        f64::from(out.width.max(out.height)) / f64::from(out.width.min(out.height));
                    let target = f64::from(print.long_mm) / f64::from(print.short_mm);
                    assert!((ratio - target).abs() / target < 0.002, "{}", print.id);
                }
            }
        }
    }

    #[test]
    fn fr_040_bar_is_painted_and_text_drawn() {
        let (out, p) = framed(FrameFit::Crop, 1200, 900, PrintSize::ALL[1]);
        let Some(bar) = p.frame.as_ref().map(|f| f.layout.bar) else {
            panic!("frame expected");
        };
        // The photo area is black (source), the bar background is light.
        let px = |x: u32, y: u32| out.data[((y * out.width + x) * 3) as usize];
        assert_eq!(px(10, 10), 0);
        assert!(px(5, bar.y + 2) > 200, "bar background");
        // Some dark text pixels exist inside the bar.
        let dark = (bar.y..bar.y + bar.height)
            .flat_map(|y| (0..out.width).map(move |x| (x, y)))
            .filter(|&(x, y)| px(x, y) < 80)
            .count();
        assert!(dark > 50, "frame text drawn: {dark}");
    }
}
