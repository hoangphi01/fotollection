// SPDX-License-Identifier: GPL-3.0-only
//! RollSense's commands (FR-141 S7, ADR-0056): the pixels' pass over the library in the
//! background (each photo's hash and CLIP embedding, cached), the duplicates proven on their
//! pictures (FR-015), and the facets the view lists. The pass runs on its own pool on every
//! core: on the two background threads 15,000 photos would pass NFR-022's 30 minutes.

// Tauri commands take `AppHandle` and `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use fotollection_core::cancel::CancellationToken;
use fotollection_core::library::{CullFilter, GallerySection, PhotoId, PhotoRecord};
use fotollection_core::sorting::clip::{Embedder, MODEL_BLAKE3};
use fotollection_core::sorting::dupes::{self, DupKind};
use fotollection_core::sorting::facets::Facets;
use fotollection_core::sorting::pass::{Pixels, picture};
use fotollection_core::sorting::queue::{Scope, SortQueue};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::commands::progress_reporter;
use crate::events::{SortingFinished, SortingQueueChanged, SortingRead, TaskKind};
use crate::state::{AppState, IpcError};

/// The bundled model, compiled in once here (the core takes its bytes).
static MODEL: &[u8] = include_bytes!("../../crates/core/assets/models/tinyclip-39m.w8.onnx");

fn poisoned(e: impl std::fmt::Display) -> IpcError {
    IpcError::internal(e.to_string())
}

/// The model, loaded and checked on first use.
fn embedder() -> Result<&'static Embedder, IpcError> {
    static EMBEDDER: OnceLock<Result<Embedder, String>> = OnceLock::new();
    EMBEDDER
        .get_or_init(|| Embedder::load(MODEL, MODEL_BLAKE3).map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| IpcError::new("sorting.clip_failed", e.clone()))
}

/// Photos read between two writes to the library: small enough that the cards fill often,
/// large enough that the write lock is rare.
const BATCH: usize = 256;
/// How often the UI is told to refetch the facets while a pass reads.
const SHOW_EVERY: std::time::Duration = std::time::Duration::from_secs(2);

/// The pass's pool, on every core, made on first use.
fn pool() -> Result<&'static rayon::ThreadPool, IpcError> {
    static POOL: OnceLock<Result<rayon::ThreadPool, String>> = OnceLock::new();
    POOL.get_or_init(|| {
        fotollection_core::pool::contained(None, "sorting").map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(|e| IpcError::internal(e.clone()))
}

/// RollSense's queue (FR-141; owner, 2026-11-20: ten folders pressed in turn are all read, one
/// after another): the scopes asked for, the running pass's job id, and whether the worker
/// thread lives. Its lock is taken only between passes, never across the pixels' work, and never
/// with another lock held (LESSONS, rust-concurrency).
#[derive(Debug, Default)]
pub struct SortingState {
    queue: SortQueue,
    job: Option<u32>,
    worker: bool,
}

impl SortingState {
    /// Nothing runs and nothing waits (the demo waits for this; the demo is debug only).
    #[cfg(debug_assertions)]
    pub fn is_idle(&self) -> bool {
        self.queue.running().is_none() && self.queue.waiting().next().is_none() && !self.worker
    }
}

/// One scope as the UI knows it: the folders' current ids (none: the Workspace).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QueuedScope {
    pub folders: Vec<u16>,
}

/// The queue for the progress bar and its list: the scope being read, its job (for its stop),
/// and the scopes waiting, in order.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SortingQueue {
    pub running: Option<QueuedScope>,
    pub job_id: Option<u32>,
    pub waiting: Vec<QueuedScope>,
}

/// A scope by the folders' paths (ids are renumbered when a folder is removed); unknown ids
/// are dropped, and a set that names none is refused.
fn scope_of(state: &AppState, folders: &[u16]) -> Result<Scope, IpcError> {
    if folders.is_empty() {
        return Ok(Scope::new(Vec::new()));
    }
    let library = state.library.read().map_err(poisoned)?;
    let paths: Vec<std::path::PathBuf> = folders
        .iter()
        .filter_map(|&f| library.folders().get(usize::from(f)).cloned())
        .collect();
    if paths.is_empty() {
        return Err(IpcError::new(
            "culling.empty",
            "There are no photos to analyse",
        ));
    }
    Ok(Scope::new(paths))
}

