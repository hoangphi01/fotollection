// SPDX-License-Identifier: GPL-3.0-only
//! Project documents (FR-023, ADR-0016): New, Open, Save, Save As and Relink.
//!
//! The `.fotollection` file is a document that the user saves explicitly, like a `.docx`.
//! The JSON session autosave keeps working underneath for crash safety and remembers which
//! document it belongs to. Paths only ever come from Rust-side dialogs or the OS, never from
//! the webview (SECURITY.md §2).

// Tauri injects `State`, `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use fotollection_core::bundle::{self, Bundle, BundleError, Opened};
use fotollection_core::fonts::{self, LoadedFont, UserFont};
use fotollection_core::frame::LogoKind;
use fotollection_core::library::Library;
use fotollection_core::logo;
use fotollection_core::project::{self, FolderRow, PathKind, ProjectFile};
use fotollection_core::settings_store::StoredSettings;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use crate::commands::{changed, import_paths};
use crate::events::{DocumentChanged, MenuAction};
use crate::i18n::strings;
use crate::menu::Action;
use crate::state::{AppState, IpcError, PendingRelink};

pub const EXTENSION: &str = "fotollection";

/// What the UI shows about the current document.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentState {
    /// The file name without its extension; `None` for an untitled session.
    pub name: Option<String>,
    pub edited: bool,
    /// Bumps when another project replaces the library (thumbnail URLs depend on it).
    pub generation: u32,
    /// Folders of the opened project that still need relinking.
    pub missing: Vec<MissingFolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MissingFolder {
    pub id: u32,
    /// Where the folder was when the project was saved.
    pub path: String,
    pub name: String,
}

fn bundle_error(e: &BundleError) -> IpcError {
    IpcError::new(e.code(), e.to_string())
}

fn poisoned(e: impl std::fmt::Display) -> IpcError {
    IpcError::internal(e.to_string())
}

fn current(state: &AppState) -> DocumentState {
    let (name, edited, generation) = state.document.lock().map_or((None, false, 0), |d| {
        let name = d
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned());
        (name, d.edited, d.generation)
    });
    let missing = state
        .pending
        .lock()
        .ok()
        .and_then(|p| {
            p.as_ref().map(|p| {
                p.missing
                    .iter()
                    .map(|m| MissingFolder {
                        id: m.id,
                        name: Path::new(&m.saved)
                            .file_name()
                            .map_or_else(|| m.saved.clone(), |n| n.to_string_lossy().into_owned()),
                        path: m.saved.clone(),
                    })
                    .collect()
            })
        })
        .unwrap_or_default();
    DocumentState {
        name,
        edited,
        generation,
        missing,
    }
}

/// Tells the UI and updates the window title: `Trip — Edited — Fotollection`.
pub fn emit(app: &AppHandle) {
    let state = app.state::<AppState>();
    let document = current(&state);
    let language = state.ui.read().map(|u| u.language).unwrap_or_default();
    let t = strings(language);
    let title = match (&document.name, document.edited) {
        (Some(name), true) => format!("{name} — {} — Fotollection", t.edited),
        (Some(name), false) => format!("{name} — Fotollection"),
        (None, _) => "Fotollection".to_owned(),
    };
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&title);
    }
    let _ = DocumentChanged { document }.emit(app);
}

#[tauri::command]
#[specta::specta]
pub fn get_document(state: State<'_, AppState>) -> DocumentState {
    current(&state)
}

// ---------------------------------------------------------------------------------------
// Save

/// The bundle for the current session, with folders relative to `base` where possible.
/// Rows of folders that are still missing are kept, so saving never drops edits.
fn build_bundle(state: &AppState, base: &Path) -> Result<Bundle, IpcError> {
    let mut file = project::snapshot(&*state.library.read().map_err(poisoned)?);
    if let Some(pending) = state.pending.lock().map_err(poisoned)?.as_ref() {
        keep_missing_rows(&mut file, pending);
    }
    project::relocate(&mut file, base);

    let render = state.settings.read().map_err(poisoned)?.0.clone();
    let frame = state.frame.read().map_err(poisoned)?;
    let mut settings = StoredSettings {
        render,
        frame: frame.settings.clone(),
        cameras: frame.cameras.clone(),
        fonts: frame.fonts.clone(),
        ui: fotollection_core::settings::UiSettings::default(),
    };
    settings.fonts = bundle::used_fonts(&settings);
    let mut font_files = std::collections::BTreeMap::new();
    for font in &settings.fonts {
        let path = state
            .font_dir
            .join(format!("{}.{}", font.id, font.format.extension()));
        let bytes = std::fs::read(&path)
            .map_err(|e| IpcError::new("project.write_failed", e.to_string()))?;
        font_files.insert(font.id, bytes);
    }
    let mut logos = std::collections::BTreeMap::new();
    for camera in &mut settings.cameras {
        if camera.logo == LogoKind::Image {
            match std::fs::read(state.logo_dir.join(format!("{}.png", camera.id))) {
                Ok(bytes) => {
                    logos.insert(camera.id, bytes);
                }
                // The file is gone: save the camera with its text logo instead.
                Err(_) => camera.logo = LogoKind::Text,
            }
        }
    }
    Ok(Bundle {
        project: file,
        settings,
        fonts: font_files,
        logos,
    })
}

