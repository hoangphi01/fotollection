// SPDX-License-Identifier: GPL-3.0-only
//! IPC commands. Contract documented in docs/ARCHITECTURE.md §6.
//!
//! File system paths never come from the webview: folders are chosen through dialogs
//! opened here in Rust, or dropped onto the window (SECURITY.md §2).

// Tauri injects `State`, `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use fotollection_core::PrintSize;
use fotollection_core::cancel::CancellationToken;
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, default_inflight, run_export};
use fotollection_core::import::{self, ImportedFile};
use fotollection_core::library::{PhotoId, PhotoSummary, flags};
use fotollection_core::metadata;
use fotollection_core::plan::{DpiBand, effective_dpi};
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs;
use rayon::prelude::*;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use crate::events::{
    ExportFinished, ExportProgress, LibraryChanged, TaskKind, TaskProgress, ThumbReady,
};
use crate::state::{AppState, IpcError};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub app_version: String,
    pub core_version: String,
}

/// Versions shown in the About section.
#[tauri::command]
#[specta::specta]
pub fn app_info() -> AppInfo {
    AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        core_version: fotollection_core::VERSION.to_owned(),
    }
}

/// Supported print sizes for the settings panel (FR-006).
#[tauri::command]
#[specta::specta]
pub fn list_print_sizes() -> Vec<PrintSize> {
    PrintSize::ALL.to_vec()
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RejectedSummary {
    pub file_name: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub added: u32,
    pub duplicates: u32,
    /// Up to 50 rejected files, with their reason codes.
    pub rejected: Vec<RejectedSummary>,
    pub rejected_total: u32,
}

const MAX_REPORTED_REJECTIONS: usize = 50;

/// Lets progress events through at most ~30 times per second, plus the final one
/// (ARCHITECTURE.md §5).
#[derive(Default)]
struct Throttle(Mutex<Option<Instant>>);

impl Throttle {
    fn due(&self, done: u32, total: u32) -> bool {
        self.0.lock().is_ok_and(|mut last| {
            let due =
                done >= total || last.is_none_or(|t| t.elapsed() >= Duration::from_millis(33));
            if due {
                *last = Some(Instant::now());
            }
            due
        })
    }
}

/// Counts finished items and emits throttled `TaskProgress` events.
fn progress_reporter(app: &AppHandle, task: TaskKind, total: u32) -> impl Fn() + Sync + '_ {
    let done = AtomicU32::new(0);
    let throttle = Throttle::default();
    let _ = TaskProgress {
        task,
        done: 0,
        total,
    }
    .emit(app);
    move || {
        let n = done.fetch_add(1, Ordering::Relaxed) + 1;
        if throttle.due(n, total) {
            let _ = TaskProgress {
                task,
                done: n,
                total,
            }
            .emit(app);
        }
    }
}

/// Imports folders and/or files (FR-001, FR-002). Used by the dialog and by drag-and-drop.
pub fn import_paths(app: &AppHandle, paths: Vec<PathBuf>) -> ImportSummary {
    let state = app.state::<AppState>();
    let mut summary = ImportSummary::default();
    let mut new_ids = Vec::new();

    for root in paths {
        let (folder_root, scan) = if root.is_dir() {
            (root.clone(), import::scan(&root))
        } else {
            let parent = root.parent().map(PathBuf::from).unwrap_or_default();
            let mut scan = import::ScanResult::default();
            match import::check_file(&root) {
                Ok(file) => scan.accepted.push(file),
                Err(reason) => scan.rejected.push(import::RejectedFile {
                    path: root.clone(),
                    reason,
                }),
            }
            (parent, scan)
        };

        // EXIF reads are independent and I/O-bound: do them in parallel.
        let total = u32::try_from(scan.accepted.len()).unwrap_or(u32::MAX);
        let tick = progress_reporter(app, TaskKind::Import, total);
        let with_meta: Vec<(ImportedFile, metadata::PhotoMeta)> =
            state.background_pool.install(|| {
                scan.accepted
                    .into_par_iter()
                    .map(|file| {
                        let meta = metadata::read_path(&file.path);
                        tick();
                        (file, meta)
                    })
                    .collect()
            });

        if let Ok(mut library) = state.library.write() {
            let folder = library.add_folder(&folder_root);
            for (file, meta) in &with_meta {
                match library.push(folder, file, meta) {
                    Some(id) => {
                        summary.added += 1;
                        new_ids.push(id);
                    }
                    None => summary.duplicates += 1,
                }
            }
        }
        summary.rejected_total += u32::try_from(scan.rejected.len()).unwrap_or(u32::MAX);
        for rejected in scan.rejected {
            if summary.rejected.len() >= MAX_REPORTED_REJECTIONS {
                break;
            }
            summary.rejected.push(RejectedSummary {
                file_name: file_name(&rejected.path),
                code: rejected.reason.code().to_owned(),
            });
        }
    }

    let count = state
        .library
        .read()
        .map_or(0, |l| u32::try_from(l.len()).unwrap_or(u32::MAX));
    let _ = LibraryChanged { count }.emit(app);
    warm_thumbnails(app, new_ids);
    summary
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Generates T1 thumbnails in the background, low priority (OPTIMIZATION.md §4.1).
fn warm_thumbnails(app: &AppHandle, ids: Vec<PhotoId>) {
    if ids.is_empty() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let total = u32::try_from(ids.len()).unwrap_or(u32::MAX);
        let tick = progress_reporter(&app, TaskKind::Thumbnails, total);
        state.background_pool.install(|| {
            ids.par_iter().for_each(|&id| {
                let record = state.library.read().ok().and_then(|l| l.record(id));
                if let Some(record) = record
                    && state.thumbs.cached_t1(&record).is_none()
                    && state.thumbs.make_t1(&record, state.codec.as_ref()).is_ok()
                {
                    let _ = ThumbReady { id }.emit(&app);
                }
                tick();
            });
        });
    });
}

