// SPDX-License-Identifier: GPL-3.0-only
//! Best-shot suggestion commands (FR-041): analyse the library in the background, list the
//! groups, and keep the best photo of chosen groups as one undoable step.

// Tauri commands take `AppHandle` and `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use fotollection_core::cancel::CancellationToken;
use fotollection_core::culling::{Group, suggest};
use fotollection_core::library::{Flag, PhotoId};
use rayon::prelude::*;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::commands::{HistoryState, edit_photos, progress_reporter};
use crate::events::{AnalysisFinished, TaskKind};
use crate::state::{AppState, IpcError};

/// Most groups one `keep_best` call may change (IPC input bound, SECURITY.md §2).
const MAX_GROUP_IDS: usize = 5_000;

fn poisoned(e: impl std::fmt::Display) -> IpcError {
    IpcError::internal(e.to_string())
}

/// Analyses every photo of the library on the background pool, reusing cached results
/// (DATABASE.md §8), then groups and ranks them. Returns the job id for `cancel_job`; a
/// second call while one runs returns the running job.
#[tauri::command]
#[specta::specta]
pub fn analyze_library(app: AppHandle, state: State<'_, AppState>) -> Result<u32, IpcError> {
    let mut analyses = state.analyses.lock().map_err(poisoned)?;
    if let Some(job_id) = analyses.running {
        return Ok(job_id);
    }
    let records: Vec<_> = {
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
    let job_id = state.next_job.fetch_add(1, Ordering::Relaxed);
    let cancel = CancellationToken::new();
    state
        .jobs
        .lock()
        .map_err(poisoned)?
        .insert(job_id, cancel.clone());
    analyses.running = Some(job_id);
    let generation = analyses.generation;
    drop(analyses);

    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let total = u32::try_from(records.len()).unwrap_or(u32::MAX);
        let tick = progress_reporter(&app, TaskKind::Analyze, total);
        let failed = AtomicU32::new(0);
        let results: HashMap<_, _> = state.background_pool.install(|| {
            records
                .par_iter()
                .filter_map(|record| {
                    if cancel.is_cancelled() {
                        return None;
                    }
                    let result = state
                        .analysis_cache
                        .analyze(record, state.codec.as_ref())
                        .inspect_err(|_| {
                            failed.fetch_add(1, Ordering::Relaxed);
                        })
                        .ok();
                    tick();
                    Some((record.id, result?))
                })
                .collect()
        });
        let cancelled = cancel.is_cancelled();
        let analyzed = u32::try_from(results.len()).unwrap_or(u32::MAX);
        let mut groups = 0;
        if let Ok(mut analyses) = state.analyses.lock() {
            analyses.running = None;
            // Keep what finished even when cancelled: it is cached on disk anyway.
            if analyses.generation == generation {
                analyses.by_photo.extend(results);
                analyses.groups = suggest(&records, &analyses.by_photo);
                groups = u32::try_from(analyses.groups.len()).unwrap_or(u32::MAX);
            }
        }
        if let Ok(mut jobs) = state.jobs.lock() {
            jobs.remove(&job_id);
        }
        let _ = AnalysisFinished {
            job_id,
            analyzed,
            failed: failed.load(Ordering::Relaxed),
            groups,
            cancelled,
        }
        .emit(&app);
    });
    Ok(job_id)
}

/// Groups of similar shots from the last analysis, in capture order, best photo first.
#[tauri::command]
#[specta::specta]
pub fn list_groups(state: State<'_, AppState>) -> Result<Vec<Group>, IpcError> {
    Ok(state.analyses.lock().map_err(poisoned)?.groups.clone())
}

/// Picks the best photo of each chosen group and rejects the others, as one undoable step
/// (FR-041, FR-022). Nothing is deleted.
#[tauri::command]
#[specta::specta]
pub fn keep_best(
    app: AppHandle,
    state: State<'_, AppState>,
    groups: Vec<PhotoId>,
) -> Result<HistoryState, IpcError> {
    if groups.is_empty() || groups.len() > MAX_GROUP_IDS {
        return Err(IpcError::new(
            "culling.invalid_groups",
            "Choose between 1 and 5,000 groups",
        ));
    }
    let flags: HashMap<PhotoId, Flag> = {
        let analyses = state.analyses.lock().map_err(poisoned)?;
        let mut flags = HashMap::new();
        for id in &groups {
            let group = analyses
                .groups
                .iter()
                .find(|g| g.id == *id)
                .ok_or_else(|| IpcError::new("culling.unknown_group", "Unknown group"))?;
            for (rank, photo) in group.result.photos.iter().enumerate() {
                flags.insert(photo.id, if rank == 0 { Flag::Pick } else { Flag::Reject });
            }
        }
        flags
    };
    let ids: Vec<PhotoId> = flags.keys().copied().collect();
    edit_photos(&app, &state, &ids, |id, e| e.flag = flags.get(&id).copied())
}

/// Keeps the chosen photo of one group and rejects the others, as one undoable step: for
/// groups where the suggestion is too close to call, or the user prefers another shot.
#[tauri::command]
#[specta::specta]
pub fn keep_one(
    app: AppHandle,
    state: State<'_, AppState>,
    group: PhotoId,
    photo: PhotoId,
) -> Result<HistoryState, IpcError> {
    let ids: Vec<PhotoId> = {
        let analyses = state.analyses.lock().map_err(poisoned)?;
        let group = analyses
            .groups
            .iter()
            .find(|g| g.id == group)
            .ok_or_else(|| IpcError::new("culling.unknown_group", "Unknown group"))?;
        group.result.photos.iter().map(|p| p.id).collect()
    };
    if !ids.contains(&photo) {
        return Err(IpcError::new("photo.unknown", "Unknown photo"));
    }
    edit_photos(&app, &state, &ids, |id, e| {
        e.flag = Some(if id == photo {
            Flag::Pick
        } else {
            Flag::Reject
        });
    })
}
