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

/// The sheets of the photos `filter` keeps, in collage order (FR-035).
pub fn sheet_pairs(library: &Library, filter: &CullFilter) -> Vec<SheetPair> {
    let photos: Vec<(PhotoId, Option<u32>)> = library
        .filtered_ids(filter)
        .into_iter()
        .map(|id| (id, library.edits(id).collage))
        .collect();
    sheet::pairs(&photos)
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
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    Ok(sheet_pairs(&library, &filter))
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
        let sheets: Vec<(
            fotollection_core::library::PhotoRecord,
            Option<fotollection_core::library::PhotoRecord>,
        )> = {
            let library = state
                .library
                .read()
                .map_err(|e| IpcError::internal(e.to_string()))?;
            sheet_pairs(&library, &filter)
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

/// Bytes for `fotothumb://localhost/sheet/{first}/{second}`: the sheet with the current
/// settings, `SMALL_SHEET` or `LARGE_SHEET` pixels on its long edge.
pub fn sheet_bytes(state: &AppState, sheet: SheetPair, small: bool) -> Result<Vec<u8>, IpcError> {
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
    let (place_a, place_b) = (place(&first), second.as_ref().and_then(place));
    // The grid's sheets come from the photos' thumbnails: no photo is decoded again when a
    // setting changes (FR-035, OPTIMIZATION §5).
    let thumbnail = |r: &fotollection_core::library::PhotoRecord| state.thumbs.cached_t1(r);
    state
        .previews
        .render_sheet_jpeg(
            [
                Some((&first, place_a.as_deref())),
                second.as_ref().map(|r| (r, place_b.as_deref())),
            ],
            &settings,
            if small { SMALL_SHEET } else { LARGE_SHEET },
            small.then_some(&thumbnail as &dyn Fn(&_) -> _),
            state.codec.as_ref(),
        )
        .map_err(|e| crate::commands::shown_error(&e, &first))
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
}
