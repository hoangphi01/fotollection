// SPDX-License-Identifier: GPL-3.0-only
//! Collage sheets (FR-035, ADR-0032): two photos on one print, like a half-frame camera put
//! two frames on one negative. The geometry is in millimetres; each photo is then planned
//! and rendered as an ordinary print the size of its photo area, and the two are placed on
//! a sheet canvas.

use chrono::NaiveDateTime;
use serde::Serialize;
use specta::Type;

use crate::image::RgbImage;
use crate::library::PhotoId;
use crate::plan::{CropFrame, DpiBand, PlanInput, plan};
use crate::print_size::PrintSize;
use crate::render::{RenderError, blit, filled, render};
use crate::settings::{BorderColor, CellRatio, CollageSettings, RenderSettings, StampSettings};

/// The sheet's pixel density never goes past this, however sharp the photos (ADR-0032).
pub const MAX_SHEET_DPI: f32 = 600.0;
/// The cut line's width and colour, and its darker ticks at the sheet's edges.
const CUT_LINE_MM: f32 = 0.2;
const CUT_LINE: [u8; 3] = [200, 200, 200];
const CUT_TICK_MM: f32 = 4.0;
const CUT_TICK: [u8; 3] = [120, 120, 120];
/// The same line on black paper: still faint, its ticks lighter.
const CUT_LINE_ON_BLACK: [u8; 3] = [70, 70, 70];
const CUT_TICK_ON_BLACK: [u8; 3] = [150, 150, 150];
const MM_PER_IN: f32 = 25.4;

/// A rectangle on the sheet, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectMm {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// One photo's place on the sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellLayout {
    pub cell: RectMm,
    /// The photo area, whole millimetres, in the photo's orientation.
    pub area: (u32, u32),
    /// Where the area's top-left corner sits on the sheet.
    pub x: f32,
    pub y: f32,
}

/// A laid-out sheet: its size and where each photo goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetLayout {
    /// The sheet in millimetres, as printed (width × height).
    pub width: f32,
    pub height: f32,
    /// The cells sit side by side (a landscape sheet) or one above the other.
    pub side_by_side: bool,
    /// The two cells; `None` for an empty one (an odd last photo, or a postcard's).
    pub cells: [Option<CellLayout>; 2],
    /// A postcard sheet's writing half (FR-073); its photo is in `cells[0]`.
    pub card: Option<RectMm>,
}

/// The sheet's print size: the chosen one, or 10 × 15 when it is Original, which has no
/// size of its own until a photo gives it one.
pub fn sheet_print(settings: &RenderSettings) -> PrintSize {
    settings
        .validate()
        .ok()
        .filter(|p| !p.is_original())
        .unwrap_or(PrintSize::ALL[1])
}

/// What the 3 mm border takes off each edge of a photo area, both sides together (FR-067).
pub fn border_inset(settings: &RenderSettings) -> f32 {
    if settings.border {
        2.0 * crate::frame::CROP_BORDER_MM
    } else {
        0.0
    }
}

/// The sheet turns to suit its first photo: a landscape photo gets landscape cells, so the
/// sheet stands up; a portrait one gets the half-frame look. `turned` is the user's choice
/// the other way.
pub fn side_by_side(first: (u32, u32), turned: bool) -> bool {
    let landscape_photo = first.0 >= first.1;
    landscape_photo == turned
}

/// The quarter turns (0 or 1) that make a photo lie the way its cell does, for sheets that
/// are cut in two (FR-035): standing cells on a sheet whose photos sit side by side, lying
/// ones otherwise. `first` and `photo` are upright sizes before any turn in the cell; the
/// sheet keeps the orientation its first photo and `turned` give it. A square photo never turns.
pub fn fitting_turns(turned: bool, first: (u32, u32), photo: (u32, u32)) -> u8 {
    let standing_cells = side_by_side(first, turned);
    let lies = photo.0 > photo.1;
    let stands = photo.0 < photo.1;
    u8::from((standing_cells && lies) || (!standing_cells && stands))
}

/// The sheet's `turned` that lays it landscape, its photos side by side, whatever the shape
/// of its first photo (FR-035, Auto turn).
pub fn landscape_turned(first: (u32, u32)) -> bool {
    first.0 >= first.1
}

/// The `turned` to lay a sheet out with, given its first photo as it sits on the sheet
/// (`size`, its own `cell_turns` included): a photo turned in its cell keeps the sheet as it
/// was, since only the photo turns (FR-035). A square photo looks the same either way.
pub fn layout_turned(turned: bool, cell_turns: u8, size: (u32, u32)) -> bool {
    if cell_turns % 2 == 1 && size.0 != size.1 {
        !turned
    } else {
        turned
    }
}

/// The sheet split across its long edge: side by side on a landscape sheet, one above the
/// other on a standing one.
pub fn cells(print: PrintSize, side_by_side: bool) -> (f32, f32, [RectMm; 2]) {
    #[allow(clippy::cast_precision_loss)]
    let (short, long) = (print.short_mm as f32, print.long_mm as f32);
    if side_by_side {
        let (w, h) = (long, short);
        let half = w / 2.0;
        (
            w,
            h,
            [
                RectMm {
                    x: 0.0,
                    y: 0.0,
                    width: half,
                    height: h,
                },
                RectMm {
                    x: half,
                    y: 0.0,
                    width: half,
                    height: h,
                },
            ],
        )
    } else {
        let (w, h) = (short, long);
        let half = h / 2.0;
        (
            w,
            h,
            [
                RectMm {
                    x: 0.0,
                    y: 0.0,
                    width: w,
                    height: half,
                },
                RectMm {
                    x: 0.0,
                    y: half,
                    width: w,
                    height: half,
                },
            ],
        )
    }
}

