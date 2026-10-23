// SPDX-License-Identifier: GPL-3.0-only
//! `RenderPlan`: a pure description of how one output photo is produced. Preview and
//! export consume the same plan, so what the user sees is what gets printed
//! (ARCHITECTURE.md §4).

use chrono::NaiveDateTime;

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

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlan {
    pub print: PrintSize,
    pub crop: CropRect,
    pub out_width: u32,
    pub out_height: u32,
    pub stamp: Option<StampSpec>,
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
) -> RenderPlan {
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
    let stamp = match (settings.stamp.enabled, capture) {
        (true, Some(dt)) => Some(StampSpec {
            text: format_date(dt, settings.stamp.format),
            color: settings.stamp.color.rgb(),
            corner: settings.stamp.corner,
            font_px: STAMP_SIZE_MM * px_per_mm,
            margin_px: (SAFE_AREA_MM + STAMP_INSET_MM) * px_per_mm,
        }),
        _ => None,
    };

    RenderPlan {
        print,
        crop,
        out_width,
        out_height,
        stamp,
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
        let p = plan(6000, 4500, None, &settings, size("10x15"), Some(1600));
        assert_eq!((p.out_width, p.out_height), (1600, 1067));
        assert!(p.stamp.is_none(), "no capture date, no stamp");
    }
}
