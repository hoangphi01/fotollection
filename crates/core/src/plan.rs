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

/// A plain border round the photo when there is no info frame (FR-067): the photo sits at
/// `photo` on a canvas of the print's size filled with `background`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mat {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background: [u8; 3],
    pub photo: frame::Rect,
}

/// The white of a border without an info frame, the same as the white frame's.
pub const MAT_WHITE: [u8; 3] = [250, 250, 249];

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
    /// Without a frame, the 3 mm border's canvas (FR-067).
    pub mat: Option<Mat>,
    /// A shared photo's fill (FR-075): the whole output above, centred on a canvas of the
    /// chosen ratio.
    pub pad: Option<Mat>,
    /// Degrees the photo is straightened by (FR-006); `crop` is then in the straightened
    /// frame.
    pub straighten: f32,
    /// Resample in linear light: share exports only (FR-072); the screen keeps the faster
    /// resize (OPTIMIZATION §5).
    pub linear_light: bool,
    /// The colour look's table (FR-079), applied after the crop and before the stamp, the
    /// frame and the border, which are never tinted. `None` when the look changes nothing.
    pub look: Option<std::sync::Arc<crate::looks::Table>>,
    /// The look's dehaze (FR-115), drawn just before its table; 0 draws nothing.
    pub dehaze: i8,
    /// The small "fotollection.org" on a print (FR-121), in canvas pixels, drawn last but
    /// for a share's fill; `None` when it is off, on a share or nowhere fits.
    pub credit: Option<crate::credit::CreditSpec>,
}

