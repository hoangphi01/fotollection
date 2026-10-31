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
    // One order for the whole library, so reordering one folder keeps the others' order.
    let placed: Vec<(PhotoId, u32)> = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .edited()
        .filter_map(|(id, e)| e.collage.map(|p| (id, p)))
        .collect();
    let order = sheet::reorder(&placed, &ids);
    let position = |id: PhotoId| {
        order
            .iter()
            .position(|i| *i == id)
            .and_then(|p| u32::try_from(p).ok())
    };
    edit_photos(&app, &state, &order, |id, e| {
        e.collage = position(id);
        if ids.contains(&id) {
            e.sheet_turned = false;
        }
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

/// Sets (or, with `None`, resets to the print crop) a photo's crop on collage sheets, with
/// its straightening, which the photo shares with its prints. One undoable step (FR-035).
#[tauri::command]
#[specta::specta]
pub fn set_collage_crop(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
    crop: Option<fotollection_core::plan::CropFrame>,
    straighten: f32,
) -> Result<HistoryState, IpcError> {
    let degrees = fotollection_core::straighten::sanitize(straighten);
    edit_photos(&app, &state, &[id], |_, e| {
        e.collage_crop = crop.map(fotollection_core::plan::CropFrame::sanitized);
        e.straighten = degrees;
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
                    upright_size(&state, r)?,
                    r.edits.sheet_crop(),
                    r.edits.straighten,
                ))
            };
        let a = fact(&first)?;
        let b = second.as_ref().map(fact).transpose()?;
        Ok(sheet::info(
            sheet::sheet_print(&settings),
            [Some(a), b],
            first.edits.sheet_turned,
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
    let (first, turned) = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let first = library
            .record(sheet.first)
            .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
        let other = partner.and_then(|id| library.record(id));
        (other, first.edits.sheet_turned)
    };
    let other = match &first {
        Some(r) => Some(upright_size(state, r)?),
        None => None,
    };
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
        straighten: record.edits.straighten,
        crop_disabled: false,
        output_width_cm: area_w / 10.0,
        output_height_cm: area_h / 10.0,
        print_grows: false,
        unit: fotollection_core::settings::LengthUnit::Cm,
    })
}

/// Bytes for `fotothumb://localhost/sheet/{first}/{second}`: the sheet with the current
/// settings, `SMALL_SHEET` or `LARGE_SHEET` pixels on its long edge.
pub fn sheet_bytes(state: &AppState, sheet: SheetPair, small: bool) -> Result<Vec<u8>, String> {
    let (first, second) = {
        let library = state.library.read().map_err(|e| e.to_string())?;
        let first = library.record(sheet.first).ok_or("unknown photo")?;
        let second = match sheet.second {
            Some(id) => Some(library.record(id).ok_or("unknown photo")?),
            None => None,
        };
        (first, second)
    };
    let settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
    let names = place_names(state);
    let place = |r: &fotollection_core::library::PhotoRecord| {
        crate::state::place_for(r, &names, settings.place_format)
    };
    let (place_a, place_b) = (place(&first), second.as_ref().and_then(place));
    state
        .previews
        .render_sheet_jpeg(
            [
                Some((&first, place_a.as_deref())),
                second.as_ref().map(|r| (r, place_b.as_deref())),
            ],
            &settings,
            if small { SMALL_SHEET } else { LARGE_SHEET },
            state.codec.as_ref(),
        )
        .map_err(|e| e.to_string())
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
