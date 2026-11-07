// SPDX-License-Identifier: GPL-3.0-only
//! Collage mode (FR-035, FR-048, ADR-0032): the sheets of the photos shown, their order,
//! turning a sheet, what the panel says about a sheet, and its preview.

// Tauri injects `State`, `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::library::{CullFilter, Library, PhotoId};
use fotollection_core::sheet::{self, SheetInfo};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::commands::{CropGeometry, HistoryState, edit_photos, place_names, upright_size};
use crate::state::{AppState, IpcError};

/// Long edge of a sheet in the grid, and in the preview.
pub const SMALL_SHEET: u32 = 640;
pub const LARGE_SHEET: u32 = fotollection_core::preview::PREVIEW_LONG_EDGE;

/// Two photos on one print: the first and, unless the last photo was odd, the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SheetPair {
    pub first: PhotoId,
    pub second: Option<PhotoId>,
}

/// The sheets of the photos `filter` keeps, in collage order (FR-035); one photo each with
/// `single` (postcards, FR-073).
pub fn sheet_pairs(library: &Library, filter: &CullFilter, single: bool) -> Vec<SheetPair> {
    let photos: Vec<(PhotoId, Option<u32>)> = library
        .filtered_ids(filter)
        .into_iter()
        .map(|id| (id, library.edits(id).collage))
        .collect();
    sheet::pairs(&photos, single)
        .into_iter()
        .map(|(first, second)| SheetPair { first, second })
        .collect()
}

/// `pairs`, or only the sheet holding the photo `only` (none when no sheet holds it).
pub fn only_sheet(pairs: Vec<SheetPair>, only: Option<PhotoId>) -> Vec<SheetPair> {
    match only {
        Some(id) => pairs
            .into_iter()
            .filter(|p| p.first == id || p.second == Some(id))
            .take(1)
            .collect(),
        None => pairs,
    }
}

/// The sheets of the photos shown, in order (FR-035).
#[tauri::command]
#[specta::specta]
pub fn list_sheets(
    state: State<'_, AppState>,
    filter: CullFilter,
) -> Result<Vec<SheetPair>, IpcError> {
    let single = postcards(&state);
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    Ok(sheet_pairs(&library, &filter, single))
}

/// A postcard whose typed text does not all fit its lines (FR-073).
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PostcardCut {
    pub photo: PhotoId,
    pub fit: fotollection_core::postcard::Fit,
}

/// The postcards whose message or address is cut on the print, for the red warning on the
/// card and beside its field (FR-073); empty while postcards are off. Settings and the
/// font are read first, then the library: no lock is taken while another is held.
#[tauri::command]
#[specta::specta]
pub fn postcard_cuts(state: State<'_, AppState>) -> Result<Vec<PostcardCut>, IpcError> {
    let settings = state
        .settings
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .0
        .clone();
    if !settings.collage.postcard.enabled {
        return Ok(Vec::new());
    }
    let font = state
        .frame
        .read()
        .ok()
        .and_then(|f| crate::commands::postcard_font_of(&f, &settings));
    let print = sheet::sheet_print(&settings);
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    Ok(library
        .sorted_ids()
        .into_iter()
        .filter_map(|id| {
            let text = library.record(id)?.edits.postcard?;
            let fit = sheet::postcard_fit(print, &settings, &text, font.as_ref().map(|f| &f.font));
            (fit.message_cut || fit.address_cut).then_some(PostcardCut { photo: id, fit })
        })
        .collect())
}

/// Sets one part of a postcard's text on a photo's edits, the other part kept (FR-073).
pub fn set_part(
    edits: &mut fotollection_core::library::PhotoEdits,
    part: fotollection_core::settings::PostcardPart,
    text: &str,
) {
    use fotollection_core::settings::PostcardPart;
    let mut both = edits.postcard.clone().unwrap_or_default();
    match part {
        PostcardPart::Message => text.clone_into(&mut both.message),
        PostcardPart::Address => text.clone_into(&mut both.address),
    }
    edits.postcard = both.sanitized();
}

/// One part of a photo's postcard text, empty when there is none.
fn part_of(
    library: &Library,
    id: PhotoId,
    part: fotollection_core::settings::PostcardPart,
) -> String {
    use fotollection_core::settings::PostcardPart;
    library
        .record(id)
        .and_then(|r| r.edits.postcard)
        .map(|t| match part {
            PostcardPart::Message => t.message,
            PostcardPart::Address => t.address,
        })
        .unwrap_or_default()
}