/// Appends the folders still waiting for a relink, and their photo rows, to `file`.
fn keep_missing_rows(file: &mut ProjectFile, pending: &PendingRelink) {
    let first_folder = file.folders.iter().map(|f| f.id).max().unwrap_or(0) + 1;
    let mut next_photo = file.photos.iter().map(|p| p.id).max().unwrap_or(0);
    for (next_folder, missing) in (first_folder..).zip(&pending.missing) {
        file.folders.push(FolderRow {
            id: next_folder,
            path: missing.saved.clone(),
            path_kind: PathKind::Absolute,
            absolute: None,
        });
        for row in pending
            .file
            .photos
            .iter()
            .filter(|r| r.folder == missing.id)
        {
            next_photo += 1;
            file.photos.push(project::PhotoRow {
                id: next_photo,
                folder: next_folder,
                ..row.clone()
            });
        }
    }
}

fn save_to(app: &AppHandle, path: &Path) -> Result<DocumentState, IpcError> {
    let state = app.state::<AppState>();
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let bundle = build_bundle(&state, base)?;
    bundle::save(path, &bundle).map_err(|e| bundle_error(&e))?;
    {
        let mut doc = state.document.lock().map_err(poisoned)?;
        doc.path = Some(path.to_path_buf());
        doc.edited = false;
    }
    // The session now belongs to this document.
    state.project_dirty.store(true, Ordering::Relaxed);
    emit(app);
    Ok(current(&state))
}

fn pick_save_path(app: &AppHandle) -> Option<PathBuf> {
    let state = app.state::<AppState>();
    let language = state.ui.read().map(|u| u.language).unwrap_or_default();
    let t = strings(language);
    let (name, dir) = {
        let doc_path = state.document.lock().ok().and_then(|d| d.path.clone());
        let first_folder = state
            .library
            .read()
            .ok()
            .and_then(|l| l.folders().first().cloned());
        match (doc_path, first_folder) {
            (Some(p), _) => (
                p.file_name().map(|n| n.to_string_lossy().into_owned()),
                p.parent().map(Path::to_path_buf),
            ),
            // Next to the photos by default, so the folder can be stored relative to it.
            (None, Some(folder)) => (
                folder
                    .file_name()
                    .map(|n| format!("{}.{EXTENSION}", n.to_string_lossy())),
                folder.parent().map(Path::to_path_buf),
            ),
            (None, None) => (None, None),
        }
    };
    let mut dialog = app
        .dialog()
        .file()
        .set_title(t.save_title)
        .add_filter(t.project_filter, &[EXTENSION])
        .set_file_name(name.unwrap_or_else(|| format!("{}.{EXTENSION}", t.untitled)));
    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }
    let path = dialog.blocking_save_file()?.into_path().ok()?;
    Some(if path.extension().is_some_and(|e| e == EXTENSION) {
        path
    } else {
        path.with_extension(EXTENSION)
    })
}

/// Save: to the current document, or Save As when there is none yet.
#[tauri::command]
#[specta::specta]
pub async fn save_document(app: AppHandle) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .state::<AppState>()
            .document
            .lock()
            .map_err(poisoned)?
            .path
            .clone();
        match path.or_else(|| pick_save_path(&app)) {
            Some(path) => save_to(&app, &path).map(Some),
            None => Ok(None),
        }
    })
    .await
    .map_err(poisoned)?
}

#[tauri::command]
#[specta::specta]
pub async fn save_document_as(app: AppHandle) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || match pick_save_path(&app) {
        Some(path) => save_to(&app, &path).map(Some),
        None => Ok(None),
    })
    .await
    .map_err(poisoned)?
}

