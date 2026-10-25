// SPDX-License-Identifier: GPL-3.0-only
//! `RenderPlan`: a pure description of how one output photo is produced. Preview and
//! export consume the same plan, so what the user sees is what gets printed
//! (ARCHITECTURE.md §4).

use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::fonts::LoadedFont;
use crate::frame::{self, FrameFit, FrameLayout, FrameSettings, FrameText};
use crate::logo::LogoImage;
use crate::print_size::PrintSize;
use crate::settings::{Corner, DateFormat, RenderSettings};

/// Default lab trim margin per edge (FR-028), plus breathing room for the stamp.
pub const SAFE_AREA_MM: f32 = 2.0;
const STAMP_INSET_MM: f32 = 2.0;
/// Stamp text size (em height) on the print.
const STAMP_SIZE_MM: f32 = 3.6;

/// A rectangle in source pixels (after orientation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StampSpec {
    pub text: String,
    pub color: [u8; 3],
    pub corner: Corner,
    /// Font em size in output pixels.
    pub font_px: f32,
    /// Distance from each output edge to the stamp, in output pixels.
    pub margin_px: f32,
    /// The user's line under the date (FR-036).
    pub caption: Option<String>,
}

/// Info frame to compose around the photo (FR-040).
#[derive(Debug, Clone, PartialEq)]
pub struct FramePlan {
    pub layout: FrameLayout,
    pub logo: Option<Arc<LogoImage>>,
    pub font: Option<LoadedFont>,
}

/// What the caller knows about the frame for one photo.
#[derive(Debug, Clone)]
pub struct FrameRequest {
    pub settings: FrameSettings,
    pub text: FrameText,
    /// The camera's user logo, when the profile uses one.
    pub logo: Option<Arc<LogoImage>>,
    /// A user font for the frame text (FR-042).
    pub font: Option<LoadedFont>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlan {
    pub print: PrintSize,
    /// Source pixels to use (after orientation).
    pub crop: CropRect,
    /// Size of the rendered photo. Without a frame this is the whole output.
    pub out_width: u32,
    pub out_height: u32,
    pub stamp: Option<StampSpec>,
    /// When set, the photo is placed on a canvas at the print ratio with an info bar.
    pub frame: Option<FramePlan>,
    /// Degrees the photo is straightened by (FR-006); `crop` is then in the straightened
    /// frame.
    pub straighten: f32,
}

impl RenderPlan {
    /// Final output size, including the frame canvas.
    pub fn output_size(&self) -> (u32, u32) {
        self.frame
            .as_ref()
            .map_or((self.out_width, self.out_height), |f| {
                (f.layout.canvas_width, f.layout.canvas_height)
            })
    }
}

/// Largest centred crop of `width × height` with the print's aspect ratio, matching the
/// photo's orientation (FR-006). Square prints yield a square crop.
pub fn center_crop(width: u32, height: u32, print: PrintSize) -> CropRect {
    let (long, short) = (u64::from(print.long_mm), u64::from(print.short_mm));
    // Target ratio as (rw : rh) in the photo's own orientation.
    let (rw, rh) = if width >= height {
        (long, short)
    } else {
        (short, long)
    };
    center_crop_to(width, height, rw, rh)
}

/// Largest centred crop of `width × height` with the ratio `rw : rh`.
pub fn center_crop_to(width: u32, height: u32, rw: u64, rh: u64) -> CropRect {
    let (rw, rh) = (rw.max(1), rh.max(1));
    let (w, h) = (u64::from(width), u64::from(height));
    let (cw, ch) = if w * rh >= h * rw {
        ((h * rw + rh / 2) / rh, h) // too wide: trim the sides
    } else {
        (w, (w * rh + rw / 2) / rw) // too tall: trim top and bottom
    };
    let (cw, ch) = (cw.min(w), ch.min(h));
    CropRect {
        x: u32::try_from((w - cw) / 2).unwrap_or(0),
        y: u32::try_from((h - ch) / 2).unwrap_or(0),
        width: u32::try_from(cw).unwrap_or(width),
        height: u32::try_from(ch).unwrap_or(height),
    }
}

/// Print resolution quality bands (FR-026, PRD §5 "Minimum resolution per size").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DpiBand {
    /// ≥ 300 DPI
    Excellent,
    /// ≥ 200 DPI
    Good,
    /// ≥ 150 DPI
    Fair,
    /// < 150 DPI
    Poor,
}