/// The other postcards a card's text could go to (FR-077): the ticked ones among those
/// shown when some are ticked, else every one shown; never the card itself.
fn other_cards(
    library: &Library,
    filter: &CullFilter,
    source: PhotoId,
    ticked: &[PhotoId],
) -> Vec<PhotoId> {
    let ticked: std::collections::HashSet<PhotoId> = ticked.iter().copied().collect();
    library
        .filtered_ids(filter)
        .into_iter()
        .filter(|id| *id != source && (ticked.is_empty() || ticked.contains(id)))
        .collect()
}

/// How many other postcards lack this card's text, and how many have a different one: the
/// counts on "Use for N postcards" (FR-077). Zero while postcards are off.
#[derive(Debug, Clone, Copy, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TextGaps {
    pub blank: u32,
    pub differ: u32,
}

#[tauri::command]
#[specta::specta]
pub fn postcard_text_gaps(
    state: State<'_, AppState>,
    filter: CullFilter,
    source: PhotoId,
    part: fotollection_core::settings::PostcardPart,
    ticked: Vec<PhotoId>,
) -> Result<TextGaps, IpcError> {
    crate::commands::check_ticked(&ticked)?;
    if !postcards(&state) {
        return Ok(TextGaps::default());
    }
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    let text = part_of(&library, source, part);
    if text.is_empty() {
        return Ok(TextGaps::default());
    }
    let mut gaps = TextGaps::default();
    for id in other_cards(&library, &filter, source, &ticked) {
        let other = part_of(&library, id, part);
        if other.is_empty() {
            gaps.blank += 1;
        } else if other != text {
            gaps.differ += 1;
        }
    }
    Ok(gaps)
}

/// This card's message or address on the other postcards (FR-077): those still blank, or
/// with `overwrite` every other one too. One undoable step; returns how many changed.
#[tauri::command]
#[specta::specta]
pub fn apply_postcard_text(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: CullFilter,
    source: PhotoId,
    part: fotollection_core::settings::PostcardPart,
    overwrite: bool,
    ticked: Vec<PhotoId>,
) -> Result<u32, IpcError> {
    crate::commands::check_ticked(&ticked)?;
    let (text, ids) = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let text = part_of(&library, source, part);
        let ids: Vec<PhotoId> = other_cards(&library, &filter, source, &ticked)
            .into_iter()
            .filter(|id| {
                let other = part_of(&library, *id, part);
                other != text && (overwrite || other.is_empty())
            })
            .collect();
        (text, ids)
    };
    if text.is_empty() {
        return Err(IpcError::new(
            "postcard.no_text",
            "This postcard has no text to use",
        ));
    }
    if ids.is_empty() {
        return Ok(0);
    }
    edit_photos(&app, &state, &ids, |_, e| set_part(e, part, &text))?;
    if part == fotollection_core::settings::PostcardPart::Address {
        remember_address(&state, &text);
    }
    Ok(u32::try_from(ids.len()).unwrap_or(u32::MAX))
}

/// Puts an address first among those this machine offers on the next cards, and saves it
/// (FR-077). The list's lock is let go before the settings are written.
pub fn remember_address(state: &AppState, address: &str) {
    let changed = state.recent_addresses.lock().is_ok_and(|mut recent| {
        let before = recent.clone();
        fotollection_core::settings::remember_address(&mut recent, address);
        *recent != before
    });
    if changed {
        state.save_settings();
    }
}

