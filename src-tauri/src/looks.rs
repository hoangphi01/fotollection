// SPDX-License-Identifier: GPL-3.0-only
//! The library of looks the Looks card lists (FR-079): the built-in recipes and the looks
//! saved on this machine with *Save as look*. Picking one goes through `set_photo_look`.
// Tauri hands commands their state by value.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::library::PhotoId;
use fotollection_core::look_library::{self, LookEntry, SaveLookError};
use fotollection_core::looks::LookSettings;
use tauri::State;

use crate::state::{AppState, IpcError};

fn poisoned<E: std::fmt::Display>(e: E) -> IpcError {
    IpcError::internal(e.to_string())
}

/// Every look the card lists: the built-in ones, then this machine's, oldest first.
#[tauri::command]
#[specta::specta]
pub fn list_looks(state: State<'_, AppState>) -> Result<Vec<LookEntry>, IpcError> {
    let saved = state.saved_looks.lock().map_err(poisoned)?;
    Ok(look_library::entries(&saved))
}

/// Saves `look` under `name` (FR-079) and returns it as the card lists it. The name is
/// cleaned and the values clamped in Rust, whatever the webview sent.
#[tauri::command]
#[specta::specta]
pub fn save_look(
    state: State<'_, AppState>,
    name: String,
    look: LookSettings,
) -> Result<LookEntry, IpcError> {
    let entry = {
        let mut saved = state.saved_looks.lock().map_err(poisoned)?;
        let new = look_library::new_saved(&name, &look, &saved).map_err(|e| match e {
            SaveLookError::NoName => IpcError::new("look.no_name", e.to_string()),
            SaveLookError::Full => IpcError::new("look.full", e.to_string()),
        })?;
        saved.push(new);
        look_library::entries(&saved).pop()
    };
    state.save_settings();
    entry.ok_or_else(|| IpcError::internal("the saved look is missing"))
}

/// Forgets a saved look (`user:<id>`) and returns the list left. Photos that used it keep
/// their values: a look is copied into a photo, never linked.
#[tauri::command]
#[specta::specta]
pub fn delete_look(state: State<'_, AppState>, id: String) -> Result<Vec<LookEntry>, IpcError> {
    let left = {
        let mut saved = state.saved_looks.lock().map_err(poisoned)?;
        let own = id.strip_prefix("user:").unwrap_or_default();
        let before = saved.len();
        saved.retain(|s| s.id != own);
        if saved.len() == before {
            return Err(IpcError::new("look.unknown", "no saved look with this id"));
        }
        look_library::entries(&saved)
    };
    state.save_settings();
    Ok(left)
}

/// Bytes for `fotothumb://localhost/looktile/{id}?look=<library id>`: the photo through that
/// look of the library, small and square (FR-079).
pub fn look_tile_bytes(state: &AppState, id: PhotoId, look: &str) -> Result<Vec<u8>, IpcError> {
    let entry = {
        let saved = state.saved_looks.lock().map_err(poisoned)?;
        look_library::entries(&saved)
            .into_iter()
            .find(|e| e.id == look)
            .ok_or_else(|| IpcError::new("look.unknown", "no look with this id"))?
    };
    let record = state
        .library
        .read()
        .map_err(poisoned)?
        .record(id)
        .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
    let thumbnail =
        |r: &fotollection_core::library::PhotoRecord| crate::commands::thumbnail_bytes(state, r.id);
    state
        .previews
        .look_tile_jpeg(&record, &entry.look, &thumbnail, state.codec.as_ref())
        .map_err(|e| crate::commands::shown_error(&e, &record))
}

/// Imports a LUT (FR-070): a `.cube` or a `HaldCLUT` PNG chosen in a Rust-side dialog, parsed
/// and kept by its content hash, then saved as a look of the user's named by its title or file
/// name. `None` when the dialog is cancelled; a refused file says why (`look.lut_*`).
#[tauri::command]
#[specta::specta]
pub async fn import_lut(app: tauri::AppHandle) -> Result<Option<LookEntry>, IpcError> {
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let picked = app
            .dialog()
            .file()
            .set_title("Choose a LUT (.cube or HaldCLUT PNG)")
            .add_filter("LUT", &["cube", "png"])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let size = std::fs::metadata(&path)
            .map_err(|e| IpcError::new("look.lut_unreadable", e.to_string()))?
            .len();
        if size > fotollection_core::lut::MAX_BYTES as u64 {
            return Err(IpcError::new(
                "look.lut_too_large",
                "the LUT is larger than 8 MB",
            ));
        }
        let bytes = std::fs::read(&path)
            .map_err(|e| IpcError::new("look.lut_unreadable", e.to_string()))?;
        let (hash, lut) = fotollection_core::lut::import(&bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let name = lut
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "LUT".to_owned());
        let look = LookSettings {
            lut: Some(hash),
            ..LookSettings::default()
        };
        let entry = {
            let mut saved = state.saved_looks.lock().map_err(poisoned)?;
            let new = look_library::new_saved(&name, &look, &saved).map_err(|e| match e {
                SaveLookError::NoName => IpcError::new("look.no_name", e.to_string()),
                SaveLookError::Full => IpcError::new("look.full", e.to_string()),
            })?;
            saved.push(new);
            look_library::entries(&saved).pop()
        };
        state.save_settings();
        entry
            .map(Some)
            .ok_or_else(|| IpcError::internal("the imported look is missing"))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}