// ---------------------------------------------------------------------------------------
// Open / New

/// Empties the library, the undo history and the per-id caches.
fn reset_library(state: &AppState) -> Result<(), IpcError> {
    *state.library.write().map_err(poisoned)? = Library::new();
    state.history.lock().map_err(poisoned)?.clear();
    state.previews.clear();
    *state.pending.lock().map_err(poisoned)? = None;
    Ok(())
}

/// Installs the bundle's fonts, camera profiles, logos and print settings
/// ([`bundle::merge_settings`]): fonts already here with the same content are reused and
/// ids are remapped. The interface language stays this machine's.
fn install_settings(state: &AppState, bundle: &Bundle) -> Result<(), IpcError> {
    let mut frame = state.frame.write().map_err(poisoned)?;
    let mut local = StoredSettings {
        render: state.settings.read().map_err(poisoned)?.0.clone(),
        frame: frame.settings.clone(),
        cameras: frame.cameras.clone(),
        fonts: frame.fonts.clone(),
        ui: state.ui.read().map(|u| *u).unwrap_or_default(),
    };
    let font_path = |f: &UserFont| {
        state
            .font_dir
            .join(format!("{}.{}", f.id, f.format.extension()))
    };
    let installed =
        bundle::merge_settings(&mut local, bundle, &|f| std::fs::read(font_path(f)).ok())
            .map_err(|e| bundle_error(&e))?;
    local
        .render
        .validate()
        .map_err(|e| IpcError::new("settings.invalid", e.to_string()))?;

    for (font, bytes) in installed.new_fonts {
        let (_, parsed) =
            fonts::load(bytes.clone()).map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        std::fs::create_dir_all(&state.font_dir)
            .map_err(|e| IpcError::new("font.write_failed", e.to_string()))?;
        fotollection_core::thumbs::write_atomic(&font_path(&font), &bytes)
            .map_err(|e| IpcError::new("font.write_failed", e.to_string()))?;
        frame.loaded_fonts.insert(
            font.id,
            LoadedFont {
                id: font.id,
                font: parsed,
            },
        );
    }
    for (camera, bytes) in installed.logos {
        let image = logo::decode_png(&bytes).map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        std::fs::create_dir_all(&state.logo_dir)
            .map_err(|e| IpcError::new("logo.write_failed", e.to_string()))?;
        fotollection_core::thumbs::write_atomic(
            &state.logo_dir.join(format!("{camera}.png")),
            &bytes,
        )
        .map_err(|e| IpcError::new("logo.write_failed", e.to_string()))?;
        frame.logos.insert(camera, Arc::new(image));
    }
    frame.settings = local.frame;
    frame.cameras = local.cameras;
    frame.fonts = local.fonts;
    drop(frame);
    state.settings.write().map_err(poisoned)?.0 = local.render;
    state.ensure_camera_profiles();
    // The app's settings follow the opened project, but opening is not an edit.
    state.save_settings();
    Ok(())
}

/// Opens a `.fotollection` file from a trusted path (a Rust dialog or the OS).
pub fn open_path(app: &AppHandle, path: &Path) -> Result<DocumentState, IpcError> {
    let state = app.state::<AppState>();
    let _guard = state.document_lock.lock().map_err(poisoned)?;
    // Everything is validated before anything changes.
    let opened = bundle::open(path).map_err(|e| bundle_error(&e))?;
    let file = match opened {
        Opened::Bundle(bundle) => {
            install_settings(&state, &bundle)?;
            bundle.project
        }
        // A JSON project from before bundles: tables only, settings stay as they are.
        Opened::Legacy(file) => file,
    };
    reset_library(&state)?;

    let resolved = project::resolve_folders(&file, path.parent());
    let found: Vec<(u32, PathBuf)> = resolved
        .iter()
        .filter_map(|r| r.path.clone().map(|p| (r.id, p)))
        .collect();
    let missing: Vec<_> = resolved.into_iter().filter(|r| r.path.is_none()).collect();
    import_paths(app, found.iter().map(|(_, p)| p.clone()).collect());
    if let Ok(mut library) = state.library.write() {
        project::restore_into(&mut library, &file, &found);
    }
    if !missing.is_empty() {
        *state.pending.lock().map_err(poisoned)? = Some(PendingRelink { file, missing });
    }
    {
        let mut doc = state.document.lock().map_err(poisoned)?;
        doc.path = Some(path.to_path_buf());
        doc.generation = doc.generation.wrapping_add(1);
    }
    finish_without_edit(app, &state);
    Ok(current(&state))
}

