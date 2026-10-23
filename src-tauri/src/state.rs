// SPDX-License-Identifier: GPL-3.0-only
//! Shared app state. Heavy work never runs on the UI thread (NFR-005).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{JpegCodec, default_codec};
use fotollection_core::frame::{self, CameraProfile, FrameSettings, LogoKind};
use fotollection_core::library::{Library, PhotoRecord, flags};
use fotollection_core::logo::{self, LogoImage};
use fotollection_core::plan::FrameRequest;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;
use fotollection_core::settings_store::{self, Loaded, StoredSettings};
use fotollection_core::thumbs::ThumbCache;
use serde::Serialize;
use specta::Type;

/// Info-frame settings, camera profiles and decoded user logos (FR-040).
#[derive(Clone, Default)]
pub struct FrameState {
    pub settings: FrameSettings,
    pub cameras: Vec<CameraProfile>,
    pub logos: HashMap<u32, Arc<LogoImage>>,
}

impl FrameState {
    pub fn profile_for(&self, make: Option<&str>, model: Option<&str>) -> Option<&CameraProfile> {
        self.cameras.iter().find(|c| c.matches(make, model))
    }

    /// The frame for one photo, or `None` when frames are off.
    pub fn request(&self, record: &PhotoRecord) -> Option<FrameRequest> {
        if !self.settings.enabled {
            return None;
        }
        let shot = &record.shot;
        let profile = self.profile_for(shot.make.as_deref(), shot.model.as_deref());
        let capture = (record.flags & flags::DATE_FROM_FILE == 0).then_some(record.capture);
        let text = frame::build_text(shot, profile, &self.settings, capture, None);
        let logo = profile
            .filter(|p| p.logo == LogoKind::Image)
            .and_then(|p| self.logos.get(&p.id).cloned());
        Some(FrameRequest {
            settings: self.settings.clone(),
            text,
            logo,
        })
    }
}

pub struct AppState {
    pub library: RwLock<Library>,
    /// Current settings and a revision that bumps on every change (for preview URLs).
    pub settings: RwLock<(RenderSettings, u32)>,
    pub frame: RwLock<FrameState>,
    /// Chosen through the Rust-side dialog only, never sent by the webview (SECURITY.md §2).
    pub export_destination: Mutex<Option<PathBuf>>,
    pub jobs: Mutex<HashMap<u32, CancellationToken>>,
    pub next_job: AtomicU32,
    pub codec: Arc<dyn JpegCodec>,
    pub thumbs: ThumbCache,
    pub previews: PreviewCache,
    /// Serves what the user is looking at right now (protocol requests).
    pub interactive_pool: rayon::ThreadPool,
    /// Import metadata reads and background T1 generation.
    pub background_pool: rayon::ThreadPool,
    /// `settings.json` and the `logos/` folder in the app config dir (DATABASE.md §6).
    pub store_path: PathBuf,
    pub logo_dir: PathBuf,
    /// False when the file was written by a newer app: never overwrite it.
    pub store_writable: AtomicBool,
    store_lock: Mutex<()>,
}

impl AppState {
    pub fn new(
        thumb_dir: PathBuf,
        config_dir: &std::path::Path,
    ) -> Result<Self, rayon::ThreadPoolBuildError> {
        let store_path = config_dir.join("settings.json");
        let logo_dir = config_dir.join("logos");
        let (stored, writable) = match settings_store::load(&store_path) {
            Loaded::Ok(stored) => (stored, true),
            Loaded::Missing => (StoredSettings::default(), true),
            Loaded::NewerVersion(v) => {
                eprintln!("settings.json has schema {v}; using defaults without saving");
                (StoredSettings::default(), false)
            }
            Loaded::Corrupt(reason) => {
                eprintln!("settings.json is invalid ({reason}); moved aside, using defaults");
                let _ = settings_store::quarantine(&store_path);
                (StoredSettings::default(), true)
            }
        };
        let logos = stored
            .cameras
            .iter()
            .filter(|c| c.logo == LogoKind::Image)
            .filter_map(|c| {
                let bytes = std::fs::read(logo_dir.join(format!("{}.png", c.id))).ok()?;
                Some((c.id, Arc::new(logo::decode_png(&bytes).ok()?)))
            })
            .collect();

        Ok(Self {
            library: RwLock::new(Library::new()),
            settings: RwLock::new((stored.render, 1)),
            frame: RwLock::new(FrameState {
                settings: stored.frame,
                cameras: stored.cameras,
                logos,
            }),
            export_destination: Mutex::new(None),
            jobs: Mutex::new(HashMap::new()),
            next_job: AtomicU32::new(1),
            codec: default_codec(),
            thumbs: ThumbCache::new(thumb_dir),
            previews: PreviewCache::new(),
            interactive_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("fotollection-ui-{i}"))
                .build()?,
            background_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("fotollection-bg-{i}"))
                .build()?,
            store_path,
            logo_dir,
            store_writable: AtomicBool::new(writable),
            store_lock: Mutex::new(()),
        })
    }

    /// Bumps the settings revision so previews refresh. Returns the new revision.
    pub fn bump_revision(&self) -> u32 {
        self.settings.write().map_or(0, |mut s| {
            s.1 = s.1.wrapping_add(1);
            s.1
        })
    }

    /// Writes `settings.json` atomically (DATABASE.md §6). Errors are logged, not fatal.
    pub fn persist(&self) {
        if !self.store_writable.load(Ordering::Relaxed) {
            return;
        }
        let (Ok(render), Ok(frame)) = (self.settings.read(), self.frame.read()) else {
            return;
        };
        let stored = StoredSettings {
            render: render.0.clone(),
            frame: frame.settings.clone(),
            cameras: frame.cameras.clone(),
        };
        drop((render, frame));
        let _guard = self.store_lock.lock();
        if let Err(e) = settings_store::save(&self.store_path, &stored) {
            eprintln!("could not save settings: {e}");
        }
    }

    /// Ensures every camera in the library has a profile row. Returns true if any were added.
    pub fn ensure_camera_profiles(&self) -> bool {
        let cameras: Vec<_> = match self.library.read() {
            Ok(lib) => lib.cameras().to_vec(),
            Err(_) => return false,
        };
        let Ok(mut frame) = self.frame.write() else {
            return false;
        };
        let mut added = false;
        for (make, model) in cameras {
            if frame
                .profile_for(make.as_deref(), model.as_deref())
                .is_none()
            {
                let id = frame.cameras.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                frame.cameras.push(CameraProfile::new(id, make, model));
                added = true;
            }
        }
        added
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
