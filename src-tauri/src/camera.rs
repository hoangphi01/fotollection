// SPDX-License-Identifier: GPL-3.0-only
//! Tethered capture from a camera plugged in by cable (FR-062, FR-063, ADR-0028).
//!
//! **Connect camera** finds the body, runs the maker's handshake and starts a session on
//! a thread of its own. That thread owns the camera, so no OS object crosses threads.
//! While connected it does three things:
//! - reads live-view frames as fast as the camera makes them (kept in memory only, served
//!   by the `fotothumb` protocol);
//! - reads the settings about every 300 ms;
//! - runs the UI's jobs one at a time: capture, a step of ISO or shutter speed, the file
//!   format.
//!
//! Shots are written atomically into a session folder in the app's data directory, which
//! becomes a tab. **Save all** moves them to a folder the user picks, keeping every edit;
//! **Delete photos** removes them. The camera logic is written against [`CameraLink`], so
//! it is tested with a scripted camera.
#![allow(clippy::needless_pass_by_value)]
// The session loop and the Sony driver run on Windows (WPD) for now; on macOS they are
// exercised by the tests until the ImageCaptureCore link lands (ADR-0028, STATUS).
#![cfg_attr(not(windows), allow(dead_code))]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fotollection_core::library::PhotoId;
use fotollection_core::project::{self, ProjectFile};
use fotollection_oscamera::ptp::{self, ObjectInfo, op};
use fotollection_oscamera::vendor::sony::{
    self, HANDSHAKE, SHUTTER_PRESS, SHUTTER_STEP_GAP, Settings, control_data, file_format,
    live_view_image, object_format, photo_ready, prop, set_value_data, shutter_speed,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use crate::document::{DocumentState, poisoned, rebuild_library};
use crate::state::{AppState, IpcError};

/// How long a press may take to become a photo before it counts as "did not shoot".
const SHOT_DEADLINE: Duration = Duration::from_secs(6);
/// How often the settings are read while connected.
const SETTINGS_EVERY: Duration = Duration::from_millis(300);
/// Failed calls in a row before the camera counts as gone.
const FAILURES_BEFORE_GONE: u32 = 5;
/// The longest file name taken from the camera.
const MAX_NAME: usize = 64;

// ---------------------------------------------------------------------------------------
// Types shared with the UI

/// A problem to show, with a stable code the UI translates (`errors.camera.*`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraIssue {
    pub code: String,
    pub message: String,
}

impl CameraIssue {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
        }
    }
}

impl From<CameraIssue> for IpcError {
    fn from(i: CameraIssue) -> Self {
        IpcError::new(&i.code, i.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CaptureFormat {
    Jpeg,
    Raw,
    /// A value we have not named (RAW+JPEG, another quality); shown as is.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExposureSetting {
    Iso,
    Shutter,
}

/// A shutter speed in seconds, as numerator / denominator (1/250 is `{ 1, 250 }`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ShutterSpeed {
    pub numerator: u16,
    pub denominator: u16,
}

/// What the capture screen shows. `None` when the camera has not filled a value yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraSettings {
    pub iso: Option<u32>,
    pub iso_auto: bool,
    pub shutter: Option<ShutterSpeed>,
    /// f-number × 100 (200 is f/2.0).
    pub aperture: Option<u16>,
    pub format: Option<CaptureFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraInfo {
    pub name: String,
    pub settings: CameraSettings,
}

// ---------------------------------------------------------------------------------------
// Events (core → UI)

/// Closing the app was held back because shots exist only in Fotollection's data: a camera
/// is connected, or session tabs are not saved yet (ADR-0028). The UI asks Save all, Delete
/// photos or Cancel, then calls `quit_app`.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct QuitBlocked {
    /// The session tabs not saved yet.
    pub sessions: Vec<u16>,
    pub connected: bool,
}

/// The camera connected or went away; `issue` says why it stopped, when it did.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CameraStatus {
    pub connected: bool,
    pub name: String,
    pub issue: Option<CameraIssue>,
}

/// A new live-view frame is at `fotothumb://localhost/camera/live?seq={seq}`.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CameraFrame {
    pub seq: u32,
}

/// The camera's settings changed.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CameraSettingsChanged {
    pub settings: CameraSettings,
}

/// A shot is in the session tab. `ms` runs from the press to now (NFR-018); 0 when the
/// camera's own shutter took it.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CameraShot {
    pub id: PhotoId,
    pub folder: u16,
    pub ms: u32,
}

/// A press did not give a photo.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CameraShotFailed {
    pub issue: CameraIssue,
}

// ---------------------------------------------------------------------------------------
// The link to a camera, and Sony's session on it

/// What one PTP operation brought back.
#[derive(Debug, Clone, Default)]
pub(crate) struct Answer {
    pub code: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkError {
    Timeout,
    Failed(String),
}

/// One camera, as the session needs it: an operation, and whether it was unplugged.
pub(crate) trait CameraLink {
    fn send(&mut self, code: u16, params: &[u32], out: Option<&[u8]>) -> Result<Answer, LinkError>;
    fn unplugged(&mut self) -> bool;
}

/// A photo read from the camera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Shot {
    pub name: String,
    pub format: CaptureFormat,
    pub bytes: Vec<u8>,
}

/// A press could be a photo; the caller waits for it with [`Sony::wait_and_fetch`].
pub(crate) struct Sony<L> {
    link: L,
    props: Vec<sony::Property>,
}

const ACCESS_DENIED: u16 = 0x200F;

impl<L: CameraLink> Sony<L> {
    pub fn new(link: L) -> Self {
        Self {
            link,
            props: Vec::new(),
        }
    }

    pub fn unplugged(&mut self) -> bool {
        self.link.unplugged()
    }

    fn call(
        &mut self,
        code: u16,
        params: &[u32],
        out: Option<&[u8]>,
    ) -> Result<Answer, CameraIssue> {
        let answer = self.link.send(code, params, out).map_err(|e| match e {
            LinkError::Timeout => CameraIssue::new("camera.timeout", "The camera did not answer"),
            LinkError::Failed(m) => CameraIssue::new("camera.failed", m),
        })?;
        if answer.code == ptp::rc::OK {
            Ok(answer)
        } else {
            Err(CameraIssue::new(
                "camera.refused",
                format!(
                    "The camera refused operation 0x{code:04X} (0x{:04X})",
                    answer.code
                ),
            ))
        }
    }

    /// Unlocks the body for the computer ([`HANDSHAKE`]). A refusal usually means another
    /// app, such as Imaging Edge Remote, holds the camera.
    pub fn handshake(&mut self) -> Result<(), CameraIssue> {
        for (code, params) in HANDSHAKE {
            self.call(code, params, None).map_err(|e| {
                CameraIssue::new(
                    "camera.busy",
                    format!("Handshake step 0x{code:04X}: {}", e.message),
                )
            })?;
        }
        Ok(())
    }

    /// Reads every setting again.
    pub fn refresh(&mut self) -> Result<CameraSettings, CameraIssue> {
        let answer = self.call(sony::op::ALL_SETTINGS, &[], None)?;
        self.props = Settings::parse(&answer.data)
            .map_err(|e| CameraIssue::new("camera.bad_answer", e.to_string()))?;
        Ok(self.settings())
    }

    fn current(&self, code: u16) -> Option<i128> {
        self.props
            .iter()
            .find(|p| p.code == code)
            .and_then(|p| p.current.as_int())
    }