/// Effective DPI of the centred crop of a `width × height` photo printed at `print`.
pub fn effective_dpi(width: u32, height: u32, print: PrintSize) -> (u32, DpiBand) {
    let crop = center_crop(width, height, print);
    let long_px = f64::from(crop.width.max(crop.height));
    let long_in = f64::from(print.long_mm) / 25.4;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let dpi = (long_px / long_in).round() as u32;
    let band = match dpi {
        300.. => DpiBand::Excellent,
        200..300 => DpiBand::Good,
        150..200 => DpiBand::Fair,
        _ => DpiBand::Poor,
    };
    (dpi, band)
}

pub fn format_date(dt: NaiveDateTime, format: DateFormat) -> String {
    match format {
        DateFormat::Iso => dt.format("%Y-%m-%d").to_string(),
        DateFormat::Dmy => dt.format("%d.%m.%Y").to_string(),
        DateFormat::Retro => dt.format("'%y %m %d").to_string(),
    }
}

/// User crop as a centre and zoom on the upright photo (FR-006). The crop rectangle is
/// derived at render time for whatever target ratio applies (print size, or the photo area
/// of a framed print), so a crop survives size changes (FR-025, DATABASE.md N4).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct CropFrame {
    /// Centre, 0–1 of the upright photo's width and height.
    pub cx: f32,
    pub cy: f32,
    /// 1 = the largest crop at the target ratio; up to `MAX_ZOOM`.
    pub zoom: f32,
}

pub const MAX_ZOOM: f32 = 8.0;

impl CropFrame {
    /// The same crop after the photo turns a quarter turn (FR-006): the centre moves with
    /// the picture, the zoom stays.
    #[must_use]
    pub fn turned(self, clockwise: bool) -> Self {
        let (cx, cy) = if clockwise {
            (1.0 - self.cy, self.cx)
        } else {
            (self.cy, 1.0 - self.cx)
        };
        Self {
            cx,
            cy,
            zoom: self.zoom,
        }
    }

    /// Clamps values coming over IPC or from a project file (SECURITY.md §2).
    #[must_use]
    pub fn sanitized(self) -> Self {
        let unit = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.5
            }
        };
        Self {
            cx: unit(self.cx),
            cy: unit(self.cy),
            zoom: if self.zoom.is_finite() {
                self.zoom.clamp(1.0, MAX_ZOOM)
            } else {
                1.0
            },
        }
    }
}

/// The crop rectangle for ratio `rw : rh`: centred and as large as possible, or as set by
/// the user's `CropFrame`, always kept inside the photo.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn crop_rect(width: u32, height: u32, rw: u64, rh: u64, frame: Option<CropFrame>) -> CropRect {
    let base = center_crop_to(width, height, rw, rh);
    let Some(f) = frame.map(CropFrame::sanitized) else {
        return base;
    };
    let w = (f64::from(base.width) / f64::from(f.zoom)).round().max(1.0);
    let h = (f64::from(base.height) / f64::from(f.zoom))
        .round()
        .max(1.0);
    let x = (f64::from(f.cx) * f64::from(width) - w / 2.0).clamp(0.0, f64::from(width) - w);
    let y = (f64::from(f.cy) * f64::from(height) - h / 2.0).clamp(0.0, f64::from(height) - h);
    CropRect {
        x: x.round() as u32,
        y: y.round() as u32,
        width: w as u32,
        height: h as u32,
    }
}

/// The crop at ratio `rw` : `rh` for `input`, straightened when it is (FR-006).
fn crop_at(input: &PlanInput<'_>, rw: u64, rh: u64) -> CropRect {
    if crate::straighten::is_straightened(input.straighten) {
        crate::straighten::crop_rect(
            input.width,
            input.height,
            rw,
            rh,
            input.crop,
            input.straighten,
        )
    } else {
        crop_rect(input.width, input.height, rw, rh, input.crop)
    }
}

/// Everything needed to plan one output photo.
#[derive(Debug, Clone, Copy)]
pub struct PlanInput<'a> {
    /// Upright source size.
    pub width: u32,
    pub height: u32,
    pub capture: Option<NaiveDateTime>,
    pub settings: &'a RenderSettings,
    pub print: PrintSize,
    /// Downsizes the output for previews; `None` keeps full resolution.
    pub max_long_edge: Option<u32>,
    pub frame: Option<&'a FrameRequest>,
    pub crop: Option<CropFrame>,
    /// The photo's place, already written in the chosen format (FR-005).
    pub place: Option<&'a str>,
    /// Straightening in degrees, ±15 (FR-006); 0 is the lossless path.
    pub straighten: f32,
}