/// The queue in the UI's terms, each scope's folders by their current ids.
fn queue_view(state: &AppState) -> SortingQueue {
    let (running, job_id, waiting) = match state.sorting.lock() {
        Ok(s) => (
            s.queue.running().cloned(),
            s.job,
            s.queue.waiting().cloned().collect::<Vec<_>>(),
        ),
        Err(_) => (None, None, Vec::new()),
    };
    let ids = |scope: &Scope| -> QueuedScope {
        let folders = state.library.read().map_or_else(
            |_| Vec::new(),
            |library| {
                let all = library.folders();
                scope
                    .folders()
                    .iter()
                    .filter_map(|p| all.iter().position(|f| f == p))
                    .filter_map(|i| u16::try_from(i).ok())
                    .collect()
            },
        );
        QueuedScope { folders }
    };
    SortingQueue {
        running: running.as_ref().map(ids),
        job_id,
        waiting: waiting.iter().map(ids).collect(),
    }
}

fn queue_changed(app: &AppHandle) {
    let _ = SortingQueueChanged(queue_view(&app.state::<AppState>())).emit(app);
}

/// Asks RollSense to read the photos of `folders` (every folder when empty: the Workspace;
/// owner, 2026-11-18: "only the tab chosen"), cached ones at once, then prove the duplicates.
/// The scope joins the queue unless the running or a waiting scope already covers it, and the
/// worker reads one scope at a time (owner, 2026-11-20). Returns the queue.
#[tauri::command]
#[specta::specta]
pub fn analyze_sorting(
    app: AppHandle,
    state: State<'_, AppState>,
    folders: Vec<u16>,
) -> Result<SortingQueue, IpcError> {
    let scope = scope_of(&state, &folders)?;
    let has_photos = {
        let library = state.library.read().map_err(poisoned)?;
        !records_of(&library, &scope).is_empty()
    };
    if !has_photos {
        return Err(IpcError::new(
            "culling.empty",
            "There are no photos to analyse",
        ));
    }
    embedder()?;
    pool()?;
    let start = {
        let mut sorting = state.sorting.lock().map_err(poisoned)?;
        sorting.queue.push(scope);
        let start = !sorting.worker;
        sorting.worker = true;
        start
    };
    if start {
        let worker = app.clone();
        fotollection_core::supervise::spawn(
            "sorting",
            move || run_queue(&worker),
            worker_ended(&app),
        );
    }
    queue_changed(&app);
    Ok(queue_view(&state))
}

/// The queue as it stands (the UI asks on start).
#[tauri::command]
#[specta::specta]
pub fn sorting_queue(state: State<'_, AppState>) -> SortingQueue {
    queue_view(&state)
}

/// Stops the scope being read; the next one waiting starts.
#[tauri::command]
#[specta::specta]
pub fn stop_sorting(state: State<'_, AppState>) -> Result<(), IpcError> {
    let job = state.sorting.lock().map_err(poisoned)?.job;
    cancel(&state, job)
}

/// Takes a waiting scope out of the queue.
#[tauri::command]
#[specta::specta]
pub fn remove_sorting(
    app: AppHandle,
    state: State<'_, AppState>,
    folders: Vec<u16>,
) -> Result<(), IpcError> {
    let scope = scope_of(&state, &folders)?;
    let removed = state.sorting.lock().map_err(poisoned)?.queue.remove(&scope);
    if removed {
        queue_changed(&app);
    }
    Ok(())
}

/// *Stop all*: nothing waits any more and the scope being read stops.
#[tauri::command]
#[specta::specta]
pub fn cancel_sorting(app: AppHandle, state: State<'_, AppState>) -> Result<(), IpcError> {
    stop_all(&state)?;
    queue_changed(&app);
    Ok(())
}