/// Opens a folder dialog and imports the chosen folder. `None` if the user cancelled.
#[tauri::command]
#[specta::specta]
pub async fn pick_and_import_folder(app: AppHandle) -> Result<Option<ImportSummary>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_title("Open a folder of photos")
            .blocking_pick_folder();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        Ok(Some(import_paths(&app, vec![path])))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// All photos, ordered by capture time (FR-003).
#[tauri::command]
#[specta::specta]
pub fn list_photos(state: State<'_, AppState>) -> Result<Vec<PhotoSummary>, IpcError> {
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    Ok(library.summaries())
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsState {
    pub settings: RenderSettings,
    /// Bumps on every change; previews use it to refresh.
    pub revision: u32,
}

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsState, IpcError> {
    let guard = state
        .settings
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    Ok(SettingsState {
        settings: guard.0.clone(),
        revision: guard.1,
    })
}

/// Validates and applies render settings (SECURITY.md §2: validate all IPC input).
#[tauri::command]
#[specta::specta]
pub fn set_settings(
    state: State<'_, AppState>,
    settings: RenderSettings,
) -> Result<SettingsState, IpcError> {
    settings
        .validate()
        .map_err(|e| IpcError::new("settings.invalid", e.to_string()))?;
    let mut guard = state
        .settings
        .write()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    guard.0 = settings;
    guard.1 = guard.1.wrapping_add(1);
    Ok(SettingsState {
        settings: guard.0.clone(),
        revision: guard.1,
    })
}

/// Chooses the export destination with a Rust-side dialog. Returns its display path.
#[tauri::command]
#[specta::specta]
pub async fn pick_export_folder(app: AppHandle) -> Result<Option<String>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_title("Choose where to export")
            .blocking_pick_folder();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let display = path.display().to_string();
        let state = app.state::<AppState>();
        let mut dest = state
            .export_destination
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        *dest = Some(path);
        Ok(Some(display))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Starts exporting every photo to the chosen destination (FR-008, FR-009, FR-011).
/// Progress and the result arrive as `ExportProgress` / `ExportFinished` events.
#[tauri::command]
#[specta::specta]
pub fn start_export(app: AppHandle, state: State<'_, AppState>) -> Result<u32, IpcError> {
    let destination = state
        .export_destination
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .clone()
        .ok_or_else(|| IpcError::new("export.no_destination", "Choose a destination first"))?;
    let records: Vec<_> = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        library
            .sorted_ids()
            .into_iter()
            .filter_map(|id| library.record(id))
            .collect()
    };
    if records.is_empty() {
        return Err(IpcError::new(
            "export.empty",
            "There are no photos to export",
        ));
    }
    let settings = state
        .settings
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .0
        .clone();

    let job_id = state.next_job.fetch_add(1, Ordering::Relaxed);
    let cancel = CancellationToken::new();
    state
        .jobs
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .insert(job_id, cancel.clone());

    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let options = ExportOptions {
            destination: &destination,
            settings: &settings,
            quality: EXPORT_QUALITY,
            max_inflight: default_inflight(),
        };
        let throttle = Throttle::default();
        let on_progress = |done: u32, total: u32| {
            if throttle.due(done, total) {
                let _ = ExportProgress {
                    job_id,
                    done,
                    total,
                }
                .emit(&app);
            }
        };
        let summary = run_export(
            &records,
            &options,
            state.codec.as_ref(),
            &cancel,
            &on_progress,
        );
        if let Ok(mut jobs) = state.jobs.lock() {
            jobs.remove(&job_id);
        }
        let _ = ExportFinished { job_id, summary }.emit(&app);
    });
    Ok(job_id)
}

