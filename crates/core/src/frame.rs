// SPDX-License-Identifier: GPL-3.0-only
//! EXIF info frame ("Shot on …", FR-040): settings, per-camera profiles, text building
//! and a pure layout that keeps the whole canvas at the print ratio.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::library::{Library, ShotInfo};
use crate::print_size::PrintSize;

/// How the photo sits in the space left above the info bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FrameFit {
    /// Crop the photo to fill the photo area (no margins).
    Crop,
    /// Keep the whole photo, with a thin margin around it.
    Fit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FrameColor {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FrameDateFormat {
    /// `2026/10/18`
    Slash,
    /// `2026-10-18`
    Iso,
    /// `18.10.2026`
    Dmy,
}

/// Which items the frame shows, LUMIX Lab style.
#[allow(clippy::struct_excessive_bools)] // one switch per field is the UI
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrameFields {
    pub logo: bool,
    pub camera: bool,
    pub lens: bool,
    pub focal: bool,
    pub aperture: bool,
    pub shutter: bool,
    pub iso: bool,
    pub photo_style: bool,
    pub date: bool,
    pub place: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrameSettings {
    pub enabled: bool,
    pub fit: FrameFit,
    pub color: FrameColor,
    pub date_format: FrameDateFormat,
    pub fields: FrameFields,
    /// A user font id for the frame text (FR-042); `None` uses Inter.
    #[serde(default)]
    pub font: Option<u32>,
    /// Bundled brand logos in their own colours or in one colour (ADR-0020). Absent before
    /// settings v4.
    #[serde(default)]
    pub logo_style: crate::brands::LogoStyle,
    /// Size of the camera logo relative to the default, 0.5–2.0, the same for every photo
    /// (FR-040). Absent before settings v7: 1.0.
    #[serde(default = "default_logo_scale")]
    pub logo_scale: f32,
}

/// Smallest and largest logo scale (FR-040).
pub const LOGO_SCALE_RANGE: (f32, f32) = (0.5, 2.0);

fn default_logo_scale() -> f32 {
    1.0
}

/// A logo scale from the UI or a file, kept finite and in range.
pub fn sanitize_logo_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.clamp(LOGO_SCALE_RANGE.0, LOGO_SCALE_RANGE.1)
    } else {
        1.0
    }
}

impl Default for FrameSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            fit: FrameFit::Crop,
            color: FrameColor::White,
            date_format: FrameDateFormat::Slash,
            font: None,
            logo_style: crate::brands::LogoStyle::default(),
            logo_scale: 1.0,
            fields: FrameFields {
                logo: true,
                camera: true,
                lens: true,
                focal: true,
                aperture: true,
                shutter: true,
                iso: true,
                photo_style: true,
                date: true,
                place: true,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LogoKind {
    /// The brand name, typeset (the default when no logo is bundled for the brand).
    Text,
    /// A PNG the user added for this camera (ADR-0014).
    Image,
    None,
    /// The bundled logo of the camera's brand (ADR-0020); the default when there is one.
    Brand,
}

/// User edits for one camera, remembered across sessions (DATABASE.md §6). Every field is
/// optional: unset fields fall back to EXIF or to derived defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CameraProfile {
    pub id: u32,
    pub make: Option<String>,
    pub model: Option<String>,
    pub display_name: Option<String>,
    pub brand_text: Option<String>,
    /// For manual lenses that report nothing in EXIF (e.g. "Industar-50-2").
    pub lens: Option<String>,
    pub focal_mm: Option<f32>,
    pub f_number: Option<f32>,
    pub photo_style: Option<String>,
    pub logo: LogoKind,
}

impl CameraProfile {
    pub fn new(id: u32, make: Option<String>, model: Option<String>) -> Self {
        let logo = if make
            .as_deref()
            .and_then(crate::brands::Brand::from_make)
            .is_some()
        {
            LogoKind::Brand
        } else {
            LogoKind::Text
        };
        Self {
            id,
            make,
            model,
            display_name: None,
            brand_text: None,
            lens: None,
            focal_mm: None,
            f_number: None,
            photo_style: None,
            logo,
        }
    }

    /// The bundled brand of this camera, if any (ADR-0020).
    pub fn brand(&self) -> Option<crate::brands::Brand> {
        self.make
            .as_deref()
            .and_then(crate::brands::Brand::from_make)
    }

    pub fn matches(&self, make: Option<&str>, model: Option<&str>) -> bool {
        self.make.as_deref() == make && self.model.as_deref() == model
    }