/// Builds the plan for one photo.
pub fn plan(input: &PlanInput<'_>) -> RenderPlan {
    if let Some(request) = input.frame.filter(|f| f.settings.enabled) {
        return plan_framed(input, request);
    }
    let (width, height, print) = (input.width, input.height, input.print);
    let (long, short) = (u64::from(print.long_mm), u64::from(print.short_mm));
    let (rw, rh) = if print.is_original() {
        // The exact pixel ratio: Original never loses a pixel to mm rounding (FR-045).
        (u64::from(width), u64::from(height))
    } else if width >= height {
        (long, short)
    } else {
        (short, long)
    };
    let crop = crop_at(input, rw, rh);
    let (mut out_width, mut out_height) = (crop.width, crop.height);
    if let Some(limit) = input.max_long_edge {
        let long = out_width.max(out_height);
        if long > limit && long > 0 {
            let scale = f64::from(limit) / f64::from(long);
            out_width = scale_dim(out_width, scale);
            out_height = scale_dim(out_height, scale);
        }
    }

    #[allow(clippy::cast_precision_loss)] // print sizes and pixel counts are far below 2^24
    let px_per_mm = out_width.max(out_height) as f32 / print.long_mm as f32;
    RenderPlan {
        print,
        crop,
        out_width,
        out_height,
        stamp: stamp_spec(input, px_per_mm),
        frame: None,
        straighten: crate::straighten::sanitize(input.straighten),
    }
}

fn stamp_spec(input: &PlanInput<'_>, px_per_mm: f32) -> Option<StampSpec> {
    let settings = input.settings;
    if !settings.stamp.enabled {
        return None;
    }
    // A photo with no date at all still gets its place and the caption the user typed.
    let date = input
        .capture
        .map(|dt| format_date(dt, settings.stamp.format));
    let place = input.place.filter(|_| settings.stamp.place);
    let text = [date.as_deref(), place]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("  ");
    let caption = settings.stamp.caption.trim();
    if text.is_empty() && caption.is_empty() {
        return None;
    }
    Some(StampSpec {
        text,
        color: settings.stamp.color.rgb(),
        corner: settings.stamp.corner,
        font_px: STAMP_SIZE_MM * px_per_mm,
        margin_px: (SAFE_AREA_MM + STAMP_INSET_MM) * px_per_mm,
        caption: (!caption.is_empty()).then(|| caption.to_owned()),
    })
}

/// Crop keeps the print ratio and fills the space above the info bar; Fit shows the whole
/// photo and adds the bar below it, so the print grows longer (FR-040).
fn plan_framed(input: &PlanInput<'_>, request: &FrameRequest) -> RenderPlan {
    let (width, height, print) = (input.width, input.height, input.print);
    let long = input
        .max_long_edge
        .map_or(width.max(height), |m| m.min(width.max(height)));
    let (cw, ch) = frame::canvas_size(width, height, print, long, request.settings.fit);
    let lay = |text: &frame::FrameText| {
        frame::layout(cw, ch, width, height, &request.settings, text, print)
    };
    let (logo, font) = (request.logo.as_deref(), request.font.as_ref());
    let mut layout = lay(&request.text);
    // A long lens name that would run into the logo goes on a line of its own.
    if !crate::render::frame_text_clears_logo(&layout, logo, font)
        && let Some(wrapped) = request.text.wrapped()
    {
        layout = lay(&wrapped);
    }
    crate::render::shrink_frame_text_beside_logo(&mut layout, logo, font);
    let photo = layout.photo;
    let crop = match request.settings.fit {
        FrameFit::Crop => crop_at(input, u64::from(photo.width), u64::from(photo.height)),
        // Fit shows the whole photo; a straightened one shows all it can without corners.
        FrameFit::Fit if crate::straighten::is_straightened(input.straighten) => {
            crate::straighten::crop_rect(
                width,
                height,
                u64::from(width),
                u64::from(height),
                None,
                input.straighten,
            )
        }
        FrameFit::Fit => CropRect {
            x: 0,
            y: 0,
            width,
            height,
        },
    };
    #[allow(clippy::cast_precision_loss)]
    let px_per_mm = cw.max(ch) as f32 / print.long_mm as f32;
    RenderPlan {
        print,
        crop,
        out_width: photo.width,
        out_height: photo.height,
        stamp: stamp_spec(input, px_per_mm),
        frame: Some(FramePlan {
            layout,
            logo: request.logo.clone(),
            font: request.font.clone(),
        }),
        straighten: crate::straighten::sanitize(input.straighten),
    }
}