impl RenderPlan {
    /// Final output size, including the frame canvas.
    pub fn output_size(&self) -> (u32, u32) {
        if let Some(pad) = &self.pad {
            return (pad.canvas_width, pad.canvas_height);
        }
        match (&self.frame, &self.mat) {
            (Some(f), _) => (f.layout.canvas_width, f.layout.canvas_height),
            (None, Some(m)) => (m.canvas_width, m.canvas_height),
            (None, None) => (self.out_width, self.out_height),
        }
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

/// Whether `print` can be laid the other way from the photo (FR-120): not a square size,
/// not *Original* (the photo's own shape), and not the info frame's Fit, which keeps the
/// whole photo.
pub fn can_turn(print: PrintSize, fit: Option<FrameFit>) -> bool {
    !print.is_original() && print.short_mm != print.long_mm && fit != Some(FrameFit::Fit)
}

/// The print size is laid the other way for this plan (FR-120).
fn turned(input: &PlanInput<'_>, fit: Option<FrameFit>) -> bool {
    input.turned && input.ratio.is_none() && can_turn(input.print, fit)
}

/// [`effective_dpi`] with the print size laid the other way from the photo when `turned`
/// (FR-120): the verdict of the crop that prints.
pub fn effective_dpi_turned(
    width: u32,
    height: u32,
    print: PrintSize,
    turned: bool,
) -> (u32, DpiBand) {
    if !turned || !can_turn(print, None) {
        return effective_dpi(width, height, print);
    }
    let (long, short) = (u64::from(print.long_mm), u64::from(print.short_mm));
    let (rw, rh) = if width >= height {
        (short, long)
    } else {
        (long, short)
    };
    let crop = center_crop_to(width, height, rw, rh);
    let long_px = f64::from(crop.width.max(crop.height));
    let long_in = f64::from(print.long_mm) / 25.4;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let dpi = (long_px / long_in).round() as u32;
    (dpi, dpi_band(dpi))
}

/// Effective DPI of the centred crop of a `width × height` photo printed at `print`.
pub fn effective_dpi(width: u32, height: u32, print: PrintSize) -> (u32, DpiBand) {
    let crop = center_crop(width, height, print);
    let long_px = f64::from(crop.width.max(crop.height));
    let long_in = f64::from(print.long_mm) / 25.4;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let dpi = (long_px / long_in).round() as u32;
    (dpi, dpi_band(dpi))
}

/// The part of `print` the photo fills, in whole millimetres: the print less the 3 mm border
/// on each side when it is on (FR-067). Original keeps its 10 cm short edge for the whole
/// print, so its photo is 94 mm on the short edge and its long edge follows the photo; the
/// border makes the print longer. One rule for planning, the crop editor and the advice.
pub fn print_photo_area(print: PrintSize, border: bool) -> PrintSize {
    if !border {
        return print;
    }
    let inset = 2 * border_mm();
    let short = print.short_mm.saturating_sub(inset).max(1);
    let long = if print.is_original() {
        let scaled = u64::from(print.long_mm) * u64::from(short) / u64::from(print.short_mm.max(1));
        u32::try_from(scaled).unwrap_or(u32::MAX).max(short)
    } else {
        print.long_mm.saturating_sub(inset).max(1)
    };
    PrintSize::custom(short, long)
}

/// The quality band of an effective DPI (FR-026).
pub fn dpi_band(dpi: u32) -> DpiBand {
    match dpi {
        300.. => DpiBand::Excellent,
        200..300 => DpiBand::Good,
        150..200 => DpiBand::Fair,
        _ => DpiBand::Poor,
    }
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
    /// A crop ratio `w : h` that wins over the print's, whatever the photo's orientation: a
    /// collage cell, which may stand when its photo lies (FR-035). Without a frame only.
    pub ratio: Option<(u64, u64)>,
    /// The small "fotollection.org" (FR-121): `Some` when this machine prints it, with the
    /// faces the best-shot analysis found in the upright photo, to keep it off them (empty
    /// when none or not analysed). Ignored on a share and on a collage cell, whose sheet
    /// draws its own.
    pub credit: Option<&'a [crate::culling::FaceSummary]>,
    /// The print size laid the other way from the photo (FR-120): a portrait photo cropped
    /// across a flat 15 × 5. Ignored where nothing turns ([`can_turn`]) and on a collage
    /// cell, whose `ratio` wins.
    pub turned: bool,
}

/// Builds the plan for one photo.
pub fn plan(input: &PlanInput<'_>) -> RenderPlan {
    let Some(fill) = share_fill(input) else {
        return plan_content(input);
    };
    // The limit bounds the whole canvas: the content is planned at full size first, then
    // again within the limit the canvas leaves it (no pixels are touched here).
    let unbounded = plan_content(&PlanInput {
        max_long_edge: None,
        ..*input
    });
    let (cw, ch) = unbounded.output_size();
    let (fw, fh) = fill_canvas(cw, ch, fill.ratio);
    let content = match input.max_long_edge {
        Some(limit) if fw.max(fh) > limit => {
            let scale = f64::from(limit) / f64::from(fw.max(fh));
            plan_content(&PlanInput {
                max_long_edge: Some(scale_dim(cw.max(ch), scale)),
                ..*input
            })
        }
        _ => unbounded,
    };
    let (cw, ch) = content.output_size();
    let (canvas_width, canvas_height) = fill_canvas(cw, ch, fill.ratio);
    RenderPlan {
        pad: Some(Mat {
            canvas_width,
            canvas_height,
            background: fill.color,
            photo: frame::Rect {
                x: (canvas_width - cw) / 2,
                y: (canvas_height - ch) / 2,
                width: cw,
                height: ch,
            },
        }),
        ..content
    }
}

/// The fill a shared photo gets (FR-075): share exports only, and never a collage cell, which
/// has its own paper.
fn share_fill(input: &PlanInput<'_>) -> Option<crate::settings::ShareFill> {
    let settings = input.settings;
    (input.ratio.is_none()
        && settings.share_fill.enabled
        && matches!(settings.export_target, crate::share::ExportTarget::Share(_)))
    .then(|| settings.share_fill.sanitized())
}

/// The smallest canvas of ratio `w : h` that holds `width × height` whole.
pub fn fill_canvas(width: u32, height: u32, ratio: [u16; 2]) -> (u32, u32) {
    let (rw, rh) = (u64::from(ratio[0].max(1)), u64::from(ratio[1].max(1)));
    let (w, h) = (u64::from(width), u64::from(height));
    // The side that binds: the content is wider than the ratio, or taller.
    let (cw, ch) = if w * rh >= h * rw {
        (w, (w * rh).div_ceil(rw))
    } else {
        ((h * rw).div_ceil(rh), h)
    };
    let clamp = |v: u64| u32::try_from(v).unwrap_or(u32::MAX);
    (clamp(cw).max(width), clamp(ch).max(height))
}

fn plan_content(input: &PlanInput<'_>) -> RenderPlan {
    if let Some(request) = input.frame.filter(|f| f.settings.enabled) {
        return plan_framed(input, request);
    }
    let (width, height, print) = (input.width, input.height, input.print);
    // With the border, the photo is cropped to the print less 3 mm on every side (FR-067).
    let area = print_photo_area(print, input.settings.border);
    let (long, short) = (u64::from(area.long_mm), u64::from(area.short_mm));
    let (rw, rh) = if let Some(ratio) = input.ratio {
        ratio
    } else if print.is_original() {
        // The exact pixel ratio: Original never loses a pixel to mm rounding (FR-045); with
        // the border, the print grows by it instead of cropping the photo.
        (u64::from(width), u64::from(height))
    } else if (width >= height) != turned(input, None) {
        (long, short)
    } else {
        (short, long)
    };
    let crop = crop_at(input, rw, rh);
    let (mut out_width, mut out_height) = (crop.width, crop.height);
    // With the border, the canvas grows by 3 mm a side: the photo gets less of the limit,
    // so the whole print still fits it, as in a framed print (FR-067).
    let limit = input.max_long_edge.map(|limit| {
        if input.settings.border {
            let photo_mm = f64::from(area.long_mm);
            let print_mm = photo_mm + f64::from(2 * border_mm());
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let fitted = (f64::from(limit) * photo_mm / print_mm).floor() as u32;
            fitted.max(1)
        } else {
            limit
        }
    });
    if let Some(limit) = limit {
        let long = out_width.max(out_height);
        if long > limit && long > 0 {
            let scale = f64::from(limit) / f64::from(long);
            out_width = scale_dim(out_width, scale);
            out_height = scale_dim(out_height, scale);
        }
    }

    #[allow(clippy::cast_precision_loss)] // print sizes and pixel counts are far below 2^24
    let px_per_mm = if input.settings.border {
        out_width.min(out_height) as f32 / short as f32
    } else {
        out_width.max(out_height) as f32 / print.long_mm as f32
    };
    let mat = input.settings.border.then(|| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let m = (frame::CROP_BORDER_MM * px_per_mm).round().max(1.0) as u32;
        Mat {
            canvas_width: out_width + 2 * m,
            canvas_height: out_height + 2 * m,
            background: input.settings.border_color.rgb(),
            photo: frame::Rect {
                x: m,
                y: m,
                width: out_width,
                height: out_height,
            },
        }
    });
    let stamp = stamp_spec(input, px_per_mm);
    let credit = content_credit(
        input,
        px_per_mm,
        mat.as_ref(),
        crop,
        (out_width, out_height),
        stamp.as_ref(),
    );
    RenderPlan {
        print,
        crop,
        out_width,
        out_height,
        stamp,
        frame: None,
        mat,
        pad: None,
        straighten: crate::straighten::sanitize(input.straighten),
        linear_light: false,
        look: input.settings.look.table(),
        dehaze: input.settings.look.dehaze(),
        credit,
    }
}

/// Where an unframed print's credit goes (FR-121): on its border under the photo, else in a
/// corner of the photo clear of the stamp and of faces.
fn content_credit(
    input: &PlanInput<'_>,
    px_per_mm: f32,
    mat: Option<&Mat>,
    crop: CropRect,
    (out_width, out_height): (u32, u32),
    stamp: Option<&StampSpec>,
) -> Option<crate::credit::CreditSpec> {
    let faces = credit_faces(input)?;
    let photo = mat.map_or(
        frame::Rect {
            x: 0,
            y: 0,
            width: out_width,
            height: out_height,
        },
        |m| m.photo,
    );
    let mut places = Vec::new();
    // The border under the photo first (FR-067).
    if let Some(m) = mat {
        let bottom = photo.y + photo.height;
        places.push(crate::credit::Place {
            area: boxf(photo.x, bottom, photo.width, m.canvas_height - bottom),
            paper: Some(m.background),
            corners: vec![Corner::BottomRight],
            fit: crate::credit::Fit::Centred,
            inset_x: 0.0,
            inset_y: 0.0,
        });
    }
    let inset = if mat.is_some() {
        1.5
    } else {
        SAFE_AREA_MM + 1.0
    } * px_per_mm;
    places.push(photo_place(photo, stamp, inset));
    let mut avoid = stamp_boxes(stamp, photo);
    avoid.extend(face_boxes(input, faces, crop, photo));
    crate::credit::place(px_per_mm, &places, &avoid)
}

/// The faces to keep the credit off, when this print gets one (FR-121): never on a share
/// and never on a collage cell, whose sheet places its own.
fn credit_faces<'a>(input: &PlanInput<'a>) -> Option<&'a [crate::culling::FaceSummary]> {
    input.credit.filter(|_| {
        input.ratio.is_none()
            && matches!(
                input.settings.export_target,
                crate::share::ExportTarget::Print
            )
    })
}