/// The addresses typed on this machine's last postcards, newest first (FR-077).
#[tauri::command]
#[specta::specta]
pub fn recent_addresses(state: State<'_, AppState>) -> Result<Vec<String>, IpcError> {
    state
        .recent_addresses
        .lock()
        .map(|r| r.clone())
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Forgets one remembered address (FR-077); returns the list left.
#[tauri::command]
#[specta::specta]
pub fn forget_address(
    state: State<'_, AppState>,
    address: String,
) -> Result<Vec<String>, IpcError> {
    let left = {
        let mut recent = state
            .recent_addresses
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        recent.retain(|a| *a != address);
        recent.clone()
    };
    state.save_settings();
    Ok(left)
}

/// Whether collage sheets are postcards (FR-073): one photo each. Read it before taking the
/// library lock, so no lock is ever taken while another is held.
pub fn postcards(state: &AppState) -> bool {
    state
        .settings
        .read()
        .is_ok_and(|s| s.0.collage.postcard.enabled)
}

/// The collage order after a drag: `ids` are the photos shown, in their new order. A turn
/// belongs to a sheet, so a new order starts every sheet the way its first photo suits.
/// One undoable step (FR-035).
#[tauri::command]
#[specta::specta]
pub fn set_collage_order(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
) -> Result<HistoryState, IpcError> {
    let mut seen = std::collections::HashSet::new();
    if !ids.iter().all(|id| seen.insert(*id)) {
        return Err(IpcError::new(
            "photo.invalid_selection",
            "A photo is listed twice",
        ));
    }
    // One order for the whole library: the view's photos swap the positions they hold, so
    // reordering one folder keeps the others' order (FR-035).
    let placed: Vec<(PhotoId, u32)> = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .edited()
        .filter_map(|(id, e)| e.collage.map(|p| (id, p)))
        .collect();
    let positions: std::collections::HashMap<PhotoId, u32> =
        sheet::reorder(&placed, &ids).into_iter().collect();
    edit_photos(&app, &state, &ids, |id, e| {
        e.collage = positions.get(&id).copied();
        e.sheet_turned = false;
    })
}

/// Turns the sheet that `first` starts against its photo, or back (FR-035). Undoable.
#[tauri::command]
#[specta::specta]
pub fn turn_sheet(
    app: AppHandle,
    state: State<'_, AppState>,
    first: PhotoId,
) -> Result<HistoryState, IpcError> {
    edit_photos(&app, &state, &[first], |_, e| {
        e.sheet_turned = !e.sheet_turned;
    })
}

/// Auto turn (FR-035), for prints cut in two: every sheet shown lies landscape, its two
/// photos standing side by side, each photo turned to stand if it lies, so no standing photo
/// sits next to a lying one. With `revert`, every sheet and photo goes back to its own
/// orientation. A photo that turns gets its sheet crop centred again, as with Rotate photo.
/// One undoable step; returns how many photos changed (0: nothing to do).
#[tauri::command]
#[specta::specta]
pub async fn auto_turn_collage(
    app: AppHandle,
    filter: CullFilter,
    revert: bool,
) -> Result<u32, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        // A postcard is held the way the user chose and its photo stays upright on it:
        // nothing to turn (FR-073).
        if postcards(&state) {
            return Ok(0);
        }
        let sheets: Vec<(
            fotollection_core::library::PhotoRecord,
            Option<fotollection_core::library::PhotoRecord>,
        )> = {
            let library = state
                .library
                .read()
                .map_err(|e| IpcError::internal(e.to_string()))?;
            sheet_pairs(&library, &filter, false)
                .into_iter()
                .filter_map(|p| {
                    Some((
                        library.record(p.first)?,
                        p.second.and_then(|id| library.record(id)),
                    ))
                })
                .collect()
        };
        // Per photo: its quarter turns in the cell and, for a sheet's first photo, the sheet's turn.
        let mut wanted: std::collections::HashMap<PhotoId, (u8, Option<bool>)> =
            std::collections::HashMap::new();
        for (first, second) in &sheets {
            if revert {
                wanted.insert(first.id, (0, Some(false)));
                if let Some(r) = second {
                    wanted.insert(r.id, (0, None));
                }
                continue;
            }
            let base = upright_size(&state, first)?;
            let turned = sheet::landscape_turned(base);
            wanted.insert(
                first.id,
                (sheet::fitting_turns(turned, base, base), Some(turned)),
            );
            if let Some(r) = second {
                let size = upright_size(&state, r)?;
                wanted.insert(r.id, (sheet::fitting_turns(turned, base, size), None));
            }
        }
        let ids: Vec<PhotoId> = sheets
            .iter()
            .flat_map(|(a, b)| std::iter::once(a).chain(b.as_ref()))
            .filter(|r| {
                wanted.get(&r.id).is_some_and(|(t, sheet)| {
                    *t != r.edits.sheet_turns() || sheet.is_some_and(|s| s != r.edits.sheet_turned)
                })
            })
            .map(|r| r.id)
            .collect();
        if ids.is_empty() {
            return Ok(0);
        }
        edit_photos(&app, &state, &ids, |id, e| {
            let Some((turns, sheet)) = wanted.get(&id).copied() else {
                return;
            };
            if let Some(turned) = sheet {
                e.sheet_turned = turned;
            }
            if turns == e.sheet_turns() {
                return;
            }
            let zoom = e.sheet_crop().map_or(1.0, |c| c.zoom);
            e.collage_rotation = turns;
            e.collage_crop = Some(
                fotollection_core::plan::CropFrame {
                    cx: 0.5,
                    cy: 0.5,
                    zoom,
                }
                .sanitized(),
            );
        })?;
        Ok(u32::try_from(ids.len()).unwrap_or(u32::MAX))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Sets a photo's crop and straightening on collage sheets, apart from its print's; `None`
/// (Reset in the crop editor) centres it. One undoable step (FR-035).
#[tauri::command]
#[specta::specta]
pub fn set_collage_crop(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
    crop: Option<fotollection_core::plan::CropFrame>,
    straighten: f32,
) -> Result<HistoryState, IpcError> {
    use fotollection_core::plan::CropFrame;
    let degrees = fotollection_core::straighten::sanitize(straighten);
    let centred = CropFrame {
        cx: 0.5,
        cy: 0.5,
        zoom: 1.0,
    };
    edit_photos(&app, &state, &[id], |_, e| {
        e.collage_crop = Some(crop.unwrap_or(centred).sanitized());
        e.collage_straighten = Some(degrees);
    })
}

/// The photo's size as it sits in its collage cell: upright, then its own turns there
/// (FR-035).
pub(crate) fn sheet_size(
    state: &AppState,
    record: &fotollection_core::library::PhotoRecord,
) -> Result<(u32, u32), IpcError> {
    let (w, h) = upright_size(state, record)?;
    Ok(if record.edits.sheet_turns() % 2 == 1 {
        (h, w)
    } else {
        (w, h)
    })
}

/// Turns a photo a quarter clockwise in its collage cell, so a landscape photo can fill a
/// standing cell (FR-035); its print and the other modes keep their orientation. The
/// sheet crop starts again from the centre, at its zoom, since the old frame no longer
/// lies on the same pixels. One undoable step.
#[tauri::command]
#[specta::specta]
pub fn rotate_collage_photo(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
) -> Result<HistoryState, IpcError> {
    use fotollection_core::plan::CropFrame;
    edit_photos(&app, &state, &[id], |_, e| {
        let zoom = e.sheet_crop().map_or(1.0, |c| c.zoom);
        e.collage_rotation = (e.sheet_turns() + 1) % 4;
        e.collage_crop = Some(
            CropFrame {
                cx: 0.5,
                cy: 0.5,
                zoom,
            }
            .sanitized(),
        );
    })
}

/// What the panel says about a sheet: its size, each photo's area, the suggested size,
/// the photo left inside the border and the print quality (FR-035, FR-026).
#[tauri::command]
#[specta::specta]
pub async fn sheet_info(app: AppHandle, sheet: SheetPair) -> Result<SheetInfo, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (first, second) = {
            let library = state
                .library
                .read()
                .map_err(|e| IpcError::internal(e.to_string()))?;
            let record = |id: PhotoId| {
                library
                    .record(id)
                    .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))
            };
            (record(sheet.first)?, sheet.second.map(record).transpose()?)
        };
        let settings = state
            .settings
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .0
            .clone();
        let fact =
            |r: &fotollection_core::library::PhotoRecord| -> Result<sheet::SheetPhoto, IpcError> {
                Ok((
                    r.id,
                    sheet_size(&state, r)?,
                    r.edits.sheet_crop(),
                    r.edits.sheet_straighten(),
                ))
            };
        let a = fact(&first)?;
        let b = second.as_ref().map(fact).transpose()?;
        let turned = sheet::layout_turned(first.edits.sheet_turned, first.edits.sheet_turns(), a.1);
        Ok(sheet::info(
            sheet::sheet_print(&settings),
            [Some(a), b],
            turned,
            &settings,
        ))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// What the crop editor needs for a photo of a collage sheet (FR-035, FR-006): the crop
