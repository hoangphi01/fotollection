// SPDX-License-Identifier: GPL-3.0-only
//! Settings → *Data* (FR-116, FR-082): what Fotollection keeps on this computer, moving it to
//! another one in a `.fotosettings`, and starting afresh. Every path comes from Rust (a dialog
//! or the app's own folders), never from the webview (SECURITY §2); each store is asked of
//! its owner (ADR-0048): this module owns no data.

// Tauri commands take `AppHandle` and `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fotollection_core::bundle::BundleError;
use fotollection_core::fonts::{self, LoadedFont};
use fotollection_core::frame::{CameraProfile, FrameSettings, LogoKind};
use fotollection_core::settings::{RenderSettings, UiSettings};
use fotollection_core::settings_bundle::{self, ImportMode, SettingsBundle, SettingsImportSummary};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::state::{AppState, IpcError};

/// One store Fotollection keeps on this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DataKind {
    Settings,
    Looks,
    Luts,
    Fonts,
    Logos,
    Addresses,
    CameraSessions,
    Cache,
    Logs,
}

/// A store's size: bytes on disk (f64: TypeScript has no u64), how many items, and for the
/// LUTs how many no look uses.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StoreUsage {
    pub kind: DataKind,
    pub bytes: f64,
    pub count: u32,
    pub unused: u32,
}

fn settings_error(e: &BundleError) -> IpcError {
    let code = match e {
        BundleError::NotAProject => "settings_bundle.not_settings",
        BundleError::NewerVersion(_) => "settings_bundle.newer",
        BundleError::Corrupt(_) => "settings_bundle.corrupt",
        BundleError::TooLarge => "settings_bundle.too_large",
        BundleError::Write(_) => "settings_bundle.write_failed",
        BundleError::TooManyFonts => "font.too_many",
    };
    IpcError::new(code, e.to_string())
}

fn config_dir(state: &AppState) -> PathBuf {
    state
        .store_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn luts_dir(state: &AppState) -> PathBuf {
    config_dir(state).join("looks")
}

/// Bytes and files under `dir`, recursively; a folder not there is empty.
fn folder_usage(dir: &Path) -> (u64, u32) {
    let mut total = (0u64, 0u32);
    let mut todo = vec![dir.to_path_buf()];
    while let Some(dir) = todo.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            match entry.file_type() {
                Ok(t) if t.is_dir() => todo.push(entry.path()),
                Ok(t) if t.is_file() => {
                    total.0 += entry.metadata().map_or(0, |m| m.len());
                    total.1 = total.1.saturating_add(1);
                }
                _ => {}
            }
        }
    }
    total
}

fn lut_files(state: &AppState) -> Vec<(String, PathBuf)> {
    std::fs::read_dir(luts_dir(state))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let stem = path.file_stem()?.to_str()?.to_owned();
            let ext = path.extension()?.to_str()?;
            (stem.len() == 64 && matches!(ext, "cube" | "png")).then_some((stem, path))
        })
        .collect()
}

/// Every LUT something refers to: the saved looks, the defaults, the open document, its
/// photos and the folders waiting for a relink.
fn luts_in_use(state: &AppState) -> BTreeSet<String> {
    let mut used = state
        .stored_settings()
        .map(|s| settings_bundle::used_luts(&s))
        .unwrap_or_default();
    if let Ok(render) = state.settings.read() {
        used.extend(render.0.look.lut.clone());
    }
    if let Ok(library) = state.library.read() {
        let file = fotollection_core::project::snapshot(&library);
        used.extend(
            file.photos
                .iter()
                .filter_map(|p| p.look.as_ref()?.lut.clone()),
        );
    }
    if let Ok(pending) = state.pending.lock()
        && let Some(pending) = pending.as_ref()
    {
        used.extend(
            pending
                .file
                .photos
                .iter()
                .filter_map(|p| p.look.as_ref()?.lut.clone()),
        );
    }
    used
}

