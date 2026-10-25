// SPDX-License-Identifier: GPL-3.0-only
//! "Open with" (FR-054, ADR-0024). A photo handed over by the operating system (at launch,
//! by macOS while running, or by a second launch through the single-instance plugin) joins
//! the session as its folder's tab and opens in the preview; a project file keeps going to
//! the document flow. Paths come from the OS, never from the webview, and are checked like
//! imported files (SECURITY.md §2).

// Tauri commands take `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use fotollection_core::import;
use fotollection_core::library::PhotoId;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::state::AppState;

/// A photo to show: its id and the folder tab it belongs to. Kept until the UI takes it, so a
/// request that arrives before the page listens is not lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct OpenPhoto {
    pub id: PhotoId,
    pub folder: u16,
}

/// The existing files among command-line arguments, without the program itself.
pub fn from_args<I, S>(args: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    args.into_iter()
        .skip(1)
        .map(|a| PathBuf::from(a.into()))
        .filter(|p| p.is_file())
        .collect()
}

/// Paths handed over before the app state exists: on a cold launch from Finder, macOS sends
/// `Opened` before `setup` has run. They are routed once start-up is over ([`take_early`]).
static EARLY: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

/// The paths that arrived before the app state existed, once.
pub fn take_early() -> Vec<PathBuf> {
    EARLY
        .lock()
        .map(|mut early| std::mem::take(&mut *early))
        .unwrap_or_default()
}

/// Sends a path the OS handed over to the right place: a project to the document flow, any
/// other file to [`open_photo`] (which rejects what is not a photo).
pub fn route(app: &AppHandle, path: PathBuf) {
    if app.try_state::<AppState>().is_none() {
        if let Ok(mut early) = EARLY.lock() {
            early.push(path);
        }
        return;
    }
    let project = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(crate::document::EXTENSION));
    if project {
        crate::document::request_open(app, path);
    } else {
        let app = app.clone();
        std::thread::spawn(move || open_photo(&app, &path));
    }
}

/// Brings the main window forward, as a viewer opened from Finder or Explorer should be.
pub fn focus(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Adds the photo's folder as a tab (only that folder, not its subfolders) unless the photo
/// is already in the library, then asks the UI to show it.
fn open_photo(app: &AppHandle, path: &Path) {
    let Ok(file) = import::check_file(path) else {
        return;
    };
    let state = app.state::<AppState>();
    // macOS can hand a file over while the last session is still being restored: let that
    // finish first, so the two never change the library at the same time.
    while !state
        .startup_done
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let known = |state: &AppState| {
        let library = state.library.read().ok()?;
        let id = library.find_path(&file.path)?;
        Some(OpenPhoto {
            id,
            folder: library.folder_of(id)?,
        })
    };
    let Some(request) = known(&state).or_else(|| {
        let folder = file.path.parent()?;
        crate::commands::import_paths_with(app, vec![folder.to_path_buf()], false);
        known(&state)
    }) else {
        return;
    };
    if let Ok(mut slot) = state.requested_photo.lock() {
        *slot = Some(request);
    }
    focus(app);
    let _ = request.emit(app);
}

/// The photo the OS asked to show, once: the UI calls it on `OpenPhoto` and after start-up.
#[tauri::command]
#[specta::specta]
pub fn take_photo_request(state: State<'_, AppState>) -> Option<OpenPhoto> {
    state.requested_photo.lock().ok()?.take()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_054_command_line_keeps_existing_files_only() -> Result<(), std::io::Error> {
        let dir = std::env::temp_dir().join(format!("fotollection-args-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let photo = dir.join("IMG_1.jpg");
        std::fs::write(&photo, [0xFF, 0xD8, 0xFF])?;
        let args = [
            OsString::from("/Applications/Fotollection"),
            photo.clone().into(),
            dir.join("missing.jpg").into(),
            dir.clone().into(),
        ];
        assert_eq!(from_args(args), vec![photo]);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