    /// Default camera name shown after "Shot on".
    pub fn default_display_name(&self) -> Option<String> {
        Library::camera_label(self.make.as_deref(), self.model.as_deref())
    }

    /// Default brand wordmark: the first word of the make, upper-cased.
    pub fn default_brand(&self) -> Option<String> {
        self.make
            .as_deref()
            .and_then(|m| m.split_whitespace().next())
            .map(str::to_uppercase)
    }

    /// Clamps user input to sane values (SECURITY.md §2: validate IPC input).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let clean = |s: Option<String>| {
            s.map(|s| {
                s.chars()
                    .filter(|c| !c.is_control())
                    .take(60)
                    .collect::<String>()
                    .trim()
                    .to_owned()
            })
            .filter(|s| !s.is_empty())
        };
        self.display_name = clean(self.display_name);
        self.brand_text = clean(self.brand_text);
        self.lens = clean(self.lens);
        self.photo_style = clean(self.photo_style);
        self.focal_mm = self
            .focal_mm
            .filter(|f| f.is_finite() && (0.5..=5000.0).contains(f));
        self.f_number = self
            .f_number
            .filter(|f| f.is_finite() && (0.5..=128.0).contains(f));
        self
    }
}

pub fn format_aperture(f: f32) -> String {
    let rounded = (f * 10.0).round() / 10.0;
    if (rounded - rounded.round()).abs() < f32::EPSILON {
        format!("F{rounded:.0}")
    } else {
        format!("F{rounded:.1}")
    }
}

pub fn format_shutter(seconds: f32) -> String {
    if seconds >= 1.0 {
        let rounded = (seconds * 10.0).round() / 10.0;
        if (rounded - rounded.round()).abs() < f32::EPSILON {
            format!("{rounded:.0}s")
        } else {
            format!("{rounded:.1}s")
        }
    } else {
        format!("1/{:.0}s", (1.0 / seconds).round())
    }
}

pub fn format_focal(mm: f32) -> String {
    format!("{:.0}mm", mm.round())
}

fn format_frame_date(dt: NaiveDateTime, format: FrameDateFormat) -> String {
    match format {
        FrameDateFormat::Slash => dt.format("%Y/%m/%d").to_string(),
        FrameDateFormat::Iso => dt.format("%Y-%m-%d").to_string(),
        FrameDateFormat::Dmy => dt.format("%d.%m.%Y").to_string(),
    }
}

/// What to draw on the left of the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameLogo {
    Text(String),
    /// Draw the camera's user logo (looked up by the renderer).
    Image(u32),
    None,
}

/// The text content of one photo's frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameText {
    pub logo: FrameLogo,
    /// Right-aligned lines; the first one is the headline.
    pub lines: Vec<String>,
    /// Where a line may wrap when it would run into the logo: the line, and the byte
    /// offset that ends the lens (a long lens name then sits on a line of its own).
    pub wrap: Option<(usize, usize)>,
}

impl FrameText {
    /// The text with the lens on its own line, when there is a place to wrap.
    pub fn wrapped(&self) -> Option<Self> {
        let (index, at) = self.wrap?;
        let line = self.lines.get(index)?;
        let (lens, rest) = (line.get(..at)?, line.get(at..)?.trim_start());
        if rest.is_empty() {
            return None;
        }
        let mut lines = self.lines.clone();
        lines.splice(index..=index, [lens.to_owned(), rest.to_owned()]);
        Some(Self {
            logo: self.logo.clone(),
            lines,
            wrap: None,
        })
    }
}