/// fills the photo's area, and the printed size is the photo left inside the border.
pub fn cell_geometry(
    state: &AppState,
    record: &fotollection_core::library::PhotoRecord,
    size: (u32, u32),
    sheet: SheetPair,
    settings: &fotollection_core::settings::RenderSettings,
) -> Result<CropGeometry, IpcError> {
    let partner = if sheet.first == record.id {
        sheet.second
    } else if sheet.second == Some(record.id) {
        Some(sheet.first)
    } else {
        return Err(IpcError::new(
            "photo.unknown",
            "The photo is not on this sheet",
        ));
    };
    let (first, partner) = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let first = library
            .record(sheet.first)
            .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
        (first, partner.and_then(|id| library.record(id)))
    };
    let other = match &partner {
        Some(r) => Some(sheet_size(state, r)?),
        None => None,
    };
    let first_size = if sheet.first == record.id {
        size
    } else {
        other.unwrap_or(size)
    };
    let turned = sheet::layout_turned(
        first.edits.sheet_turned,
        first.edits.sheet_turns(),
        first_size,
    );
    let photos = if sheet.first == record.id {
        [Some(size), other]
    } else {
        [other, Some(size)]
    };
    let layout = sheet::layout(
        sheet::sheet_print(settings),
        photos,
        turned,
        &settings.collage,
    );
    let index = usize::from(sheet.first != record.id);
    let cell = layout.cells[index].ok_or_else(|| IpcError::internal("an empty cell"))?;
    let inset = sheet::border_inset(settings);
    #[allow(clippy::cast_precision_loss)]
    let (area_w, area_h) = (cell.area.0 as f32, cell.area.1 as f32);
    // The crop has the ratio of the photo inside the border.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (target_width, target_height) = (
        ((area_w - inset).max(1.0) * 10.0).round() as u32,
        ((area_h - inset).max(1.0) * 10.0).round() as u32,
    );
    Ok(CropGeometry {
        width: size.0,
        height: size.1,
        target_width,
        target_height,
        print_width_cm: (area_w - inset).max(1.0) / 10.0,
        print_height_cm: (area_h - inset).max(1.0) / 10.0,
        crop: record.edits.sheet_crop(),
        straighten: record.edits.sheet_straighten(),
        crop_disabled: false,
        output_width_cm: area_w / 10.0,
        output_height_cm: area_h / 10.0,
        print_grows: false,
        unit: fotollection_core::settings::LengthUnit::Cm,
    })
}