/// Empties the queue and stops the running pass (also on a project switch).
pub fn stop_all(state: &AppState) -> Result<(), IpcError> {
    let job = {
        let mut sorting = state.sorting.lock().map_err(poisoned)?;
        sorting.queue.clear();
        sorting.job
    };
    cancel(state, job)
}

fn cancel(state: &AppState, job: Option<u32>) -> Result<(), IpcError> {
    if let Some(job) = job
        && let Some(token) = state.jobs.lock().map_err(poisoned)?.get(&job)
    {
        token.cancel();
    }
    Ok(())
}

/// The photos of a scope, listed when its turn comes (a folder imported into while it waited
/// is read whole; a folder removed meanwhile has none).
fn records_of(library: &fotollection_core::library::Library, scope: &Scope) -> Vec<PhotoRecord> {
    let folders = library.folders();
    let ids: Vec<u16> = scope
        .folders()
        .iter()
        .filter_map(|p| folders.iter().position(|f| f == p))
        .filter_map(|i| u16::try_from(i).ok())
        .collect();
    library
        .sorted_ids()
        .into_iter()
        .filter(|&id| {
            scope.is_workspace() || library.folder_of(id).is_some_and(|f| ids.contains(&f))
        })
        .filter_map(|id| library.record(id))
        .collect()
}

/// The worker: one scope at a time, each lock taken alone and let go before the pixels' work.
fn run_queue(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (Ok(embedder), Ok(pool)) = (embedder(), pool()) else {
        return;
    };
    loop {
        let job_id = state.next_job.fetch_add(1, Ordering::Relaxed);
        let cancel = CancellationToken::new();
        let next = match state.sorting.lock() {
            Ok(mut sorting) => {
                let next = sorting.queue.start_next();
                if next.is_some() {
                    sorting.job = Some(job_id);
                } else {
                    sorting.worker = false;
                }
                next
            }
            Err(_) => None,
        };
        let Some(scope) = next else {
            queue_changed(app);
            return;
        };
        if let Ok(mut jobs) = state.jobs.lock() {
            jobs.insert(job_id, cancel.clone());
        }
        queue_changed(app);
        let records = state
            .library
            .read()
            .map_or_else(|_| Vec::new(), |library| records_of(&library, &scope));
        let (analysed, failed) =
            read_batches(app, &state, pool, embedder, &records, &cancel, job_id);
        if !cancel.is_cancelled() {
            let candidates = state
                .library
                .read()
                .map_or_else(|_| Vec::new(), |library| library.duplicate_candidates());
            prove_duplicates(app, &state, pool, &records, &candidates, &cancel);
        }
        if let Ok(mut jobs) = state.jobs.lock() {
            jobs.remove(&job_id);
        }
        if let Ok(mut sorting) = state.sorting.lock() {
            sorting.queue.finish();
            sorting.job = None;
        }
        let _ = SortingFinished {
            job_id,
            analysed,
            failed,
            cancelled: cancel.is_cancelled(),
        }
        .emit(app);
    }
}