/// The photo area's ratio `w : h`, in the photo's orientation.
fn area_ratio(photo: (u32, u32), print: PrintSize, ratio: CellRatio) -> (f64, f64) {
    let landscape = photo.0 >= photo.1;
    let (long, short) = match ratio {
        CellRatio::Photo => {
            let (w, h) = (f64::from(photo.0.max(1)), f64::from(photo.1.max(1)));
            return (w, h);
        }
        CellRatio::Sheet => (f64::from(print.long_mm), f64::from(print.short_mm)),
        CellRatio::FourThree => (4.0, 3.0),
        CellRatio::Square => (1.0, 1.0),
    };
    if landscape {
        (long, short)
    } else {
        (short, long)
    }
}

/// Whole millimetres, rounded down, forgiving a twentieth of a millimetre: a photo's pixel
/// ratio after the preview's downscale (1365 × 2048 for 4000 × 6000) must give the same
/// area as the full-size photo, 60 mm and not 59.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn floor_mm(v: f64) -> u32 {
    ((v + 0.05).floor().max(1.0)) as u32
}

/// The largest area at `ratio` inside `available`, rounded down to the millimetre.
fn fit(available: (f64, f64), ratio: (f64, f64)) -> (u32, u32) {
    let scale = (available.0 / ratio.0).min(available.1 / ratio.1);
    (floor_mm(ratio.0 * scale), floor_mm(ratio.1 * scale))
}

/// The suggested photo area for a photo in a cell: the largest at the chosen ratio that
/// leaves the margin free, in whole millimetres (a 2:3 photo in 75 × 100 with 5 mm: 60 × 90).
pub fn suggested_area(
    cell: (f32, f32),
    photo: (u32, u32),
    print: PrintSize,
    settings: &CollageSettings,
) -> (u32, u32) {
    if settings.fill {
        return (floor_mm(f64::from(cell.0)), floor_mm(f64::from(cell.1)));
    }
    fit(
        available(cell, settings),
        area_ratio(photo, print, settings.ratio),
    )
}

fn available(cell: (f32, f32), settings: &CollageSettings) -> (f64, f64) {
    #[allow(clippy::cast_precision_loss)]
    let m = 2.0
        * f64::from(
            settings
                .margin_mm
                .min(crate::settings::MAX_COLLAGE_MARGIN_MM),
        );
    (
        (f64::from(cell.0) - m).max(1.0),
        (f64::from(cell.1) - m).max(1.0),
    )
}

/// The photo area for a photo in a cell: the suggested one, or the size the user typed,
/// clamped so it always fits inside the margin (FR-035).
pub fn photo_area(
    cell: (f32, f32),
    photo: (u32, u32),
    print: PrintSize,
    settings: &CollageSettings,
) -> (u32, u32) {
    let settings = settings.clone().sanitized();
    // Filled: the whole cell, as it stands, whatever the photo's orientation.
    if settings.fill {
        return (floor_mm(f64::from(cell.0)), floor_mm(f64::from(cell.1)));
    }
    let suggested = suggested_area(cell, photo, print, &settings);
    let Some(size) = settings.size else {
        return suggested;
    };
    let room = available(cell, &settings);
    let landscape = photo.0 >= photo.1;
    if settings.lock_ratio {
        // The short edge typed; the long one follows the chosen ratio.
        let (rw, rh) = area_ratio(photo, print, settings.ratio);
        let (short, long_per_short) = (f64::from(size.short_mm), rw.max(rh) / rw.min(rh));
        let long = short * long_per_short;
        let (w, h) = if rw >= rh {
            (long, short)
        } else {
            (short, long)
        };
        if w <= room.0 + 1e-6 && h <= room.1 + 1e-6 {
            (floor_mm(w), floor_mm(h))
        } else {
            suggested
        }
    } else {
        let (w, h) = if landscape {
            (size.long_mm, size.short_mm)
        } else {
            (size.short_mm, size.long_mm)
        };
        (
            floor_mm(f64::from(w).min(room.0)),
            floor_mm(f64::from(h).min(room.1)),
        )
    }
}

/// Lays out a sheet for one or two photos (upright pixel sizes): each photo centred in its
/// half, so the white between them stays with or without the cut line.
pub fn layout(
    print: PrintSize,
    photos: [Option<(u32, u32)>; 2],
    turned: bool,
    settings: &CollageSettings,
) -> SheetLayout {
    let first = photos[0].or(photos[1]).unwrap_or((3, 2));
    let postcard = settings.postcard.enabled;
    // A postcard is held the way the user chose; its photo takes the left or bottom half.
    let side = if postcard {
        settings.postcard.orientation == crate::settings::PostcardOrientation::Landscape
    } else {
        side_by_side(first, turned)
    };
    let (width, height, rects) = cells(print, side);
    let place = |cell: RectMm, photo: (u32, u32)| {
        let area = photo_area((cell.width, cell.height), photo, print, settings);
        #[allow(clippy::cast_precision_loss)]
        let (aw, ah) = (area.0 as f32, area.1 as f32);
        let centred = (
            cell.x + (cell.width - aw) / 2.0,
            cell.y + (cell.height - ah) / 2.0,
        );
        CellLayout {
            cell,
            area,
            x: centred.0,
            y: centred.1,
        }
    };
    if postcard {
        let (photo_half, card) = if side {
            (rects[0], rects[1])
        } else {
            (rects[1], rects[0])
        };
        return SheetLayout {
            width,
            height,
            side_by_side: side,
            cells: [photos[0].map(|p| place(photo_half, p)), None],
            card: Some(card),
        };
    }
    SheetLayout {
        width,
        height,
        side_by_side: side,
        cells: [
            photos[0].map(|p| place(rects[0], p)),
            photos[1].map(|p| place(rects[1], p)),
        ],
        card: None,
    }
}