/// What a sheet is drawn from: its photos, the settings, their places and the card's font.
struct SheetInputs {
    first: fotollection_core::library::PhotoRecord,
    second: Option<fotollection_core::library::PhotoRecord>,
    settings: fotollection_core::settings::RenderSettings,
    places: [Option<String>; 2],
    card_font: Option<fotollection_core::fonts::LoadedFont>,
}

fn sheet_inputs(state: &AppState, sheet: SheetPair) -> Result<SheetInputs, IpcError> {
    let unknown = || IpcError::new("photo.unknown", "unknown photo");
    let (first, second) = {
        let library = state.library.read().map_err(crate::document::poisoned)?;
        let first = library.record(sheet.first).ok_or_else(unknown)?;
        let second = match sheet.second {
            Some(id) => Some(library.record(id).ok_or_else(unknown)?),
            None => None,
        };
        (first, second)
    };
    let settings = state
        .settings
        .read()
        .map_err(crate::document::poisoned)?
        .0
        .clone();
    let names = place_names(state);
    let place = |r: &fotollection_core::library::PhotoRecord| {
        crate::state::place_for(r, &names, settings.place_format)
    };
    let places = [place(&first), second.as_ref().and_then(place)];
    let card_font = state
        .frame
        .read()
        .ok()
        .and_then(|f| crate::commands::postcard_font_of(&f, &settings));
    Ok(SheetInputs {
        first,
        second,
        settings,
        places,
        card_font,
    })
}

