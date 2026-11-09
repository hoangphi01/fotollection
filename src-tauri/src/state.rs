// SPDX-License-Identifier: GPL-3.0-only
//! Shared app state. Heavy work never runs on the UI thread (NFR-005).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

// Locks that heal after a panic (ADR-0048).
use fotollection_core::lock::{Mutex, RwLock};

use fotollection_core::brands;
use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{JpegCodec, default_codec};
use fotollection_core::culling::cache::AnalysisCache;
use fotollection_core::culling::{Group, PhotoAnalysis};
use fotollection_core::fonts::{self, LoadedFont, UserFont};
use fotollection_core::frame::{self, CameraProfile, FrameSettings, LogoKind};
use fotollection_core::geocode::PlaceFormat;
use fotollection_core::history::History;
use fotollection_core::library::{Library, PhotoId, PhotoRecord};
use fotollection_core::logo::{self, LogoImage};
use fotollection_core::plan::FrameRequest;
use fotollection_core::preview::PreviewCache;
use fotollection_core::project;
use fotollection_core::settings::{RenderSettings, UiSettings};
use fotollection_core::settings_store::{self, Loaded, StoredSettings};
use fotollection_core::thumbs::ThumbCache;
use serde::Serialize;
use specta::Type;

/// The place printed on one photo, written in `format` (FR-005): set by hand, borrowed, or
/// from its own GPS, with the user's names for renamed places (ADR-0027).
pub fn place_for(
    record: &PhotoRecord,
    names: &HashMap<u32, String>,
    format: PlaceFormat,
) -> Option<String> {
    fotollection_core::places::effective(record, names, format).map(|p| p.label)
}

/// Info-frame settings, camera profiles and decoded user logos (FR-040).
#[derive(Clone, Default)]
pub struct FrameState {
    pub settings: FrameSettings,
    pub cameras: Vec<CameraProfile>,
    pub logos: HashMap<u32, Arc<LogoImage>>,
    /// User fonts (FR-042): table rows plus parsed fonts, keyed by id.
    pub fonts: Vec<UserFont>,
    pub loaded_fonts: HashMap<u32, LoadedFont>,
}

impl FrameState {
    pub fn font(&self, id: Option<u32>) -> Option<LoadedFont> {
        id.and_then(|id| self.loaded_fonts.get(&id).cloned())
    }

    pub fn profile_for(&self, make: Option<&str>, model: Option<&str>) -> Option<&CameraProfile> {
        self.cameras.iter().find(|c| c.matches(make, model))
    }

    /// The frame for one photo, or `None` when frames are off. `shared` is the look every
    /// photo without its own has: its name, or the photo's own look's, fills the photo-style
    /// line (FR-040, FR-079), never *Original*'s.
    pub fn request(
        &self,
        record: &PhotoRecord,
        place: Option<&str>,
        shared: &fotollection_core::looks::LookSettings,
    ) -> Option<FrameRequest> {
        if !self.settings.enabled {
            return None;
        }
        let shot = &record.shot;
        let profile = self.profile_for(shot.make.as_deref(), shot.model.as_deref());
        let capture = record.stamp_date();
        let look = record.edits.look.as_ref().unwrap_or(shared);
        let look_name = look
            .name
            .as_deref()
            .filter(|_| look.base.as_deref() != Some("builtin:original") && look.table().is_some());
        let text = frame::build_text(shot, profile, &self.settings, capture, place, look_name);
        let dark = self.settings.color == frame::FrameColor::Black;
        let logo = profile.and_then(|p| match p.logo {
            LogoKind::Image => self.logos.get(&p.id).cloned(),
            // A bundled logo that fails to decode falls back to the typeset name.
            LogoKind::Brand => p
                .brand()
                .and_then(|b| brands::logo(b, dark, self.settings.logo_style).ok()),
            LogoKind::Text | LogoKind::None => None,
        });
        Some(FrameRequest {
            settings: self.settings.clone(),
            text,
            logo,
            font: self.font(self.settings.font),
        })
    }
}

/// Best-shot analyses of the current library and the groups built from them (FR-041).
#[derive(Default)]
pub struct Analyses {
    /// Bumps when the library is replaced: photo ids are reused, so a job started before
    /// must not store its results.
    pub generation: u32,
    pub by_photo: HashMap<PhotoId, PhotoAnalysis>,
    pub groups: Vec<Group>,
    /// Job id of the analysis in progress, if any.
    pub running: Option<u32>,
}

