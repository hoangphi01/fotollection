// SPDX-License-Identifier: GPL-3.0-only
//! The disk cache (DATABASE.md §8): its size for the settings, *Clear cache*, and a prune in
//! the background after start-up and after thumbnails are made, so it never grows past the
//! limit. Thumbnails and analyses come back by themselves when they are needed again.

// Tauri commands take `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::path::Path;

use fotollection_core::cache_limit;
use fotollection_core::settings::UiSettings;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

/// The cache's size, shown in the Settings window (FR-086): in all and by kind.
#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CacheUsage {
    /// Bytes (as f64: TypeScript has no u64).
    pub bytes: f64,
    pub files: u32,
    /// The limit the cache is kept under, in bytes (the user's choice, FR-086).
    pub limit_bytes: f64,
    pub thumbnails_bytes: f64,
    pub previews_bytes: f64,
    pub analyses_bytes: f64,
}

/// What the previews keep in memory, for the Settings window (FR-086, FR-091).
#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    /// The machine's RAM, in bytes.
    pub total: f64,
    /// What the previews may keep, in bytes: the chosen share of it.
    pub budget: f64,
    /// What they keep now, in bytes.
    pub held: f64,
}

fn dirs(state: &AppState) -> [&Path; 2] {
    [state.thumbs.dir(), state.analysis_cache.dir()]
}

fn ui(state: &AppState) -> UiSettings {
    state.ui.read().map(|u| u.clone()).unwrap_or_default()
}

#[allow(clippy::cast_precision_loss)]
fn to_ipc(kinds: cache_limit::Kinds, limit: u64) -> CacheUsage {
    let all = [kinds.thumbnails, kinds.previews, kinds.analyses];
    CacheUsage {
        bytes: all.iter().map(|u| u.bytes).sum::<u64>() as f64,
        files: u32::try_from(all.iter().map(|u| u.files).sum::<u64>()).unwrap_or(u32::MAX),
        limit_bytes: limit as f64,
        thumbnails_bytes: kinds.thumbnails.bytes as f64,
        previews_bytes: kinds.previews.bytes as f64,
        analyses_bytes: kinds.analyses.bytes as f64,
    }
}

/// How much the cache holds; about 0.2 s for 70,000 files (OPTIMIZATION.md §5).
#[tauri::command]
#[specta::specta]
pub async fn cache_usage(app: AppHandle) -> CacheUsage {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        to_ipc(
            cache_limit::usage_by_kind(&dirs(&state)),
            ui(&state).cache_limit_bytes(),
        )
    })
    .await
    .unwrap_or(to_ipc(
        cache_limit::Kinds::default(),
        UiSettings::default().cache_limit_bytes(),
    ))
}

/// *Clear cache*: deletes every thumbnail, standard preview and analysis; returns what is
/// left (nothing).
#[tauri::command]
#[specta::specta]
pub fn clear_cache(state: State<'_, AppState>) -> CacheUsage {
    cache_limit::clear(&dirs(&state));
    state.look_thumbs.clear();
    to_ipc(
        cache_limit::usage_by_kind(&dirs(&state)),
        ui(&state).cache_limit_bytes(),
    )
}

/// This computer, as the Settings window's *For this computer* shows it (FR-086).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MachineInfo {
    /// The processor's name, or `None` when the system does not say.
    pub cpu: Option<String>,
    pub cores: u32,
    /// The RAM, in bytes.
    pub memory: f64,
    /// Bytes free on the cache's disk, or `None` when the system does not say.
    pub free_disk: Option<f64>,
}

/// What this computer has, for *Auto* to explain its choice (FR-086); about 10 ms.
#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_precision_loss)]
pub async fn machine_info(app: AppHandle) -> MachineInfo {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        MachineInfo {
            cpu: fotollection_core::machine::cpu_name(),
            cores: fotollection_core::machine::cores(),
            memory: fotollection_core::preview::machine_memory() as f64,
            free_disk: free_disk(&state).map(|b| b as f64),
        }
    })
    .await
    .unwrap_or(MachineInfo {
        cpu: None,
        cores: 1,
        memory: 0.0,
        free_disk: None,
    })
}

/// Bytes free on the disk that holds the cache.
fn free_disk(state: &AppState) -> Option<u64> {
    fotollection_core::preview::free_disk(state.thumbs.dir()).or_else(|| {
        state
            .thumbs
            .dir()
            .parent()
            .and_then(fotollection_core::preview::free_disk)
    })
}

/// What the previews keep in memory against their budget (FR-086).
#[tauri::command]
#[specta::specta]
#[allow(clippy::cast_precision_loss)]
pub fn memory_info(state: State<'_, AppState>) -> MemoryInfo {
    let total = fotollection_core::preview::machine_memory();
    MemoryInfo {
        total: total as f64,
        budget: (total / 100 * u64::from(ui(&state).preview_memory_percent)) as f64,
        held: (state.previews.usage().total() + state.look_thumbs.bytes()) as f64,
    }
}

/// The memory and cache settings the Settings window's *Default* (`auto` false) or *Auto*
/// button would set, with every other setting as it is; nothing is saved (FR-086).
#[tauri::command]
#[specta::specta]
pub fn suggest_memory_settings(state: State<'_, AppState>, auto: bool) -> UiSettings {
    let current = ui(&state);
    if auto {
        let free = free_disk(&state).unwrap_or(0);
        current.auto_for(fotollection_core::preview::machine_memory(), free)
    } else {
        current.memory_defaults()
    }
}

/// Watches the machine's memory and gives the previews' back while it runs short (FR-091,
/// ADR-0041): every 5 s, under a tenth available, the previews keep a tenth of their budget
/// until memory comes back, then the user's share again (audit 2026-11-06: a single trim
/// let them fill up again at once). A plain thread for the app's life; it holds no lock
/// while it sleeps.
pub fn watch_memory(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let state = app.state::<AppState>();
            let full =
                fotollection_core::preview::machine_budget(ui(&state).preview_memory_percent);
            let freed = state
                .previews
                .follow_memory(full, fotollection_core::preview::available_memory_percent());
            // Coloured thumbnails are made again in a millisecond each: they go first.
            if freed > 0 {
                state.look_thumbs.clear();
                eprintln!(
                    "memory: short, the previews gave back {} MB",
                    freed / 1_000_000
                );
            }
        }
    });
}

/// Keeps the cache under its limit, on a thread of its own so nothing waits for it.
pub fn prune_in_background(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || prune(&app.state::<AppState>()));
}

/// Keeps the cache under its limit now, on the caller's thread (a plain one: it reads every
/// file's time, about 0.2 s for 70,000 files).
pub fn prune(state: &AppState) {
    let pruned = cache_limit::prune(&dirs(state), ui(state).cache_limit_bytes());
    if pruned.removed > 0 {
        eprintln!(
            "cache: removed {} files ({} MB), {} MB left",
            pruned.removed,
            pruned.freed_bytes / 1_000_000,
            pruned.left.bytes / 1_000_000
        );
    }
}