/// Builds the frame text from EXIF, the camera profile overrides and the field switches.
pub fn build_text(
    shot: &ShotInfo,
    profile: Option<&CameraProfile>,
    settings: &FrameSettings,
    capture: Option<NaiveDateTime>,
    place: Option<&str>,
) -> FrameText {
    let f = settings.fields;
    let fallback = CameraProfile::new(0, shot.make.clone(), shot.model.clone());
    let p = profile.unwrap_or(&fallback);

    let mut lines = Vec::new();
    if f.camera
        && let Some(name) = p.display_name.clone().or_else(|| p.default_display_name())
    {
        lines.push(format!("Shot on  {name}"));
    }

    let mut exposure = Vec::new();
    let mut lens_len = 0;
    if f.lens
        && let Some(lens) = p.lens.clone().or_else(|| shot.lens.clone())
    {
        lens_len = lens.len();
        exposure.push(lens);
    }
    if f.focal
        && let Some(mm) = p.focal_mm.or(shot.focal_mm)
    {
        exposure.push(format_focal(mm));
    }
    if f.aperture
        && let Some(fnum) = p.f_number.or(shot.f_number)
    {
        exposure.push(format_aperture(fnum));
    }
    if f.shutter
        && let Some(s) = shot.exposure_s
    {
        exposure.push(format_shutter(s));
    }
    if f.iso
        && let Some(iso) = shot.iso
    {
        exposure.push(format!("ISO{iso}"));
    }
    let mut wrap = None;
    if !exposure.is_empty() {
        if lens_len > 0 && exposure.len() > 1 {
            wrap = Some((lines.len(), lens_len));
        }
        lines.push(exposure.join("  "));
    }

    if f.photo_style
        && let Some(style) = p.photo_style.clone()
    {
        lines.push(style);
    }

    let mut when_where = Vec::new();
    if f.date
        && let Some(dt) = capture
    {
        when_where.push(format_frame_date(dt, settings.date_format));
    }
    if f.place
        && let Some(place) = place
    {
        when_where.push(place.to_owned());
    }
    if !when_where.is_empty() {
        lines.push(when_where.join(" "));
    }

    let logo = if f.logo {
        match p.logo {
            LogoKind::Image => FrameLogo::Image(p.id),
            LogoKind::Brand if p.brand().is_some() => FrameLogo::Image(p.id),
            LogoKind::None => FrameLogo::None,
            LogoKind::Text | LogoKind::Brand => p
                .brand_text
                .clone()
                .or_else(|| p.default_brand())
                .map_or(FrameLogo::None, FrameLogo::Text),
        }
    } else {
        FrameLogo::None
    };
    FrameText { logo, lines, wrap }
}

/// An axis-aligned rectangle in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// One right-aligned text line.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLine {
    pub text: String,
    pub font_px: f32,
    pub bold: bool,
    pub color: [u8; 3],
    /// Right edge of the text, and top of the line box, in canvas pixels.
    pub right_x: f32,
    pub top_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameLayout {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background: [u8; 3],
    /// Where the photo goes. In `Fit` mode it is the fitted photo rectangle.
    pub photo: Rect,
    pub bar: Rect,
    pub logo: FrameLogo,
    /// Left edge and vertical centre of the logo slot, and its height.
    pub logo_x: f32,
    pub logo_center_y: f32,
    pub logo_height: f32,
    pub logo_max_width: f32,
    pub logo_color: [u8; 3],
    pub lines: Vec<PlacedLine>,
}

/// Bar height as a fraction of the canvas short edge.
const BAR_FRACTION: f32 = 0.24;
/// Margin around the photo in `Fit` mode, as a fraction of the short edge.
const FIT_MARGIN_FRACTION: f32 = 0.035;
/// Horizontal padding inside the bar, as a fraction of the canvas width.
const PAD_FRACTION: f32 = 0.055;