pub struct AppState {
    pub library: RwLock<Library>,
    /// Current settings and a revision that bumps on every change (for preview URLs).
    pub settings: RwLock<(RenderSettings, u32)>,
    pub frame: RwLock<FrameState>,
    /// Interface language and other per-machine UI preferences (FR-020).
    pub ui: RwLock<UiSettings>,
    /// Addresses last typed on postcards, newest first (FR-077); this machine's only.
    pub recent_addresses: Mutex<Vec<String>>,
    /// The looks saved with *Save as look* (FR-079), oldest first; this machine's only.
    pub saved_looks: Mutex<Vec<fotollection_core::look_library::SavedLook>>,
    /// Chosen through the Rust-side dialog only, never sent by the webview (SECURITY.md §2).
    pub jobs: Mutex<HashMap<u32, CancellationToken>>,
    pub next_job: AtomicU32,
    /// Bumped when another library replaces this one: background preview work for the old
    /// one stops (FR-091).
    pub library_generation: AtomicU64,
    /// The photos left to prefetch and its one worker (FR-091, `prefetch.rs`).
    pub prefetch: crate::prefetch::Prefetch,
    pub codec: Arc<dyn JpegCodec>,
    pub thumbs: ThumbCache,
    /// `analysis/` next to `thumbs/` in the cache dir (DATABASE.md §8).
    pub analysis_cache: AnalysisCache,
    pub analyses: Mutex<Analyses>,
    pub previews: PreviewCache,
    /// Thumbnails through each photo's colour, in memory (FR-114, ADR-0047), and the batch
    /// whose thumbnails are being made: a newer batch stops an older one's.
    pub look_thumbs: fotollection_core::look_thumbs::LookThumbs,
    pub look_warm: AtomicU64,
    /// Serves what the user is looking at right now (protocol requests).
    pub interactive_pool: rayon::ThreadPool,
    /// Import metadata reads and background T1 generation.
    pub background_pool: rayon::ThreadPool,
    /// `settings.json` and the `logos/` folder in the app config dir (DATABASE.md §6).
    pub store_path: PathBuf,
    pub logo_dir: PathBuf,
    pub font_dir: PathBuf,
    /// False when the file was written by a newer app: never overwrite it.
    pub store_writable: AtomicBool,
    store_lock: Mutex<()>,
    /// A setting changed since `settings.json` was last written: the autosave thread writes
    /// it within a quarter second, so a slider's drag is one write, not one per step (FR-079).
    pub settings_dirty: AtomicBool,
    /// Session project (FR-023): autosaved by a background thread when dirty.
    pub project_path: PathBuf,
    pub project_dirty: AtomicBool,
    /// The last session is back (or there was none): the start-up screen may go (FR-060).
    pub startup_done: AtomicBool,
    /// False when the project was written by a newer app: never overwrite it.
    pub project_writable: AtomicBool,
    pub history: Mutex<History>,
    /// The `.fotollection` document this session belongs to (FR-023).
    pub document: Mutex<Document>,
    /// The opened project and its folders that were not found yet (Relink dialog).
    pub pending: Mutex<Option<PendingRelink>>,
    /// Serializes opening, restoring and starting projects.
    pub document_lock: Mutex<()>,
    /// A `.fotollection` the OS asked us to open, waiting for the UI's go-ahead.
    pub requested_open: Mutex<Option<PathBuf>>,
    /// A photo the OS asked to show ("Open with", FR-054), until the UI takes it.
    pub requested_photo: Mutex<Option<crate::open_with::OpenPhoto>>,
    /// What start-up has to tell the user (a damaged settings file), taken once by the UI.
    pub startup_notices: Mutex<Vec<IpcError>>,
    /// The installer link found by the last "Check for updates" (FR-061).
    pub update_download: crate::updates::PendingDownload,
    /// The folder the last export wrote into (FR-093), for *Open folder*: kept in Rust, so the
    /// webview never names a path to open. One path; replaced by the next export.
    pub last_export: Mutex<Option<PathBuf>>,
    /// The settings *Copy settings* took (FR-097), until the next copy or another project;
    /// one photo's develop edits, a few hundred bytes.
    pub copied: Mutex<Option<crate::settings_copy::Copied>>,
}

/// The current document: none until the first Save As or Open.
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub path: Option<PathBuf>,
    /// A name given before the first save (double-click on "Untitled"); Save As offers it.
    pub pending_name: Option<String>,
    /// Changes not saved into the document yet.
    pub edited: bool,
    /// Bumps whenever another project replaces the library, so the UI drops cached
    /// thumbnails of reused photo ids.
    pub generation: u32,
}

/// Folders of an opened project that could not be found on this machine.
#[derive(Debug, Clone)]
pub struct PendingRelink {
    pub file: project::ProjectFile,
    pub missing: Vec<project::ResolvedFolder>,
}