    pub fn settings(&self) -> CameraSettings {
        let iso = self.current(prop::ISO).and_then(|v| u32::try_from(v).ok());
        let iso_auto = iso == Some(0x00FF_FFFF);
        CameraSettings {
            iso: iso.filter(|&v| v > 0 && !iso_auto),
            iso_auto,
            shutter: self
                .current(prop::SHUTTER_SPEED)
                .and_then(|v| u32::try_from(v).ok())
                .and_then(shutter_speed)
                .filter(|&(n, d)| n != 0xFFFF && d != 0xFFFF)
                .map(|(numerator, denominator)| ShutterSpeed {
                    numerator,
                    denominator,
                }),
            aperture: self
                .current(prop::F_NUMBER)
                .and_then(|v| u16::try_from(v).ok())
                .filter(|&v| v > 0 && v != 0xFFFF),
            format: self
                .current(prop::FILE_FORMAT)
                .and_then(|v| u16::try_from(v).ok())
                .map(|v| match v {
                    file_format::JPEG => CaptureFormat::Jpeg,
                    file_format::RAW => CaptureFormat::Raw,
                    _ => CaptureFormat::Other,
                }),
        }
    }

    /// Whether the last settings read says a photo is waiting.
    pub fn photo_waiting(&self) -> bool {
        self.current(prop::SHOT_STATE).is_some_and(photo_ready)
    }

    /// Presses the shutter as a person would: half, full, then release, with time for
    /// autofocus between steps.
    pub fn press(&mut self, sleep: &mut dyn FnMut(Duration)) -> Result<(), CameraIssue> {
        for (i, (code, value)) in SHUTTER_PRESS.into_iter().enumerate() {
            if i > 0 {
                sleep(SHUTTER_STEP_GAP);
            }
            self.call(
                sony::op::CONTROL,
                &[u32::from(code)],
                Some(&control_data(code, value)),
            )?;
        }
        Ok(())
    }

    /// Waits until the camera has a photo, then reads it. Through WPD the "new photo" event
    /// does not arrive, so this reads the shot state (sony.md).
    pub fn wait_and_fetch(
        &mut self,
        deadline: Duration,
        sleep: &mut dyn FnMut(Duration),
    ) -> Result<Shot, CameraIssue> {
        let start = Instant::now();
        loop {
            self.refresh()?;
            if self.photo_waiting() {
                return self.fetch();
            }
            if start.elapsed() >= deadline {
                return Err(CameraIssue::new(
                    "camera.no_shot",
                    "The camera did not take a photo: it may not have found focus",
                ));
            }
            sleep(Duration::from_millis(100));
        }
    }

    /// Reads the waiting photo.
    pub fn fetch(&mut self) -> Result<Shot, CameraIssue> {
        let info = self.call(op::GET_OBJECT_INFO, &[sony::handle::PHOTO], None)?;
        let info = ObjectInfo::parse(&info.data)
            .map_err(|e| CameraIssue::new("camera.bad_answer", e.to_string()))?;
        let bytes = self
            .call(op::GET_OBJECT, &[sony::handle::PHOTO], None)?
            .data;
        if bytes.len() != info.size as usize {
            return Err(CameraIssue::new(
                "camera.bad_answer",
                format!(
                    "The photo arrived incomplete ({} of {} bytes)",
                    bytes.len(),
                    info.size
                ),
            ));
        }
        let format = match info.format {
            object_format::JPEG => CaptureFormat::Jpeg,
            object_format::RAW => CaptureFormat::Raw,
            _ => CaptureFormat::Other,
        };
        Ok(Shot {
            name: safe_name(&info.filename, format),
            format,
            bytes,
        })
    }

    /// The current live-view frame, or `None` when the next one is not ready yet.
    pub fn live(&mut self) -> Result<Option<Vec<u8>>, CameraIssue> {
        let _ = self
            .link
            .send(op::GET_OBJECT_INFO, &[sony::handle::LIVE_VIEW], None);
        let answer = self
            .link
            .send(op::GET_OBJECT, &[sony::handle::LIVE_VIEW], None)
            .map_err(|e| CameraIssue::new("camera.failed", format!("{e:?}")))?;
        match answer.code {
            ptp::rc::OK => live_view_image(&answer.data)
                .map(|image| Some(image.to_vec()))
                .map_err(|e| CameraIssue::new("camera.bad_answer", e.to_string())),
            ACCESS_DENIED | ptp::rc::DEVICE_BUSY => Ok(None),
            code => Err(CameraIssue::new(
                "camera.refused",
                format!("Live view refused (0x{code:04X})"),
            )),
        }
    }

    pub fn step(&mut self, setting: ExposureSetting, delta: i8) -> Result<(), CameraIssue> {
        let code = match setting {
            ExposureSetting::Iso => prop::ISO,
            ExposureSetting::Shutter => prop::SHUTTER_SPEED,
        };
        let value = i16::from(delta.signum());
        for _ in 0..delta.unsigned_abs() {
            self.call(
                sony::op::CONTROL,
                &[u32::from(code)],
                Some(&control_data(code, value)),
            )?;
        }
        Ok(())
    }

    pub fn set_format(&mut self, format: CaptureFormat) -> Result<(), CameraIssue> {
        let value = match format {
            CaptureFormat::Jpeg => file_format::JPEG,
            CaptureFormat::Raw => file_format::RAW,
            CaptureFormat::Other => return Ok(()),
        };
        self.call(
            sony::op::SET_VALUE,
            &[u32::from(prop::FILE_FORMAT)],
            Some(&set_value_data(value)),
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------
// Files

/// A file name from the camera made safe to write: only its last part, plain characters,
/// at most [`MAX_NAME`] long, with the extension its format calls for.
fn safe_name(name: &str, format: CaptureFormat) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let clean: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ' '))
        .take(MAX_NAME)
        .collect();
    let clean = clean.trim_matches(|c| c == '.' || c == ' ').to_owned();
    let ext = match format {
        CaptureFormat::Jpeg => "JPG",
        CaptureFormat::Raw => "ARW",
        CaptureFormat::Other => "bin",
    };
    let has_ext = Path::new(&clean)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext));
    match (clean.is_empty(), has_ext) {
        (true, _) => format!("shot.{ext}"),
        (false, true) => clean,
        (false, false) => format!("{clean}.{ext}"),
    }
}

/// `dir/name`, or `dir/stem (2).ext`, `(3)`… when the name is taken: nothing is overwritten.
fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = path.extension().and_then(|e| e.to_str());
    (2..10_000u32)
        .map(|n| match ext {
            Some(ext) => dir.join(format!("{stem} ({n}).{ext}")),
            None => dir.join(format!("{stem} ({n})")),
        })
        .find(|p| !p.exists())
        .unwrap_or(first)
}

/// Writes a shot as `name.part`, then renames it, so a half-written file never shows.
fn write_shot(dir: &Path, shot: &Shot) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = unique_path(dir, &shot.name);
    let mut part = path.clone().into_os_string();
    part.push(".part");
    let part = PathBuf::from(part);
    std::fs::write(&part, &shot.bytes)?;
    std::fs::rename(&part, &path)?;
    Ok(path)
}

/// Moves one file, across volumes by copying, checking the size, and only then removing.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    let copied = std::fs::copy(from, to)?;
    if copied != std::fs::metadata(from)?.len() {
        let _ = std::fs::remove_file(to);
        return Err(std::io::Error::other(format!(
            "{} was not copied whole",
            from.display()
        )));
    }
    std::fs::remove_file(from)
}

