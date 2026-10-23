// SPDX-License-Identifier: GPL-3.0-only
//! `RenderPlan`: a pure description of how one output photo is produced. Preview and
//! export consume the same plan, so what the user sees is what gets printed
//! (ARCHITECTURE.md §4).

use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::Serialize;
use specta::Type;

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
}

/// Info frame to compose around the photo (FR-040).
#[derive(Debug, Clone, PartialEq)]
pub struct FramePlan {
    pub layout: FrameLayout,
    pub logo: Option<Arc<LogoImage>>,
}

/// What the caller knows about the frame for one photo.
#[derive(Debug, Clone)]
pub struct FrameRequest {
    pub settings: FrameSettings,
    pub text: FrameText,
    /// The camera's user logo, when the profile uses one.
    pub logo: Option<Arc<LogoImage>>,
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

/// Builds the plan for a source of `width × height` (already oriented).
/// `max_long_edge` downsizes the output for previews; `None` keeps full resolution.
pub fn plan(
    width: u32,
    height: u32,
    capture: Option<NaiveDateTime>,
    settings: &RenderSettings,
    print: PrintSize,
    max_long_edge: Option<u32>,
    frame: Option<&FrameRequest>,
) -> RenderPlan {
    if let Some(request) = frame.filter(|f| f.settings.enabled) {
        return plan_framed(
            width,
            height,
            capture,
            settings,
            print,
            max_long_edge,
            request,
        );
    }
    let crop = center_crop(width, height, print);
    let (mut out_width, mut out_height) = (crop.width, crop.height);
    if let Some(limit) = max_long_edge {
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
        stamp: stamp_spec(settings, capture, px_per_mm),
        frame: None,
    }
}

fn stamp_spec(
    settings: &RenderSettings,
    capture: Option<NaiveDateTime>,
    px_per_mm: f32,
) -> Option<StampSpec> {
    match (settings.stamp.enabled, capture) {
        (true, Some(dt)) => Some(StampSpec {
            text: format_date(dt, settings.stamp.format),
            color: settings.stamp.color.rgb(),
            corner: settings.stamp.corner,
            font_px: STAMP_SIZE_MM * px_per_mm,
            margin_px: (SAFE_AREA_MM + STAMP_INSET_MM) * px_per_mm,
        }),
        _ => None,
    }
}

/// The whole canvas keeps the print ratio; the photo fills (Crop) or fits (Fit) the space
/// above the info bar (FR-040).
fn plan_framed(
    width: u32,
    height: u32,
    capture: Option<NaiveDateTime>,
    settings: &RenderSettings,
    print: PrintSize,
    max_long_edge: Option<u32>,
    request: &FrameRequest,
) -> RenderPlan {
    let long = max_long_edge.map_or(width.max(height), |m| m.min(width.max(height)));
    let (cw, ch) = frame::canvas_size(width, height, print, long);
    let layout = frame::layout(
        cw,
        ch,
        width,
        height,
        &request.settings,
        &request.text,
        print,
    );
    let photo = layout.photo;
    let crop = match request.settings.fit {
        FrameFit::Crop => center_crop_to(
            width,
            height,
            u64::from(photo.width),
            u64::from(photo.height),
        ),
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
        stamp: stamp_spec(settings, capture, px_per_mm),
        frame: Some(FramePlan {
            layout,
            logo: request.logo.clone(),
        }),
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
        let p = plan(6000, 4500, None, &settings, size("10x15"), Some(1600), None);
        assert_eq!((p.out_width, p.out_height), (1600, 1067));
        assert!(p.stamp.is_none(), "no capture date, no stamp");
    }
}