/// Canvas at the print ratio, in the photo's orientation, with the given long edge.
pub fn canvas_size(
    src_width: u32,
    src_height: u32,
    print: PrintSize,
    long_edge: u32,
) -> (u32, u32) {
    let ratio = if print.is_original() {
        // The photo's exact ratio (FR-045), not the mm-rounded one.
        f64::from(src_width.min(src_height).max(1)) / f64::from(src_width.max(src_height).max(1))
    } else {
        f64::from(print.short_mm) / f64::from(print.long_mm)
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let short = (f64::from(long_edge) * ratio).round() as u32;
    if src_width >= src_height {
        (long_edge, short.max(1))
    } else {
        (short.max(1), long_edge)
    }
}

/// Lays out the frame for a canvas, a source photo size and the text.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn layout(
    canvas_width: u32,
    canvas_height: u32,
    src_width: u32,
    src_height: u32,
    settings: &FrameSettings,
    text: &FrameText,
    print: PrintSize,
) -> FrameLayout {
    let (cw, ch) = (canvas_width as f32, canvas_height as f32);
    let short = cw.min(ch);
    let bar_h = (short * BAR_FRACTION).round().max(1.0);
    let bar_top = (ch - bar_h).max(0.0);
    let px_per_mm = cw.max(ch) / print.long_mm as f32;
    let safe = crate::plan::SAFE_AREA_MM * px_per_mm;

    let photo = match settings.fit {
        FrameFit::Crop => Rect {
            x: 0,
            y: 0,
            width: canvas_width,
            height: bar_top as u32,
        },
        FrameFit::Fit => {
            let m = (short * FIT_MARGIN_FRACTION).max(safe);
            let (aw, ah) = ((cw - 2.0 * m).max(1.0), (bar_top - m).max(1.0));
            let scale = (aw / src_width as f32).min(ah / src_height as f32);
            let (pw, ph) = (src_width as f32 * scale, src_height as f32 * scale);
            Rect {
                x: (m + (aw - pw) / 2.0).round() as u32,
                y: (m + (ah - ph) / 2.0).round() as u32,
                width: pw.round().max(1.0) as u32,
                height: ph.round().max(1.0) as u32,
            }
        }
    };

    let (background, primary, secondary) = match settings.color {
        FrameColor::White => ([250, 250, 249], [17, 17, 17], [138, 138, 138]),
        FrameColor::Black => ([13, 13, 13], [242, 242, 242], [143, 143, 143]),
    };

    let pad = (cw * PAD_FRACTION).max(safe * 1.5);
    let headline = bar_h * 0.15;
    let detail = bar_h * 0.105;
    let gap = |px: f32| px * 0.5;
    let sizes: Vec<f32> = (0..text.lines.len())
        .map(|i| if i == 0 { headline } else { detail })
        .collect();
    let block: f32 = sizes.iter().map(|s| s * 1.2).sum::<f32>()
        + sizes.iter().skip(1).map(|s| gap(*s)).sum::<f32>();
    // Shrink everything if many lines would not fit comfortably.
    let fit = (bar_h * 0.8 / block.max(1.0)).min(1.0);
    let mut y = bar_top + (bar_h - block * fit) / 2.0;
    let lines = text
        .lines
        .iter()
        .zip(&sizes)
        .enumerate()
        .map(|(i, (line, size))| {
            let font_px = size * fit;
            if i > 0 {
                y += gap(font_px);
            }
            let placed = PlacedLine {
                text: line.clone(),
                font_px,
                bold: i == 0,
                color: if i == 0 { primary } else { secondary },
                right_x: cw - pad,
                top_y: y,
            };
            y += font_px * 1.2;
            placed
        })
        .collect();

    FrameLayout {
        canvas_width,
        canvas_height,
        background,
        photo,
        bar: Rect {
            x: 0,
            y: bar_top as u32,
            width: canvas_width,
            height: bar_h as u32,
        },
        logo: text.logo.clone(),
        logo_x: pad,
        logo_center_y: bar_top + bar_h / 2.0,
        logo_height: bar_h * 0.22 * sanitize_logo_scale(settings.logo_scale),
        // A larger logo may take more of the bar, but never reaches the text.
        logo_max_width: cw * (0.35 * sanitize_logo_scale(settings.logo_scale)).min(0.5),
        logo_color: primary,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot() -> ShotInfo {
        ShotInfo {
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            lens: None,
            focal_mm: None,
            f_number: None,
            exposure_s: Some(1.0 / 400.0),
            iso: Some(200),
        }
    }

    fn at(day: u32) -> Option<NaiveDateTime> {
        chrono::NaiveDate::from_ymd_opt(2026, 10, day).and_then(|d| d.and_hms_opt(9, 0, 0))
    }

    #[test]
    fn fr_040_exposure_formatting() {
        assert_eq!(format_aperture(4.0), "F4");
        assert_eq!(format_aperture(2.8), "F2.8");
        assert_eq!(format_shutter(1.0 / 400.0), "1/400s");
        assert_eq!(format_shutter(0.5), "1/2s");
        assert_eq!(format_shutter(2.0), "2s");
        assert_eq!(format_focal(50.2), "50mm");
    }

    #[test]
    fn fr_040_profile_overrides_exif_lens() {
        let mut profile = CameraProfile::new(1, Some("Panasonic".into()), Some("DC-GX9".into()));
        // A known brand starts on its bundled logo (ADR-0020); this test types the name.
        assert_eq!(profile.logo, LogoKind::Brand);
        profile.logo = LogoKind::Text;
        profile.display_name = Some("LUMIX GX9".into());
        profile.brand_text = Some("LUMIX".into());
        profile.lens = Some("Индустар-50-2".into());
        profile.focal_mm = Some(50.0);
        profile.f_number = Some(4.0);
        profile.photo_style = Some("L. Monochrome D".into());
        let text = build_text(
            &shot(),
            Some(&profile),
            &FrameSettings::default(),
            at(18),
            Some("Budapest"),
        );
        assert_eq!(text.logo, FrameLogo::Text("LUMIX".into()));
        assert_eq!(
            text.lines,
            [
                "Shot on  LUMIX GX9",
                "Индустар-50-2  50mm  F4  1/400s  ISO200",
                "L. Monochrome D",
                "2026/10/18 Budapest",
            ]
        );
    }

    #[test]
    fn fr_040_hidden_fields_and_missing_data_drop_lines() {
        let mut settings = FrameSettings::default();
        settings.fields.iso = false;
        settings.fields.shutter = false;
        settings.fields.logo = false;
        // No profile: defaults from EXIF; no lens/focal/aperture in EXIF; no style.
        let text = build_text(&shot(), None, &settings, at(18), None);
        assert_eq!(text.logo, FrameLogo::None);
        assert_eq!(text.lines, ["Shot on  Panasonic DC-GX9", "2026/10/18"]);
    }

    #[test]
    fn fr_040_canvas_matches_print_ratio_crop_and_fit() {
        let text = build_text(&shot(), None, &FrameSettings::default(), at(18), None);
        for print in PrintSize::ALL {
            for (sw, sh) in [(4608, 3456), (3456, 4608), (6000, 4000)] {
                for fit in [FrameFit::Crop, FrameFit::Fit] {
                    let settings = FrameSettings {
                        fit,
                        ..FrameSettings::default()
                    };
                    let (cw, ch) = canvas_size(sw, sh, print, sw.max(sh));
                    let l = layout(cw, ch, sw, sh, &settings, &text, print);
                    let ratio = f64::from(cw.max(ch)) / f64::from(cw.min(ch));
                    let target = f64::from(print.long_mm) / f64::from(print.short_mm);
                    assert!(
                        (ratio - target).abs() / target < 0.002,
                        "{}: {ratio}",
                        print.id
                    );
                    if print.short_mm != print.long_mm {
                        assert_eq!(sw >= sh, cw >= ch, "orientation kept");
                    }
                    // Photo area never overlaps the bar and stays inside the canvas.
                    assert!(l.photo.y + l.photo.height <= l.bar.y + 1);
                    assert!(l.photo.x + l.photo.width <= cw);
                    if fit == FrameFit::Fit {
                        let src = f64::from(sw) / f64::from(sh);
                        let placed = f64::from(l.photo.width) / f64::from(l.photo.height);
                        assert!(
                            (src - placed).abs() / src < 0.01,
                            "fit keeps the photo ratio"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn fr_045_original_frame_canvas_keeps_the_photo_ratio() {
        for (sw, sh) in [(4608, 3456), (3456, 4608), (8000, 2000)] {
            let print = PrintSize::original(sw, sh);
            let (cw, ch) = canvas_size(sw, sh, print, sw.max(sh));
            assert_eq!((cw, ch), (sw, sh), "the canvas is the photo's own ratio");
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn fr_040_text_inside_safe_area() {
        let print = PrintSize::ALL[1];
        let mut profile = CameraProfile::new(1, Some("Panasonic".into()), Some("DC-GX9".into()));
        profile.photo_style = Some("L. Monochrome D".into());
        let text = build_text(
            &shot(),
            Some(&profile),
            &FrameSettings::default(),
            at(18),
            Some("Budapest"),
        );
        let (cw, ch) = canvas_size(4608, 3456, print, 4608);
        let l = layout(cw, ch, 4608, 3456, &FrameSettings::default(), &text, print);
        let safe = crate::plan::SAFE_AREA_MM * cw as f32 / print.long_mm as f32;
        assert!(l.logo_x >= safe);
        for line in &l.lines {
            assert!(line.right_x <= cw as f32 - safe);
            assert!(line.top_y >= l.bar.y as f32);
            assert!(
                line.top_y + line.font_px * 1.2 <= ch as f32 - safe,
                "{line:?}"
            );
        }
    }

    #[test]
    fn fr_040_logo_scale_resizes_the_logo_for_every_photo() {
        let print = PrintSize::ALL[1];
        let text = build_text(&shot(), None, &FrameSettings::default(), at(18), None);
        let (cw, ch) = canvas_size(4608, 3456, print, 4608);
        let at_scale = |logo_scale| {
            let settings = FrameSettings {
                logo_scale,
                ..FrameSettings::default()
            };
            layout(cw, ch, 4608, 3456, &settings, &text, print)
        };
        let normal = at_scale(1.0);
        let big = at_scale(2.0);
        assert!((big.logo_height - 2.0 * normal.logo_height).abs() < 0.01);
        assert!(big.logo_max_width > normal.logo_max_width);
        // Out-of-range and broken values are clamped, never trusted.
        assert!((at_scale(9.0).logo_height - big.logo_height).abs() < 0.01);
        assert!((at_scale(f32::NAN).logo_height - normal.logo_height).abs() < 0.01);
        assert!((at_scale(0.1).logo_height - 0.5 * normal.logo_height).abs() < 0.01);
    }
}