#[tauri::command]
#[specta::specta]
pub fn cancel_job(state: State<'_, AppState>, job_id: u32) -> Result<(), IpcError> {
    if let Some(token) = state
        .jobs
        .lock()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .get(&job_id)
    {
        token.cancel();
    }
    Ok(())
}

/// Bytes for `fotothumb://localhost/thumb/{id}`: cached T1, else T0, else a fresh T1.
pub fn thumbnail_bytes(state: &AppState, id: PhotoId) -> Option<Vec<u8>> {
    let record = state.library.read().ok()?.record(id)?;
    if let Some(bytes) = state.thumbs.cached_t1(&record) {
        return Some(bytes);
    }
    if record.flags & flags::HAS_EMBEDDED_THUMB != 0
        && let Some(bytes) = thumbs::t0(&record, state.codec.as_ref())
    {
        return Some(bytes);
    }
    state.thumbs.make_t1(&record, state.codec.as_ref()).ok()
}

/// Bytes for `fotothumb://localhost/preview/{id}`, rendered with the current settings.
pub fn preview_bytes(state: &AppState, id: PhotoId) -> Result<Vec<u8>, String> {
    let record = state
        .library
        .read()
        .ok()
        .and_then(|l| l.record(id))
        .ok_or_else(|| "unknown photo".to_owned())?;
    let settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
    state
        .previews
        .render_jpeg(&record, &settings, state.codec.as_ref())
        .map_err(|e| e.to_string())
}

/// Everything the preview inspector shows about one photo.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoDetails {
    pub id: PhotoId,
    pub file_name: String,
    pub folder_path: String,
    /// File size in bytes (as f64: TypeScript has no u64).
    pub file_bytes: f64,
    /// Upright pixel size (after EXIF orientation).
    pub width: u32,
    pub height: u32,
    /// Camera-local capture time, `YYYY-MM-DDTHH:MM:SS`.
    pub capture: String,
    /// `true` when no EXIF date exists and the file time is used.
    pub date_from_file: bool,
    pub orientation: u8,
    pub gps: Option<GpsPoint>,
    /// Effective DPI of the crop at the current print size (FR-026).
    pub print_dpi: u32,
    pub print_quality: DpiBand,
}

#[derive(Debug, Clone, Copy, Serialize, Type)]
pub struct GpsPoint {
    pub lat: f32,
    pub lon: f32,
}

/// Details for the preview inspector (FR-002, FR-007, FR-026). Reads only the JPEG header.
#[tauri::command]
#[specta::specta]
pub async fn photo_details(app: AppHandle, id: PhotoId) -> Result<PhotoDetails, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let record = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .record(id)
            .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
        let bytes = std::fs::read(&record.path)
            .map_err(|e| IpcError::new("import.unreadable", e.to_string()))?;
        let (w, h) = state
            .codec
            .dimensions(&bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let (width, height) = if record.orientation >= 5 {
            (h, w)
        } else {
            (w, h)
        };
        let print = state
            .settings
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .0
            .validate()
            .unwrap_or(PrintSize::ALL[1]);
        let (print_dpi, print_quality) = effective_dpi(width, height, print);
        Ok(PhotoDetails {
            id,
            file_name: record.file_name.clone(),
            folder_path: record
                .path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            #[allow(clippy::cast_precision_loss)] // exact below 2^53 bytes
            file_bytes: record.size as f64,
            width,
            height,
            capture: record.capture.format("%Y-%m-%dT%H:%M:%S").to_string(),
            date_from_file: record.flags & flags::DATE_FROM_FILE != 0,
            orientation: record.orientation,
            gps: record.gps.map(|(lat, lon)| GpsPoint { lat, lon }),
            print_dpi,
            print_quality,
        })
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}