/// What the panel says about one photo of a sheet (FR-035, FR-026).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CellInfo {
    pub id: PhotoId,
    /// The photo area, as printed (width × height), in millimetres.
    pub area_width_mm: u32,
    pub area_height_mm: u32,
    /// The suggested area for this photo, the same way round.
    pub suggested_width_mm: u32,
    pub suggested_height_mm: u32,
    /// The photo left inside the 3 mm border (FR-067), or the whole area without it.
    pub photo_width_mm: f32,
    pub photo_height_mm: f32,
    /// Effective DPI of the photo at that size, and its band.
    pub dpi: u32,
    pub quality: DpiBand,
}

/// A sheet as the panel and the grid see it.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SheetInfo {
    pub width_mm: f32,
    pub height_mm: f32,
    pub side_by_side: bool,
    pub cells: Vec<CellInfo>,
}

/// A photo as `info` needs it: its id, upright pixel size, its crop on the sheet and its
/// straightening in degrees.
pub type SheetPhoto = (PhotoId, (u32, u32), Option<CropFrame>, f32);

/// Whether a postcard's text fits its lines on the print (FR-073), for the warning on the
/// card. The card half depends on how the card is held, not on the photo.
pub fn postcard_fit(
    print: PrintSize,
    settings: &RenderSettings,
    text: &crate::library::PostcardText,
    font: Option<&ab_glyph::FontArc>,
) -> crate::postcard::Fit {
    let sheet = layout(print, [Some((1000, 1000)), None], false, &settings.collage);
    let Some(card) = sheet.card else {
        return crate::postcard::Fit::default();
    };
    crate::postcard::fit(&crate::postcard::CardSpec {
        card,
        sheet_width: sheet.width,
        settings: &settings.collage.postcard,
        text: Some(text),
        font,
        paper: settings.border_color,
    })
}

/// Facts about a laid-out sheet, from the photos' sizes only (no decode).
pub fn info(
    print: PrintSize,
    photos: [Option<SheetPhoto>; 2],
    turned: bool,
    settings: &RenderSettings,
) -> SheetInfo {
    let sheet = layout(
        print,
        photos.map(|p| p.map(|(_, size, _, _)| size)),
        turned,
        &settings.collage,
    );
    let inset = border_inset(settings);
    let cells = sheet
        .cells
        .iter()
        .zip(photos)
        .filter_map(|(cell, photo)| {
            let (cell, (id, size, frame, degrees)) = ((*cell)?, photo?);
            let suggested = suggested_area(
                (cell.cell.width, cell.cell.height),
                size,
                print,
                &settings.collage,
            );
            #[allow(clippy::cast_precision_loss)]
            let (pw, ph) = (
                (cell.area.0 as f32 - inset).max(1.0),
                (cell.area.1 as f32 - inset).max(1.0),
            );
            // The photo's own crop, zoom and straightening included, at the ratio left inside
            // the border: the pixels the sheet really prints.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (rw, rh) = ((pw * 10.0).round() as u64, (ph * 10.0).round() as u64);
            let crop = if crate::straighten::is_straightened(degrees) {
                crate::straighten::crop_rect(size.0, size.1, rw, rh, frame, degrees)
            } else {
                crate::plan::crop_rect(size.0, size.1, rw, rh, frame)
            };
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let dpi = (crop.width.max(crop.height) as f32 / pw.max(ph) * MM_PER_IN).round() as u32;
            Some(CellInfo {
                id,
                area_width_mm: cell.area.0,
                area_height_mm: cell.area.1,
                suggested_width_mm: suggested.0,
                suggested_height_mm: suggested.1,
                photo_width_mm: pw,
                photo_height_mm: ph,
                dpi,
                quality: crate::plan::dpi_band(dpi),
            })
        })
        .collect();
    SheetInfo {
        width_mm: sheet.width,
        height_mm: sheet.height,
        side_by_side: sheet.side_by_side,
        cells,
    }
}

/// One photo to render into a cell: the upright image and its own edits.
#[derive(Debug, Clone, Copy)]
pub struct CellSource<'a> {
    pub image: &'a RgbImage,
    pub capture: Option<NaiveDateTime>,
    pub crop: Option<CropFrame>,
    pub straighten: f32,
    pub place: Option<&'a str>,
    /// The photo's own stamp (FR-044); `None` follows the settings.
    pub stamp: Option<&'a StampSettings>,
    /// The photo's own line under the date (FR-036); `None` uses the stamp's.
    pub caption: Option<&'a str>,
    /// The text typed on the photo's postcard (FR-073).
    pub postcard: Option<&'a crate::library::PostcardText>,
}