/// Reads `records` in batches, each kept as it is read, so the cards fill while the window stays
/// free (owner, 2026-11-18); a stopped pass puts back the snapshot taken before it. Returns the
/// photos read and those that failed.
fn read_batches(
    app: &AppHandle,
    state: &AppState,
    pool: &rayon::ThreadPool,
    embedder: &Embedder,
    records: &[PhotoRecord],
    cancel: &CancellationToken,
    job_id: u32,
) -> (u32, u32) {
    let total = u32::try_from(records.len()).unwrap_or(u32::MAX);
    let tick = progress_reporter(app, TaskKind::Sort, total);
    let failed = AtomicU32::new(0);
    // In batches, each kept as it is read, so the cards fill while the window stays
    // free (owner, 2026-11-18); a stopped pass puts the snapshot back.
    let before = state.library.read().ok().map(|l| l.sense_snapshot());
    let mut analysed = 0u32;
    let mut shown = std::time::Instant::now();
    for batch in records.chunks(BATCH) {
        if cancel.is_cancelled() {
            break;
        }
        let results: Vec<(PhotoId, std::path::PathBuf, Pixels)> = pool.install(|| {
            batch
                .par_iter()
                .filter_map(|r| {
                    if cancel.is_cancelled() {
                        return None;
                    }
                    let p = state
                        .sorting_cache
                        .pass(r, &state.thumbs, state.codec.as_ref(), embedder)
                        .inspect_err(|_| {
                            failed.fetch_add(1, Ordering::Relaxed);
                        })
                        .ok();
                    tick();
                    Some((r.id, r.path.clone(), p?))
                })
                .collect()
        });
        analysed += u32::try_from(results.len()).unwrap_or(u32::MAX);
        if let Ok(mut library) = state.library.write() {
            for (id, path, p) in &results {
                library.set_pixels(*id, path, p);
            }
        }
        if shown.elapsed() >= SHOW_EVERY {
            shown = std::time::Instant::now();
            let _ = SortingRead { job_id, analysed }.emit(app);
        }
    }
    if cancel.is_cancelled() {
        // Stopped: nothing is kept (what was read stays cached, so the next pass is quick).
        if let (Some(before), Ok(mut library)) = (before, state.library.write()) {
            library.restore_sense(before);
        }
    }
    (analysed, failed.load(Ordering::Relaxed))
}

/// Each candidate pair proven on its two pictures (FR-015: a re-save or a colour version,
/// never two screens of one app); the proven photos kept in the library. Its progress is shown
/// as its own task, with the pass's stop (owner, 2026-11-19: a stop with no word of what runs);
/// stopped, the duplicates stay as they were.
fn prove_duplicates(
    app: &AppHandle,
    state: &AppState,
    pool: &rayon::ThreadPool,
    records: &[PhotoRecord],
    candidates: &[(PhotoId, PhotoId)],
    cancel: &CancellationToken,
) {
    let rules = fotollection_core::sorting::dup_rules();
    let by_id: std::collections::HashMap<PhotoId, &PhotoRecord> =
        records.iter().map(|r| (r.id, r)).collect();
    // Only the scope's photos: a candidate pair may reach a photo of another folder, which this
    // pass does not read; every photo counted is ticked, so the progress always ends (a bar
    // stuck at 1/16, 2026-11-19).
    let mut wanted: Vec<PhotoId> = candidates
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .filter(|id| by_id.contains_key(id))
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let tick = progress_reporter(
        app,
        TaskKind::Duplicates,
        u32::try_from(wanted.len()).unwrap_or(u32::MAX),
    );
    let prints: std::collections::HashMap<PhotoId, dupes::Fingerprint> = pool.install(|| {
        wanted
            .par_iter()
            .filter_map(|id| {
                let print = (!cancel.is_cancelled())
                    .then(|| {
                        let image =
                            picture(by_id.get(id)?, &state.thumbs, state.codec.as_ref()).ok()?;
                        dupes::fingerprint(&image, rules.grid)
                    })
                    .flatten();
                tick();
                Some((*id, print?))
            })
            .collect()
    });
    if cancel.is_cancelled() {
        return;
    }
    let proven: Vec<(usize, usize, DupKind)> = candidates
        .iter()
        .filter_map(|(a, b)| {
            let kind = dupes::same_picture(prints.get(a)?, prints.get(b)?, rules)?;
            Some((a.0 as usize, b.0 as usize, kind))
        })
        .collect();
    let n = records
        .iter()
        .map(|r| r.id.0 as usize + 1)
        .max()
        .unwrap_or(0);
    // Each group's keeper left out (the largest file, then the oldest capture), so *Duplicates*
    // shows only the copies that can go.
    let rank = |m: usize| {
        let r = u32::try_from(m).ok().and_then(|id| by_id.get(&PhotoId(id)));
        r.map(|r| (r.size, std::cmp::Reverse(r.capture)))
    };
    let ids: Vec<PhotoId> = dupes::spare(&dupes::groups(n, &proven), rank)
        .into_iter()
        .filter_map(|m| Some(PhotoId(u32::try_from(m).ok()?)))
        .collect();
    if let Ok(mut library) = state.library.write() {
        let scope: Vec<PhotoId> = records.iter().map(|r| r.id).collect();
        library.set_duplicates_in(&scope, &ids);
    }
}

