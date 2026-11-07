// SPDX-License-Identifier: GPL-3.0-only
//! Copy, paste and reset develop settings (FR-097), as Lightroom's *Copy settings*,
//! *Paste settings* (*Sync* on ticked photos) and *Reset*. A paste or a reset over many
//! photos is one undoable step, named so in the History list.
//!
//! State kept (`STYLE_RULE` §2): [`AppState::copied`], one photo's develop edits and the
//! parts chosen, replaced by the next copy and emptied when another project opens.
// Tauri hands commands their `State` and `AppHandle` by value.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::history::StepKind;
use fotollection_core::library::{PhotoEdits, PhotoId, SettingsParts};
use tauri::{AppHandle, State};

use crate::commands::{HistoryState, edit_photos_as};
use crate::state::{AppState, IpcError};

/// What was copied: the source photo's develop edits, with the crop written both as its print
/// crop and as its collage crop, so a paste lands in whichever the target is edited in.
#[derive(Debug, Clone)]
pub struct Copied {
    source: PhotoEdits,
    parts: SettingsParts,
}

fn nothing() -> IpcError {
    IpcError::new("photo.nothing_copied", "Copy settings from a photo first")
}

/// Takes `parts` of photo `id`'s develop settings for a later paste (FR-097). On a collage
/// sheet (`collage`) the crop taken is the photo's collage crop. Returns the parts taken.
#[tauri::command]
#[specta::specta]
pub fn copy_photo_settings(
    state: State<'_, AppState>,
    id: PhotoId,
    parts: SettingsParts,
    collage: bool,
) -> Result<SettingsParts, IpcError> {
    if !parts.any() {
        return Err(nothing());
    }
    let edits = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        if library.record(id).is_none() {
            return Err(IpcError::new("photo.unknown", "Unknown photo"));
        }
        library.edits(id)
    };
    let (crop, straighten) = if collage {
        (
            edits.collage_crop.or(edits.crop),
            edits.collage_straighten.unwrap_or(edits.straighten),
        )
    } else {
        (edits.crop, edits.straighten)
    };
    let source = PhotoEdits {
        crop,
        straighten,
        collage_crop: crop,
        collage_straighten: Some(straighten),
        ..edits
    };
    *state
        .copied
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))? = Some(Copied { source, parts });
    Ok(parts)
}

/// The parts last copied, if any: the UI shows *Paste* only then.
#[tauri::command]
#[specta::specta]
pub fn copied_settings(state: State<'_, AppState>) -> Result<Option<SettingsParts>, IpcError> {
    Ok(state
        .copied
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .as_ref()
        .map(|c| c.parts))
}

/// Gives the copied settings to every photo in `ids`, as one step (FR-097). On a collage
/// sheet (`collage`) the crop lands as the collage crop.
#[tauri::command]
#[specta::specta]
pub fn paste_photo_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    collage: bool,
) -> Result<HistoryState, IpcError> {
    let Copied { source, parts } = state
        .copied
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .clone()
        .ok_or_else(nothing)?;
    edit_photos_as(&app, &state, &ids, Some(StepKind::Paste), |_, e| {
        *e = e.with_parts_of(&source, parts, collage);
    })
}

/// Gives every photo in `ids` back to the shared settings, as one step (FR-097, *Reset*):
/// no look, RAW look, crop or stamp of its own; on a collage sheet its collage crop too.
#[tauri::command]
#[specta::specta]
pub fn reset_photo_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    collage: bool,
) -> Result<HistoryState, IpcError> {
    edit_photos_as(&app, &state, &ids, Some(StepKind::Reset), |_, e| {
        *e = e.reset_develop(collage);
    })
}