#[allow(clippy::cast_precision_loss)]
fn boxf(x: u32, y: u32, w: u32, h: u32) -> crate::credit::BoxF {
    crate::credit::BoxF {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
    }
}

/// The photo's corners, inset by `inset` pixels, the one away from the stamp first.
fn photo_place(photo: frame::Rect, stamp: Option<&StampSpec>, inset: f32) -> crate::credit::Place {
    crate::credit::Place {
        area: boxf(photo.x, photo.y, photo.width, photo.height),
        paper: None,
        corners: crate::credit::photo_corners(stamp.map(|s| s.corner)),
        fit: crate::credit::Fit::Corner,
        inset_x: inset,
        inset_y: inset,
    }
}

/// The date stamp's lines on the canvas, the photo placed at `photo`.
#[allow(clippy::cast_precision_loss)]
fn stamp_boxes(stamp: Option<&StampSpec>, photo: frame::Rect) -> Vec<crate::credit::BoxF> {
    stamp.map_or_else(Vec::new, |spec| {
        crate::stamp::layout(photo.width, photo.height, spec)
            .into_iter()
            .map(|l| crate::credit::BoxF {
                x: photo.x as f32 + l.x,
                y: photo.y as f32 + l.y,
                w: l.width,
                h: l.height,
            })
            .collect()
    })
}