/// The worker's end when its thread dies: nothing runs or waits, so a press starts it again.
fn worker_ended(app: &AppHandle) -> impl FnOnce() + Send + 'static {
    let app = app.clone();
    move || {
        let state = app.state::<AppState>();
        let job = state.sorting.lock().ok().and_then(|mut s| {
            let job = s.job.take();
            *s = SortingState::default();
            job
        });
        if let (Some(job), Ok(mut jobs)) = (job, state.jobs.lock()) {
            jobs.remove(&job);
        }
        queue_changed(&app);
    }
}

/// Everything RollSense lists, with counts, over the photos of `folders` (all when empty).
#[tauri::command]
#[specta::specta]
pub fn sort_facets(state: State<'_, AppState>, folders: Vec<u16>) -> Result<Facets, IpcError> {
    Ok(state.library.read().map_err(poisoned)?.facets(&folders))
}

/// The grid's sections under RollSense's time picks (FR-141 C2): its events when a day is
/// ticked, its days for a month or a year, none otherwise.
#[tauri::command]
#[specta::specta]
pub fn gallery_sections(
    state: State<'_, AppState>,
    filter: CullFilter,
) -> Result<Vec<GallerySection>, IpcError> {
    Ok(state
        .library
        .read()
        .map_err(poisoned)?
        .gallery_sections(&filter))
}

/// Most photos one delete may take (IPC input bound, SECURITY.md §2): a whole roll.
const MAX_DELETE_IDS: usize = 100_000;

/// What a delete did: the files moved to the Trash (or deleted for good), and the photos it
/// could not move, by their ids in the library rebuilt after it (a volume with no Trash, such
/// as a NAS share, refuses; ADR-0055 asks again before deleting those for good).
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub removed: u32,
    pub not_removed: Vec<PhotoId>,
}

/// What moving the photos marked to delete will free and risk (FR-141), for the summary shown
/// before it: their bytes on disk, and how many sit on a volume that seems to have no Trash
/// (a network share), whose files would be deleted for good.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TrashCheck {
    pub bytes: f64,
    pub without_trash: u32,
}