/// Renders a sheet. `max_long_edge` bounds the sheet's long edge for previews; `None`
/// keeps the density of the less sharp photo, at most 600 DPI (ADR-0032). A postcard
/// sheet (FR-073) draws its writing half after the photo, its text in `card_font` (the
/// user's, FR-042) or the bundled typewriter face.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn render_sheet(
    settings: &RenderSettings,
    print: PrintSize,
    sources: [Option<CellSource<'_>>; 2],
    turned: bool,
    max_long_edge: Option<u32>,
    card_font: Option<&ab_glyph::FontArc>,
) -> Result<RgbImage, RenderError> {
    let sizes = sources.map(|s| s.map(|s| (s.image.width, s.image.height)));
    let sheet = layout(print, sizes, turned, &settings.collage);
    let inset = border_inset(settings);
    let input =
        |source: &CellSource<'_>, cell: &CellLayout, own: &RenderSettings, max: Option<u32>| {
            let (a, b) = cell.area;
            let area = PrintSize::custom(a.min(b), a.max(b));
            // The crop takes the area's own shape, inside the border, even when the photo
            // lies and the cell stands (a filled cell).
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let ratio = (
                ((a as f32 - inset).max(1.0) * 10.0).round() as u64,
                ((b as f32 - inset).max(1.0) * 10.0).round() as u64,
            );
            plan(&PlanInput {
                width: source.image.width,
                height: source.image.height,
                capture: source.capture,
                settings: own,
                print: area,
                max_long_edge: max,
                frame: None,
                crop: source.crop,
                place: source.place,
                straighten: source.straighten,
                ratio: Some(ratio),
            })
        };
    let owns: [Option<RenderSettings>; 2] =
        sources.map(|s| s.map(|s| settings.for_photo(s.stamp, s.caption).into_owned()));

    // The density of the less sharp photo, so neither is enlarged.
    let mut density = MAX_SHEET_DPI / MM_PER_IN;
    for ((source, cell), own) in sources.iter().zip(&sheet.cells).zip(&owns) {
        if let (Some(source), Some(cell), Some(own)) = (source, cell, own) {
            let p = input(source, cell, own, None);
            let photo_long = (cell.area.0.max(cell.area.1) as f32 - inset).max(1.0);
            density = density.min(p.out_width.max(p.out_height) as f32 / photo_long);
        }
    }
    if let Some(max) = max_long_edge {
        density = density.min(max as f32 / sheet.width.max(sheet.height));
    }
    let density = density.max(1.0 / MM_PER_IN);

    let px = |mm: f32| (mm * density).round().max(1.0) as u32;
    let mut canvas = filled(
        px(sheet.width),
        px(sheet.height),
        settings.border_color.rgb(),
    );
    for ((source, cell), own) in sources.iter().zip(&sheet.cells).zip(&owns) {
        let (Some(source), Some(cell), Some(own)) = (source, cell, own) else {
            continue;
        };
        let photo_long = (cell.area.0.max(cell.area.1) as f32 - inset).max(1.0);
        let mut p = input(source, cell, own, Some(px(photo_long)));
        // Two stamps share the sheet: each is half the size it would have on the sheet
        // printed alone, whatever the photo's own area (FR-004).
        if let Some(stamp) = p.stamp.as_mut() {
            stamp.font_px *= stamp_scale(print, cell.area);
            stamp.margin_px /= 2.0;
        }
        let image = render(source.image, &p)?;
        // Centred on the area's centre, so pixel rounding never shifts it.
        let cx = (cell.x + cell.area.0 as f32 / 2.0) * density - image.width as f32 / 2.0;
        let cy = (cell.y + cell.area.1 as f32 / 2.0) * density - image.height as f32 / 2.0;
        blit(
            &mut canvas,
            &image,
            cx.round().max(0.0) as u32,
            cy.round().max(0.0) as u32,
        );
    }
    if let Some(card) = sheet.card {
        crate::postcard::draw(
            &mut canvas,
            density,
            &crate::postcard::CardSpec {
                card,
                sheet_width: sheet.width,
                settings: &settings.collage.postcard,
                text: sources[0].and_then(|s| s.postcard),
                font: card_font,
                paper: settings.border_color,
            },
        );
    }
    // A sheet with one photo has nothing to cut apart, and filled halves meet at an edge
    // that needs no line.
    let collage = &settings.collage;
    if collage.cut_line && !collage.fill && sheet.cells.iter().all(Option::is_some) {
        draw_cut_line(&mut canvas, &sheet, density, settings.border_color);
    }
    Ok(canvas)
}

/// What turns a stamp sized for a photo's own area into half the sheet's stamp (FR-004).
#[allow(clippy::cast_precision_loss)]
fn stamp_scale(print: PrintSize, area: (u32, u32)) -> f32 {
    print.short_mm as f32 / 2.0 / area.0.min(area.1).max(1) as f32
}

/// A thin light line between the cells, with darker ticks at the sheet's edges.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn draw_cut_line(canvas: &mut RgbImage, sheet: &SheetLayout, density: f32, paper: BorderColor) {
    let (cut_line, cut_tick) = match paper {
        BorderColor::White => (CUT_LINE, CUT_TICK),
        BorderColor::Black => (CUT_LINE_ON_BLACK, CUT_TICK_ON_BLACK),
    };
    let line = (CUT_LINE_MM * density).round().max(1.0) as u32;
    let tick_len = (CUT_TICK_MM * density).round().max(1.0) as u32;
    let (w, h) = (canvas.width, canvas.height);
    if sheet.side_by_side {
        let x = (w / 2).saturating_sub(line / 2);
        fill_rect(canvas, x, 0, line, h, cut_line);
        fill_rect(canvas, x, 0, line, tick_len, cut_tick);
        fill_rect(
            canvas,
            x,
            h.saturating_sub(tick_len),
            line,
            tick_len,
            cut_tick,
        );
    } else {
        let y = (h / 2).saturating_sub(line / 2);
        fill_rect(canvas, 0, y, w, line, cut_line);
        fill_rect(canvas, 0, y, tick_len, line, cut_tick);
        fill_rect(
            canvas,
            w.saturating_sub(tick_len),
            y,
            tick_len,
            line,
            cut_tick,
        );
    }
}

