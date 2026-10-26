// SPDX-License-Identifier: GPL-3.0-only
//! The disk cache (DATABASE.md §8): its size for the settings, *Clear cache*, and a prune in
//! the background after start-up and after thumbnails are made, so it never grows past the
//! limit. Thumbnails and analyses come back by themselves when they are needed again.

// Tauri commands take `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::path::Path;

use fotollection_core::cache_limit;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

/// The cache's size, shown under *Clear cache*.
#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CacheUsage {
    /// Bytes (as f64: TypeScript has no u64).
    pub bytes: f64,
    pub files: u32,
    /// The limit the cache is kept under, in bytes.
    pub limit_bytes: f64,
}

fn dirs(state: &AppState) -> [&Path; 2] {
    [state.thumbs.dir(), state.analysis_cache.dir()]
}

#[allow(clippy::cast_precision_loss)]
fn to_ipc(usage: cache_limit::Usage) -> CacheUsage {
    CacheUsage {
        bytes: usage.bytes as f64,
        files: u32::try_from(usage.files).unwrap_or(u32::MAX),
        limit_bytes: cache_limit::LIMIT_BYTES as f64,
    }
}

/// How much the cache holds; about 0.2 s for 70,000 files (OPTIMIZATION.md §5).
#[tauri::command]
#[specta::specta]
pub async fn cache_usage(app: AppHandle) -> CacheUsage {
    tauri::async_runtime::spawn_blocking(move || {
        to_ipc(cache_limit::usage(&dirs(&app.state::<AppState>())))
    })
    .await
    .unwrap_or(to_ipc(cache_limit::Usage::default()))
}

/// *Clear cache*: deletes every thumbnail and analysis; returns what is left (nothing).
#[tauri::command]
#[specta::specta]
pub fn clear_cache(state: State<'_, AppState>) -> CacheUsage {
    cache_limit::clear(&dirs(&state));
    to_ipc(cache_limit::usage(&dirs(&state)))
}

/// Keeps the cache under its limit, on the background pool so nothing waits for it.
pub fn prune_in_background(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        state.background_pool.install(|| {
            let pruned = cache_limit::prune(&dirs(&state), cache_limit::LIMIT_BYTES);
            if pruned.removed > 0 {
                eprintln!(
                    "cache: removed {} files ({} MB), {} MB left",
                    pruned.removed,
                    pruned.freed_bytes / 1_000_000,
                    pruned.left.bytes / 1_000_000
                );
            }
        });
    });
}