/// Whether `path`'s volume seems to have no Trash. macOS: a volume under `/Volumes` without
/// `.Trashes` (network shares; a new USB drive may make one on its first delete). Windows: a
/// network path (`\\server\share`). Elsewhere, and when unsure, a Trash is assumed: the move
/// itself still refuses, and the user is asked again (ADR-0055).
fn without_trash(path: &std::path::Path) -> bool {
    if cfg!(target_os = "macos") {
        let mut parts = path.components().skip(1);
        if let (Some(first), Some(volume)) = (parts.next(), parts.next())
            && first.as_os_str() == "Volumes"
        {
            return !std::path::Path::new("/Volumes")
                .join(volume.as_os_str())
                .join(".Trashes")
                .exists();
        }
        return false;
    }
    let text = path.to_string_lossy();
    cfg!(windows) && text.starts_with(r"\\") && !text.starts_with(r"\\?\")
}

/// The summary before a move to the Trash (FR-141): the photos' bytes, and how many seem to
/// sit on a volume without a Trash.
#[tauri::command]
#[specta::specta]
pub fn trash_check(state: State<'_, AppState>, ids: Vec<PhotoId>) -> Result<TrashCheck, IpcError> {
    if ids.len() > MAX_DELETE_IDS {
        return Err(IpcError::new(
            "photo.invalid_selection",
            "Choose between 1 and 100,000 photos to delete",
        ));
    }
    let library = state.library.read().map_err(poisoned)?;
    let records: Vec<PhotoRecord> = ids.iter().filter_map(|&id| library.record(id)).collect();
    drop(library);
    #[allow(clippy::cast_precision_loss)]
    let bytes = records.iter().map(|r| r.size as f64).sum();
    let without = records.iter().filter(|r| without_trash(&r.path)).count();
    Ok(TrashCheck {
        bytes,
        without_trash: u32::try_from(without).unwrap_or(u32::MAX),
    })
}

/// Stops a move to the Trash before its next file; the photos not moved stay marked.
#[tauri::command]
#[specta::specta]
pub fn cancel_trash(state: State<'_, AppState>) {
    state.trash_cancel.store(true, Ordering::Relaxed);
}

/// Moves the chosen photos' files to the system's Trash (FR-141, ADR-0055), then rebuilds the
/// library from its folders so they leave it; every other photo keeps its edits. The paths
/// come from the library, never from the webview.
#[tauri::command]
#[specta::specta]
pub async fn trash_photos(app: AppHandle, ids: Vec<PhotoId>) -> Result<DeleteReport, IpcError> {
    remove_photos(app, ids, false).await
}

/// Deletes the chosen photos' files for good, after the user said so a second time (photos
/// a volume without a Trash refused), then rebuilds the library.
#[tauri::command]
#[specta::specta]
pub async fn delete_photos(app: AppHandle, ids: Vec<PhotoId>) -> Result<DeleteReport, IpcError> {
    remove_photos(app, ids, true).await
}

async fn remove_photos(
    app: AppHandle,
    ids: Vec<PhotoId>,
    for_good: bool,
) -> Result<DeleteReport, IpcError> {
    if ids.is_empty() || ids.len() > MAX_DELETE_IDS {
        return Err(IpcError::new(
            "photo.invalid_selection",
            "Choose between 1 and 100,000 photos to delete",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let _guard = state.document_lock.lock().map_err(poisoned)?;
        let (paths, folders, snapshot) = {
            let library = state.library.read().map_err(poisoned)?;
            let paths: Vec<std::path::PathBuf> = ids
                .iter()
                .filter_map(|&id| library.record(id).map(|r| r.path))
                .collect();
            if paths.len() != ids.len() {
                return Err(IpcError::new(
                    "photo.unknown",
                    "A photo to delete is not in the library",
                ));
            }
            let folders: Vec<std::path::PathBuf> = library
                .folders_in_order()
                .into_iter()
                .map(|(_, p)| p.clone())
                .collect();
            (
                paths,
                folders,
                fotollection_core::project::snapshot(&library),
            )
        };
        {
            let refused = state.trash_refused.lock().map_err(poisoned)?;
            if for_good && !paths.iter().all(|p| refused.contains(p)) {
                return Err(IpcError::new(
                    "photo.not_refused",
                    "Only photos the Trash refused can be deleted for good",
                ));
            }
        }
        let mut kept = std::collections::HashSet::new();
        let mut removed = 0u32;
        state.trash_cancel.store(false, Ordering::Relaxed);
        let tick = progress_reporter(
            &app,
            TaskKind::Trash,
            u32::try_from(paths.len()).unwrap_or(u32::MAX),
        );
        for path in &paths {
            if state.trash_cancel.load(Ordering::Relaxed) {
                break;
            }
            tick();
            let done = if for_good {
                std::fs::remove_file(path).is_ok()
            } else {
                trash::delete(path).is_ok()
            };
            if done {
                removed += 1;
            } else {
                kept.insert(path.clone());
            }
        }
        if !for_good {
            kept.clone_into(&mut *state.trash_refused.lock().map_err(poisoned)?);
        }
        if removed > 0 {
            crate::document::rebuild_library(&app, &state, folders, &snapshot)?;
        }
        let library = state.library.read().map_err(poisoned)?;
        let not_removed = library
            .sorted_ids()
            .into_iter()
            .filter(|&id| library.record(id).is_some_and(|r| kept.contains(&r.path)))
            .collect();
        Ok(DeleteReport {
            removed,
            not_removed,
        })
    })
    .await
    .map_err(poisoned)?
}