pub(crate) fn fill_rect(
    canvas: &mut RgbImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 3],
) {
    let block = filled(
        width.min(canvas.width.saturating_sub(x)).max(1),
        height.min(canvas.height.saturating_sub(y)).max(1),
        color,
    );
    blit(canvas, &block, x, y);
}

/// New collage positions after a reorder in one view (FR-035). `placed` holds the photos that
/// already have a position; `view` is the view's photos in their new order. The view's photos
/// share the positions they already held, in their new order, and those without one take
/// fresh positions after every other photo. Photos outside the view keep theirs, so a reorder
/// in one folder never touches another, and only the view's photos are written.
pub fn reorder(placed: &[(PhotoId, u32)], view: &[PhotoId]) -> Vec<(PhotoId, u32)> {
    let held: std::collections::HashMap<PhotoId, u32> = placed.iter().copied().collect();
    let mut slots: Vec<u32> = view.iter().filter_map(|id| held.get(id).copied()).collect();
    slots.sort_unstable();
    let mut next = placed
        .iter()
        .map(|(_, p)| *p)
        .max()
        .map_or(0, |p| p.saturating_add(1));
    while slots.len() < view.len() {
        slots.push(next);
        next = next.saturating_add(1);
    }
    view.iter().copied().zip(slots).collect()
}