/// Moves every file of `src` into `dest` without overwriting any. Returns the files that
/// got a new name, as (old, new). `src` is removed when it ends up empty. On an error the
/// files already moved go back, so the session tab is never left half moved.
fn move_all(src: &Path, dest: &Path) -> std::io::Result<Vec<(String, String)>> {
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let result = move_each(src, dest, &mut moved);
    if result.is_err() {
        for (from, to) in moved.iter().rev() {
            let _ = move_file(to, from);
        }
    } else {
        let _ = std::fs::remove_dir(src);
    }
    result
}

fn move_each(
    src: &Path,
    dest: &Path,
    moved: &mut Vec<(PathBuf, PathBuf)>,
) -> std::io::Result<Vec<(String, String)>> {
    std::fs::create_dir_all(dest)?;
    let mut renamed = Vec::new();
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if Path::new(&name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("part"))
        {
            continue;
        }
        let target = unique_path(dest, &name);
        move_file(&entry.path(), &target)?;
        moved.push((entry.path(), target.clone()));
        let new = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if new != name {
            renamed.push((name, new));
        }
    }
    Ok(renamed)
}

/// Where Save all puts the photos: the folder the user picked when it is empty, otherwise a
/// folder named after the session inside it, so shots never land loose among other files
/// (for example straight in Pictures).
fn save_target(picked: &Path, session: &Path) -> PathBuf {
    let empty = std::fs::read_dir(picked).map_or(true, |mut d| d.next().is_none());
    if empty {
        return picked.to_path_buf();
    }
    let name = session
        .file_name()
        .map_or_else(|| "Camera".to_owned(), |n| n.to_string_lossy().into_owned());
    unique_path(picked, &name)
}

/// Points the session folder's rows of a project snapshot at `dest`, with the new names of
/// renamed files, so a rebuild restores every edit there.
fn retarget(snapshot: &mut ProjectFile, session: &Path, dest: &Path, renamed: &[(String, String)]) {
    let session_key = session.display().to_string();
    let Some(row) = snapshot.folders.iter_mut().find(|f| f.path == session_key) else {
        return;
    };
    row.path = dest.display().to_string();
    let id = row.id;
    for photo in snapshot.photos.iter_mut().filter(|p| p.folder == id) {
        if let Some((_, new)) = renamed.iter().find(|(old, _)| *old == photo.path) {
            photo.path.clone_from(new);
        }
    }
}

/// Deletes a session folder, but only one under `root`: never anything the user chose.
fn delete_session(root: &Path, dir: &Path) -> std::io::Result<()> {
    if dir.parent() != Some(root) {
        return Err(std::io::Error::other("not a camera session folder"));
    }
    match std::fs::remove_dir_all(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------------------
// State

enum Job {
    Capture,
    Step(ExposureSetting, i8),
    Format(CaptureFormat),
}

struct Running {
    /// Tells this session from the next one, so a camera thread that ends late never
    /// clears a newer session.
    id: u64,
    jobs: Sender<Job>,
    dir: PathBuf,
    /// Set by Disconnect: the camera thread stops before the jobs still queued.
    stop: Arc<AtomicBool>,
    /// A press is on its way; another one is refused until it is answered (a held key).
    pending: Arc<AtomicBool>,
}

/// What the camera thread knows about its own session.
pub(crate) struct Session {
    id: u64,
    stop: Arc<AtomicBool>,
    pending: Arc<AtomicBool>,
}

/// The camera session, if any, and the latest live-view frame (managed by Tauri).
pub struct CameraState {
    root: PathBuf,
    live: Mutex<Option<Arc<Vec<u8>>>>,
    running: Mutex<Option<Running>>,
    next_id: AtomicU64,
    /// Held while a Connect is in progress, so two at once cannot both start a session.
    connecting: Mutex<()>,
    /// Session folders whose camera thread has not ended yet: Save all and Delete wait.
    busy: Mutex<Vec<PathBuf>>,
    /// The user answered the closing question: closing goes ahead.
    allow_quit: AtomicBool,
}

impl CameraState {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            live: Mutex::new(None),
            running: Mutex::new(None),
            next_id: AtomicU64::new(1),
            connecting: Mutex::new(()),
            busy: Mutex::new(Vec::new()),
            allow_quit: AtomicBool::new(false),
        }
    }

    /// Ends session `id` if it is still the current one; a later session is left alone.
    fn clear(&self, id: u64) {
        if let Ok(mut running) = self.running.lock()
            && running.as_ref().is_some_and(|r| r.id == id)
        {
            *running = None;
            if let Ok(mut live) = self.live.lock() {
                *live = None;
            }
        }
    }

    fn is_current(&self, id: u64) -> bool {
        self.running
            .lock()
            .is_ok_and(|r| r.as_ref().is_some_and(|r| r.id == id))
    }

    fn is_busy(&self, dir: &Path) -> bool {
        self.busy.lock().is_ok_and(|b| b.iter().any(|d| d == dir))
    }

    /// The camera thread of `dir` has ended: its folder may be saved or deleted now.
    fn release(&self, id: u64, dir: &Path) {
        self.clear(id);
        if let Ok(mut busy) = self.busy.lock() {
            busy.retain(|d| d != dir);
        }
    }

    /// Whether the user still has shots only Fotollection holds: a camera connected, or a
    /// session tab not yet saved (asked before the app closes, ADR-0028).
    pub fn has_unsaved(&self, library_folders: &[PathBuf]) -> bool {
        self.running.lock().is_ok_and(|r| r.is_some())
            || library_folders.iter().any(|f| self.is_session(f))
    }

    /// The latest live-view frame, for the protocol.
    pub fn live_frame(&self) -> Option<Arc<Vec<u8>>> {
        self.live.lock().ok().and_then(|f| f.clone())
    }

    fn is_session(&self, dir: &Path) -> bool {
        dir.parent() == Some(self.root.as_path())
    }

    fn running_dir(&self) -> Option<PathBuf> {
        self.running
            .lock()
            .ok()
            .and_then(|r| r.as_ref().map(|r| r.dir.clone()))
    }

    fn send(&self, job: Job) -> Result<(), IpcError> {
        let running = self.running.lock().map_err(poisoned)?;
        let running = running
            .as_ref()
            .ok_or_else(|| IpcError::new("camera.not_connected", "No camera is connected"))?;
        running
            .jobs
            .send(job)
            .map_err(|_| IpcError::new("camera.not_connected", "The camera stopped"))
    }
}

/// `YYYY-MM-DD HHmmss`, the session folder's name.
fn session_name() -> String {
    chrono::Local::now().format("%Y-%m-%d %H%M%S").to_string()
}

// ---------------------------------------------------------------------------------------
// Commands

/// Finds a camera, runs its handshake and starts the session.
#[tauri::command]
#[specta::specta]
pub async fn camera_connect(app: AppHandle) -> Result<CameraInfo, IpcError> {
    tauri::async_runtime::spawn_blocking(move || connect(&app))
        .await
        .map_err(poisoned)?
}

#[cfg(not(windows))]
fn connect(_app: &AppHandle) -> Result<CameraInfo, IpcError> {
    Err(IpcError::new(
        "camera.unavailable_platform",
        "Connecting a camera works on Windows for now",
    ))
}