/// The machine's settings, whether they may be saved, and what the user is told once the
/// window is up (ADR-0048): a damaged file is recovered as far as it can be, never in silence.
fn load_settings(store_path: &Path) -> (StoredSettings, bool, Vec<IpcError>) {
    let mut notices = Vec::new();
    let (stored, writable) = match settings_store::load_machine(store_path) {
        Loaded::Ok(stored) => (stored, true),
        Loaded::Missing => (StoredSettings::default(), true),
        Loaded::Recovered(recovered) => {
            let _ = settings_store::keep_damaged(store_path);
            notices.push(if recovered.from_backup {
                IpcError::new(
                    "settings.restored",
                    "settings restored from settings.json.bak",
                )
            } else {
                IpcError::new(
                    "settings.recovered",
                    format!(
                        "parts back to their defaults: {}",
                        recovered.reset.join(", ")
                    ),
                )
            });
            (recovered.settings, true)
        }
        Loaded::NewerVersion(v) => {
            notices.push(IpcError::new(
                "settings.newer",
                format!("settings.json has schema {v}; defaults used, the file left alone"),
            ));
            (StoredSettings::default(), false)
        }
        Loaded::Corrupt(reason) => {
            let _ = settings_store::quarantine(store_path);
            notices.push(IpcError::new(
                "settings.corrupt",
                format!("settings.json unreadable ({reason}); kept as settings.json.corrupt"),
            ));
            (StoredSettings::default(), true)
        }
    };
    (stored, writable, notices)
}

impl AppState {
    pub fn new(
        thumb_dir: PathBuf,
        config_dir: &std::path::Path,
        project_path: PathBuf,
    ) -> Result<Self, rayon::ThreadPoolBuildError> {
        let store_path = config_dir.join("settings.json");
        let logo_dir = config_dir.join("logos");
        let font_dir = config_dir.join("fonts");
        // Imported LUTs, by content hash (FR-070).
        fotollection_core::lut::set_dir(config_dir.join("looks"));
        let (stored, writable, notices) = load_settings(&store_path);
        let logos = stored
            .cameras
            .iter()
            .filter(|c| c.logo == LogoKind::Image)
            .filter_map(|c| {
                let bytes = std::fs::read(logo_dir.join(format!("{}.png", c.id))).ok()?;
                // A logo whose decoder panics is skipped, never the start (ADR-0048).
                let image = fotollection_core::supervise::contained(|| logo::decode_png(&bytes))?;
                Some((c.id, Arc::new(image.ok()?)))
            })
            .collect();

        let loaded_fonts = stored
            .fonts
            .iter()
            .filter_map(|f| {
                let path = font_dir.join(format!("{}.{}", f.id, f.format.extension()));
                let bytes = std::fs::read(path).ok()?;
                let (_, font) =
                    fotollection_core::supervise::contained(|| fonts::load(bytes))?.ok()?;
                Some((f.id, LoadedFont { id: f.id, font }))
            })
            .collect();

        Ok(Self {
            library: RwLock::new(Library::new()),
            settings: RwLock::new((stored.render, 1)),
            frame: RwLock::new(FrameState {
                settings: stored.frame,
                cameras: stored.cameras,
                logos,
                fonts: stored.fonts,
                loaded_fonts,
            }),
            ui: RwLock::new(stored.ui.clone()),
            recent_addresses: Mutex::new(stored.recent_addresses),
            saved_looks: Mutex::new(stored.looks),
            jobs: Mutex::new(HashMap::new()),
            next_job: AtomicU32::new(1),
            library_generation: AtomicU64::new(0),
            prefetch: crate::prefetch::Prefetch::default(),
            codec: default_codec(),
            analysis_cache: AnalysisCache::new(
                thumb_dir
                    .parent()
                    .map_or_else(|| thumb_dir.join("analysis"), |p| p.join("analysis")),
            ),
            analyses: Mutex::new(Analyses::default()),
            previews: PreviewCache::with(
                fotollection_core::preview::machine_budget(stored.ui.preview_memory_percent),
                Some(thumb_dir.clone()),
            ),
            thumbs: ThumbCache::new(thumb_dir),
            look_thumbs: fotollection_core::look_thumbs::LookThumbs::default(),
            look_warm: AtomicU64::new(0),
            // A panic in a job stops there, never the app (ADR-0048, `pool::contained`).
            interactive_pool: fotollection_core::pool::contained(Some(2), "ui")?,
            background_pool: fotollection_core::pool::contained(Some(2), "bg")?,
            store_path,
            logo_dir,
            font_dir,
            store_writable: AtomicBool::new(writable),
            store_lock: Mutex::new(()),
            settings_dirty: AtomicBool::new(false),
            project_path,
            project_dirty: AtomicBool::new(false),
            startup_done: AtomicBool::new(false),
            project_writable: AtomicBool::new(true),
            history: Mutex::new(History::default()),
            document: Mutex::new(Document::default()),
            pending: Mutex::new(None),
            document_lock: Mutex::new(()),
            requested_open: Mutex::new(None),
            requested_photo: Mutex::new(None),
            startup_notices: Mutex::new(notices),
            update_download: crate::updates::PendingDownload::default(),
            last_export: Mutex::new(None),
            copied: Mutex::new(None),
        })
    }