/// Refreshes the UI after a library swap, which itself is not an unsaved change.
fn finish_without_edit(app: &AppHandle, state: &AppState) {
    changed(app, state);
    if let Ok(mut doc) = state.document.lock() {
        doc.edited = false;
    }
    emit(app);
}

#[tauri::command]
#[specta::specta]
pub async fn open_document(app: AppHandle) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let language = app
            .state::<AppState>()
            .ui
            .read()
            .map(|u| u.language)
            .unwrap_or_default();
        let t = strings(language);
        let picked = app
            .dialog()
            .file()
            .set_title(t.open_title)
            .add_filter(t.project_filter, &[EXTENSION])
            .blocking_pick_file();
        match picked.and_then(|p| p.into_path().ok()) {
            Some(path) => open_path(&app, &path).map(Some),
            None => Ok(None),
        }
    })
    .await
    .map_err(poisoned)?
}

/// Opens the file the OS asked us to open (double-click, `open -a`), after the UI has
/// dealt with unsaved changes. The path never comes from the webview.
#[tauri::command]
#[specta::specta]
pub async fn open_requested_document(app: AppHandle) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .state::<AppState>()
            .requested_open
            .lock()
            .map_err(poisoned)?
            .take();
        match path {
            Some(path) => open_path(&app, &path).map(Some),
            None => Ok(None),
        }
    })
    .await
    .map_err(poisoned)?
}

/// Called for files handed over by the OS: remember the path and let the UI run the
/// same flow as Open (unsaved-changes question first).
pub fn request_open(app: &AppHandle, path: PathBuf) {
    if let Ok(mut slot) = app.state::<AppState>().requested_open.lock() {
        *slot = Some(path);
    }
    let _ = MenuAction {
        action: Action::OpenRequested,
    }
    .emit(app);
}

#[tauri::command]
#[specta::specta]
pub fn new_document(app: AppHandle, state: State<'_, AppState>) -> Result<DocumentState, IpcError> {
    let _guard = state.document_lock.lock().map_err(poisoned)?;
    reset_library(&state)?;
    {
        let mut doc = state.document.lock().map_err(poisoned)?;
        doc.path = None;
        doc.generation = doc.generation.wrapping_add(1);
    }
    finish_without_edit(&app, &state);
    Ok(current(&state))
}

/// Points a missing folder of the opened project at its new location and restores its
/// edits (FR-023).
#[tauri::command]
#[specta::specta]
pub async fn relink_folder(
    app: AppHandle,
    folder_id: u32,
) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let saved = state
            .pending
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .and_then(|p| {
                p.missing
                    .iter()
                    .find(|m| m.id == folder_id)
                    .map(|m| m.saved.clone())
            })
            .ok_or_else(|| IpcError::new("project.unknown_folder", "This folder is not missing"))?;
        let language = state.ui.read().map(|u| u.language).unwrap_or_default();
        let mut dialog = app
            .dialog()
            .file()
            .set_title(strings(language).relink_title);
        if let Some(parent) = Path::new(&saved).parent().filter(|p| p.is_dir()) {
            dialog = dialog.set_directory(parent);
        }
        let Some(folder) = dialog
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok())
        else {
            return Ok(None);
        };
        let _guard = state.document_lock.lock().map_err(poisoned)?;
        import_paths(&app, vec![folder.clone()]);
        {
            let mut pending = state.pending.lock().map_err(poisoned)?;
            if let Some(p) = pending.as_mut() {
                if let Ok(mut library) = state.library.write() {
                    project::restore_into(&mut library, &p.file, &[(folder_id, folder)]);
                }
                p.missing.retain(|m| m.id != folder_id);
                if p.missing.is_empty() {
                    *pending = None;
                }
            }
        }
        // The new location is a change the document should remember.
        changed(&app, &state);
        Ok(Some(current(&state)))
    })
    .await
    .map_err(poisoned)?
}

/// Restores the document of the last session (path and unsaved flag) from its autosave.
pub fn restore_reference(state: &AppState, file: &ProjectFile) {
    if let (Some(doc), Ok(mut current)) = (&file.document, state.document.lock()) {
        current.path = Some(PathBuf::from(&doc.path));
        current.edited = doc.edited;
    }
}
