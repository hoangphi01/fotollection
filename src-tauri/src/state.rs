// SPDX-License-Identifier: GPL-3.0-only
//! Shared app state. Heavy work never runs on the UI thread (NFR-005).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex, RwLock};

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{JpegCodec, default_codec};
use fotollection_core::library::Library;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::ThumbCache;
use serde::Serialize;
use specta::Type;

pub struct AppState {
    pub library: RwLock<Library>,
    /// Current settings and a revision that bumps on every change (for preview URLs).
    pub settings: RwLock<(RenderSettings, u32)>,
    /// Chosen through the Rust-side dialog only, never sent by the webview (SECURITY.md §2).
    pub export_destination: Mutex<Option<PathBuf>>,
    pub jobs: Mutex<HashMap<u32, CancellationToken>>,
    pub next_job: AtomicU32,
    pub codec: Arc<dyn JpegCodec>,
    pub thumbs: ThumbCache,
    pub previews: PreviewCache,
    /// Low-priority pool for thumbnails and protocol requests, separate from export.
    pub io_pool: rayon::ThreadPool,
}

impl AppState {
    pub fn new(thumb_dir: PathBuf) -> Result<Self, rayon::ThreadPoolBuildError> {
        Ok(Self {
            library: RwLock::new(Library::new()),
            settings: RwLock::new((RenderSettings::default(), 1)),
            export_destination: Mutex::new(None),
            jobs: Mutex::new(HashMap::new()),
            next_job: AtomicU32::new(1),
            codec: default_codec(),
            thumbs: ThumbCache::new(thumb_dir),
            previews: PreviewCache::new(),
            io_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(3)
                .thread_name(|i| format!("fotollection-io-{i}"))
                .build()?,
        })
    }
}

/// Error shape for every IPC command: a stable code plus a human-readable message.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: String,
    pub message: String,
}

impl IpcError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}
