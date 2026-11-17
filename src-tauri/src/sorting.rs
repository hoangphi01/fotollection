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

/// Reads every photo's pixels (cached ones at once), then proves the duplicates. Returns
/// the job id for `cancel_job`; a second call while one runs returns the running job.
#[tauri::command]
#[specta::specta]
pub fn analyze_sorting(app: AppHandle, state: State<'_, AppState>) -> Result<u32, IpcError> {
    let mut running = state.sorting.lock().map_err(poisoned)?;
    if let Some(job_id) = *running {
        return Ok(job_id);
    }
    let records: Vec<PhotoRecord> = {
        let library = state.library.read().map_err(poisoned)?;
        library
            .sorted_ids()
            .into_iter()
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
            let candidates = state.library.write().map_or_else(
                |_| Vec::new(),
                |mut library| {
                    for (id, path, p) in &results {
                        library.set_pixels(*id, path, p);
                    }
                    library.duplicate_candidates()
                },
            );
            if !cancelled {
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
    let ids: Vec<PhotoId> = dupes::groups(n, &proven)
        .into_iter()
        .flat_map(|g| g.members)
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

/// Everything RollSense lists, with counts.
#[tauri::command]
#[specta::specta]
pub fn sort_facets(state: State<'_, AppState>) -> Result<Facets, IpcError> {
    Ok(state.library.read().map_err(poisoned)?.facets())
}