fn scale_dim(value: u32, scale: f64) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let scaled = (f64::from(value) * scale).round() as u32;
    scaled.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_006_crop_centre_follows_a_quarter_turn() {
        let crop = CropFrame {
            cx: 0.2,
            cy: 0.3,
            zoom: 1.5,
        };
        // Top-left area moves to the top-right after a clockwise turn.
        assert_eq!(
            crop.turned(true),
            CropFrame {
                cx: 0.7,
                cy: 0.2,
                zoom: 1.5
            }
        );
        assert_eq!(crop.turned(true).turned(false), crop);
        let four = (0..4).fold(crop, |c, _| c.turned(true));
        assert!((four.cx - crop.cx).abs() < 1e-6 && (four.cy - crop.cy).abs() < 1e-6);
    }
    use crate::settings::RenderSettings;

    fn size(id: &str) -> PrintSize {
        PrintSize::by_id(id).unwrap_or(PrintSize::ALL[1])
    }

    fn ratio(c: CropRect) -> f64 {
        f64::from(c.width.max(c.height)) / f64::from(c.width.min(c.height))
    }

    #[test]
    fn fr_006_crop_matches_ratio_for_portrait_and_landscape() {
        // 4:3 camera sensor, landscape and portrait, onto 10×15 (2:3).
        let landscape = center_crop(6000, 4500, size("10x15"));
        assert_eq!((landscape.width, landscape.height), (6000, 4000));
        assert_eq!((landscape.x, landscape.y), (0, 250));

        let portrait = center_crop(4500, 6000, size("10x15"));
        assert_eq!((portrait.width, portrait.height), (4000, 6000));
        assert_eq!((portrait.x, portrait.y), (250, 0));

        for id in ["9x13", "13x18", "15x20", "15x15", "5x15"] {
            let print = size(id);
            let c = center_crop(6000, 4000, print);
            let target = f64::from(print.long_mm) / f64::from(print.short_mm);
            assert!((ratio(c) - target).abs() < 0.002, "{id}: {}", ratio(c));
            assert!(c.x + c.width <= 6000 && c.y + c.height <= 4000);
        }
    }

    #[test]
    fn fr_006_crop_frame_clamped_inside_photo() {
        // Centre pushed to a corner: the rectangle slides back inside.
        let corner = CropFrame {
            cx: 0.0,
            cy: 1.0,
            zoom: 2.0,
        };
        let r = crop_rect(6000, 4000, 3, 2, Some(corner));
        assert_eq!((r.x, r.width, r.height), (0, 3000, 2000));
        assert_eq!(r.y + r.height, 4000);
        // Garbage is sanitized rather than trusted.
        let bad = CropFrame {
            cx: f32::NAN,
            cy: 7.0,
            zoom: 1000.0,
        }
        .sanitized();
        assert_eq!((bad.cx, bad.cy, bad.zoom), (0.5, 1.0, MAX_ZOOM));
    }

    #[test]
    fn fr_006_zoom_and_pan_keep_ratio() {
        for zoom in [1.0, 1.5, 2.7, 8.0] {
            for (cx, cy) in [(0.5, 0.5), (0.2, 0.8), (0.9, 0.1)] {
                let r = crop_rect(4608, 3456, 3, 2, Some(CropFrame { cx, cy, zoom }));
                let ratio = f64::from(r.width) / f64::from(r.height);
                assert!((ratio - 1.5).abs() < 0.01, "zoom {zoom}: {ratio}");
                assert!(r.x + r.width <= 4608 && r.y + r.height <= 3456);
            }
        }
    }

    #[test]
    fn fr_025_crop_survives_size_change() {
        // The same centre/zoom gives the same centre on another print ratio.
        let f = CropFrame {
            cx: 0.4,
            cy: 0.45,
            zoom: 1.5,
        };
        for (rw, rh) in [(3, 2), (4, 3), (1, 1)] {
            let r = crop_rect(6000, 4000, rw, rh, Some(f));
            let cx = (f64::from(r.x) + f64::from(r.width) / 2.0) / 6000.0;
            let cy = (f64::from(r.y) + f64::from(r.height) / 2.0) / 4000.0;
            assert!(
                (cx - 0.4).abs() < 0.01 && (cy - 0.45).abs() < 0.01,
                "{rw}:{rh}"
            );
        }
    }

    #[test]
    fn fr_026_effective_dpi_bands() {
        // 16 MP 4:3 onto 10x15: crop 4608x3072, long edge 150 mm = 5.906 in → 780 DPI.
        assert_eq!(
            effective_dpi(4608, 3456, size("10x15")),
            (780, DpiBand::Excellent)
        );
        // 20x30 from 12 MP (4000x3000): crop 4000x2667 over 11.81 in → 339 DPI.
        assert_eq!(
            effective_dpi(4000, 3000, size("20x30")).1,
            DpiBand::Excellent
        );
        // A 1600 px web image on 20x30 → 135 DPI.
        assert_eq!(effective_dpi(1600, 1067, size("20x30")).1, DpiBand::Poor);
        assert_eq!(effective_dpi(2600, 1733, size("20x30")).1, DpiBand::Good);
    }

    #[test]
    fn fr_004_formats_dates() {
        let dt = chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
            .and_then(|d| d.and_hms_opt(9, 5, 0))
            .unwrap_or_default();
        assert_eq!(format_date(dt, DateFormat::Iso), "2026-07-04");
        assert_eq!(format_date(dt, DateFormat::Dmy), "04.07.2026");
        assert_eq!(format_date(dt, DateFormat::Retro), "'26 07 04");
    }

    #[test]
    fn fr_007_preview_plan_scales_down_and_keeps_ratio() {
        let settings = RenderSettings::default();
        let p = plan(&PlanInput {
            width: 6000,
            height: 4500,
            capture: None,
            settings: &settings,
            print: size("10x15"),
            max_long_edge: Some(1600),
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
        });
        assert_eq!((p.out_width, p.out_height), (1600, 1067));
        assert!(p.stamp.is_none(), "no capture date, no stamp");
    }

    #[test]
    fn fr_036_a_photo_without_a_date_still_prints_its_caption() {
        let mut settings = RenderSettings::default();
        settings.stamp.caption = "KY NIEM BUDAPEST".into();
        settings.stamp.place = true;
        let p = plan(&PlanInput {
            width: 6000,
            height: 4000,
            capture: None,
            settings: &settings,
            print: size("10x15"),
            max_long_edge: Some(1600),
            frame: None,
            crop: None,
            place: Some("Budapest"),
            straighten: 0.0,
        });
        let Some(stamp) = p.stamp else {
            panic!("the caption is printed without a date");
        };
        assert_eq!(stamp.text, "Budapest");
        assert_eq!(stamp.caption.as_deref(), Some("KY NIEM BUDAPEST"));
    }

    #[test]
    fn fr_045_original_keeps_every_pixel() {
        let mut settings = RenderSettings {
            print_size: PrintSize::ORIGINAL_ID.into(),
            ..RenderSettings::default()
        };
        settings.stamp.enabled = false;
        for (w, h) in [
            (4608, 3456),
            (3000, 4000),
            (6000, 4000),
            (8000, 2000),
            (4001, 2999),
        ] {
            let print = settings.print_for(w, h).unwrap_or(size("10x15"));
            assert!(print.is_original());
            let p = plan(&PlanInput {
                width: w,
                height: h,
                capture: None,
                settings: &settings,
                print,
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
            });
            assert_eq!(
                (p.crop.x, p.crop.y, p.crop.width, p.crop.height),
                (0, 0, w, h),
                "{w}x{h} must not be cropped"
            );
        }
    }

    #[test]
    fn fr_045_original_print_is_10cm_on_the_short_edge() {
        let mm = |w, h| {
            let p = PrintSize::original(w, h);
            (p.short_mm, p.long_mm)
        };
        assert_eq!(mm(4608, 3456), (100, 133)); // 4:3 → 10 × 13.3 cm
        assert_eq!(mm(3456, 4608), (100, 133)); // portrait, same print
        assert_eq!(mm(6000, 4000), (100, 150)); // 3:2 → 10 × 15 cm
        assert_eq!(mm(3000, 3000), (100, 100));
        assert_eq!(mm(20_000, 1000), (100, 1000)); // panoramas capped at 1 m
        // The placeholder from validate() is only used before a photo is known.
        let settings = RenderSettings {
            print_size: PrintSize::ORIGINAL_ID.into(),
            ..RenderSettings::default()
        };
        assert!(settings.validate().is_ok_and(PrintSize::is_original));
    }
}