#[cfg(windows)]
fn connect(app: &AppHandle) -> Result<CameraInfo, IpcError> {
    let camera = app.state::<CameraState>();
    let already = || IpcError::new("camera.already_connected", "A camera is already connected");
    let _connecting = camera.connecting.try_lock().map_err(|_| already())?;
    if camera.running.lock().map_err(poisoned)?.is_some() {
        return Err(already());
    }
    if !camera.busy.lock().map_err(poisoned)?.is_empty() {
        return Err(finishing());
    }
    let id = camera.next_id.fetch_add(1, Ordering::Relaxed);
    let dir = camera.root.join(session_name());
    let (jobs, inbox) = mpsc::channel();
    let (ready, started) = mpsc::channel();
    let (stop, pending) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
    );
    // Stored before the thread starts, so the thread can only ever end its own session.
    *camera.running.lock().map_err(poisoned)? = Some(Running {
        id,
        jobs,
        dir: dir.clone(),
        stop: stop.clone(),
        pending: pending.clone(),
    });
    camera.busy.lock().map_err(poisoned)?.push(dir.clone());
    let session = Session {
        id,
        stop: stop.clone(),
        pending,
    };
    let handle = app.clone();
    let session_dir = dir.clone();
    let spawned = std::thread::Builder::new()
        .name("camera".into())
        .spawn(move || {
            wpd::run(&handle, &session_dir, &inbox, &ready, &session);
            handle.state::<CameraState>().release(id, &session_dir);
        });
    if let Err(e) = spawned {
        camera.release(id, &dir);
        return Err(IpcError::internal(e.to_string()));
    }
    let info = match started.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok(info)) if camera.is_current(id) => info,
        Ok(Ok(_)) => {
            return Err(IpcError::new(
                "camera.unplugged",
                "The camera was unplugged or turned off",
            ));
        }
        Ok(Err(e)) => {
            camera.clear(id);
            return Err(e);
        }
        Err(_) => {
            stop.store(true, Ordering::Relaxed);
            camera.clear(id);
            return Err(IpcError::new("camera.timeout", "The camera did not answer"));
        }
    };
    let _ = CameraStatus {
        connected: true,
        name: info.name.clone(),
        issue: None,
    }
    .emit(app);
    Ok(info)
}

/// The camera thread of the last session is still ending (a shot being written).
fn finishing() -> IpcError {
    IpcError::new(
        "camera.finishing",
        "The camera is finishing its last photo: try again in a moment",
    )
}

/// Ends the session; the session folder and its tab stay. Jobs still queued are dropped,
/// and the folder stays busy until the camera thread has ended.
#[tauri::command]
#[specta::specta]
pub fn camera_disconnect(camera: State<'_, CameraState>) -> Result<(), IpcError> {
    if let Some(running) = camera.running.lock().map_err(poisoned)?.take() {
        running.stop.store(true, Ordering::Relaxed);
        // Dropping the job sender also ends the camera thread's loop.
    }
    *camera.live.lock().map_err(poisoned)? = None;
    Ok(())
}

/// Takes a photo; the result comes as `camera-shot` or `camera-shot-failed`. One press at a
/// time: another is refused (`camera.busy`) until the first is answered, so a held key or
/// a double click never queues a burst.
#[tauri::command]
#[specta::specta]
pub fn camera_capture(camera: State<'_, CameraState>) -> Result<(), IpcError> {
    let running = camera.running.lock().map_err(poisoned)?;
    let running = running
        .as_ref()
        .ok_or_else(|| IpcError::new("camera.not_connected", "No camera is connected"))?;
    if running.pending.swap(true, Ordering::AcqRel) {
        return Err(IpcError::new("camera.busy", "A photo is still being taken"));
    }
    running.jobs.send(Job::Capture).map_err(|_| {
        running.pending.store(false, Ordering::Release);
        IpcError::new("camera.not_connected", "The camera stopped")
    })
}

/// Moves ISO or shutter speed by `delta` steps (−1 darker or slower… as the camera counts).
#[tauri::command]
#[specta::specta]
pub fn camera_step(
    camera: State<'_, CameraState>,
    setting: ExposureSetting,
    delta: i8,
) -> Result<(), IpcError> {
    if delta == 0 || delta.unsigned_abs() > 10 {
        return Err(IpcError::new(
            "camera.bad_step",
            "A step is between −10 and 10",
        ));
    }
    camera.send(Job::Step(setting, delta))
}

#[tauri::command]
#[specta::specta]
pub fn camera_set_format(
    camera: State<'_, CameraState>,
    format: CaptureFormat,
) -> Result<(), IpcError> {
    if format == CaptureFormat::Other {
        return Err(IpcError::new("camera.bad_format", "Choose JPEG or RAW"));
    }
    camera.send(Job::Format(format))
}

/// The library folders that are camera sessions (tabs that offer Save all).
#[tauri::command]
#[specta::specta]
pub fn camera_sessions(
    state: State<'_, AppState>,
    camera: State<'_, CameraState>,
) -> Result<Vec<u16>, IpcError> {
    let library = state.library.read().map_err(poisoned)?;
    Ok(library
        .folders()
        .iter()
        .enumerate()
        .filter(|(_, p)| camera.is_session(p))
        .filter_map(|(i, _)| u16::try_from(i).ok())
        .collect())
}

/// The session folder of tab `folder`, refusing other folders and the one still shooting.
fn session_folder(app: &AppHandle, folder: u16) -> Result<PathBuf, IpcError> {
    let state = app.state::<AppState>();
    let camera = app.state::<CameraState>();
    let dir = state
        .library
        .read()
        .map_err(poisoned)?
        .folders()
        .get(usize::from(folder))
        .cloned()
        .ok_or_else(|| IpcError::new("folder.unknown", "Unknown folder"))?;
    if !camera.is_session(&dir) {
        return Err(IpcError::new(
            "camera.not_session",
            "This folder is not a camera session",
        ));
    }
    if camera.running_dir().as_deref() == Some(dir.as_path()) {
        return Err(IpcError::new(
            "camera.still_connected",
            "Close the capture screen first",
        ));
    }
    if camera.is_busy(&dir) {
        return Err(finishing());
    }
    Ok(dir)
}

/// Save all (ADR-0028): asks where, moves the session's photos there without overwriting
/// any, and points the tab at them with every edit kept. `None` when the dialog is
/// cancelled.
#[tauri::command]
#[specta::specta]
pub async fn save_camera_session(
    app: AppHandle,
    folder: u16,
) -> Result<Option<DocumentState>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let session = session_folder(&app, folder)?;
        let state = app.state::<AppState>();
        let language = state.ui.read().map(|u| u.language).unwrap_or_default();
        let mut dialog = app
            .dialog()
            .file()
            .set_title(crate::i18n::strings(language).camera_save_title);
        if let Ok(pictures) = app.path().picture_dir() {
            dialog = dialog.set_directory(pictures);
        }
        let Some(dest) = dialog
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok())
        else {
            return Ok(None);
        };
        let camera = app.state::<CameraState>();
        let dest = save_target(&dest, &session);
        if dest.starts_with(&camera.root) {
            return Err(IpcError::new(
                "camera.bad_destination",
                "Choose a folder outside Fotollection's own data",
            ));
        }
        let _guard = state.document_lock.lock().map_err(poisoned)?;
        let (mut snapshot, folders) = {
            let library = state.library.read().map_err(poisoned)?;
            if library.folders().contains(&dest) {
                return Err(IpcError::new(
                    "camera.destination_open",
                    "That folder is already a tab: choose another one",
                ));
            }
            let folders: Vec<PathBuf> = library
                .folders()
                .iter()
                .map(|f| {
                    if *f == session {
                        dest.clone()
                    } else {
                        f.clone()
                    }
                })
                .collect();
            (project::snapshot(&library), folders)
        };
        let renamed = move_all(&session, &dest)
            .map_err(|e| IpcError::new("camera.save_failed", e.to_string()))?;
        retarget(&mut snapshot, &session, &dest, &renamed);
        rebuild_library(&app, &state, folders, &snapshot).map(Some)
    })
    .await
    .map_err(poisoned)?
}