/// Bytes for `fotothumb://localhost/sheet/{first}/{second}`: the sheet with the current
/// settings, `SMALL_SHEET` or `LARGE_SHEET` pixels on its long edge, its photos' clipping
/// marked when `clip` (FR-098, `J`).
pub fn sheet_bytes(
    state: &AppState,
    sheet: SheetPair,
    small: bool,
    clip: bool,
) -> Result<Vec<u8>, IpcError> {
    let SheetInputs {
        first,
        second,
        settings,
        places,
        card_font,
    } = sheet_inputs(state, sheet)?;
    // The grid's sheets come from the photos' thumbnails: no photo is decoded again when a
    // setting changes (FR-035, OPTIMIZATION §5).
    let thumbnail = |r: &fotollection_core::library::PhotoRecord| state.thumbs.cached_t1(r);
    state
        .previews
        .render_sheet_jpeg(
            [
                Some((&first, places[0].as_deref())),
                second.as_ref().map(|r| (r, places[1].as_deref())),
            ],
            &settings,
            if small { SMALL_SHEET } else { LARGE_SHEET },
            small.then_some(&thumbnail as &dyn Fn(&_) -> _),
            card_font.as_ref().map(|f| &f.font),
            clip,
            state.codec.as_ref(),
        )
        .map_err(|e| crate::commands::shown_error(&e, &first))
}

/// Sets the warmth and tint of the photo `id` on a sheet so the spot clicked at (`fx`, `fy`)
/// of the sheet's preview turns neutral grey (FR-098, `W` in Collage mode), as one undo step;
/// a point off that photo (the other photo, the card, the paper, the stamp) is `ERR-IPC-0026`.
#[tauri::command]
#[specta::specta]
pub async fn pick_white_balance_on_sheet(
    app: AppHandle,
    sheet: SheetPair,
    id: PhotoId,
    fx: f32,
    fy: f32,
) -> Result<crate::commands::HistoryState, IpcError> {
    crate::commands::check_pick_point(fx, fy)?;
    let cell = if sheet.first == id {
        0
    } else if sheet.second == Some(id) {
        1
    } else {
        return Err(IpcError::new("photo.unknown", "unknown photo"));
    };
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let SheetInputs {
            first,
            second,
            settings,
            places,
            card_font,
        } = sheet_inputs(&state, sheet)?;
        let picked = if cell == 0 {
            first.clone()
        } else {
            second.clone().unwrap_or_else(|| first.clone())
        };
        let sample = state
            .previews
            .neutral_sample_on_sheet(
                [
                    Some((&first, places[0].as_deref())),
                    second.as_ref().map(|r| (r, places[1].as_deref())),
                ],
                &settings,
                card_font.as_ref().map(|f| &f.font),
                cell,
                (fx, fy),
                state.codec.as_ref(),
            )
            .map_err(|e| crate::commands::shown_error(&e, &picked))?
            .ok_or_else(crate::commands::not_on_photo)?;
        crate::commands::set_white_balance_from(&app, &state, &picked, &settings, sample)
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_035_a_sheet_exported_from_its_preview_is_the_one_holding_the_photo() {
        let pair = |a: u32, b: Option<u32>| SheetPair {
            first: PhotoId(a),
            second: b.map(PhotoId),
        };
        let all = vec![pair(1, Some(2)), pair(3, Some(4)), pair(5, None)];
        assert_eq!(only_sheet(all.clone(), None), all);
        assert_eq!(
            only_sheet(all.clone(), Some(PhotoId(4))),
            vec![pair(3, Some(4))]
        );
        assert_eq!(
            only_sheet(all.clone(), Some(PhotoId(5))),
            vec![pair(5, None)]
        );
        assert_eq!(only_sheet(all, Some(PhotoId(9))), Vec::new());
    }

    /// FR-077: an address put on a card keeps its message, and a blank one leaves no text.
    #[test]
    fn fr_077_setting_one_part_keeps_the_other() {
        use fotollection_core::library::{PhotoEdits, PostcardText};
        use fotollection_core::settings::PostcardPart;
        let mut edits = PhotoEdits {
            postcard: Some(PostcardText {
                message: "Ahoj Praha!".into(),
                address: String::new(),
            }),
            ..PhotoEdits::default()
        };
        set_part(
            &mut edits,
            PostcardPart::Address,
            "Národní muzeum\n110 00 Praha 1",
        );
        let Some(text) = edits.postcard.clone() else {
            panic!("the card has text");
        };
        assert_eq!(text.message, "Ahoj Praha!");
        assert_eq!(text.address, "Národní muzeum\n110 00 Praha 1");
        set_part(&mut edits, PostcardPart::Message, "");
        set_part(&mut edits, PostcardPart::Address, "  ");
        assert_eq!(edits.postcard, None);
    }
}