/// Photos in collage order (FR-035): first those the user placed, by their position, then
/// the others in the view's order, so a photo added later joins at the end instead of
/// pushing every sheet along. Paired 1 + 2, 3 + 4…, an odd last photo alone; with `single`
/// (postcards, FR-073), one photo per sheet.
pub fn pairs(photos: &[(PhotoId, Option<u32>)], single: bool) -> Vec<(PhotoId, Option<PhotoId>)> {
    let mut placed: Vec<(u32, usize, PhotoId)> = photos
        .iter()
        .enumerate()
        .filter_map(|(i, (id, position))| position.map(|p| (p, i, *id)))
        .collect();
    placed.sort_unstable();
    let order: Vec<PhotoId> = placed
        .into_iter()
        .map(|(_, _, id)| id)
        .chain(
            photos
                .iter()
                .filter(|(_, p)| p.is_none())
                .map(|(id, _)| *id),
        )
        .collect();
    order
        .chunks(if single { 1 } else { 2 })
        .map(|pair| (pair[0], pair.get(1).copied()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::MAT_WHITE;

    fn print() -> PrintSize {
        PrintSize::by_id("10x15").unwrap_or(PrintSize::ALL[1])
    }

    #[test]
    fn fr_035_a_10x15_holds_two_10x7_5_cells_turned_to_the_first_photo() {
        // A portrait photo: the half-frame look, two 7.5 × 10 cells side by side.
        let (w, h, cells_lr) = cells(print(), side_by_side((4000, 6000), false));
        assert_eq!((w, h), (150.0, 100.0));
        assert_eq!(
            (cells_lr[1].x, cells_lr[1].width, cells_lr[1].height),
            (75.0, 75.0, 100.0)
        );
        // A landscape photo: the sheet stands up, two 10 × 7.5 cells one above the other.
        let (w, h, cells_tb) = cells(print(), side_by_side((6000, 4000), false));
        assert_eq!((w, h), (100.0, 150.0));
        assert_eq!(
            (cells_tb[1].y, cells_tb[1].width, cells_tb[1].height),
            (75.0, 100.0, 75.0)
        );
        // The user turns it back.
        assert!(side_by_side((6000, 4000), true));
    }

    #[test]
    fn fr_035_the_suggested_size_leaves_the_margin_free() {
        let s = CollageSettings::default();
        // 2:3 in a 7.5 × 10 cell, 5 mm a side: 6 × 9 cm.
        assert_eq!(
            suggested_area((75.0, 100.0), (4000, 6000), print(), &s),
            (60, 90)
        );
        assert_eq!(
            suggested_area((100.0, 75.0), (6000, 4000), print(), &s),
            (90, 60)
        );
        // 4:3 and square in the same cell.
        let four = CollageSettings {
            ratio: CellRatio::FourThree,
            ..s.clone()
        };
        assert_eq!(
            suggested_area((75.0, 100.0), (4000, 6000), print(), &four),
            (65, 86)
        );
        let square = CollageSettings {
            ratio: CellRatio::Square,
            ..s.clone()
        };
        assert_eq!(
            suggested_area((75.0, 100.0), (4000, 6000), print(), &square),
            (65, 65)
        );
        // The sheet's ratio on a 4:3 photo is still 2:3.
        let sheet = CollageSettings {
            ratio: CellRatio::Sheet,
            ..s.clone()
        };
        assert_eq!(
            suggested_area((75.0, 100.0), (3000, 4000), print(), &sheet),
            (60, 90)
        );
    }

    #[test]
    fn fr_035_a_typed_size_keeps_the_ratio_when_locked_and_is_clamped() {
        let typed = |short, long, lock| CollageSettings {
            size: Some(crate::settings::CellSize {
                short_mm: short,
                long_mm: long,
            }),
            lock_ratio: lock,
            ..CollageSettings::default()
        };
        let cell = (75.0, 100.0);
        // Locked: the short edge counts, 50 mm gives 50 × 75.
        assert_eq!(
            photo_area(cell, (4000, 6000), print(), &typed(50, 75, true)),
            (50, 75)
        );
        // Too large for the cell: back to the suggested size.
        assert_eq!(
            photo_area(cell, (4000, 6000), print(), &typed(70, 105, true)),
            (60, 90)
        );
        // Unlocked: both edges, each clamped inside the margin.
        assert_eq!(
            photo_area(cell, (4000, 6000), print(), &typed(55, 80, false)),
            (55, 80)
        );
        assert_eq!(
            photo_area(cell, (4000, 6000), print(), &typed(90, 120, false)),
            (65, 90)
        );
    }

    #[test]
    fn fr_035_without_the_cut_line_the_photos_keep_their_white() {
        let mut s = CollageSettings::default();
        let photos = [Some((4000, 6000)), Some((4000, 6000))];
        let with_line = layout(print(), photos, false, &s);
        s.cut_line = false;
        let without = layout(print(), photos, false, &s);
        // The same places: the line goes, the 15 mm of white between the photos stays.
        assert_eq!(with_line, without);
        let [Some(a), Some(b)] = without.cells else {
            panic!("two cells");
        };
        assert!((a.x - 7.5).abs() < 1e-3 && (b.x - 82.5).abs() < 1e-3);
        assert!((b.x - (a.x + 60.0) - 15.0).abs() < 1e-3);
    }

    #[test]
    fn fr_035_a_filled_cell_has_no_white_and_crops_to_the_cell() -> Result<(), RenderError> {
        let s = CollageSettings {
            fill: true,
            ..CollageSettings::default()
        };
        // Two landscape photos on a standing 10 × 15: each fills its 10 × 7.5 half.
        let sheet = layout(print(), [Some((6000, 4000)), Some((6000, 4000))], false, &s);
        let [Some(a), Some(b)] = sheet.cells else {
            panic!("two cells");
        };
        assert_eq!((a.area, a.x, a.y), ((100, 75), 0.0, 0.0));
        assert_eq!((b.area, b.y), ((100, 75), 75.0));
        // A portrait photo in a landscape half is cropped to the half, not letterboxed.
        let red = filled(1200, 1800, [200, 30, 30]);
        let blue = filled(1800, 1200, [30, 30, 200]);
        let mut settings = RenderSettings {
            collage: s,
            ..RenderSettings::default()
        };
        settings.stamp.enabled = false;
        // The cut line stays on: filled halves draw none.
        let source = |image| CellSource {
            image,
            capture: None,
            crop: None,
            straighten: 0.0,
            place: None,
            stamp: None,
            caption: None,
            postcard: None,
        };
        // The first photo lies, so the sheet stands; the portrait one is cropped to lie too.
        let out = render_sheet(
            &settings,
            print(),
            [Some(source(&blue)), Some(source(&red))],
            false,
            Some(1500),
            None,
        )?;
        assert_eq!((out.width, out.height), (1000, 1500));
        let at = |x: u32, y: u32| {
            let i = ((y * out.width + x) * 3) as usize;
            [out.data[i], out.data[i + 1], out.data[i + 2]]
        };
        assert_eq!(at(2, 2), [30, 30, 200], "no white at the corner");
        assert_eq!(
            at(998, 1497),
            [200, 30, 30],
            "the second photo reaches its corner"
        );
        assert_eq!(
            at(500, 750),
            [200, 30, 30],
            "no cut line between filled halves"
        );
        Ok(())
    }

    #[test]
    fn fr_004_each_stamp_on_a_sheet_is_half_the_sheets_own() {
        // 10 × 15 sheet, a 60 × 90 photo area: medium is 3.6 % of 100 mm alone, 1.8 mm here.
        let scale = stamp_scale(print(), (60, 90));
        let mm = crate::settings::StampSize::Medium.of_short_edge() * 60.0 * scale;
        assert!((mm - 1.8).abs() < 1e-4, "{mm}");
    }

    #[test]
    fn fr_035_the_border_leaves_5_4_by_8_4_of_photo() {
        let settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        let sheet = info(
            print(),
            [
                Some((PhotoId(1), (4000, 6000), None, 0.0)),
                Some((PhotoId(2), (4000, 6000), None, 0.0)),
            ],
            false,
            &settings,
        );
        let a = &sheet.cells[0];
        assert_eq!((a.area_width_mm, a.area_height_mm), (60, 90));
        assert!((a.photo_width_mm - 54.0).abs() < 1e-3 && (a.photo_height_mm - 84.0).abs() < 1e-3);
        assert_eq!(a.quality, DpiBand::Excellent);
    }

    #[test]
    fn fr_035_photos_pair_in_order_and_follow_their_position() {
        let ids: Vec<(PhotoId, Option<u32>)> = (0..5).map(|i| (PhotoId(i), None)).collect();
        assert_eq!(
            pairs(&ids, false),
            vec![
                (PhotoId(0), Some(PhotoId(1))),
                (PhotoId(2), Some(PhotoId(3))),
                (PhotoId(4), None)
            ]
        );
        // The user moved photo 4 to the front.
        let moved = vec![
            (PhotoId(0), Some(1)),
            (PhotoId(1), Some(2)),
            (PhotoId(2), Some(3)),
            (PhotoId(3), Some(4)),
            (PhotoId(4), Some(0)),
        ];
        assert_eq!(pairs(&moved, false)[0], (PhotoId(4), Some(PhotoId(0))));
        assert_eq!(
            pairs(&moved, true)[..2],
            [(PhotoId(4), None), (PhotoId(0), None)]
        );
        // A photo added later, first by date, joins at the end instead of shifting every sheet.
        let mut added = vec![(PhotoId(9), None)];
        added.extend(moved);
        let after = pairs(&added, false);
        assert_eq!(after[0], (PhotoId(4), Some(PhotoId(0))));
        assert_eq!(after[2], (PhotoId(3), Some(PhotoId(9))));
    }

    #[test]
    fn fr_035_a_reorder_in_one_folder_keeps_the_other_folders_order() {
        let id = PhotoId;
        // Folder A (1, 2, 3) was arranged first, then folder B (10, 11).
        let placed = [(id(1), 0), (id(2), 1), (id(3), 2), (id(10), 3), (id(11), 4)];
        // In folder B's tab the user swaps its two photos: only B's photos move.
        assert_eq!(
            reorder(&placed, &[id(11), id(10)]),
            [(id(11), 3), (id(10), 4)]
        );
        // A view with a new photo: it joins after every placed photo.
        assert_eq!(
            reorder(&placed, &[id(2), id(1), id(12)]),
            [(id(2), 0), (id(1), 1), (id(12), 5)]
        );
        // Nothing placed yet: positions from 0.
        assert_eq!(reorder(&[], &[id(7), id(6)]), [(id(7), 0), (id(6), 1)]);
    }

    #[test]
    fn fr_035_the_preview_source_gives_the_same_area_as_the_photo() {
        let s = CollageSettings::default();
        // 4000 × 6000 downscaled to the 2048 px preview source is 1365 × 2048.
        assert_eq!(
            photo_area((75.0, 100.0), (1365, 2048), print(), &s),
            photo_area((75.0, 100.0), (4000, 6000), print(), &s)
        );
    }

    /// FR-073: a postcard sheet puts its one photo on the left (landscape) or bottom
    /// (portrait) half, the dashed stamp box at the top right and the lines and typed text in
    /// the other half; nothing of the photo reaches the postcard half.
    #[test]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn fr_073_a_postcard_sheet_draws_one_photo_and_the_card() -> Result<(), RenderError> {
        use crate::settings::{PostcardOrientation, PostcardSettings};
        let image = filled(1800, 1200, [200, 30, 30]);
        let text = crate::library::PostcardText {
            message: "Liebe Grüße aus Budapest!".into(),
            address: "Anna Schmidt".into(),
        };
        let source = CellSource {
            image: &image,
            capture: None,
            crop: None,
            straighten: 0.0,
            place: None,
            stamp: None,
            caption: None,
            postcard: Some(&text),
        };
        for orientation in [
            PostcardOrientation::Landscape,
            PostcardOrientation::Portrait,
        ] {
            let mut settings = RenderSettings::default();
            settings.stamp.enabled = false;
            settings.collage.postcard = PostcardSettings {
                enabled: true,
                orientation,
                ..PostcardSettings::default()
            };
            let sheet = layout(
                print(),
                [Some((1800, 1200)), None],
                false,
                &settings.collage,
            );
            assert!(sheet.cells[1].is_none());
            let (Some(cell), Some(card)) = (sheet.cells[0], sheet.card) else {
                panic!("a photo and a card");
            };
            let out = render_sheet(
                &settings,
                print(),
                [Some(source), None],
                false,
                Some(1520),
                None,
            )?;
            let density = out.width as f32 / sheet.width;
            let at = |x_mm: f32, y_mm: f32| {
                let (x, y) = ((x_mm * density) as usize, (y_mm * density) as usize);
                let i = (y * out.width as usize + x) * 3;
                [out.data[i], out.data[i + 1], out.data[i + 2]]
            };
            let landscape = orientation == PostcardOrientation::Landscape;
            assert_eq!(sheet.side_by_side, landscape);
            // The photo is red in the middle of its half, which is left or bottom.
            let photo_mid = (
                cell.cell.x + cell.cell.width / 2.0,
                cell.cell.y + cell.cell.height / 2.0,
            );
            assert_eq!(at(photo_mid.0, photo_mid.1), [200, 30, 30]);
            if landscape {
                assert!(card.x > cell.cell.x);
            } else {
                assert!(card.y < cell.cell.y);
            }
            // The card half has no red at all, and some ink: box lines and the typed text.
            let (x0, y0) = ((card.x * density) as u32, (card.y * density) as u32);
            let (x1, y1) = (
                ((card.x + card.width) * density) as u32,
                ((card.y + card.height) * density) as u32,
            );
            let mut inked = 0usize;
            for y in y0..y1.min(out.height) {
                for x in x0..x1.min(out.width) {
                    let i = ((y * out.width + x) * 3) as usize;
                    let p = &out.data[i..i + 3];
                    assert!(!(p[0] > 150 && p[1] < 80), "red in the card at {x},{y}");
                    inked += usize::from(p[0] < 200);
                }
            }
            assert!(inked > 500, "{inked} inked pixels");
            // The stamp box's top edge is dashed: ink and paper alternate along it.
            let card_layout =
                crate::postcard::layout(sheet.width, card, 4, crate::postcard::ADDRESS_LINES);
            let edge_y = card_layout.stamp.y + 0.1;
            let along: Vec<bool> = (0..40)
                .map(|i| at(card_layout.stamp.x + 1.0 + i as f32 * 0.5, edge_y)[0] < 200)
                .collect();
            assert!(
                along.iter().any(|&a| a) && along.iter().any(|&a| !a),
                "{along:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn fr_035_a_lone_photo_stays_centred_and_has_no_cut_line() -> Result<(), RenderError> {
        let s = CollageSettings {
            cut_line: false,
            ..CollageSettings::default()
        };
        let alone = layout(print(), [Some((4000, 6000)), None], false, &s);
        let Some(a) = alone.cells[0] else {
            panic!("one cell");
        };
        assert!((a.x - 7.5).abs() < 1e-3, "centred in its half: {}", a.x);
        let image = filled(1200, 1800, [200, 30, 30]);
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let source = CellSource {
            image: &image,
            capture: None,
            crop: None,
            straighten: 0.0,
            place: None,
            stamp: None,
            caption: None,
            postcard: None,
        };
        let out = render_sheet(
            &settings,
            print(),
            [Some(source), None],
            false,
            Some(1500),
            None,
        )?;
        let middle = ((500 * out.width + 750) * 3) as usize;
        assert_eq!(
            &out.data[middle..middle + 3],
            &MAT_WHITE,
            "no line through an empty half"
        );
        Ok(())
    }

    #[test]
    fn fr_035_the_quality_follows_the_photos_own_crop() {
        let settings = RenderSettings::default();
        let dpi = |zoom| {
            let frame = CropFrame {
                cx: 0.5,
                cy: 0.5,
                zoom,
            };
            info(
                print(),
                [Some((PhotoId(1), (4000, 6000), Some(frame), 0.0)), None],
                false,
                &settings,
            )
            .cells[0]
                .dpi
        };
        assert!((f64::from(dpi(1.0)) / f64::from(dpi(2.0)) - 2.0).abs() < 0.05);
    }

    #[test]
    fn fr_035_a_sheet_renders_at_the_print_ratio_with_both_photos() -> Result<(), RenderError> {
        let a = filled(1200, 1800, [200, 30, 30]);
        let b = filled(1800, 1200, [30, 30, 200]);
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        let source = |image| CellSource {
            image,
            capture: None,
            crop: None,
            straighten: 0.0,
            place: None,
            stamp: None,
            caption: None,
            postcard: None,
        };
        let out = render_sheet(
            &settings,
            print(),
            [Some(source(&a)), Some(source(&b))],
            false,
            Some(1500),
            None,
        )?;
        // A portrait first photo: a landscape sheet, 3:2, at most 1500 px long.
        assert_eq!(out.width, 1500);
        assert_eq!(out.height, 1000);
        let at = |x: u32, y: u32| {
            let i = ((y * out.width + x) * 3) as usize;
            [out.data[i], out.data[i + 1], out.data[i + 2]]
        };
        assert_eq!(
            at(375, 500),
            [200, 30, 30],
            "the first photo in the left cell"
        );
        assert_eq!(at(1125, 500), [30, 30, 200], "the second in the right cell");
        assert_eq!(at(10, 10), MAT_WHITE, "the margin is white");
        assert_eq!(at(750, 500), CUT_LINE, "the cut line in the middle");
        Ok(())
    }

    #[test]
    fn fr_067_black_paper_colours_the_margins_and_the_border() -> Result<(), RenderError> {
        let a = filled(1200, 1800, [200, 30, 30]);
        let mut settings = RenderSettings::default();
        settings.stamp.enabled = false;
        settings.border = true;
        settings.border_color = BorderColor::Black;
        let source = |image| CellSource {
            image,
            capture: None,
            crop: None,
            straighten: 0.0,
            place: None,
            stamp: None,
            caption: None,
            postcard: None,
        };
        let out = render_sheet(
            &settings,
            print(),
            [Some(source(&a)), Some(source(&a))],
            false,
            Some(1500),
            None,
        )?;
        let at = |x: u32, y: u32| {
            let i = ((y * out.width + x) * 3) as usize;
            [out.data[i], out.data[i + 1], out.data[i + 2]]
        };
        assert_eq!(at(10, 10), BorderColor::Black.rgb(), "the margin is black");
        assert_eq!(at(750, 500), CUT_LINE_ON_BLACK, "a darker cut line");
        assert_eq!(at(375, 500), [200, 30, 30], "the photo itself");
        Ok(())
    }

    #[test]
    fn fr_035_turning_a_photo_in_its_cell_keeps_the_sheet() {
        let s = CollageSettings::default();
        // A portrait photo: two side by side, the half-frame look.
        let upright = (4000, 6000);
        let before = layout(print(), [Some(upright), Some(upright)], false, &s);
        // Turned a quarter in its cell, it lies; the sheet must not stand up with it.
        let lying = (6000, 4000);
        let turned = layout_turned(false, 1, lying);
        let after = layout(print(), [Some(lying), Some(upright)], turned, &s);
        assert_eq!(before.side_by_side, after.side_by_side);
        assert_eq!((before.width, before.height), (after.width, after.height));
        // Two quarter turns, or a square photo, change nothing; "Turn sheet" still turns it.
        assert!(!layout_turned(false, 2, lying));
        assert!(!layout_turned(false, 1, (5000, 5000)));
        assert!(layout_turned(true, 0, upright));
    }

    #[test]
    fn fr_035_auto_turn_makes_every_photo_lie_like_its_cell() {
        let (portrait, landscape, square) = ((4000, 6000), (6000, 4000), (5000, 5000));
        // A portrait first photo: side by side, standing cells; a landscape partner turns.
        assert_eq!(fitting_turns(false, portrait, portrait), 0);
        assert_eq!(fitting_turns(false, portrait, landscape), 1);
        // A landscape first photo: one above the other, lying cells; a portrait one turns.
        assert_eq!(fitting_turns(false, landscape, landscape), 0);
        assert_eq!(fitting_turns(false, landscape, portrait), 1);
        // A sheet turned by hand turns its first photo too, so both still match.
        assert_eq!(fitting_turns(true, portrait, portrait), 1);
        assert_eq!(fitting_turns(false, portrait, square), 0);
        // Auto turn lays every sheet landscape: side by side whatever its first photo.
        for first in [portrait, landscape, square] {
            assert!(side_by_side(first, landscape_turned(first)));
            assert_eq!(fitting_turns(landscape_turned(first), first, landscape), 1);
            assert_eq!(fitting_turns(landscape_turned(first), first, portrait), 0);
        }
    }
}