/// Delete photos: removes the session folder and closes its tab (the UI asks first).
#[tauri::command]
#[specta::specta]
pub async fn discard_camera_session(
    app: AppHandle,
    folder: u16,
) -> Result<DocumentState, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let session = session_folder(&app, folder)?;
        let state = app.state::<AppState>();
        let camera = app.state::<CameraState>();
        let _guard = state.document_lock.lock().map_err(poisoned)?;
        let (snapshot, folders) = {
            let library = state.library.read().map_err(poisoned)?;
            let folders: Vec<PathBuf> = library
                .folders()
                .iter()
                .filter(|f| **f != session)
                .cloned()
                .collect();
            (project::snapshot(&library), folders)
        };
        delete_session(&camera.root, &session)
            .map_err(|e| IpcError::new("camera.delete_failed", e.to_string()))?;
        rebuild_library(&app, &state, folders, &snapshot)
    })
    .await
    .map_err(poisoned)?
}

/// Why closing the app should wait for the user, if it should (ADR-0028): `None` when no
/// shot exists only in Fotollection's data, or the user already answered.
pub fn should_block_quit(app: &AppHandle) -> Option<QuitBlocked> {
    let camera = app.state::<CameraState>();
    if camera.allow_quit.load(Ordering::Relaxed) {
        return None;
    }
    let folders = app
        .state::<AppState>()
        .library
        .read()
        .ok()?
        .folders()
        .to_vec();
    if !camera.has_unsaved(&folders) {
        return None;
    }
    Some(QuitBlocked {
        sessions: folders
            .iter()
            .enumerate()
            .filter(|(_, p)| camera.is_session(p))
            .filter_map(|(i, _)| u16::try_from(i).ok())
            .collect(),
        connected: camera.running.lock().is_ok_and(|r| r.is_some()),
    })
}

/// Closes the app after the user answered the closing question: the camera is let go, and
/// the app waits (at most 10 s) for its thread to write the last shot.
#[tauri::command]
#[specta::specta]
pub async fn quit_app(app: AppHandle) -> Result<(), IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let camera = app.state::<CameraState>();
        camera.allow_quit.store(true, Ordering::Relaxed);
        if let Some(running) = camera.running.lock().map_err(poisoned)?.take() {
            running.stop.store(true, Ordering::Relaxed);
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && camera.busy.lock().is_ok_and(|b| !b.is_empty()) {
            std::thread::sleep(Duration::from_millis(50));
        }
        app.exit(0);
        Ok(())
    })
    .await
    .map_err(poisoned)?
}

/// Session folders left from a run that ended without an answer (a crash, a forced quit)
/// come back as tabs, so their shots can still be saved or deleted (ADR-0028).
pub fn reopen_sessions(app: &AppHandle) {
    let camera = app.state::<CameraState>();
    let Ok(entries) = std::fs::read_dir(&camera.root) else {
        return;
    };
    let open: Vec<PathBuf> = app
        .state::<AppState>()
        .library
        .read()
        .map(|l| l.folders().to_vec())
        .unwrap_or_default();
    let left: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !open.contains(p))
        .filter(|p| {
            std::fs::read_dir(p).is_ok_and(|mut d| {
                d.any(|f| {
                    f.is_ok_and(|f| {
                        f.path()
                            .extension()
                            .is_some_and(|e| !e.eq_ignore_ascii_case("part"))
                    })
                })
            })
        })
        .collect();
    if !left.is_empty() {
        crate::commands::import_paths_with(app, left, false);
    }
}

// ---------------------------------------------------------------------------------------
// The camera thread

/// What one pass of the session loop asks the app to do.
#[derive(Debug, PartialEq, Eq)]
enum Outgoing {
    Frame(Vec<u8>),
    Settings(CameraSettings),
    /// The camera's own shutter took a photo.
    BodyShot(Shot),
    Gone(CameraIssue),
}

/// The loop's memory between passes.
struct Pass {
    last_settings: Option<CameraSettings>,
    last_poll: Option<Instant>,
    failures: u32,
}

impl Pass {
    fn new() -> Self {
        Self {
            last_settings: None,
            last_poll: None,
            failures: 0,
        }
    }

    fn failed(&mut self, issue: CameraIssue, out: &mut Vec<Outgoing>) {
        self.failures += 1;
        if self.failures >= FAILURES_BEFORE_GONE {
            out.push(Outgoing::Gone(issue));
        }
    }
}

/// One pass: unplugged? a live-view frame; the settings when due, and a photo the camera's
/// own shutter took.
fn pass<L: CameraLink>(sony: &mut Sony<L>, memory: &mut Pass) -> Vec<Outgoing> {
    let mut out = Vec::new();
    if sony.unplugged() {
        out.push(Outgoing::Gone(CameraIssue::new(
            "camera.unplugged",
            "The camera was unplugged or turned off",
        )));
        return out;
    }
    match sony.live() {
        Ok(Some(frame)) => {
            memory.failures = 0;
            out.push(Outgoing::Frame(frame));
        }
        Ok(None) => memory.failures = 0,
        Err(issue) => memory.failed(issue, &mut out),
    }
    if memory
        .last_poll
        .is_none_or(|t| t.elapsed() >= SETTINGS_EVERY)
    {
        memory.last_poll = Some(Instant::now());
        match sony.refresh() {
            Ok(settings) => {
                if memory.last_settings != Some(settings) {
                    memory.last_settings = Some(settings);
                    out.push(Outgoing::Settings(settings));
                }
                if sony.photo_waiting() {
                    match sony.fetch() {
                        Ok(shot) => out.push(Outgoing::BodyShot(shot)),
                        Err(issue) => memory.failed(issue, &mut out),
                    }
                }
            }
            Err(issue) => memory.failed(issue, &mut out),
        }
    }
    out
}

/// Writes a shot into the session folder and adds it to its tab.
fn keep_shot(app: &AppHandle, dir: &Path, shot: &Shot, pressed: Option<Instant>) {
    let path = match write_shot(dir, shot) {
        Ok(p) => p,
        Err(e) => {
            let _ = CameraShotFailed {
                issue: CameraIssue::new("camera.save_failed", e.to_string()),
            }
            .emit(app);
            return;
        }
    };
    // Only the new file: re-reading the whole session would grow with every shot.
    crate::commands::import_paths_with(app, vec![path.clone()], false);
    let state = app.state::<AppState>();
    let found = state.library.read().ok().and_then(|library| {
        let id = library.find_path(&path)?;
        Some((id, library.folder_of(id)?))
    });
    if let Some((id, folder)) = found {
        let ms = pressed.map_or(0, |t| {
            u32::try_from(t.elapsed().as_millis()).unwrap_or(u32::MAX)
        });
        let _ = CameraShot { id, folder, ms }.emit(app);
        return;
    }
    // Saved in the session folder, but the library would not take it (a RAW without the
    // Windows codec, for one): say why, so the press is never left hanging.
    let issue = match fotollection_core::import::check_file(&path) {
        Err(reason) => CameraIssue::new(reason.code(), "The shot was saved but cannot be shown"),
        Ok(_) => CameraIssue::new(
            "camera.not_imported",
            "The shot was saved but cannot be shown",
        ),
    };
    let _ = CameraShotFailed { issue }.emit(app);
}