/// What each store holds now (FR-116), asked of its owner.
// Byte counts as f64 for TypeScript: exact far beyond any disk.
#[allow(clippy::cast_precision_loss)]
#[tauri::command]
#[specta::specta]
pub async fn data_usage(app: AppHandle) -> Result<Vec<StoreUsage>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let row = |kind, (bytes, count): (u64, u32), unused| StoreUsage {
            kind,
            bytes: bytes as f64,
            count,
            unused,
        };
        let stored = state.stored_settings().unwrap_or_default();
        let settings_bytes = std::fs::metadata(&state.store_path).map_or(0, |m| m.len());
        let used = luts_in_use(&state);
        let luts = lut_files(&state);
        let lut_bytes: u64 = luts
            .iter()
            .map(|(_, p)| std::fs::metadata(p).map_or(0, |m| m.len()))
            .sum();
        let unused = luts.iter().filter(|(h, _)| !used.contains(h)).count();
        let n = |v: usize| u32::try_from(v).unwrap_or(u32::MAX);
        let cache = fotollection_core::cache_limit::usage_by_kind(&crate::cache::dirs(&state));
        let camera = app.state::<crate::camera::CameraState>();
        let sessions = std::fs::read_dir(camera.root())
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .count();
        Ok(vec![
            row(DataKind::Settings, (settings_bytes, 1), 0),
            row(DataKind::Looks, (0, n(stored.looks.len())), 0),
            row(DataKind::Luts, (lut_bytes, n(luts.len())), n(unused)),
            row(DataKind::Fonts, folder_usage(&state.font_dir), 0),
            row(DataKind::Logos, folder_usage(&state.logo_dir), 0),
            row(
                DataKind::Addresses,
                (0, n(stored.recent_addresses.len())),
                0,
            ),
            row(
                DataKind::CameraSessions,
                (folder_usage(camera.root()).0, n(sessions)),
                0,
            ),
            row(
                DataKind::Cache,
                (
                    cache.thumbnails.bytes + cache.previews.bytes + cache.analyses.bytes,
                    u32::try_from(
                        cache.thumbnails.files + cache.previews.files + cache.analyses.files,
                    )
                    .unwrap_or(u32::MAX),
                ),
                0,
            ),
            row(
                DataKind::Logs,
                fotollection_core::error_log::dir().map_or((0, 0), |d| folder_usage(&d)),
                0,
            ),
        ])
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Shows a store's folder in Finder or Explorer; the folder is Rust's own, never a path the
/// webview sent.
#[tauri::command]
#[specta::specta]
pub fn open_data_folder(app: AppHandle, kind: DataKind) -> Result<(), IpcError> {
    let state = app.state::<AppState>();
    let folder = match kind {
        DataKind::Settings | DataKind::Looks | DataKind::Addresses => config_dir(&state),
        DataKind::Luts => luts_dir(&state),
        DataKind::Fonts => state.font_dir.clone(),
        DataKind::Logos => state.logo_dir.clone(),
        DataKind::CameraSessions => app
            .state::<crate::camera::CameraState>()
            .root()
            .to_path_buf(),
        DataKind::Cache => state
            .thumbs
            .dir()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default(),
        DataKind::Logs => fotollection_core::error_log::dir().unwrap_or_default(),
    };
    std::fs::create_dir_all(&folder)
        .map_err(|e| IpcError::new("data.folder_missing", e.to_string()))?;
    app.opener()
        .open_path(folder.to_string_lossy(), None::<&str>)
        .map_err(|e| IpcError::new("data.folder_missing", e.to_string()))
}

/// The fonts, logos and LUTs this machine's settings refer to, with the settings: what a
/// `.fotosettings` carries.
fn this_machine(state: &AppState) -> Result<SettingsBundle, IpcError> {
    let settings = state
        .stored_settings()
        .ok_or_else(|| IpcError::internal("settings unavailable"))?;
    let mut fonts = std::collections::BTreeMap::new();
    for font in &settings.fonts {
        let path = state
            .font_dir
            .join(format!("{}.{}", font.id, font.format.extension()));
        if let Ok(bytes) = std::fs::read(path) {
            fonts.insert(font.id, bytes);
        }
    }
    let mut settings = settings;
    // A font whose file is gone leaves the table, as a project does with it.
    settings.fonts.retain(|f| fonts.contains_key(&f.id));
    let mut logos = std::collections::BTreeMap::new();
    for camera in &mut settings.cameras {
        if camera.logo == LogoKind::Image {
            match std::fs::read(state.logo_dir.join(format!("{}.png", camera.id))) {
                Ok(bytes) => {
                    logos.insert(camera.id, bytes);
                }
                Err(_) => camera.logo = LogoKind::Text,
            }
        }
    }
    let luts = settings_bundle::used_luts(&settings)
        .into_iter()
        .filter_map(|hash| fotollection_core::lut::file_of(&hash).map(|bytes| (hash, bytes)))
        .collect();
    Ok(SettingsBundle {
        settings,
        fonts,
        logos,
        luts,
    })
}

