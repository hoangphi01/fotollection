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
use fotollection_core::library::{PhotoId, PhotoRecord};
use fotollection_core::sorting::clip::{Embedder, MODEL_BLAKE3};
use fotollection_core::sorting::dupes::{self, DupKind};
use fotollection_core::sorting::facets::Facets;
use fotollection_core::sorting::pass::{Pixels, picture};
use rayon::prelude::*;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::commands::progress_reporter;
use crate::events::{SortingFinished, TaskKind};
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

/// The pass's pool, on every core, made on first use.
fn pool() -> Result<&'static rayon::ThreadPool, IpcError> {
    static POOL: OnceLock<Result<rayon::ThreadPool, String>> = OnceLock::new();
    POOL.get_or_init(|| {
        fotollection_core::pool::contained(None, "sorting").map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(|e| IpcError::internal(e.clone()))
}

/// Reads the pixels of the photos in `folders` (every folder when empty: the Workspace;
/// owner, 2026-11-18: "only the tab chosen"), cached ones at once, then proves the duplicates.
/// Cancelled, it keeps nothing: the library is as before. Returns the job id for `cancel_job`;
/// a second call while one runs returns the running job.
#[tauri::command]
#[specta::specta]
pub fn analyze_sorting(
    app: AppHandle,
    state: State<'_, AppState>,
    folders: Vec<u16>,
) -> Result<u32, IpcError> {
    let mut running = state.sorting.lock().map_err(poisoned)?;
    if let Some(job_id) = *running {
        return Ok(job_id);
    }
    let records: Vec<PhotoRecord> = {
        let library = state.library.read().map_err(poisoned)?;
        library
            .sorted_ids()
            .into_iter()
            .filter(|&id| {
                folders.is_empty() || library.folder_of(id).is_some_and(|f| folders.contains(&f))
            })
            .filter_map(|id| library.record(id))
            .collect()
    };
    if records.is_empty() {
        return Err(IpcError::new(
            "culling.empty",
            "There are no photos to analyse",
        ));
    }
    let embedder = embedder()?;
    let pool = pool()?;
    let job_id = state.next_job.fetch_add(1, Ordering::Relaxed);
    let cancel = CancellationToken::new();
    state
        .jobs
        .lock()
        .map_err(poisoned)?
        .insert(job_id, cancel.clone());
    *running = Some(job_id);
    drop(running);

    let total = u32::try_from(records.len()).unwrap_or(u32::MAX);
    let ended = sorting_ended(&app, job_id, total);
    fotollection_core::supervise::spawn(
        "sorting",
        move || {
            let state = app.state::<AppState>();
            let tick = progress_reporter(&app, TaskKind::Sort, total);
            let failed = AtomicU32::new(0);
            let results: Vec<(PhotoId, std::path::PathBuf, Pixels)> = pool.install(|| {
                records
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
            let cancelled = cancel.is_cancelled();
            let analysed = u32::try_from(results.len()).unwrap_or(u32::MAX);
            // Cancelled: nothing is kept (what was read stays cached, so the next pass is quick).
            if !cancelled {
                let candidates = state.library.write().map_or_else(
                    |_| Vec::new(),
                    |mut library| {
                        for (id, path, p) in &results {
                            library.set_pixels(*id, path, p);
                        }
                        library.duplicate_candidates()
                    },
                );
                prove_duplicates(&state, pool, &records, &candidates);
            }
            if let Ok(mut running) = state.sorting.lock() {
                *running = None;
            }
            if let Ok(mut jobs) = state.jobs.lock() {
                jobs.remove(&job_id);
            }
            let _ = SortingFinished {
                job_id,
                analysed,
                failed: failed.load(Ordering::Relaxed),
                cancelled,
            }
            .emit(&app);
        },
        ended,
    );
    Ok(job_id)
}

/// Each candidate pair proven on its two pictures (FR-015: a re-save or a colour version,
/// never two screens of one app); the proven photos kept in the library.
fn prove_duplicates(
    state: &AppState,
    pool: &rayon::ThreadPool,
    records: &[PhotoRecord],
    candidates: &[(PhotoId, PhotoId)],
) {
    let rules = fotollection_core::sorting::dup_rules();
    let by_id: std::collections::HashMap<PhotoId, &PhotoRecord> =
        records.iter().map(|r| (r.id, r)).collect();
    let mut wanted: Vec<PhotoId> = candidates.iter().flat_map(|&(a, b)| [a, b]).collect();
    wanted.sort_unstable();
    wanted.dedup();
    let prints: std::collections::HashMap<PhotoId, dupes::Fingerprint> = pool.install(|| {
        wanted
            .par_iter()
            .filter_map(|id| {
                let image = picture(by_id.get(id)?, &state.thumbs, state.codec.as_ref()).ok()?;
                Some((*id, dupes::fingerprint(&image, rules.grid)?))
            })
            .collect()
    });
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
        library.set_duplicates(&ids);
    }
}

/// The pass's end when its thread dies: nothing running, the job gone, every photo failed.
fn sorting_ended(app: &AppHandle, job_id: u32, total: u32) -> impl FnOnce() + Send + 'static {
    let app = app.clone();
    move || {
        let state = app.state::<AppState>();
        if let Ok(mut running) = state.sorting.lock() {
            *running = None;
        }
        if let Ok(mut jobs) = state.jobs.lock() {
            jobs.remove(&job_id);
        }
        let _ = SortingFinished {
            job_id,
            analysed: 0,
            failed: total,
            cancelled: false,
        }
        .emit(&app);
    }
}

/// Everything RollSense lists, with counts, over the photos of `folders` (all when empty).
#[tauri::command]
#[specta::specta]
pub fn sort_facets(state: State<'_, AppState>, folders: Vec<u16>) -> Result<Facets, IpcError> {
    Ok(state.library.read().map_err(poisoned)?.facets(&folders))
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
        for path in &paths {
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