/// The session loop: jobs first, then a pass, until the UI disconnects or the camera goes.
fn serve<L: CameraLink>(
    app: &AppHandle,
    dir: &Path,
    name: &str,
    sony: &mut Sony<L>,
    inbox: &mpsc::Receiver<Job>,
    session: &Session,
) {
    let camera = app.state::<CameraState>();
    let mut memory = Pass::new();
    let mut seq = 0u32;
    let mut sleep = |d: Duration| std::thread::sleep(d);
    loop {
        // Disconnect: the jobs still queued are dropped, not run.
        if session.stop.load(Ordering::Relaxed) {
            break;
        }
        let job = match inbox.try_recv() {
            Ok(job) => Some(job),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => break,
        };
        match job {
            Some(Job::Capture) => {
                let pressed = Instant::now();
                match sony
                    .press(&mut sleep)
                    .and_then(|()| sony.wait_and_fetch(SHOT_DEADLINE, &mut sleep))
                {
                    Ok(shot) => keep_shot(app, dir, &shot, Some(pressed)),
                    Err(issue) => {
                        let _ = CameraShotFailed { issue }.emit(app);
                    }
                }
                session.pending.store(false, Ordering::Release);
            }
            Some(Job::Step(setting, delta)) => {
                if let Err(issue) = sony.step(setting, delta) {
                    let _ = CameraShotFailed { issue }.emit(app);
                }
            }
            Some(Job::Format(format)) => {
                if let Err(issue) = sony.set_format(format) {
                    let _ = CameraShotFailed { issue }.emit(app);
                }
            }
            None => {}
        }
        let mut idle = true;
        for outgoing in pass(sony, &mut memory) {
            match outgoing {
                Outgoing::Frame(frame) => {
                    idle = false;
                    seq = seq.wrapping_add(1);
                    if let Ok(mut live) = camera.live.lock() {
                        *live = Some(Arc::new(frame));
                    }
                    let _ = CameraFrame { seq }.emit(app);
                }
                Outgoing::Settings(settings) => {
                    let _ = CameraSettingsChanged { settings }.emit(app);
                }
                Outgoing::BodyShot(shot) => keep_shot(app, dir, &shot, None),
                Outgoing::Gone(issue) => {
                    finish(app, session.id, name, Some(issue));
                    return;
                }
            }
        }
        if idle {
            // The next frame is not ready: a short rest instead of a busy loop.
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    finish(app, session.id, name, None);
}

/// Ends session `id`: no more frames, and the UI hears why. A newer session is untouched.
fn finish(app: &AppHandle, id: u64, name: &str, issue: Option<CameraIssue>) {
    let camera = app.state::<CameraState>();
    if !camera.is_current(id) {
        // Disconnected by the user (the UI already knows), or a newer session runs that this
        // late thread must not close.
        return;
    }
    camera.clear(id);
    let _ = CameraStatus {
        connected: false,
        name: name.to_owned(),
        issue,
    }
    .emit(app);
}

/// The camera through Windows Portable Devices (ADR-0026).
#[cfg(windows)]
mod wpd {
    use std::path::Path;
    use std::sync::mpsc;
    use std::time::Duration;

    use fotollection_oscamera::ptp::{self, DeviceInfo, op};
    use fotollection_oscamera::windows::{Browser, Camera, CameraError, Event};
    use tauri::AppHandle;

    use super::{Answer, CameraInfo, CameraLink, Job, LinkError, Sony, serve};
    use crate::state::IpcError;

    /// Sony's vendor extension ID (PTP).
    const SONY: u32 = 0x11;
    const CALL: Duration = Duration::from_secs(10);
    /// A RAW file is 25 MB: reading it takes under a second, so this is generous.
    const READ: Duration = Duration::from_secs(60);

    struct Link(Camera);

    impl CameraLink for Link {
        fn send(
            &mut self,
            code: u16,
            params: &[u32],
            out: Option<&[u8]>,
        ) -> Result<Answer, LinkError> {
            let timeout = if code == op::GET_OBJECT { READ } else { CALL };
            match self.0.send(code, params, out, timeout) {
                Ok(reply) => Ok(Answer {
                    code: reply.response.code,
                    data: reply.data,
                }),
                Err(CameraError::Timeout(_)) => Err(LinkError::Timeout),
                Err(e) => Err(LinkError::Failed(e.to_string())),
            }
        }

        fn unplugged(&mut self) -> bool {
            self.0
                .take_events()
                .iter()
                .any(|t| matches!(t.event, Event::Removed))
        }
    }

    fn not_found() -> IpcError {
        IpcError::new(
            "camera.not_found",
            "No camera found: set USB Connection to PC Remote and check the cable",
        )
    }

    /// Finds a Sony body among the devices Windows sees, opens it and runs the handshake.
    fn open() -> Result<(Sony<Link>, String), IpcError> {
        let browser =
            Browser::start().map_err(|e| IpcError::new("camera.failed", e.to_string()))?;
        let mut cameras = browser.cameras(Duration::from_secs(3));
        if cameras.is_empty() {
            return Err(not_found());
        }
        let mut unsupported = None;
        for mut camera in cameras.drain(..) {
            let name = camera.name();
            if camera.open(CALL).is_err() {
                continue;
            }
            let info = camera
                .send(op::GET_DEVICE_INFO, &[], None, CALL)
                .ok()
                .and_then(|r| DeviceInfo::parse(&r.data).ok());
            match info {
                Some(info) if info.vendor_extension_id == SONY => {
                    let mut sony = Sony::new(Link(camera));
                    sony.handshake().map_err(IpcError::from)?;
                    return Ok((sony, name));
                }
                Some(info) => {
                    unsupported = Some(format!(
                        "{name} ({}) is not supported yet",
                        ptp::vendor_name(info.vendor_extension_id)
                    ));
                }
                None => {}
            }
        }
        Err(unsupported.map_or_else(not_found, |m| IpcError::new("camera.unsupported", m)))
    }

    /// The camera thread: connect, report, then serve until the session ends.
    pub(super) fn run(
        app: &AppHandle,
        dir: &Path,
        inbox: &mpsc::Receiver<Job>,
        ready: &mpsc::Sender<Result<CameraInfo, IpcError>>,
        session: &super::Session,
    ) {
        let (mut sony, name) = match open() {
            Ok(v) => v,
            Err(e) => {
                let _ = ready.send(Err(e));
                return;
            }
        };
        // Right after the first handshake the block can come back empty: read it twice.
        let _ = sony.refresh();
        let settings = sony.refresh().unwrap_or_default();
        let info = CameraInfo {
            name: name.clone(),
            settings,
        };
        if ready.send(Ok(info)).is_err() {
            return;
        }
        serve(app, dir, &name, &mut sony, inbox, session);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// A scripted Sony body: answers the handshake, a settings block whose shot state goes
    /// to "photo waiting" a few reads after a full press, and the photo.
    #[derive(Default)]
    #[allow(clippy::struct_excessive_bools)]
    struct FakeCamera {
        calls: Vec<u16>,
        controls: Vec<(u32, Vec<u8>)>,
        pressed: bool,
        reads_until_photo: Option<u32>,
        never_shoots: bool,
        unplugged: bool,
        refuse_handshake: bool,
        live: VecDeque<u16>,
        photo: Vec<u8>,
    }

    fn u16_prop(code: u16, value: u16) -> Vec<u8> {
        let mut b = code.to_le_bytes().to_vec();
        b.extend_from_slice(&0x0004u16.to_le_bytes());
        b.extend_from_slice(&[0, 1]);
        b.extend_from_slice(&value.to_le_bytes());
        b.extend_from_slice(&value.to_le_bytes());
        b.push(0);
        b
    }

    fn u32_prop(code: u16, value: u32) -> Vec<u8> {
        let mut b = code.to_le_bytes().to_vec();
        b.extend_from_slice(&0x0006u16.to_le_bytes());
        b.extend_from_slice(&[0, 1]);
        b.extend_from_slice(&value.to_le_bytes());
        b.extend_from_slice(&value.to_le_bytes());
        b.push(0);
        b
    }

    fn settings_block(shot_state: u16) -> Vec<u8> {
        let mut b = 5u32.to_le_bytes().to_vec();
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend(u16_prop(prop::SHOT_STATE, shot_state));
        b.extend(u32_prop(prop::ISO, 1600));
        b.extend(u32_prop(prop::SHUTTER_SPEED, 0x0001_00FA));
        b.extend(u16_prop(prop::F_NUMBER, 200));
        b.extend(u16_prop(prop::FILE_FORMAT, file_format::JPEG));
        b
    }

    fn ptp_string(s: &str) -> Vec<u8> {
        let units: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
        let mut out = vec![u8::try_from(units.len()).unwrap_or(0)];
        for u in units {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out
    }

    fn object_info(size: u32) -> Vec<u8> {
        let mut d = 0x0001_0000u32.to_le_bytes().to_vec();
        d.extend_from_slice(&object_format::JPEG.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&size.to_le_bytes());
        d.extend_from_slice(&[0; 14 + 8 + 18]);
        d.extend(ptp_string("../DSC00001.JPG"));
        d.push(0);
        d
    }

    #[allow(clippy::unnecessary_wraps)]
    fn ok(data: Vec<u8>) -> Result<Answer, LinkError> {
        Ok(Answer {
            code: ptp::rc::OK,
            data,
        })
    }

    impl CameraLink for FakeCamera {
        fn send(
            &mut self,
            code: u16,
            params: &[u32],
            out: Option<&[u8]>,
        ) -> Result<Answer, LinkError> {
            self.calls.push(code);
            match code {
                sony::op::HANDSHAKE | sony::op::PROPERTY_CODES if self.refuse_handshake => {
                    Ok(Answer {
                        code: 0x2002,
                        data: Vec::new(),
                    })
                }
                sony::op::HANDSHAKE | sony::op::PROPERTY_CODES => ok(vec![0; 8]),
                sony::op::CONTROL => {
                    let code = params.first().copied().unwrap_or(0);
                    let value = out.map(<[u8]>::to_vec).unwrap_or_default();
                    if code == u32::from(prop::SHUTTER_FULL) && value.first() == Some(&2) {
                        self.pressed = true;
                    }
                    self.controls.push((code, value));
                    ok(Vec::new())
                }
                sony::op::SET_VALUE => ok(Vec::new()),
                sony::op::ALL_SETTINGS => {
                    let waiting = self.pressed
                        && !self.never_shoots
                        && match self.reads_until_photo.as_mut() {
                            Some(0) | None => true,
                            Some(n) => {
                                *n -= 1;
                                false
                            }
                        };
                    ok(settings_block(if waiting { 0x8001 } else { 0 }))
                }
                op::GET_OBJECT_INFO if params == [sony::handle::LIVE_VIEW] => ok(Vec::new()),
                op::GET_OBJECT if params == [sony::handle::LIVE_VIEW] => {
                    match self.live.pop_front() {
                        Some(ptp::rc::OK) | None => {
                            let mut block = 8u32.to_le_bytes().to_vec();
                            block.extend_from_slice(&2u32.to_le_bytes());
                            block.extend_from_slice(&[0xFF, 0xD8]);
                            ok(block)
                        }
                        Some(code) => Ok(Answer {
                            code,
                            data: Vec::new(),
                        }),
                    }
                }
                op::GET_OBJECT_INFO => {
                    ok(object_info(u32::try_from(self.photo.len()).unwrap_or(0)))
                }
                op::GET_OBJECT => {
                    self.pressed = false;
                    ok(self.photo.clone())
                }
                _ => Err(LinkError::Failed(format!("unexpected 0x{code:04X}"))),
            }
        }

        fn unplugged(&mut self) -> bool {
            self.unplugged
        }
    }

    fn no_sleep() -> impl FnMut(Duration) {
        |_| {}
    }

    fn photo_camera() -> FakeCamera {
        FakeCamera {
            photo: vec![0xFF, 0xD8, 1, 2, 3, 0xFF, 0xD9],
            reads_until_photo: Some(2),
            ..FakeCamera::default()
        }
    }

    #[test]
    fn fr_063_handshake_runs_in_order() {
        let mut sony = Sony::new(FakeCamera::default());
        assert_eq!(sony.handshake(), Ok(()));
        let expected: Vec<u16> = HANDSHAKE.iter().map(|(code, _)| *code).collect();
        assert_eq!(sony.link.calls, expected);

        let mut busy = Sony::new(FakeCamera {
            refuse_handshake: true,
            ..FakeCamera::default()
        });
        assert_eq!(
            busy.handshake().map_err(|e| e.code),
            Err("camera.busy".into())
        );
    }

    #[test]
    fn fr_063_settings_are_read_for_the_screen() {
        let mut sony = Sony::new(FakeCamera::default());
        let settings = sony.refresh().ok();
        assert_eq!(
            settings,
            Some(CameraSettings {
                iso: Some(1600),
                iso_auto: false,
                shutter: Some(ShutterSpeed {
                    numerator: 1,
                    denominator: 250
                }),
                aperture: Some(200),
                format: Some(CaptureFormat::Jpeg),
            })
        );
    }

    #[test]
    fn fr_063_shot_is_saved_atomically_and_named_safely() -> std::io::Result<()> {
        let mut sony = Sony::new(photo_camera());
        let shot = sony
            .press(&mut no_sleep())
            .and_then(|()| sony.wait_and_fetch(SHOT_DEADLINE, &mut no_sleep()))
            .map_err(|e| std::io::Error::other(e.message))?;
        // The camera's name loses its path: nothing is written outside the session folder.
        assert_eq!(shot.name, "DSC00001.JPG");
        let presses: Vec<u32> = sony.link.controls.iter().map(|(c, _)| *c).collect();
        let expected: Vec<u32> = SHUTTER_PRESS.iter().map(|(c, _)| u32::from(*c)).collect();
        assert_eq!(presses, expected);

        let dir = tempfile::tempdir()?;
        let session = dir.path().join("2026-10-26 180000");
        let first = write_shot(&session, &shot)?;
        let second = write_shot(&session, &shot)?;
        assert_eq!(std::fs::read(&first)?, shot.bytes);
        assert_eq!(
            second.file_name().and_then(|n| n.to_str()),
            Some("DSC00001 (2).JPG")
        );
        let leftovers = std::fs::read_dir(&session)?
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .count();
        assert_eq!(leftovers, 0);
        Ok(())
    }

    #[test]
    fn fr_063_no_shot_reports_camera_no_shot() {
        let mut sony = Sony::new(FakeCamera {
            never_shoots: true,
            ..photo_camera()
        });
        let result = sony
            .press(&mut no_sleep())
            .and_then(|()| sony.wait_and_fetch(Duration::from_millis(20), &mut no_sleep()));
        assert_eq!(result.map_err(|e| e.code), Err("camera.no_shot".into()));
    }

    #[test]
    fn fr_063_names_from_the_camera_are_made_safe() {
        assert_eq!(
            safe_name("DSC04099.JPG", CaptureFormat::Jpeg),
            "DSC04099.JPG"
        );
        assert_eq!(
            safe_name("..\\..\\evil.ARW", CaptureFormat::Raw),
            "evil.ARW"
        );
        assert_eq!(safe_name("", CaptureFormat::Raw), "shot.ARW");
        assert_eq!(safe_name("photo", CaptureFormat::Jpeg), "photo.JPG");
        assert_eq!(safe_name("a<b>:c|d?.JPG", CaptureFormat::Jpeg), "abcd.JPG");
        assert!(safe_name(&"x".repeat(500), CaptureFormat::Jpeg).len() <= MAX_NAME + 4);
    }

    #[test]
    fn fr_063_save_all_uses_a_session_folder_inside_a_folder_that_has_files() -> std::io::Result<()>
    {
        let dir = tempfile::tempdir()?;
        let session = dir.path().join("2026-10-26 103000");
        let empty = dir.path().join("empty");
        let pictures = dir.path().join("Pictures");
        std::fs::create_dir_all(&empty)?;
        std::fs::create_dir_all(&pictures)?;
        std::fs::write(pictures.join("holiday.jpg"), b"x")?;
        assert_eq!(save_target(&empty, &session), empty);
        assert_eq!(
            save_target(&pictures, &session),
            pictures.join("2026-10-26 103000")
        );
        Ok(())
    }

    #[test]
    fn fr_063_a_late_camera_thread_never_ends_a_newer_session() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let camera = CameraState::new(dir.path().to_path_buf());
        let session = |id: u64, name: &str| Running {
            id,
            jobs: mpsc::channel().0,
            dir: dir.path().join(name),
            stop: Arc::new(AtomicBool::new(false)),
            pending: Arc::new(AtomicBool::new(false)),
        };
        // Session 1 was disconnected while a shot was pending, then session 2 started.
        *camera.running.lock().map_err(|e| e.to_string())? = Some(session(2, "second"));
        camera
            .busy
            .lock()
            .map_err(|e| e.to_string())?
            .push(dir.path().join("first"));
        assert!(camera.is_busy(&dir.path().join("first")));
        // The first thread ends late: it frees its own folder and leaves session 2 running.
        camera.release(1, &dir.path().join("first"));
        assert!(camera.is_current(2));
        assert!(!camera.is_busy(&dir.path().join("first")));
        assert!(camera.has_unsaved(&[]));
        camera.release(2, &dir.path().join("second"));
        assert!(!camera.is_current(2));
        assert!(!camera.has_unsaved(&[PathBuf::from("/Users/someone/Pictures/Trip")]));
        assert!(camera.has_unsaved(&[dir.path().join("a session")]));
        Ok(())
    }

    #[test]
    fn fr_063_save_all_moves_files_without_overwriting_and_keeps_edits() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let session = dir.path().join("session");
        let dest = dir.path().join("dest");
        std::fs::create_dir_all(&session)?;
        std::fs::create_dir_all(&dest)?;
        std::fs::write(session.join("a.JPG"), b"new a")?;
        std::fs::write(session.join("b.JPG"), b"new b")?;
        std::fs::write(dest.join("a.JPG"), b"old a")?;

        let renamed = move_all(&session, &dest)?;
        assert_eq!(renamed, vec![("a.JPG".to_owned(), "a (2).JPG".to_owned())]);
        assert_eq!(std::fs::read(dest.join("a.JPG"))?, b"old a");
        assert_eq!(std::fs::read(dest.join("a (2).JPG"))?, b"new a");
        assert_eq!(std::fs::read(dest.join("b.JPG"))?, b"new b");
        assert!(!session.exists());

        // The snapshot follows the files, so the rebuild restores each photo's edits.
        let mut snapshot: ProjectFile = serde_json::from_value(serde_json::json!({
            "format": "fotollection", "schema_version": 7, "app_version": "0.8.0",
            "modified_at": "2026-10-26T18:00:00Z",
            "folders": [{ "id": 1, "path": session.display().to_string(), "path_kind": "absolute" }],
            "photos": [{ "id": 1, "folder": 1, "path": "a.JPG", "fp_size": 5, "fp_mtime": 0, "rating": 4 }],
        }))
        .map_err(std::io::Error::other)?;
        retarget(&mut snapshot, &session, &dest, &renamed);
        assert_eq!(snapshot.folders[0].path, dest.display().to_string());
        assert_eq!(snapshot.photos[0].path, "a (2).JPG");
        assert_eq!(snapshot.photos[0].rating, Some(4));
        Ok(())
    }

    #[test]
    fn fr_063_discard_removes_only_session_files() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("camera");
        let session = root.join("2026-10-26 180000");
        let elsewhere = dir.path().join("Pictures");
        std::fs::create_dir_all(&session)?;
        std::fs::create_dir_all(&elsewhere)?;
        std::fs::write(session.join("a.JPG"), b"a")?;
        std::fs::write(elsewhere.join("keep.JPG"), b"k")?;

        assert!(delete_session(&root, &elsewhere).is_err());
        assert!(delete_session(&root, &root).is_err());
        assert!(elsewhere.join("keep.JPG").exists());
        delete_session(&root, &session)?;
        assert!(!session.exists());
        assert!(root.exists());
        Ok(())
    }

    #[test]
    fn fr_062_live_view_skips_frames_not_ready_and_reports_settings_once() {
        let mut sony = Sony::new(FakeCamera {
            live: VecDeque::from([0x200F, ptp::rc::OK]),
            ..FakeCamera::default()
        });
        let mut memory = Pass::new();
        let first = pass(&mut sony, &mut memory);
        assert!(!first.iter().any(|o| matches!(o, Outgoing::Frame(_))));
        assert!(first.iter().any(|o| matches!(o, Outgoing::Settings(_))));
        let second = pass(&mut sony, &mut memory);
        assert!(second.contains(&Outgoing::Frame(vec![0xFF, 0xD8])));
        // Settings are not due again yet, and did not change anyway.
        assert!(!second.iter().any(|o| matches!(o, Outgoing::Settings(_))));
    }

    #[test]
    fn fr_062_unplug_ends_the_session_with_a_reason() {
        let mut sony = Sony::new(FakeCamera {
            unplugged: true,
            ..FakeCamera::default()
        });
        let out = pass(&mut sony, &mut Pass::new());
        assert!(matches!(&out[..], [Outgoing::Gone(i)] if i.code == "camera.unplugged"));
    }

    #[test]
    fn fr_063_the_cameras_own_shutter_is_kept_too() {
        let mut camera = photo_camera();
        camera.pressed = true;
        camera.reads_until_photo = None;
        let mut sony = Sony::new(camera);
        let out = pass(&mut sony, &mut Pass::new());
        assert!(
            out.iter()
                .any(|o| matches!(o, Outgoing::BodyShot(s) if s.name == "DSC00001.JPG"))
        );
    }
}