    /// Bumps the settings revision so previews refresh. Returns the new revision.
    pub fn bump_revision(&self) -> u32 {
        self.settings.write().map_or(0, |mut s| {
            s.1 = s.1.wrapping_add(1);
            s.1
        })
    }

    /// A print setting changed: mark the document edited, and `settings.json` for the
    /// autosave thread to write (within 250 ms; at once on quit, `flush_settings`). Writing
    /// here, on the main thread, cost an fsync (about 4 ms) on every step of a drag.
    pub fn persist(&self) {
        self.mark_edited();
        self.settings_dirty.store(true, Ordering::Relaxed);
    }

    /// Writes `settings.json` when a change is waiting.
    pub fn flush_settings(&self) {
        if self.settings_dirty.swap(false, Ordering::Relaxed) {
            self.save_settings();
        }
    }

    /// Marks the document as having unsaved changes (FR-023).
    pub fn mark_edited(&self) {
        if let Ok(mut doc) = self.document.lock() {
            doc.edited = true;
        }
    }

    /// Writes `settings.json` atomically (DATABASE.md §6). Errors are logged, not fatal.
    pub fn save_settings(&self) {
        if !self.store_writable.load(Ordering::Relaxed) {
            return;
        }
        // Read on its own first: no lock is held while another is taken.
        let Ok(recent_addresses) = self.recent_addresses.lock().map(|r| r.clone()) else {
            return;
        };
        let Ok(looks) = self.saved_looks.lock().map(|l| l.clone()) else {
            return;
        };
        let (Ok(render), Ok(frame), Ok(ui)) =
            (self.settings.read(), self.frame.read(), self.ui.read())
        else {
            return;
        };
        let stored = StoredSettings {
            render: render.0.clone(),
            frame: frame.settings.clone(),
            cameras: frame.cameras.clone(),
            fonts: frame.fonts.clone(),
            ui: ui.clone(),
            recent_addresses,
            looks,
        };
        drop((render, frame, ui));
        let _guard = self.store_lock.lock();
        if let Err(e) = settings_store::save(&self.store_path, &stored) {
            eprintln!("could not save settings: {e}");
        }
    }

    /// Writes the session project now (atomic, DATABASE.md §4.4).
    pub fn save_project(&self) {
        if !self.project_writable.load(Ordering::Relaxed) {
            return;
        }
        let Some(mut file) = self.library.read().ok().map(|l| project::snapshot(&l)) else {
            return;
        };
        file.document = self.document.lock().ok().and_then(|d| {
            d.path.as_ref().map(|p| project::DocumentRef {
                path: p.display().to_string(),
                edited: d.edited,
            })
        });
        if let Err(e) = project::save(&self.project_path, &file) {
            eprintln!("could not save the project: {e}");
            self.project_dirty.store(true, Ordering::Relaxed);
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

/// Error shape for every IPC command: a stable code the interface translates, its permanent
/// `ERR-` code from `docs/ERROR_CONTROL.md` (shown with the message, the user's clue for a
/// report), and a human-readable message.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: String,
    pub err: String,
    pub message: String,
}

impl IpcError {
    /// The error, written to the error log as it is made (`error_log`), except a cancel,
    /// which the user asked for.
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        let message = message.into();
        if !code.ends_with(".cancelled") {
            fotollection_core::error_log::record(code, &message);
        }
        Self {
            code: code.to_owned(),
            err: fotollection_core::errors::err_code(code).to_owned(),
            message,
        }
    }

    /// A photo's error, logged with what the file is beside the message (FR-071): the log is
    /// the only clue in a bug report, the interface shows the message alone.
    pub fn for_photo(
        code: &str,
        message: impl Into<String>,
        facts: &fotollection_core::facts::ImageFacts,
    ) -> Self {
        let message = message.into();
        fotollection_core::error_log::record(code, &format!("{message} [{}]", facts.short()));
        Self {
            code: code.to_owned(),
            err: fotollection_core::errors::err_code(code).to_owned(),
            message,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}
