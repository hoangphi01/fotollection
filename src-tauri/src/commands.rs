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
use fotollection_core::codec::image_dimensions;
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, default_inflight, run_export};
use fotollection_core::frame::{CameraProfile, FrameSettings, LogoKind};
use fotollection_core::import::{self, ImportedFile};
use fotollection_core::library::{CullFilter, Flag, PhotoEdits, PhotoId, PhotoSummary, flags};
use fotollection_core::logo;
use fotollection_core::metadata;
use fotollection_core::plan::{CropFrame, DpiBand, PlanInput, effective_dpi, plan};
use fotollection_core::settings::{RenderSettings, UiSettings};
use fotollection_core::thumbs;
use rayon::prelude::*;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use crate::events::{
    ExportFinished, ExportProgress, LibraryChanged, RevisionChanged, TaskKind, TaskProgress,
    ThumbReady,
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
pub(crate) fn progress_reporter(
    app: &AppHandle,
    task: TaskKind,
    total: u32,
) -> impl Fn() + Sync + '_ {
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
    if state.ensure_camera_profiles() {
        state.persist();
    }
    // A refresh that finds nothing new changes nothing.
    if summary.added > 0 {
        state
            .project_dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
        state.mark_edited();
    }
    crate::document::emit(app);
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

/// Scans the library's folders again and adds photos that appeared since (FR-001). Photos
/// already in the library are recognised and keep every edit; nothing is re-imported.
#[tauri::command]
#[specta::specta]
pub async fn refresh_library(app: AppHandle) -> Result<ImportSummary, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let folders = app
            .state::<AppState>()
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .folders()
            .iter()
            .filter(|f| f.is_dir())
            .cloned()
            .collect();
        Ok(import_paths(&app, folders))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
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

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoList {
    /// The photos kept by the filter, in capture order.
    pub photos: Vec<PhotoSummary>,
    /// All photos in the library, filtered or not.
    pub total: u32,
    /// Photos with their own stamp set in the preview (FR-044), for the Reset button.
    pub custom_stamps: u32,
}

/// The photos kept by `filter`, ordered by capture time (FR-003, FR-022).
#[tauri::command]
#[specta::specta]
pub fn list_photos(state: State<'_, AppState>, filter: CullFilter) -> Result<PhotoList, IpcError> {
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    let custom_stamps = library.edited().filter(|(_, e)| e.stamp.is_some()).count();
    Ok(PhotoList {
        photos: library.summaries(&filter),
        total: u32::try_from(library.len()).unwrap_or(u32::MAX),
        custom_stamps: u32::try_from(custom_stamps).unwrap_or(u32::MAX),
    })
}

/// One photo's stamp as the preview edits it (FR-044).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoStamp {
    /// The stamp this photo prints with: its own, or the settings'.
    pub stamp: fotollection_core::settings::StampSettings,
    /// True when the photo has its own stamp.
    pub custom: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_photo_stamp(state: State<'_, AppState>, id: PhotoId) -> Result<PhotoStamp, IpcError> {
    let library = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    if library.record(id).is_none() {
        return Err(IpcError::new("photo.unknown", "Unknown photo"));
    }
    let own = library.edits(id).stamp;
    let custom = own.is_some();
    let stamp = match own {
        Some(stamp) => stamp,
        None => state
            .settings
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .0
            .stamp
            .clone(),
    };
    Ok(PhotoStamp { stamp, custom })
}

/// Gives one photo its own stamp, or (`None`) makes it follow the settings again; undoable.
#[tauri::command]
#[specta::specta]
pub fn set_photo_stamp(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
    stamp: Option<fotollection_core::settings::StampSettings>,
) -> Result<HistoryState, IpcError> {
    if let Some(s) = &stamp {
        s.validate()
            .map_err(|e| IpcError::new("settings.invalid", e.to_string()))?;
    }
    edit_photos(&app, &state, &[id], |_, e| e.stamp.clone_from(&stamp))
}

/// "Apply to all" in the preview: this photo's stamp becomes the setting for every photo,
/// and no photo keeps its own any more (the photo edits are one undoable step).
#[tauri::command]
#[specta::specta]
pub fn apply_stamp_to_all(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
) -> Result<HistoryState, IpcError> {
    let stamp = get_photo_stamp(state.clone(), id)?.stamp;
    state
        .settings
        .write()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .0
        .stamp = stamp;
    state.persist();
    reset_photo_stamps(app, state)
}

/// "Reset" in the gallery: every photo follows the settings' stamp again; undoable.
#[tauri::command]
#[specta::specta]
pub fn reset_photo_stamps(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<HistoryState, IpcError> {
    let ids: Vec<PhotoId> = state
        .library
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .edited()
        .filter(|(_, e)| e.stamp.is_some())
        .map(|(id, _)| id)
        .collect();
    if ids.is_empty() {
        return Ok(changed(&app, &state));
    }
    edit_photos(&app, &state, &ids, |_, e| e.stamp = None)
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
    let result = SettingsState {
        settings: guard.0.clone(),
        revision: guard.1,
    };
    drop(guard);
    state.persist();
    Ok(result)
}

/// Chooses the export destination with a Rust-side dialog. Returns its display path.
/// Interface preferences of this machine (FR-020).
#[tauri::command]
#[specta::specta]
pub fn get_ui_settings(state: State<'_, AppState>) -> Result<UiSettings, IpcError> {
    state
        .ui
        .read()
        .map(|ui| *ui)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Changes the interface language; the typed enum is the validation (FR-020).
#[tauri::command]
#[specta::specta]
pub fn set_ui_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    ui: UiSettings,
) -> Result<UiSettings, IpcError> {
    *state
        .ui
        .write()
        .map_err(|e| IpcError::internal(e.to_string()))? = ui;
    // A per-machine preference: not an edit of the document.
    state.save_settings();
    crate::menu::refresh(&app, ui.language);
    crate::document::emit(&app);
    Ok(ui)
}

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

/// Starts exporting the photos kept by `filter` to the chosen destination
/// (FR-008, FR-009, FR-011, FR-022).
/// Progress and the result arrive as `ExportProgress` / `ExportFinished` events.
#[tauri::command]
#[specta::specta]
pub fn start_export(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: CullFilter,
) -> Result<u32, IpcError> {
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
        // Exactly the photos the grid shows (FR-022).
        library
            .filtered_ids(&filter)
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

    // Snapshot the frame state so export never holds locks the UI needs.
    let frames = state
        .frame
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .clone();

    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let format = settings.place_format;
        let place_for = |record: &fotollection_core::library::PhotoRecord| {
            crate::state::place_for(record, format)
        };
        let frame_for = |record: &fotollection_core::library::PhotoRecord| {
            frames.request(record, place_for(record).as_deref())
        };
        let options = ExportOptions {
            destination: &destination,
            settings: &settings,
            quality: EXPORT_QUALITY,
            max_inflight: default_inflight(),
            place_for: &place_for,
            frame_for: &frame_for,
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
    let place = crate::state::place_for(&record, settings.place_format);
    let frame = {
        let f = state.frame.read().map_err(|e| e.to_string())?;
        f.request(&record, place.as_deref())
    };
    state
        .previews
        .render_jpeg(
            &record,
            &settings,
            frame.as_ref(),
            place.as_deref(),
            state.codec.as_ref(),
        )
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
    /// Camera profile row for this photo's camera (FR-040), if it has EXIF make/model.
    pub camera_id: Option<u32>,
    pub lens: Option<String>,
    pub focal_mm: Option<f32>,
    pub f_number: Option<f32>,
    pub exposure_s: Option<f32>,
    pub iso: Option<u32>,
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
        let (w, h) = image_dimensions(state.codec.as_ref(), &bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let (width, height) = if record.display_orientation() >= 5 {
            (h, w)
        } else {
            (w, h)
        };
        let print = state
            .settings
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .0
            .print_for(width, height)
            .unwrap_or(PrintSize::ALL[1]);
        let (print_dpi, print_quality) = effective_dpi(width, height, print);
        let shot = record.shot.clone();
        let camera_id = state.frame.read().ok().and_then(|f| {
            f.profile_for(shot.make.as_deref(), shot.model.as_deref())
                .map(|p| p.id)
        });
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
            camera_id,
            lens: shot.lens,
            focal_mm: shot.focal_mm,
            f_number: shot.f_number,
            exposure_s: shot.exposure_s,
            iso: shot.iso,
        })
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// A camera profile plus the defaults the UI shows as placeholders.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraProfileView {
    pub profile: CameraProfile,
    pub default_display_name: Option<String>,
    pub default_brand: Option<String>,
    pub has_logo_image: bool,
    /// A logo is bundled for this camera's brand (ADR-0020).
    pub has_brand_logo: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FrameConfig {
    pub settings: FrameSettings,
    pub cameras: Vec<CameraProfileView>,
    pub revision: u32,
}

fn frame_config(state: &AppState) -> Result<FrameConfig, IpcError> {
    let frame = state
        .frame
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?;
    let revision = state.settings.read().map_or(0, |s| s.1);
    Ok(FrameConfig {
        settings: frame.settings.clone(),
        cameras: frame
            .cameras
            .iter()
            .map(|p| CameraProfileView {
                default_display_name: p.default_display_name(),
                default_brand: p.default_brand(),
                has_logo_image: frame.logos.contains_key(&p.id),
                has_brand_logo: p.brand().is_some(),
                profile: p.clone(),
            })
            .collect(),
        revision,
    })
}

/// Info-frame settings and camera profiles (FR-040).
#[tauri::command]
#[specta::specta]
pub fn get_frame_config(state: State<'_, AppState>) -> Result<FrameConfig, IpcError> {
    frame_config(&state)
}

#[tauri::command]
#[specta::specta]
pub fn set_frame_settings(
    state: State<'_, AppState>,
    settings: FrameSettings,
) -> Result<FrameConfig, IpcError> {
    state
        .frame
        .write()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .settings = settings;
    state.bump_revision();
    state.persist();
    frame_config(&state)
}

/// Updates the editable fields of one camera profile. Make/model (its identity) and the
/// logo kind are kept from the stored row; logos change only through the logo commands.
#[tauri::command]
#[specta::specta]
pub fn set_camera_profile(
    state: State<'_, AppState>,
    profile: CameraProfile,
) -> Result<FrameConfig, IpcError> {
    {
        let mut frame = state
            .frame
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let has_logo = frame.logos.contains_key(&profile.id);
        let row = frame
            .cameras
            .iter_mut()
            .find(|c| c.id == profile.id)
            .ok_or_else(|| IpcError::new("camera.unknown", "Unknown camera"))?;
        let logo = match profile.logo {
            LogoKind::Image if !has_logo => row.logo,
            other => other,
        };
        *row = CameraProfile {
            id: row.id,
            make: row.make.clone(),
            model: row.model.clone(),
            logo,
            ..profile
        }
        .sanitized();
    }
    state.bump_revision();
    state.persist();
    frame_config(&state)
}

/// Picks a PNG logo with a Rust-side dialog, validates it, and copies it into the app's
/// config folder (ADR-0014, SECURITY.md §3). `None` if the user cancelled.
#[tauri::command]
#[specta::specta]
pub async fn pick_camera_logo(
    app: AppHandle,
    camera_id: u32,
) -> Result<Option<FrameConfig>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let picked = app
            .dialog()
            .file()
            .set_title("Choose a logo (PNG)")
            .add_filter("PNG image", &["png"])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let size = std::fs::metadata(&path)
            .map_err(|e| IpcError::new("logo.unreadable", e.to_string()))?
            .len();
        if size > logo::MAX_LOGO_BYTES as u64 {
            return Err(IpcError::new(
                "logo.too_large",
                "The logo file is larger than 2 MB",
            ));
        }
        let bytes =
            std::fs::read(&path).map_err(|e| IpcError::new("logo.unreadable", e.to_string()))?;
        let image = logo::decode_png(&bytes).map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        std::fs::create_dir_all(&state.logo_dir)
            .map_err(|e| IpcError::new("logo.write_failed", e.to_string()))?;
        fotollection_core::thumbs::write_atomic(
            &state.logo_dir.join(format!("{camera_id}.png")),
            &bytes,
        )
        .map_err(|e| IpcError::new("logo.write_failed", e.to_string()))?;
        {
            let mut frame = state
                .frame
                .write()
                .map_err(|e| IpcError::internal(e.to_string()))?;
            let row = frame
                .cameras
                .iter_mut()
                .find(|c| c.id == camera_id)
                .ok_or_else(|| IpcError::new("camera.unknown", "Unknown camera"))?;
            row.logo = LogoKind::Image;
            frame.logos.insert(camera_id, std::sync::Arc::new(image));
        }
        state.bump_revision();
        state.persist();
        frame_config(&state).map(Some)
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub fn clear_camera_logo(
    state: State<'_, AppState>,
    camera_id: u32,
) -> Result<FrameConfig, IpcError> {
    {
        let mut frame = state
            .frame
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        frame.logos.remove(&camera_id);
        if let Some(row) = frame.cameras.iter_mut().find(|c| c.id == camera_id)
            && row.logo == LogoKind::Image
        {
            // Back to the bundled brand logo when there is one, else the typeset name.
            row.logo = if row.brand().is_some() {
                LogoKind::Brand
            } else {
                LogoKind::Text
            };
        }
    }
    let _ = std::fs::remove_file(state.logo_dir.join(format!("{camera_id}.png")));
    state.bump_revision();
    state.persist();
    frame_config(&state)
}

/// Bumps the revision, marks the project dirty, and tells the UI (one place for all edits).
pub fn changed(app: &AppHandle, state: &AppState) -> HistoryState {
    let revision = state.bump_revision();
    state.project_dirty.store(true, Ordering::Relaxed);
    state.mark_edited();
    crate::document::emit(app);
    let (can_undo, can_redo) = state
        .history
        .lock()
        .map_or((false, false), |h| (h.can_undo(), h.can_redo()));
    let _ = RevisionChanged {
        revision,
        can_undo,
        can_redo,
    }
    .emit(app);
    // Ratings and flags are part of the photo list, so undo/redo must refresh it too.
    let count = state
        .library
        .read()
        .map_or(0, |l| u32::try_from(l.len()).unwrap_or(u32::MAX));
    let _ = LibraryChanged { count }.emit(app);
    HistoryState {
        revision,
        can_undo,
        can_redo,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryState {
    pub revision: u32,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// What the crop editor needs (FR-006): the upright photo size, the target box the crop
/// must fill, its printed size in cm, and the current crop.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CropGeometry {
    pub width: u32,
    pub height: u32,
    /// The crop must have the ratio `target_width : target_height`.
    pub target_width: u32,
    pub target_height: u32,
    /// Printed size of the cropped area, as displayed (width × height).
    pub print_width_cm: f32,
    pub print_height_cm: f32,
    pub crop: Option<CropFrame>,
    /// The frame's Fit mode shows the whole photo, so cropping is off.
    pub crop_disabled: bool,
    /// Display unit of the selected size (inches for custom sizes entered in inches).
    pub unit: fotollection_core::settings::LengthUnit,
}

#[tauri::command]
#[specta::specta]
pub async fn get_crop_geometry(app: AppHandle, id: PhotoId) -> Result<CropGeometry, IpcError> {
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
        let (w, h) = image_dimensions(state.codec.as_ref(), &bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let (width, height) = if record.display_orientation() >= 5 {
            (h, w)
        } else {
            (w, h)
        };
        let settings = state
            .settings
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .0
            .clone();
        let print = settings
            .print_for(width, height)
            .unwrap_or(PrintSize::ALL[1]);
        let place = crate::state::place_for(&record, settings.place_format);
        let frame = state
            .frame
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .request(&record, place.as_deref());
        let p = plan(&PlanInput {
            width,
            height,
            capture: None,
            settings: &settings,
            print,
            max_long_edge: None,
            frame: frame.as_ref(),
            crop: None,
            place: None,
        });
        #[allow(clippy::cast_precision_loss)]
        let (long_cm, short_cm) = (print.long_mm as f32 / 10.0, print.short_mm as f32 / 10.0);
        let (canvas_width_cm, canvas_height_cm) = if width >= height {
            (long_cm, short_cm)
        } else {
            (short_cm, long_cm)
        };
        let (canvas_w, canvas_h) = p.output_size();
        #[allow(clippy::cast_precision_loss)]
        let (print_width_cm, print_height_cm) = (
            canvas_width_cm * p.out_width as f32 / canvas_w.max(1) as f32,
            canvas_height_cm * p.out_height as f32 / canvas_h.max(1) as f32,
        );
        let crop_disabled = frame
            .as_ref()
            .is_some_and(|f| f.settings.fit == fotollection_core::frame::FrameFit::Fit);
        Ok(CropGeometry {
            width,
            height,
            target_width: p.out_width,
            target_height: p.out_height,
            print_width_cm,
            print_height_cm,
            crop: record.edits.crop,
            crop_disabled,
            unit: settings.unit(),
        })
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Sets (or, with `None`, resets) a photo's crop. Undoable (FR-023).
#[tauri::command]
#[specta::specta]
pub fn set_photo_crop(
    app: AppHandle,
    state: State<'_, AppState>,
    id: PhotoId,
    crop: Option<CropFrame>,
) -> Result<HistoryState, IpcError> {
    {
        let mut library = state
            .library
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let mut edits = library.edits(id);
        edits.crop = crop.map(CropFrame::sanitized);
        state
            .history
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .apply(&mut library, &[(id, edits)]);
    }
    Ok(changed(&app, &state))
}

/// Most photos one culling command may change (IPC input bound, SECURITY.md §2).
const MAX_EDIT_IDS: usize = 10_000;

/// Applies `edit` to each photo as one undoable step (FR-022, FR-023).
pub(crate) fn edit_photos(
    app: &AppHandle,
    state: &AppState,
    ids: &[PhotoId],
    edit: impl Fn(PhotoId, &mut PhotoEdits),
) -> Result<HistoryState, IpcError> {
    if ids.is_empty() || ids.len() > MAX_EDIT_IDS {
        return Err(IpcError::new(
            "photo.invalid_selection",
            "Select between 1 and 10,000 photos",
        ));
    }
    {
        let mut library = state
            .library
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let mut changes = Vec::with_capacity(ids.len());
        for &id in ids {
            if library.record(id).is_none() {
                return Err(IpcError::new("photo.unknown", "Unknown photo"));
            }
            let mut edits = library.edits(id);
            edit(id, &mut edits);
            changes.push((id, edits));
        }
        state
            .history
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .apply(&mut library, &changes);
    }
    Ok(changed(app, state))
}

/// Sets 0–5 stars on photos (FR-022). Larger values are rejected.
#[tauri::command]
#[specta::specta]
pub fn set_photo_rating(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    rating: u8,
) -> Result<HistoryState, IpcError> {
    if rating > 5 {
        return Err(IpcError::new(
            "photo.invalid_rating",
            "Ratings are 0 to 5 stars",
        ));
    }
    edit_photos(&app, &state, &ids, |_, e| e.rating = rating)
}

/// Turns photos a quarter turn (FR-006), lossless; their crops turn with them. Undoable.
#[tauri::command]
#[specta::specta]
pub fn rotate_photos(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    clockwise: bool,
) -> Result<HistoryState, IpcError> {
    edit_photos(&app, &state, &ids, |_, e| {
        e.rotation = (e.rotation + if clockwise { 1 } else { 3 }) % 4;
        e.crop = e.crop.map(|c| c.turned(clockwise));
    })
}

/// Flags photos as pick or reject, or clears the flag (FR-022).
#[tauri::command]
#[specta::specta]
pub fn set_photo_flag(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    flag: Option<Flag>,
) -> Result<HistoryState, IpcError> {
    edit_photos(&app, &state, &ids, |_, e| e.flag = flag)
}

#[tauri::command]
#[specta::specta]
pub fn undo(app: AppHandle, state: State<'_, AppState>) -> Result<HistoryState, IpcError> {
    {
        let mut library = state
            .library
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        state
            .history
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .undo(&mut library);
    }
    Ok(changed(&app, &state))
}

#[tauri::command]
#[specta::specta]
pub fn redo(app: AppHandle, state: State<'_, AppState>) -> Result<HistoryState, IpcError> {
    {
        let mut library = state
            .library
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        state
            .history
            .lock()
            .map_err(|e| IpcError::internal(e.to_string()))?
            .redo(&mut library);
    }
    Ok(changed(&app, &state))
}

/// Bytes for `fotothumb://localhost/source/{id}`: the upright, uncropped T2 image.
pub fn source_bytes(state: &AppState, id: PhotoId) -> Result<Vec<u8>, String> {
    let record = state
        .library
        .read()
        .ok()
        .and_then(|l| l.record(id))
        .ok_or_else(|| "unknown photo".to_owned())?;
    state
        .previews
        .source_jpeg(&record, state.codec.as_ref())
        .map_err(|e| e.to_string())
}

/// Reopens the session project: re-scans its folders and re-applies the saved edits.
pub fn restore_session(app: &AppHandle) {
    use fotollection_core::project::{self, Loaded};
    let state = app.state::<AppState>();
    let file = match project::load(&state.project_path) {
        Loaded::Ok(file) => file,
        Loaded::Missing => return,
        Loaded::NewerVersion(v) => {
            eprintln!("session project has schema {v}; not opening or overwriting it");
            state.project_writable.store(false, Ordering::Relaxed);
            return;
        }
        Loaded::Corrupt(reason) => {
            eprintln!("session project is invalid ({reason}); starting empty");
            let mut aside = state.project_path.as_os_str().to_owned();
            aside.push(".corrupt");
            let _ = std::fs::rename(&state.project_path, aside);
            return;
        }
    };
    let folders: Vec<_> = project::folder_paths(&file)
        .into_iter()
        .filter(|p| p.is_dir())
        .collect();
    if folders.is_empty() {
        return;
    }
    crate::document::restore_reference(&state, &file);
    import_paths(app, folders);
    if let Ok(mut library) = state.library.write() {
        let summary = project::restore(&mut library, &file);
        if summary.missing + summary.changed > 0 {
            eprintln!(
                "project restore: {} restored, {} missing, {} changed",
                summary.restored, summary.missing, summary.changed
            );
        }
    }
    let count = state
        .library
        .read()
        .map_or(0, |l| u32::try_from(l.len()).unwrap_or(u32::MAX));
    let _ = LibraryChanged { count }.emit(app);
    // Restoring is not an edit: keep the unsaved flag the session had.
    let edited = file.document.as_ref().is_some_and(|d| d.edited);
    changed(app, &state);
    if let Ok(mut doc) = state.document.lock() {
        doc.edited = edited;
    }
    crate::document::emit(app);
}

/// User fonts (FR-042).
#[tauri::command]
#[specta::specta]
pub fn list_fonts(
    state: State<'_, AppState>,
) -> Result<Vec<fotollection_core::fonts::UserFont>, IpcError> {
    Ok(state
        .frame
        .read()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .fonts
        .clone())
}

/// Picks an OTF/TTF with a Rust-side dialog, validates it and copies it into the config
/// folder (SECURITY.md §3). Returns the new font, or `None` if cancelled.
#[tauri::command]
#[specta::specta]
pub async fn import_font(
    app: AppHandle,
) -> Result<Option<fotollection_core::fonts::UserFont>, IpcError> {
    use fotollection_core::fonts::{self, LoadedFont, UserFont};
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let picked = app
            .dialog()
            .file()
            .set_title("Choose a font (OTF or TTF)")
            .add_filter("Fonts", &["ttf", "otf"])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let size = std::fs::metadata(&path)
            .map_err(|e| IpcError::new("font.unreadable", e.to_string()))?
            .len();
        if size > fonts::MAX_FONT_BYTES as u64 {
            return Err(IpcError::new(
                "font.too_large",
                "The font file is larger than 20 MB",
            ));
        }
        let bytes =
            std::fs::read(&path).map_err(|e| IpcError::new("font.unreadable", e.to_string()))?;
        let copy = bytes.clone();
        let (format, font) =
            fonts::load(bytes).map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let name: String = path.file_stem().map_or_else(
            || "Font".to_owned(),
            |s| {
                s.to_string_lossy()
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(60)
                    .collect()
            },
        );
        let row = {
            let mut frame = state
                .frame
                .write()
                .map_err(|e| IpcError::internal(e.to_string()))?;
            if frame.fonts.len() >= fonts::MAX_USER_FONTS {
                return Err(IpcError::new("font.too_many", "Too many fonts imported"));
            }
            let id = frame.fonts.iter().map(|f| f.id).max().unwrap_or(0) + 1;
            std::fs::create_dir_all(&state.font_dir)
                .map_err(|e| IpcError::new("font.write_failed", e.to_string()))?;
            fotollection_core::thumbs::write_atomic(
                &state.font_dir.join(format!("{id}.{}", format.extension())),
                &copy,
            )
            .map_err(|e| IpcError::new("font.write_failed", e.to_string()))?;
            let row = UserFont { id, name, format };
            frame.fonts.push(row.clone());
            frame.loaded_fonts.insert(id, LoadedFont { id, font });
            row
        };
        state.persist();
        Ok(Some(row))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Removes a user font; the stamp or frame using it falls back to Inter.
#[tauri::command]
#[specta::specta]
pub fn remove_font(app: AppHandle, state: State<'_, AppState>, id: u32) -> Result<(), IpcError> {
    let format = {
        let mut frame = state
            .frame
            .write()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let format = frame.fonts.iter().find(|f| f.id == id).map(|f| f.format);
        frame.fonts.retain(|f| f.id != id);
        frame.loaded_fonts.remove(&id);
        if frame.settings.font == Some(id) {
            frame.settings.font = None;
        }
        format
    };
    if let Some(format) = format {
        let _ = std::fs::remove_file(state.font_dir.join(format!("{id}.{}", format.extension())));
    }
    state.persist();
    changed(&app, &state);
    Ok(())
}
