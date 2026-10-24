// SPDX-License-Identifier: GPL-3.0-only
//! Executes a `RenderPlan`: crop/resize, draw the stamp inside the safe area, then (when
//! enabled) compose the info frame around the photo (FR-040).

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, FontRef, PxScale, ScaleFont, VariableFont};
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::fonts::LoadedFont;
use crate::frame::{FrameLayout, FrameLogo};
use crate::image::RgbImage;
use crate::logo::LogoImage;
use crate::plan::{FramePlan, RenderPlan};
use crate::text::{TextRun, draw_text, text_width};

static INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

/// Bundled Inter at a given weight, as a `FontArc` so built-in and user fonts (FR-042)
/// share one rendering path.
fn inter(weight: f32) -> Option<FontArc> {
    let mut font = FontRef::try_from_slice(INTER).ok()?;
    font.set_variation(b"wght", weight);
    Some(FontArc::new(font))
}

static SEMIBOLD: LazyLock<Option<FontArc>> = LazyLock::new(|| inter(600.0));
static REGULAR: LazyLock<Option<FontArc>> = LazyLock::new(|| inter(400.0));
static BLACK: LazyLock<Option<FontArc>> = LazyLock::new(|| inter(800.0));

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
        crate::stamp::draw(&mut photo, stamp);
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

/// Places the photo on a canvas at the print ratio and draws the info bar (FR-040).
fn compose_frame(photo: &RgbImage, frame: &FramePlan) -> RgbImage {
    let l = &frame.layout;
    let mut canvas = RgbImage::new(l.canvas_width, l.canvas_height);
    for px in canvas.data.as_chunks_mut::<3>().0 {
        *px = l.background;
    }
    blit(&mut canvas, photo, l.photo.x, l.photo.y);
    draw_frame_logo(&mut canvas, l, frame.logo.as_deref(), frame.font.as_ref());
    for line in &l.lines {
        let font = match &frame.font {
            Some(user) => Some(user.font.clone()),
            None if line.bold => SEMIBOLD.clone(),
            None => REGULAR.clone(),
        };
        let Some(font) = font else { continue };
        let width = text_width(&font, line.font_px, &line.text, 0.0);
        let run = TextRun {
            font: &font,
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
fn draw_frame_logo(
    canvas: &mut RgbImage,
    l: &FrameLayout,
    image: Option<&LogoImage>,
    user_font: Option<&LoadedFont>,
) {
    match (&l.logo, image) {
        (FrameLogo::Text(brand), _) => {
            let Some(font) = user_font.map(|f| f.font.clone()).or_else(|| BLACK.clone()) else {
                return;
            };
            let font = &font;
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
            // Wordmarks sit at about the cap height of the frame text; square marks and
            // badges get more height to read at the same weight.
            let wide = logo.width as f32 > 2.5 * logo.height as f32;
            let target_h = (l.logo_height * if wide { 0.85 } else { 1.5 })
                .round()
                .max(1.0);
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
    use crate::plan::{FrameRequest, PlanInput, SAFE_AREA_MM, plan};
    use crate::print_size::PrintSize;
    use crate::settings::Corner;
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
            let p = plan(&PlanInput {
                width: 6000,
                height: 4000,
                capture: capture(),
                settings: &settings,
                print,
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
            });
            let spec = p.stamp.as_ref();
            let lines = spec.map(|s| crate::stamp::layout(p.out_width, p.out_height, s));
            let Some(b) = lines.and_then(|l| l.into_iter().next()) else {
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
        let p = plan(&PlanInput {
            width: 1500,
            height: 1000,
            capture: capture(),
            settings: &settings,
            print,
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
        });
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
        let p = plan(&PlanInput {
            width: 30,
            height: 20,
            capture: None,
            settings: &settings,
            print: PrintSize::ALL[6],
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
        }); // 15x15 square
        let out = render(&source, &p).unwrap_or_else(|_| RgbImage::new(0, 0));
        assert_eq!((out.width, out.height), (20, 20));
        // First output pixel is source pixel (5, 0).
        assert_eq!(out.data[..3], source.data[15..18]);
    }

    fn framed(fit: FrameFit, w: u32, h: u32, print: PrintSize) -> (RgbImage, RenderPlan) {
        framed_with(fit, w, h, print)
    }

    fn framed_with(fit: FrameFit, w: u32, h: u32, print: PrintSize) -> (RgbImage, RenderPlan) {
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
            font: None,
        };
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let source = RgbImage::new(w, h);
        let p = plan(&PlanInput {
            width: w,
            height: h,
            capture: capture(),
            settings: &settings,
            print,
            max_long_edge: None,
            frame: Some(&request),
            crop: None,
            place: None,
        });
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

    #[test]
    fn fr_043_custom_ratio_drives_crop_and_frame() {
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        // 4 × 6 in (102 × 152 mm) and a 13 × 13 cm square.
        for (print, target) in [
            (PrintSize::custom(102, 152), 152.0 / 102.0),
            (PrintSize::custom(130, 130), 1.0),
        ] {
            let (out, p) = framed_with(FrameFit::Crop, 1200, 900, print);
            let ratio = f64::from(out.width.max(out.height)) / f64::from(out.width.min(out.height));
            assert!(
                (ratio - target).abs() / target < 0.005,
                "{ratio} vs {target}"
            );
            assert_eq!((out.width, out.height), p.output_size());
        }
    }
}