/// *Export settings…* (FR-082): this machine's settings, fonts, logos and LUTs in a
/// `.fotosettings` where the user chooses; the addresses only when `include_addresses`.
#[tauri::command]
#[specta::specta]
pub async fn export_settings(
    app: AppHandle,
    include_addresses: bool,
) -> Result<Option<String>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let bundle = this_machine(&state)?;
        let bytes = settings_bundle::to_bytes(&bundle, include_addresses)
            .map_err(|e| settings_error(&e))?;
        let picked = app
            .dialog()
            .file()
            .set_title("Export settings")
            .add_filter("Fotollection settings", &[settings_bundle::EXTENSION])
            .set_file_name(format!(
                "Fotollection settings.{}",
                settings_bundle::EXTENSION
            ))
            .blocking_save_file();
        let Some(mut path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        if path
            .extension()
            .is_none_or(|e| !e.eq_ignore_ascii_case(settings_bundle::EXTENSION))
        {
            path.set_extension(settings_bundle::EXTENSION);
        }
        fotollection_core::atomic::write_with_backup(&path, &bytes)
            .map_err(|e| IpcError::new("settings_bundle.write_failed", e.to_string()))?;
        Ok(Some(path.display().to_string()))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// *Import settings…*, first step (FR-082): a `.fotosettings` the user picks, checked whole,
/// kept until they choose how to import it; what it holds, or `None` if cancelled.
#[tauri::command]
#[specta::specta]
pub async fn inspect_settings_import(
    app: AppHandle,
) -> Result<Option<SettingsImportSummary>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_title("Import settings")
            .add_filter("Fotollection settings", &[settings_bundle::EXTENSION])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let size = std::fs::metadata(&path).map_or(u64::MAX, |m| m.len());
        if size > fotollection_core::bundle::MAX_TOTAL_BYTES {
            return Err(settings_error(&BundleError::TooLarge));
        }
        let bytes = std::fs::read(&path)
            .map_err(|e| IpcError::new("settings_bundle.corrupt", e.to_string()))?;
        let bundle = settings_bundle::from_bytes(&bytes).map_err(|e| settings_error(&e))?;
        let summary = bundle.summary();
        if let Ok(mut pending) = app.state::<AppState>().pending_import.lock() {
            *pending = Some(bundle);
        }
        Ok(Some(summary))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// *Import settings…*, second step: the file checked by [`inspect_settings_import`], merged
/// or replacing (owner, 2026-11-09). The old `settings.json` stays as `.bak`; the interface
/// reloads afterwards.
#[tauri::command]
#[specta::specta]
pub fn apply_settings_import(state: State<'_, AppState>, mode: ImportMode) -> Result<(), IpcError> {
    let bundle = state
        .pending_import
        .lock()
        .ok()
        .and_then(|mut p| p.take())
        .ok_or_else(|| IpcError::new("settings_bundle.corrupt", "no settings file was checked"))?;
    let mut local = state
        .stored_settings()
        .ok_or_else(|| IpcError::internal("settings unavailable"))?;
    let font_path = |f: &fonts::UserFont| {
        state
            .font_dir
            .join(format!("{}.{}", f.id, f.format.extension()))
    };
    let installed = settings_bundle::apply(&mut local, &bundle, mode, &|f| {
        std::fs::read(font_path(f)).ok()
    })
    .map_err(|e| settings_error(&e))?;
    for bytes in bundle.luts.values() {
        if let Err(e) = fotollection_core::lut::import(bytes) {
            fotollection_core::error_log::record("look.lut_invalid", &e.to_string());
        }
    }
    let mut new_fonts = Vec::new();
    for (font, bytes) in installed.new_fonts {
        fotollection_core::atomic::write_durable(&font_path(&font), &bytes)
            .map_err(|e| IpcError::new("font.write_failed", e.to_string()))?;
        if let Ok((_, parsed)) = fonts::load(bytes) {
            new_fonts.push(LoadedFont {
                id: font.id,
                font: parsed,
            });
        }
    }
    let mut new_logos = Vec::new();
    for (camera, bytes) in installed.logos {
        let image = fotollection_core::logo::decode_png(&bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        fotollection_core::atomic::write_durable(
            &state.logo_dir.join(format!("{camera}.png")),
            &bytes,
        )
        .map_err(|e| IpcError::new("logo.write_failed", e.to_string()))?;
        new_logos.push((camera, Arc::new(image)));
    }
    install(&state, local, new_fonts, new_logos);
    Ok(())
}

/// `local` becomes this machine's settings in memory and on disk, the lock order kept
/// (settings → frame → ui, one at a time).
fn install(
    state: &AppState,
    local: fotollection_core::settings_store::StoredSettings,
    fonts: Vec<LoadedFont>,
    logos: Vec<(u32, Arc<fotollection_core::logo::LogoImage>)>,
) {
    if let Ok(mut defaults) = state.defaults.write() {
        *defaults = (local.render.clone(), local.frame.clone());
    }
    if let Ok(mut frame) = state.frame.write() {
        frame.cameras = local.cameras;
        frame.fonts = local.fonts;
        for font in fonts {
            frame.loaded_fonts.insert(font.id, font);
        }
        for (id, logo) in logos {
            frame.logos.insert(id, logo);
        }
    }
    if let Ok(mut ui) = state.ui.write() {
        *ui = local.ui;
    }
    if let Ok(mut looks) = state.saved_looks.lock() {
        *looks = local.looks;
    }
    if let Ok(mut addresses) = state.recent_addresses.lock() {
        *addresses = local.recent_addresses;
    }
    state.save_settings();
}

/// Removes the LUT files nothing refers to (FR-116), asked first in the interface: a project
/// saved before v0.14.0 carries none and may need them. Returns how many went.
#[tauri::command]
#[specta::specta]
pub fn remove_unused_luts(state: State<'_, AppState>) -> u32 {
    let used = luts_in_use(&state);
    let gone = lut_files(&state)
        .into_iter()
        .filter(|(hash, path)| !used.contains(hash) && std::fs::remove_file(path).is_ok())
        .count();
    u32::try_from(gone).unwrap_or(u32::MAX)
}

/// Forgets every postcard address this machine remembers (FR-077, FR-116).
#[tauri::command]
#[specta::specta]
pub fn forget_all_addresses(state: State<'_, AppState>) {
    if let Ok(mut addresses) = state.recent_addresses.lock() {
        addresses.clear();
    }
    state.save_settings();
}

/// *Reset settings* (owner, 2026-11-09): the interface, the export choices, the defaults for
/// new projects and the cameras' typed names back to their defaults; the user's data (saved
/// looks, LUTs, fonts, logos, addresses, custom sizes) stays. The old file stays as `.bak`.
#[tauri::command]
#[specta::specta]
pub fn reset_settings(state: State<'_, AppState>) -> Result<(), IpcError> {
    let mut local = state
        .stored_settings()
        .ok_or_else(|| IpcError::internal("settings unavailable"))?;
    let sizes = std::mem::take(&mut local.render.custom_sizes);
    local.render = RenderSettings {
        custom_sizes: sizes,
        ..RenderSettings::default()
    };
    local.frame = FrameSettings::default();
    local.ui = UiSettings::default();
    for camera in &mut local.cameras {
        let logo = camera.logo;
        *camera = CameraProfile::new(camera.id, camera.make.clone(), camera.model.clone());
        if logo == LogoKind::Image {
            camera.logo = LogoKind::Image;
        }
    }
    install(&state, local, Vec::new(), Vec::new());
    Ok(())
}

/// *Erase data* (owner, 2026-11-09): the saved looks, LUTs, fonts, logos and addresses go, as
/// on a new install; projects and photos on disk are never touched. The old file stays as
/// `.bak`.
#[tauri::command]
#[specta::specta]
pub fn erase_data(state: State<'_, AppState>) -> Result<(), IpcError> {
    let mut local = state
        .stored_settings()
        .ok_or_else(|| IpcError::internal("settings unavailable"))?;
    for (_, path) in lut_files(&state) {
        let _ = std::fs::remove_file(path);
    }
    for font in &local.fonts {
        let _ = std::fs::remove_file(state.font_dir.join(format!(
            "{}.{}",
            font.id,
            font.format.extension()
        )));
    }
    for camera in &mut local.cameras {
        if camera.logo == LogoKind::Image {
            let _ = std::fs::remove_file(state.logo_dir.join(format!("{}.png", camera.id)));
            camera.logo = LogoKind::Text;
        }
    }
    local.looks.clear();
    local.fonts.clear();
    local.recent_addresses.clear();
    local.frame.font = None;
    local.render.collage.postcard.font = None;
    if let Ok(mut frame) = state.frame.write() {
        frame.loaded_fonts.clear();
        frame.logos.clear();
        frame.settings.font = None;
    }
    if let Ok(mut render) = state.settings.write() {
        render.0.collage.postcard.font = None;
    }
    install(&state, local, Vec::new(), Vec::new());
    Ok(())
}