/// The faces, found as fractions of the upright photo, on the canvas through the crop.
#[allow(clippy::cast_precision_loss)]
fn face_boxes(
    input: &PlanInput<'_>,
    faces: &[crate::culling::FaceSummary],
    crop: CropRect,
    photo: frame::Rect,
) -> Vec<crate::credit::BoxF> {
    if crop.width == 0 || crop.height == 0 {
        return Vec::new();
    }
    let sx = photo.width as f32 / crop.width as f32;
    let sy = photo.height as f32 / crop.height as f32;
    let (w, h) = (input.width as f32, input.height as f32);
    faces
        .iter()
        .filter(|f| [f.x, f.y, f.w, f.h].iter().all(|v| v.is_finite()))
        .map(|f| crate::credit::BoxF {
            x: photo.x as f32 + (f.x * w - crop.x as f32) * sx,
            y: photo.y as f32 + (f.y * h - crop.y as f32) * sy,
            w: f.w * w * sx,
            h: f.h * h * sy,
        })
        .collect()
}

/// The 3 mm border in whole millimetres (FR-067).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn border_mm() -> u32 {
    frame::CROP_BORDER_MM as u32
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
    // The size follows the print it is on (FR-004).
    #[allow(clippy::cast_precision_loss)]
    let short_mm = input.print.short_mm as f32;
    Some(StampSpec {
        text,
        color: settings.stamp.color.rgb(),
        corner: settings.stamp.corner,
        font_px: settings.stamp.size.of_short_edge() * short_mm * px_per_mm,
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
    let border = input.settings.border;
    // Turned (FR-120), the frame is laid out as for a photo of the other orientation: in
    // Crop the photo's size only decides the canvas's orientation; Fit never turns.
    let (ow, oh) = if turned(input, Some(request.settings.fit)) {
        (height, width)
    } else {
        (width, height)
    };
    let (cw, ch) = frame::canvas_size(ow, oh, print, long, request.settings.fit, border);
    let lay = |text: &frame::FrameText| {
        frame::layout(cw, ch, ow, oh, &request.settings, text, print, border)
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
    let stamp = stamp_spec(input, px_per_mm);
    let credit = credit_faces(input).and_then(|faces| {
        // In the bar, under the logo, then on the photo.
        let bar = layout.bar;
        let places = [
            crate::credit::Place {
                area: boxf(bar.x, bar.y, bar.width, bar.height),
                paper: Some(layout.background),
                corners: vec![Corner::BottomLeft, Corner::BottomRight],
                fit: crate::credit::Fit::Corner,
                inset_x: layout.logo_x,
                inset_y: (SAFE_AREA_MM + 0.5) * px_per_mm,
            },
            photo_place(photo, stamp.as_ref(), (SAFE_AREA_MM + 1.0) * px_per_mm),
        ];
        let mut avoid = stamp_boxes(stamp.as_ref(), photo);
        avoid.extend(face_boxes(input, faces, crop, photo));
        avoid.extend(frame_boxes(&layout, request));
        crate::credit::place(px_per_mm, &places, &avoid)
    });
    RenderPlan {
        print,
        crop,
        out_width: photo.width,
        out_height: photo.height,
        stamp,
        frame: Some(FramePlan {
            layout,
            logo: request.logo.clone(),
            font: request.font.clone(),
        }),
        mat: None,
        pad: None,
        straighten: crate::straighten::sanitize(input.straighten),
        linear_light: false,
        look: input.settings.look.table(),
        dehaze: input.settings.look.dehaze(),
        credit,
    }
}

/// The frame's logo and text lines, which the credit keeps clear of (FR-121).
fn frame_boxes(layout: &FrameLayout, request: &FrameRequest) -> Vec<crate::credit::BoxF> {
    let mut boxes = Vec::new();
    if layout.logo != frame::FrameLogo::None && layout.logo_height > 0.0 {
        boxes.push(crate::credit::BoxF {
            x: layout.logo_x,
            y: layout.logo_center_y - layout.logo_height / 2.0,
            w: layout.logo_max_width,
            h: layout.logo_height,
        });
    }
    let font: Option<&ab_glyph::FontArc> = match &request.font {
        Some(f) => Some(&f.font),
        None => crate::render::regular(),
    };
    for line in &layout.lines {
        // Bold lines run a little wider than the face measures.
        let width = font.map_or(line.right_x, |f| {
            crate::text::text_width(f, line.font_px, &line.text, 0.0) * 1.1
        });
        boxes.push(crate::credit::BoxF {
            x: line.right_x - width,
            y: line.top_y,
            w: width,
            h: line.font_px * 1.2,
        });
    }
    boxes
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
            ratio: None,
            credit: None,
            turned: false,
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
            ratio: None,
            credit: None,
            turned: false,
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
                ratio: None,
                credit: None,
                turned: false,
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

    #[test]
    fn fr_067_the_border_takes_the_chosen_colour() {
        for (color, rgb) in [
            (crate::settings::BorderColor::White, MAT_WHITE),
            (crate::settings::BorderColor::Black, [13, 13, 13]),
        ] {
            let settings = RenderSettings {
                border: true,
                border_color: color,
                ..RenderSettings::default()
            };
            let p = plan(&PlanInput {
                width: 6000,
                height: 4000,
                capture: None,
                settings: &settings,
                print: size("10x15"),
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
                ratio: None,
                credit: None,
                turned: false,
            });
            assert_eq!(p.mat.map(|m| m.background), Some(rgb));
        }
    }

    #[test]
    fn fr_067_the_border_sits_inside_the_print_size_without_a_frame() {
        let mut settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        settings.stamp.enabled = false;
        for (w, h) in [(6000, 4000), (4000, 6000), (4608, 3456)] {
            let p = plan(&PlanInput {
                width: w,
                height: h,
                capture: None,
                settings: &settings,
                print: size("10x15"),
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
                ratio: None,
                credit: None,
                turned: false,
            });
            let Some(mat) = p.mat else {
                panic!("the border needs a canvas");
            };
            // The canvas keeps the print's 2:3; the photo is 94 × 144 mm, 3 mm in.
            let (cw, ch) = p.output_size();
            let canvas = f64::from(cw.max(ch)) / f64::from(cw.min(ch));
            assert!((canvas - 1.5).abs() < 0.005, "{w}×{h}: {canvas}");
            let mm = f64::from(cw.min(ch)) / 100.0;
            assert!((f64::from(mat.photo.x) / mm - 3.0).abs() < 0.1);
            assert_eq!(mat.photo.x, mat.photo.y);
            assert_eq!(mat.photo.x * 2 + p.out_width, cw);
            assert_eq!(mat.photo.y * 2 + p.out_height, ch);
            let photo =
                f64::from(p.out_width.max(p.out_height)) / f64::from(p.out_width.min(p.out_height));
            assert!((photo - 144.0 / 94.0).abs() < 0.005, "{w}×{h}: {photo}");
        }
        // Off, nothing changes.
        settings.border = false;
        let p = plan(&PlanInput {
            width: 6000,
            height: 4000,
            capture: None,
            settings: &settings,
            print: size("10x15"),
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
            ratio: None,
            credit: None,
            turned: false,
        });
        assert!(p.mat.is_none());
        assert_eq!(p.output_size(), (6000, 4000));
    }

    #[test]
    fn fr_067_original_with_the_border_still_keeps_every_pixel() {
        let mut settings = RenderSettings {
            print_size: PrintSize::ORIGINAL_ID.into(),
            border: true,
            ..RenderSettings::default()
        };
        settings.stamp.enabled = false;
        let print = settings.print_for(4608, 3456).unwrap_or(size("10x15"));
        let p = plan(&PlanInput {
            width: 4608,
            height: 3456,
            capture: None,
            settings: &settings,
            print,
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
            ratio: None,
            credit: None,
            turned: false,
        });
        assert_eq!((p.crop.width, p.crop.height), (4608, 3456));
        assert!(p.mat.is_some_and(|m| m.photo.x > 0));
    }

    #[test]
    fn fr_067_print_quality_follows_the_photo_inside_the_border() {
        let print = size("10x15");
        assert_eq!(print_photo_area(print, false), print);
        let area = print_photo_area(print, true);
        assert_eq!((area.short_mm, area.long_mm), (94, 144));
        let (plain, _) = effective_dpi(6000, 4000, print);
        let (bordered, _) = effective_dpi(6000, 4000, area);
        assert!(bordered > plain, "a smaller area, more pixels per inch");
        // Original keeps its 10 cm short edge: a 3:2 photo is 94 × 141 inside the border.
        let original = print_photo_area(PrintSize::original(6000, 4000), true);
        assert_eq!((original.short_mm, original.long_mm), (94, 141));
    }

    #[test]
    fn fr_067_a_preview_with_the_border_stays_within_its_long_edge() {
        let settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        let p = plan(&PlanInput {
            width: 6000,
            height: 4000,
            capture: None,
            settings: &settings,
            print: size("10x15"),
            max_long_edge: Some(1600),
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
            ratio: None,
            credit: None,
            turned: false,
        });
        let (w, h) = p.output_size();
        assert!(w.max(h) <= 1600 && w.max(h) >= 1590, "{w}×{h}");
    }

    #[test]
    fn fr_004_the_stamp_comes_small_medium_or_large() {
        let font = |stamp_size| {
            let mut settings = RenderSettings::default();
            settings.stamp.size = stamp_size;
            let p = plan(&PlanInput {
                width: 6000,
                height: 4000,
                capture: chrono::NaiveDate::from_ymd_opt(2026, 7, 14)
                    .and_then(|d| d.and_hms_opt(10, 0, 0)),
                settings: &settings,
                print: size("10x15"),
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
                ratio: None,
                credit: None,
                turned: false,
            });
            p.stamp.map_or(0.0, |s| s.font_px)
        };
        let (small, medium, large) = (
            font(crate::settings::StampSize::Small),
            font(crate::settings::StampSize::Medium),
            font(crate::settings::StampSize::Large),
        );
        // 6000 px over 150 mm: 40 px a millimetre; medium is 3.6 % of the 100 mm short edge.
        assert!((medium - 3.6 * 40.0).abs() < 0.5, "{medium}");
        assert!(small < medium && medium < large);
        assert!((large / medium - 4.8 / 3.6).abs() < 1e-3);
    }
    /// FR-075: a portrait shared with a square fill keeps every pixel, centred on a square
    /// canvas within the limit; prints and collage cells never get a fill.
    #[test]
    fn fr_075_a_shared_portrait_sits_whole_in_a_square_fill() {
        use crate::settings::ShareFill;
        use crate::share::{ExportTarget, SharePreset};
        let share = RenderSettings {
            print_size: PrintSize::ORIGINAL_ID.into(),
            export_target: ExportTarget::Share(SharePreset::Instagram),
            share_fill: ShareFill {
                enabled: true,
                ..ShareFill::default()
            },
            ..RenderSettings::default()
        };
        let input = |settings: &RenderSettings, limit: Option<u32>, ratio| {
            plan(&PlanInput {
                width: 4000,
                height: 6000,
                capture: None,
                settings,
                print: PrintSize::original(4000, 6000),
                max_long_edge: limit,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
                ratio,
                credit: None,
                turned: false,
            })
        };
        let full = input(&share, None, None);
        assert_eq!(full.output_size(), (6000, 6000));
        assert_eq!((full.out_width, full.out_height), (4000, 6000));
        let Some(pad) = full.pad else {
            panic!("a shared photo with a fill has a canvas");
        };
        assert_eq!((pad.photo.x, pad.photo.y), (1000, 0));
        assert_eq!(pad.background, [255, 255, 255]);
        // Instagram: 1080 × 1080, the photo 720 × 1080 in the middle.
        let small = input(&share, Some(1080), None);
        assert_eq!(small.output_size(), (1080, 1080));
        assert_eq!((small.out_width, small.out_height), (720, 1080));
        // 4:5 for a landscape photo: the fill goes above and below.
        assert_eq!(fill_canvas(6000, 4000, [4, 5]), (6000, 7500));
        // Prints, and a collage cell, have none.
        let print = RenderSettings {
            export_target: ExportTarget::Print,
            ..share.clone()
        };
        assert!(input(&print, None, None).pad.is_none());
        assert!(input(&share, None, Some((2, 3))).pad.is_none());
    }
}

/// The print size laid the other way from the photo (FR-120).
#[cfg(test)]
mod fr_120 {
    use super::*;
    use crate::frame::{FrameFit, FrameSettings, FrameText};

    fn size(id: &str) -> PrintSize {
        PrintSize::by_id(id).unwrap_or(PrintSize::ALL[1])
    }

    fn planned(
        print: PrintSize,
        (w, h): (u32, u32),
        frame: Option<&FrameRequest>,
        turned: bool,
    ) -> RenderPlan {
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        plan(&PlanInput {
            width: w,
            height: h,
            capture: None,
            settings: &settings,
            print,
            max_long_edge: None,
            frame,
            crop: None,
            place: None,
            straighten: 0.0,
            ratio: None,
            credit: None,
            turned,
        })
    }

    #[test]
    fn fr_120_a_portrait_photo_lies_across_a_flat_strip() {
        // The owner's case: a portrait photo, 3456 × 4608, on a 5 × 15 strip.
        let upright = planned(size("5x15"), (3456, 4608), None, false);
        assert_eq!((upright.crop.width, upright.crop.height), (1536, 4608));
        let flat = planned(size("5x15"), (3456, 4608), None, true);
        assert_eq!((flat.crop.width, flat.crop.height), (3456, 1152));
        assert!(flat.out_width > flat.out_height);
        // 10 × 15 the same way, and a landscape photo stood up.
        let flat = planned(size("10x15"), (3456, 4608), None, true);
        assert_eq!((flat.crop.width, flat.crop.height), (3456, 2304));
        let standing = planned(size("10x15"), (4608, 3456), None, true);
        assert_eq!((standing.crop.width, standing.crop.height), (2304, 3456));
    }

    #[test]
    fn fr_120_the_verdict_reads_the_crop_that_prints() {
        let (dpi, _) = effective_dpi_turned(3456, 4608, size("5x15"), false);
        assert_eq!(dpi, effective_dpi(3456, 4608, size("5x15")).0);
        // Flat: the long edge is the photo's 3456 px across 150 mm, about 585 DPI.
        let (dpi, _) = effective_dpi_turned(3456, 4608, size("5x15"), true);
        assert_eq!(dpi, 585);
    }

    #[test]
    fn fr_120_square_original_and_fit_never_turn() {
        for id in ["15x15", "original"] {
            let print = PrintSize::by_id(id).unwrap_or_else(|| PrintSize::original(3456, 4608));
            let a = planned(print, (3456, 4608), None, false);
            let b = planned(print, (3456, 4608), None, true);
            assert_eq!(a.crop, b.crop, "{id}");
            assert!(!can_turn(print, None), "{id}");
        }
        let fit = FrameRequest {
            settings: FrameSettings {
                enabled: true,
                fit: FrameFit::Fit,
                ..FrameSettings::default()
            },
            text: FrameText {
                logo: crate::frame::FrameLogo::None,
                lines: vec!["Shot on a camera".into()],
                wrap: None,
            },
            logo: None,
            font: None,
        };
        let a = planned(size("10x15"), (3456, 4608), Some(&fit), false);
        let b = planned(size("10x15"), (3456, 4608), Some(&fit), true);
        assert_eq!(a.output_size(), b.output_size());
        assert!(!can_turn(size("10x15"), Some(FrameFit::Fit)));
        assert!(can_turn(size("10x15"), Some(FrameFit::Crop)));
    }

    #[test]
    fn fr_120_a_cropped_frame_turns_with_the_print() {
        let crop = FrameRequest {
            settings: FrameSettings {
                enabled: true,
                fit: FrameFit::Crop,
                ..FrameSettings::default()
            },
            text: FrameText {
                logo: crate::frame::FrameLogo::None,
                lines: vec!["Shot on a camera".into()],
                wrap: None,
            },
            logo: None,
            font: None,
        };
        let upright = planned(size("10x15"), (3456, 4608), Some(&crop), false);
        let (w, h) = upright.output_size();
        assert!(w < h);
        let flat = planned(size("10x15"), (3456, 4608), Some(&crop), true);
        let (w, h) = flat.output_size();
        assert!(w > h, "{w} × {h}");
        assert!(flat.crop.width > flat.crop.height);
    }

    #[test]
    fn fr_120_a_collage_cell_keeps_its_own_ratio() {
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let cell = |turned| {
            plan(&PlanInput {
                width: 3456,
                height: 4608,
                capture: None,
                settings: &settings,
                print: size("10x15"),
                max_long_edge: None,
                frame: None,
                crop: None,
                place: None,
                straighten: 0.0,
                ratio: Some((70, 100)),
                credit: None,
                turned,
            })
        };
        assert_eq!(cell(false).crop, cell(true).crop);
    }
}
