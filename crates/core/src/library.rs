// SPDX-License-Identifier: GPL-3.0-only
//! In-memory photo library as columnar arenas (DATABASE.md §3.3): dense `PhotoId(u32)`
//! indices, interned paths, and small fixed-width columns, so thousands of photos cost
//! about a megabyte and leave RAM for pixels (NFR-004).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::import::ImportedFile;
use crate::metadata::PhotoMeta;
use crate::plan::CropFrame;

/// Dense index of a photo within this session's library.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
pub struct PhotoId(pub u32);

/// Pick/reject flag for culling (FR-022).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Flag {
    Pick,
    Reject,
}

/// The two holding areas of the print workflow (FR-050): the print basket, and photos set
/// aside, out of the gallery but never deleted. A photo is in one of them or in neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Queue {
    Print,
    Aside,
}

/// Which part of the library a view shows (FR-050).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum QueueView {
    /// Every photo except those set aside; the basket's photos stay, marked.
    #[default]
    Gallery,
    Print,
    Aside,
}

/// Which flags a culling filter keeps (FR-022).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FlagFilter {
    #[default]
    All,
    Picks,
    Unflagged,
    Rejects,
}

/// The library filter (FR-022). The grid shows, and export writes, exactly the photos it keeps.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CullFilter {
    pub flag: FlagFilter,
    /// Minimum stars, 0–5. Values above 5 are treated as 5.
    pub min_rating: u8,
    /// Source folders shown, like browser tabs (FR-056); empty means every folder. Unknown
    /// ids match nothing.
    #[serde(default)]
    pub folders: Vec<u16>,
    /// The gallery, the print basket or the photos set aside (FR-050).
    #[serde(default)]
    pub queue: QueueView,
}

/// One source folder in the library list (FR-056).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderSummary {
    pub id: u16,
    pub name: String,
    /// Every photo of the folder, whatever the filter.
    pub count: u32,
}

impl CullFilter {
    pub fn matches(&self, edits: &PhotoEdits) -> bool {
        let flag_ok = match self.flag {
            FlagFilter::All => true,
            FlagFilter::Picks => edits.flag == Some(Flag::Pick),
            FlagFilter::Unflagged => edits.flag.is_none(),
            FlagFilter::Rejects => edits.flag == Some(Flag::Reject),
        };
        let queue_ok = match self.queue {
            QueueView::Gallery => edits.queue != Some(Queue::Aside),
            QueueView::Print => edits.queue == Some(Queue::Print),
            QueueView::Aside => edits.queue == Some(Queue::Aside),
        };
        flag_ok && queue_ok && edits.rating >= self.min_rating.min(5)
    }
}

/// User edits of one photo. Stored sparsely: photos without edits cost nothing (N5).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoEdits {
    pub crop: Option<CropFrame>,
    /// 0–5 stars.
    pub rating: u8,
    pub flag: Option<Flag>,
    /// This photo's own stamp, set in the preview (FR-044); `None` follows the settings.
    pub stamp: Option<crate::settings::StampSettings>,
    /// The user's quarter turns clockwise on top of the EXIF orientation, 0–3 (FR-006).
    #[serde(default)]
    pub rotation: u8,
    /// Straightening in degrees, ±15 (FR-006).
    #[serde(default)]
    pub straighten: f32,
    /// In the print basket or set aside (FR-050).
    #[serde(default)]
    pub queue: Option<Queue>,
}

impl PhotoEdits {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Bit flags stored per photo.
pub mod flags {
    pub const HAS_GPS: u16 = 1 << 0;
    /// No EXIF capture time: the date comes from the file's modification time.
    pub const DATE_FROM_FILE: u16 = 1 << 1;
    pub const HAS_EMBEDDED_THUMB: u16 = 1 << 2;
}

const NO_OFFSET: i16 = i16::MIN;

/// Identity of a path that ignores separator style (`C:\a\b.jpg` == `C:/a/b.jpg`), so
/// paths rebuilt from project rows (`/`-separated) match scanned paths on every OS.
fn path_key(path: &Path) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    for (i, component) in path.components().enumerate() {
        if i > 0 {
            hasher.update(b"/");
        }
        hasher.update(component.as_os_str().as_encoded_bytes());
    }
    hasher.finalize()
}

/// All strings (relative paths) live in one buffer; columns hold `(start, len)` spans.
#[derive(Default)]
struct StrArena {
    buf: String,
}

impl StrArena {
    fn intern(&mut self, s: &str) -> (u32, u32) {
        let start = u32::try_from(self.buf.len()).unwrap_or(u32::MAX);
        self.buf.push_str(s);
        (start, u32::try_from(s.len()).unwrap_or(0))
    }

    fn get(&self, (start, len): (u32, u32)) -> &str {
        self.buf
            .get(start as usize..(start as usize + len as usize))
            .unwrap_or_default()
    }
}

/// Everything the renderer and export need about one photo, materialized on demand.
#[derive(Debug, Clone)]
pub struct PhotoRecord {
    pub id: PhotoId,
    pub path: PathBuf,
    pub file_name: String,
    pub size: u64,
    pub mtime: i64,
    pub capture: NaiveDateTime,
    pub orientation: u8,
    pub flags: u16,
    pub gps: Option<(f32, f32)>,
    pub shot: ShotInfo,
    pub edits: PhotoEdits,
    /// The time the camera wrote, before its clock offset (FR-030). Equal to `capture` when
    /// the body has no offset or the date comes from the file.
    pub camera_time: NaiveDateTime,
}

impl PhotoRecord {
    /// The orientation the photo is shown and printed in: its EXIF orientation plus the
    /// user's quarter turns (FR-006).
    pub fn display_orientation(&self) -> u8 {
        crate::image::compose_orientation(self.orientation, self.edits.rotation)
    }

    /// The date stamped on the photo and written in its frame (FR-004, FR-040): the capture
    /// time corrected by its camera's clock offset (FR-030), or the file's time when the
    /// photo has none. `None` only when even the file time is unknown.
    pub fn stamp_date(&self) -> Option<NaiveDateTime> {
        (self.flags & flags::DATE_FROM_FILE == 0 || self.mtime != 0).then_some(self.capture)
    }
}

/// Camera and exposure facts from EXIF, used by the info frame (FR-040).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShotInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens: Option<String>,
    pub focal_mm: Option<f32>,
    pub f_number: Option<f32>,
    pub exposure_s: Option<f32>,
    pub iso: Option<u32>,
}

/// Grid data sent to the UI (FR-003).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoSummary {
    pub id: PhotoId,
    pub file_name: String,
    /// Index of the source folder, for taking a folder out of the library (FR-001).
    pub folder_id: u16,
    pub folder_name: String,
    /// Camera-local capture time, `YYYY-MM-DDTHH:MM:SS`, corrected by the camera's clock
    /// offset (FR-030).
    pub capture: String,
    /// The camera's clock offset changed this photo's time (FR-030).
    pub time_shifted: bool,
    pub has_gps: bool,
    pub date_from_file: bool,
    pub camera: Option<String>,
    /// 0–5 stars (FR-022).
    pub rating: u8,
    pub flag: Option<Flag>,
    /// The user's quarter turns clockwise (FR-006); the grid turns the thumbnail to match.
    pub rotation: u8,
    /// In the print basket or set aside (FR-050).
    pub queue: Option<Queue>,
}

#[derive(Default)]
pub struct Library {
    folders: Vec<PathBuf>,
    strings: StrArena,
    /// Distinct `(make, model)` pairs; photos reference them by index (N2/N3).
    cameras: Vec<(Option<String>, Option<String>)>,
    /// Distinct lens names, referenced by index.
    lenses: Vec<String>,
    /// Distinct camera bodies, each with its own clock (FR-030).
    bodies: Vec<Body>,
    /// Full-path hash → photo, for de-duplication and restoring project rows.
    seen: HashMap<blake3::Hash, PhotoId>,
    edits: HashMap<PhotoId, PhotoEdits>,
    // Columns, all indexed by PhotoId.
    folder: Vec<u16>,
    rel_path: Vec<(u32, u32)>,
    size: Vec<u64>,
    mtime: Vec<i64>,
    capture: Vec<i64>,
    offset: Vec<i16>,
    orientation: Vec<u8>,
    flags: Vec<u16>,
    lat: Vec<f32>,
    lon: Vec<f32>,
    camera: Vec<u16>,
    body: Vec<u16>,
    lens: Vec<u16>,
    focal_mm: Vec<f32>,
    f_number: Vec<f32>,
    exposure_s: Vec<f32>,
    iso: Vec<u32>,
}

const NONE_U16: u16 = u16::MAX;

/// A camera clock is corrected by at most a month (FR-030); a date further off is set by
/// hand (FR-032).
pub const MAX_CLOCK_OFFSET_S: i64 = 31 * 24 * 3600;
/// Time zones span UTC−12 to UTC+14.
pub const MAX_TZ_MINUTES: i16 = 14 * 60;

/// One camera body: its make and model (an index into `cameras`), its serial number when
/// the camera writes one, and its clock correction (FR-030).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Body {
    camera: u16,
    serial: Option<String>,
    offset_s: i64,
    tz_minutes: Option<i16>,
}

/// A camera body and its clock, as listed in the settings (FR-030).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CameraClock {
    pub id: u16,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    /// The name shown to the user ("Panasonic DC-GX9").
    pub label: Option<String>,
    /// Seconds added to the camera's time.
    #[specta(type = f64)]
    pub offset_s: i64,
    /// The time zone the corrected times are in, in minutes from UTC, when the user set one
    /// (used to match a GPX track, FR-031).
    pub tz_minutes: Option<i16>,
    /// Photos taken with this body.
    pub photos: u32,
}

/// Why a clock correction was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ClockError {
    #[error("unknown camera")]
    UnknownCamera,
    #[error("the offset is larger than a month")]
    OffsetTooLarge,
    #[error("the time zone is outside UTC−14…+14")]
    BadTimeZone,
}

fn check_clock(offset_s: i64, tz_minutes: Option<i16>) -> Result<(), ClockError> {
    if offset_s.abs() > MAX_CLOCK_OFFSET_S {
        return Err(ClockError::OffsetTooLarge);
    }
    if tz_minutes.is_some_and(|tz| tz.abs() > MAX_TZ_MINUTES) {
        return Err(ClockError::BadTimeZone);
    }
    Ok(())
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.folder.len()
    }

    pub fn is_empty(&self) -> bool {
        self.folder.is_empty()
    }

    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// Registers a source folder, returning its index (existing folders are reused).
    pub fn add_folder(&mut self, root: &Path) -> u16 {
        if let Some(i) = self.folders.iter().position(|f| f == root) {
            return u16::try_from(i).unwrap_or(u16::MAX);
        }
        self.folders.push(root.to_path_buf());
        u16::try_from(self.folders.len() - 1).unwrap_or(u16::MAX)
    }

    fn camera_index(&mut self, meta: &PhotoMeta) -> u16 {
        if meta.make.is_none() && meta.model.is_none() {
            return NONE_U16;
        }
        let key = (meta.make.clone(), meta.model.clone());
        let i = self
            .cameras
            .iter()
            .position(|c| *c == key)
            .unwrap_or_else(|| {
                self.cameras.push(key);
                self.cameras.len() - 1
            });
        u16::try_from(i).unwrap_or(NONE_U16)
    }

    fn body_index(&mut self, camera: u16, serial: Option<&String>) -> u16 {
        if camera == NONE_U16 {
            return NONE_U16;
        }
        let i = self
            .bodies
            .iter()
            .position(|b| b.camera == camera && b.serial.as_ref() == serial)
            .unwrap_or_else(|| {
                self.bodies.push(Body {
                    camera,
                    serial: serial.cloned(),
                    offset_s: 0,
                    tz_minutes: None,
                });
                self.bodies.len() - 1
            });
        u16::try_from(i).unwrap_or(NONE_U16)
    }

    /// Seconds the photo's camera clock is corrected by; 0 for a date taken from the file.
    fn clock_offset(&self, i: usize) -> i64 {
        if self.flags[i] & flags::DATE_FROM_FILE != 0 {
            return 0;
        }
        self.bodies
            .get(self.body[i] as usize)
            .map_or(0, |b| b.offset_s)
    }

    /// The capture time used everywhere (FR-030): the camera's time plus its clock offset.
    fn effective(&self, i: usize) -> i64 {
        self.capture[i].saturating_add(self.clock_offset(i))
    }

    /// Every camera body in the library and its clock (FR-030).
    pub fn clocks(&self) -> Vec<CameraClock> {
        let mut photos = vec![0u32; self.bodies.len()];
        for &b in &self.body {
            if let Some(n) = photos.get_mut(b as usize) {
                *n += 1;
            }
        }
        self.bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let (make, model) = self
                    .cameras
                    .get(b.camera as usize)
                    .cloned()
                    .unwrap_or_default();
                CameraClock {
                    id: u16::try_from(i).unwrap_or(NONE_U16),
                    label: Self::camera_label(make.as_deref(), model.as_deref()),
                    make,
                    model,
                    serial: b.serial.clone(),
                    offset_s: b.offset_s,
                    tz_minutes: b.tz_minutes,
                    photos: photos[i],
                }
            })
            .collect()
    }

    /// Corrects one body's clock (FR-030).
    pub fn set_clock(
        &mut self,
        id: u16,
        offset_s: i64,
        tz_minutes: Option<i16>,
    ) -> Result<(), ClockError> {
        check_clock(offset_s, tz_minutes)?;
        let body = self
            .bodies
            .get_mut(id as usize)
            .ok_or(ClockError::UnknownCamera)?;
        body.offset_s = offset_s;
        body.tz_minutes = tz_minutes;
        Ok(())
    }

    /// The body a photo was taken with (FR-030).
    pub fn clock_of(&self, id: PhotoId) -> Option<u16> {
        let b = *self.body.get(self.index(id)?)?;
        (b != NONE_U16).then_some(b)
    }

    /// Restores a saved clock onto the body with this make, model and serial, if the library
    /// has it (project rows, FR-030). Invalid values are ignored.
    pub fn restore_clock(&mut self, clock: &CameraClock) {
        if check_clock(clock.offset_s, clock.tz_minutes).is_err() {
            return;
        }
        let key = (clock.make.clone(), clock.model.clone());
        let Some(camera) = self.cameras.iter().position(|c| *c == key) else {
            return;
        };
        if let Some(body) = self
            .bodies
            .iter_mut()
            .find(|b| b.camera as usize == camera && b.serial == clock.serial)
        {
            body.offset_s = clock.offset_s;
            body.tz_minutes = clock.tz_minutes;
        }
    }

    fn lens_index(&mut self, lens: Option<&String>) -> u16 {
        let Some(lens) = lens else { return NONE_U16 };
        let i = self
            .lenses
            .iter()
            .position(|l| l == lens)
            .unwrap_or_else(|| {
                self.lenses.push(lens.clone());
                self.lenses.len() - 1
            });
        u16::try_from(i).unwrap_or(NONE_U16)
    }

    /// Distinct cameras seen so far, as `(make, model)`.
    pub fn cameras(&self) -> &[(Option<String>, Option<String>)] {
        &self.cameras
    }

    /// Human-readable camera name: `make model`, without repeating the make.
    pub fn camera_label(make: Option<&str>, model: Option<&str>) -> Option<String> {
        let brand = make.map(brand_name);
        match (brand.as_deref(), model) {
            (_, None) => brand,
            (Some(brand), Some(model)) if !starts_with_word(model, brand) => {
                Some(format!("{brand} {model}"))
            }
            (_, Some(model)) => Some(model.to_owned()),
        }
    }

    /// Adds a photo. Returns `None` if the same path was already imported.
    pub fn push(&mut self, folder: u16, file: &ImportedFile, meta: &PhotoMeta) -> Option<PhotoId> {
        let key = path_key(&file.path);
        if self.seen.contains_key(&key) {
            return None;
        }
        let root = self.folders.get(folder as usize)?;
        let rel = file.path.strip_prefix(root).unwrap_or(&file.path);
        let rel = rel.to_string_lossy();
        let span = self.strings.intern(&rel);
        let camera = self.camera_index(meta);
        let body = self.body_index(camera, meta.serial.as_ref());
        let lens = self.lens_index(meta.lens.as_ref());

        let mut f = 0;
        let (capture, offset) = if let Some(dt) = meta.capture {
            (
                dt.and_utc().timestamp(),
                meta.offset_minutes.unwrap_or(NO_OFFSET),
            )
        } else {
            f |= flags::DATE_FROM_FILE;
            (file.mtime, NO_OFFSET)
        };
        #[allow(clippy::cast_possible_truncation)] // f32 ≈ 1 m precision is enough for stamping
        let (lat, lon) = match meta.gps {
            Some((lat, lon)) => {
                f |= flags::HAS_GPS;
                (lat as f32, lon as f32)
            }
            None => (f32::NAN, f32::NAN),
        };
        if meta.has_embedded_thumbnail {
            f |= flags::HAS_EMBEDDED_THUMB;
        }

        let id = PhotoId(u32::try_from(self.len()).ok()?);
        self.seen.insert(key, id);
        self.folder.push(folder);
        self.rel_path.push(span);
        self.size.push(file.size);
        self.mtime.push(file.mtime);
        self.capture.push(capture);
        self.offset.push(offset);
        self.orientation.push(meta.orientation);
        self.flags.push(f);
        self.lat.push(lat);
        self.lon.push(lon);
        self.camera.push(camera);
        self.body.push(body);
        self.lens.push(lens);
        self.focal_mm.push(meta.focal_mm.unwrap_or(f32::NAN));
        self.f_number.push(meta.f_number.unwrap_or(f32::NAN));
        self.exposure_s.push(meta.exposure_s.unwrap_or(f32::NAN));
        self.iso.push(meta.iso.unwrap_or(0));
        Some(id)
    }

    fn index(&self, id: PhotoId) -> Option<usize> {
        let i = id.0 as usize;
        (i < self.len()).then_some(i)
    }

    pub fn record(&self, id: PhotoId) -> Option<PhotoRecord> {
        let i = self.index(id)?;
        let root = self.folders.get(self.folder[i] as usize)?;
        let rel = self.strings.get(self.rel_path[i]);
        let path = root.join(rel);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let gps = (self.flags[i] & flags::HAS_GPS != 0).then(|| (self.lat[i], self.lon[i]));
        let finite = |v: f32| v.is_finite().then_some(v);
        let (make, model) = self
            .cameras
            .get(self.camera[i] as usize)
            .cloned()
            .unwrap_or_default();
        let shot = ShotInfo {
            make,
            model,
            lens: self.lenses.get(self.lens[i] as usize).cloned(),
            focal_mm: finite(self.focal_mm[i]),
            f_number: finite(self.f_number[i]),
            exposure_s: finite(self.exposure_s[i]),
            iso: (self.iso[i] > 0).then_some(self.iso[i]),
        };
        Some(PhotoRecord {
            id,
            path,
            file_name,
            size: self.size[i],
            mtime: self.mtime[i],
            capture: DateTime::from_timestamp(self.effective(i), 0)?.naive_utc(),
            camera_time: DateTime::from_timestamp(self.capture[i], 0)?.naive_utc(),
            orientation: self.orientation[i],
            flags: self.flags[i],
            gps,
            shot,
            edits: self.edits(id),
        })
    }

    /// The photo at `path`, if imported.
    pub fn find_path(&self, path: &Path) -> Option<PhotoId> {
        self.seen.get(&path_key(path)).copied()
    }

    pub fn edits(&self, id: PhotoId) -> PhotoEdits {
        self.edits.get(&id).cloned().unwrap_or_default()
    }

    /// Replaces a photo's edits, returning the previous ones (for undo).
    pub fn set_edits(&mut self, id: PhotoId, edits: PhotoEdits) -> PhotoEdits {
        let old = self.edits(id);
        if edits.is_default() {
            self.edits.remove(&id);
        } else if self.index(id).is_some() {
            self.edits.insert(id, edits);
        }
        old
    }

    /// Photos that have edits, for saving the project.
    pub fn edited(&self) -> impl Iterator<Item = (PhotoId, PhotoEdits)> + '_ {
        self.edits.iter().map(|(id, e)| (*id, e.clone()))
    }

    /// Folder index and relative path of a photo (project rows, DATABASE.md §4).
    pub fn location(&self, id: PhotoId) -> Option<(u16, &str)> {
        let i = self.index(id)?;
        Some((self.folder[i], self.strings.get(self.rel_path[i])))
    }

    /// All photo IDs ordered by capture time, then path.
    pub fn sorted_ids(&self) -> Vec<PhotoId> {
        let mut ids: Vec<u32> = (0..u32::try_from(self.len()).unwrap_or(u32::MAX)).collect();
        ids.sort_by(|&a, &b| {
            let (a, b) = (a as usize, b as usize);
            self.effective(a).cmp(&self.effective(b)).then_with(|| {
                self.strings
                    .get(self.rel_path[a])
                    .cmp(self.strings.get(self.rel_path[b]))
            })
        });
        ids.into_iter().map(PhotoId).collect()
    }

    pub fn summary(&self, id: PhotoId) -> Option<PhotoSummary> {
        let i = self.index(id)?;
        let record = self.record(id)?;
        let folder_name = self.folders[self.folder[i] as usize]
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Some(PhotoSummary {
            id,
            file_name: record.file_name,
            folder_id: self.folder[i],
            folder_name,
            capture: record.capture.format("%Y-%m-%dT%H:%M:%S").to_string(),
            time_shifted: record.capture != record.camera_time,
            has_gps: record.flags & flags::HAS_GPS != 0,
            date_from_file: record.flags & flags::DATE_FROM_FILE != 0,
            camera: self
                .cameras
                .get(self.camera[i] as usize)
                .and_then(|(make, model)| Self::camera_label(make.as_deref(), model.as_deref())),
            rating: record.edits.rating,
            flag: record.edits.flag,
            rotation: record.edits.rotation % 4,
            queue: record.edits.queue,
        })
    }

    /// Photos kept by `filter`, in capture order (FR-003, FR-022).
    pub fn filtered_ids(&self, filter: &CullFilter) -> Vec<PhotoId> {
        let mut ids = self.sorted_ids();
        ids.retain(|&id| {
            (filter.folders.is_empty()
                || self
                    .folder_of(id)
                    .is_some_and(|f| filter.folders.contains(&f)))
                && filter.matches(&self.edits(id))
        });
        ids
    }

    /// Photos in the print basket and set aside (FR-050), for the sidebar.
    pub fn queue_counts(&self) -> (u32, u32) {
        self.edits.values().fold((0, 0), |(p, a), e| match e.queue {
            Some(Queue::Print) => (p + 1, a),
            Some(Queue::Aside) => (p, a + 1),
            None => (p, a),
        })
    }

    /// The source folder of a photo (FR-056).
    pub fn folder_of(&self, id: PhotoId) -> Option<u16> {
        self.index(id).map(|i| self.folder[i])
    }

    /// The folder's own name, as shown in the library list.
    pub fn folder_name(&self, folder: u16) -> Option<String> {
        let path = self.folders.get(usize::from(folder))?;
        Some(path.file_name().map_or_else(
            || path.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        ))
    }

    /// Every source folder with its photo count, in import order (FR-056).
    pub fn folder_summaries(&self) -> Vec<FolderSummary> {
        let mut counts = vec![0u32; self.folders.len()];
        for &f in &self.folder {
            if let Some(c) = counts.get_mut(usize::from(f)) {
                *c += 1;
            }
        }
        (0..self.folders.len())
            .filter_map(|i| {
                let id = u16::try_from(i).ok()?;
                Some(FolderSummary {
                    id,
                    name: self.folder_name(id)?,
                    count: counts[i],
                })
            })
            .collect()
    }

    pub fn summaries(&self, filter: &CullFilter) -> Vec<PhotoSummary> {
        self.filtered_ids(filter)
            .into_iter()
            .filter_map(|id| self.summary(id))
            .collect()
    }
}

/// The maker as people say it: EXIF writes the company ("NIKON CORPORATION", "RICOH IMAGING
/// COMPANY, LTD."), so trailing company words are dropped; a real two-word brand ("Phase
/// One", "OM Digital Solutions") is kept.
fn brand_name(make: &str) -> String {
    const COMPANY: &[&str] = &[
        "CORPORATION",
        "CORP",
        "COMPANY",
        "CO",
        "LTD",
        "INC",
        "IMAGING",
        "OPTICAL",
    ];
    let mut words: Vec<&str> = make.split_whitespace().collect();
    while words.len() > 1
        && words.last().is_some_and(|w| {
            let w = w.trim_matches(|c| c == ',' || c == '.');
            COMPANY.iter().any(|c| w.eq_ignore_ascii_case(c))
        })
    {
        words.pop();
    }
    words.join(" ").trim_end_matches(',').to_owned()
}

/// `text` begins with `word` as a whole word, ignoring case ("NIKON D750" / "Nikon").
fn starts_with_word(text: &str, word: &str) -> bool {
    text.get(..word.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(word))
        && text[word.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_040_camera_label_drops_company_words_and_repeats() {
        let label = |make: &str, model: &str| Library::camera_label(Some(make), Some(model));
        assert_eq!(
            label("NIKON CORPORATION", "NIKON D750").as_deref(),
            Some("NIKON D750")
        );
        assert_eq!(
            label("RICOH IMAGING COMPANY, LTD.", "RICOH GR III").as_deref(),
            Some("RICOH GR III")
        );
        assert_eq!(
            label("OLYMPUS CORPORATION", "E-M10 Mark III").as_deref(),
            Some("OLYMPUS E-M10 Mark III")
        );
        assert_eq!(
            label("Canon", "Canon EOS R10").as_deref(),
            Some("Canon EOS R10")
        );
        assert_eq!(
            label("Panasonic", "DC-GX9").as_deref(),
            Some("Panasonic DC-GX9")
        );
        assert_eq!(label("Phase One", "IQ4").as_deref(), Some("Phase One IQ4"));
        // "Nikon" is not a prefix word of "Nikonos V".
        assert_eq!(
            label("Nikon", "Nikonos V").as_deref(),
            Some("Nikon Nikonos V")
        );
        assert_eq!(
            Library::camera_label(Some("NIKON CORPORATION"), None).as_deref(),
            Some("NIKON")
        );
    }
    use chrono::NaiveDate;

    fn file(path: &str, mtime: i64) -> ImportedFile {
        ImportedFile {
            path: PathBuf::from(path),
            size: 10,
            mtime,
        }
    }

    fn meta_at(day: u32) -> PhotoMeta {
        PhotoMeta {
            capture: NaiveDate::from_ymd_opt(2026, 7, day).and_then(|d| d.and_hms_opt(10, 0, 0)),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    fn camera(make: &str, model: &str, serial: Option<&str>, h: u32, m: u32) -> PhotoMeta {
        PhotoMeta {
            capture: NaiveDate::from_ymd_opt(2026, 7, 1).and_then(|d| d.and_hms_opt(h, m, 0)),
            make: Some(make.into()),
            model: Some(model.into()),
            serial: serial.map(Into::into),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    #[test]
    fn fr_030_a_clock_offset_moves_the_time_sort_and_stamp_but_keeps_the_camera_time() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        // The GX9 clock is an hour behind: its 9:30 shot was really taken after the 10:00 one.
        let gx9 = lib.push(
            f,
            &file("/trip/gx9.jpg", 0),
            &camera("Panasonic", "DC-GX9", None, 9, 30),
        );
        let phone = lib.push(
            f,
            &file("/trip/phone.jpg", 0),
            &camera("Apple", "iPhone 15", None, 10, 0),
        );
        let (Some(gx9), Some(phone)) = (gx9, phone) else {
            panic!("import");
        };
        assert_eq!(lib.sorted_ids(), vec![gx9, phone]);

        let body = lib.clock_of(gx9).unwrap_or(u16::MAX);
        assert_eq!(lib.set_clock(body, 3_600, Some(120)), Ok(()));
        assert_eq!(lib.sorted_ids(), vec![phone, gx9]);
        let Some(record) = lib.record(gx9) else {
            panic!("record");
        };
        assert_eq!(record.capture.format("%H:%M").to_string(), "10:30");
        assert_eq!(record.camera_time.format("%H:%M").to_string(), "09:30");
        assert_eq!(record.stamp_date(), Some(record.capture));
        assert!(lib.summary(gx9).is_some_and(|s| s.time_shifted));
        assert!(lib.summary(phone).is_some_and(|s| !s.time_shifted));
    }

    #[test]
    fn fr_030_bodies_of_one_model_keep_their_own_clock_and_bad_values_are_refused() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        let a = lib.push(
            f,
            &file("/trip/a.jpg", 0),
            &camera("Sony", "ILCE-6400", Some("111"), 10, 0),
        );
        let b = lib.push(
            f,
            &file("/trip/b.jpg", 0),
            &camera("Sony", "ILCE-6400", Some("222"), 10, 0),
        );
        let (Some(a), Some(b)) = (a, b) else {
            panic!("import");
        };
        assert_ne!(lib.clock_of(a), lib.clock_of(b));
        assert_eq!(lib.clocks().len(), 2);
        assert_eq!(lib.cameras().len(), 1, "one frame profile for the model");

        let body = lib.clock_of(a).unwrap_or(u16::MAX);
        assert_eq!(
            lib.set_clock(body, MAX_CLOCK_OFFSET_S + 1, None),
            Err(ClockError::OffsetTooLarge)
        );
        assert_eq!(
            lib.set_clock(body, 0, Some(15 * 60)),
            Err(ClockError::BadTimeZone)
        );
        assert_eq!(lib.set_clock(999, 0, None), Err(ClockError::UnknownCamera));
        assert_eq!(lib.set_clock(body, -90, None), Ok(()));
        let times: Vec<String> = [a, b]
            .iter()
            .filter_map(|&id| lib.record(id))
            .map(|r| r.capture.format("%H:%M:%S").to_string())
            .collect();
        assert_eq!(times, vec!["09:58:30", "10:00:00"]);
    }

    #[test]
    fn fr_030_a_date_from_the_file_is_never_shifted() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        let meta = PhotoMeta {
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            orientation: 1,
            ..PhotoMeta::default()
        };
        let Some(id) = lib.push(f, &file("/trip/scan.jpg", 1_780_000_000), &meta) else {
            panic!("import");
        };
        let body = lib.clock_of(id).unwrap_or(u16::MAX);
        assert_eq!(lib.set_clock(body, 3_600, None), Ok(()));
        assert!(lib.record(id).is_some_and(|r| r.capture == r.camera_time));
    }

    #[test]
    fn fr_001_rescanning_a_folder_adds_new_photos_and_keeps_edits() {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/trip"));
        let Some(a) = lib.push(folder, &file("/trip/a.jpg", 1), &meta_at(1)) else {
            panic!("first import");
        };
        lib.set_edits(
            a,
            PhotoEdits {
                rating: 5,
                ..PhotoEdits::default()
            },
        );
        // Refresh: the same folder again, now with a new photo.
        let again = lib.add_folder(Path::new("/trip"));
        assert_eq!(again, folder, "the folder is not added twice");
        assert_eq!(lib.push(again, &file("/trip/a.jpg", 1), &meta_at(1)), None);
        assert!(
            lib.push(again, &file("/trip/b.jpg", 2), &meta_at(2))
                .is_some()
        );
        assert_eq!(lib.len(), 2);
        assert_eq!(lib.edits(a).rating, 5, "edits survive a refresh");
    }

    #[test]
    fn fr_003_sorts_by_capture_time_and_dedupes() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        assert!(lib.push(f, &file("/trip/b.jpg", 0), &meta_at(15)).is_some());
        assert!(lib.push(f, &file("/trip/a.jpg", 0), &meta_at(14)).is_some());
        assert!(
            lib.push(f, &file("/trip/a.jpg", 0), &meta_at(14)).is_none(),
            "duplicate"
        );
        let names: Vec<_> = lib
            .summaries(&CullFilter::default())
            .into_iter()
            .map(|s| s.file_name)
            .collect();
        assert_eq!(names, ["a.jpg", "b.jpg"]);
    }

    #[test]
    fn fr_023_path_key_ignores_separator_style() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        let id = lib.push(f, &file("/trip/sub/a.jpg", 0), &meta_at(14));
        // Rebuilt the way project restore does: folder joined with a `/` relative path.
        assert_eq!(lib.find_path(&Path::new("/trip").join("sub/a.jpg")), id);
        assert_eq!(lib.find_path(Path::new("/trip//sub/./a.jpg")), id);
        #[cfg(windows)]
        assert_eq!(lib.find_path(Path::new(r"\trip\sub\a.jpg")), id);
    }

    #[test]
    fn fr_002_missing_date_falls_back_to_file_time() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/scans"));
        let id = lib.push(
            f,
            &file("/scans/x.jpg", 1_700_000_000),
            &PhotoMeta::default(),
        );
        let summary = id.and_then(|id| lib.summary(id));
        assert!(summary.is_some_and(|s| s.date_from_file && s.capture.starts_with("2023-11-14")));
    }

    #[test]
    fn fr_004_photos_without_exif_are_stamped_with_the_file_date() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/scans"));
        let dated = lib.push(
            f,
            &file("/scans/a.jpg", 1_700_000_000),
            &PhotoMeta::default(),
        );
        let undated = lib.push(f, &file("/scans/b.jpg", 0), &PhotoMeta::default());
        let exif = lib.push(f, &file("/scans/c.jpg", 0), &meta_at(14));
        let date = |id: Option<PhotoId>| {
            id.and_then(|id| lib.record(id))
                .and_then(|r| r.stamp_date())
        };
        assert!(date(dated).is_some_and(|d| d.format("%Y-%m-%d").to_string() == "2023-11-14"));
        assert_eq!(date(undated), None);
        assert!(date(exif).is_some_and(|d| d.format("%d").to_string() == "14"));
    }

    #[test]
    fn fr_022_filter_matches_flags_and_rating() {
        let edits = |rating, flag| PhotoEdits {
            rating,
            flag,
            ..PhotoEdits::default()
        };
        let pick3 = edits(3, Some(Flag::Pick));
        let reject = edits(0, Some(Flag::Reject));
        let plain = edits(1, None);
        let only = |flag, min_rating| CullFilter {
            flag,
            min_rating,
            ..CullFilter::default()
        };

        assert!(
            [&pick3, &reject, &plain]
                .into_iter()
                .all(|e| CullFilter::default().matches(e))
        );
        assert!(only(FlagFilter::Picks, 0).matches(&pick3));
        assert!(!only(FlagFilter::Picks, 0).matches(&plain));
        assert!(only(FlagFilter::Rejects, 0).matches(&reject));
        assert!(only(FlagFilter::Unflagged, 0).matches(&plain));
        assert!(!only(FlagFilter::Unflagged, 0).matches(&pick3));
        assert!(only(FlagFilter::All, 3).matches(&pick3));
        assert!(!only(FlagFilter::All, 2).matches(&plain));
        // Out-of-range minimums clamp to 5 stars instead of hiding everything.
        assert!(only(FlagFilter::All, 9).matches(&edits(5, None)));
    }

    #[test]
    fn fr_022_rating_and_flag_are_undoable() {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/trip"));
        let a = lib.push(folder, &file("/trip/a.jpg", 1), &meta_at(1));
        let b = lib.push(folder, &file("/trip/b.jpg", 2), &meta_at(2));
        let (Some(a), Some(b)) = (a, b) else {
            panic!("both photos are new");
        };
        let mut history = crate::history::History::default();
        let picked = PhotoEdits {
            rating: 4,
            flag: Some(Flag::Pick),
            ..lib.edits(a)
        };
        history.apply(&mut lib, &[(a, picked)]);
        let picks = CullFilter {
            flag: FlagFilter::Picks,
            ..CullFilter::default()
        };
        assert_eq!(lib.filtered_ids(&picks), [a]);
        assert_eq!(
            lib.summary(a).map(|s| (s.rating, s.flag)),
            Some((4, Some(Flag::Pick)))
        );

        history.undo(&mut lib);
        assert_eq!(lib.filtered_ids(&picks), []);
        assert_eq!(lib.filtered_ids(&CullFilter::default()), [a, b]);
        history.redo(&mut lib);
        assert_eq!(lib.filtered_ids(&picks), [a]);
    }

    #[test]
    fn fr_056_folders_scope_the_library_like_tabs() {
        let mut lib = Library::new();
        let trip = lib.add_folder(Path::new("/photos/Trip"));
        let home = lib.add_folder(Path::new("/photos/Home"));
        let a = lib.push(trip, &file("/photos/Trip/a.jpg", 1), &meta_at(1));
        let b = lib.push(home, &file("/photos/Home/b.jpg", 2), &meta_at(2));
        let c = lib.push(trip, &file("/photos/Trip/c.jpg", 3), &meta_at(3));
        let (Some(a), Some(b), Some(c)) = (a, b, c) else {
            panic!("three new photos");
        };
        let scope = |folders: Vec<u16>| CullFilter {
            folders,
            ..CullFilter::default()
        };
        assert_eq!(lib.filtered_ids(&scope(vec![])), [a, b, c]);
        assert_eq!(lib.filtered_ids(&scope(vec![trip])), [a, c]);
        assert_eq!(lib.filtered_ids(&scope(vec![home, trip])), [a, b, c]);
        assert_eq!(lib.filtered_ids(&scope(vec![99])), []);
        // Counts ignore the filter, so a folder never disappears from the list.
        let names: Vec<_> = lib
            .folder_summaries()
            .into_iter()
            .map(|f| (f.name, f.count))
            .collect();
        assert_eq!(names, [("Trip".to_owned(), 2), ("Home".to_owned(), 1)]);
        assert_eq!(lib.folder_of(b), Some(home));
        // Older callers without `folders` still mean "every folder".
        let old: CullFilter = serde_json::from_str(r#"{"flag":"all","minRating":0}"#)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(old.folders, Vec::<u16>::new());
    }

    #[test]
    fn fr_050_the_basket_and_set_aside_photos_have_their_own_views() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/trip"));
        let ids: Vec<PhotoId> = (1..=4)
            .filter_map(|i| {
                lib.push(
                    f,
                    &file(&format!("/trip/{i}.jpg"), i),
                    &meta_at(u32::try_from(i).unwrap_or(1)),
                )
            })
            .collect();
        let queued = |q| PhotoEdits {
            queue: Some(q),
            ..PhotoEdits::default()
        };
        lib.set_edits(ids[0], queued(Queue::Print));
        lib.set_edits(ids[1], queued(Queue::Aside));
        let view = |queue| CullFilter {
            queue,
            ..CullFilter::default()
        };
        // The gallery hides set-aside photos but keeps the basket's.
        assert_eq!(
            lib.filtered_ids(&view(QueueView::Gallery)),
            [ids[0], ids[2], ids[3]]
        );
        assert_eq!(lib.filtered_ids(&view(QueueView::Print)), [ids[0]]);
        assert_eq!(lib.filtered_ids(&view(QueueView::Aside)), [ids[1]]);
        assert_eq!(lib.queue_counts(), (1, 1));
        assert_eq!(
            lib.summary(ids[0]).and_then(|s| s.queue),
            Some(Queue::Print)
        );
        // Older callers without `queue` see the gallery.
        let old: CullFilter = serde_json::from_str(r#"{"flag":"all","minRating":0}"#)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(old.queue, QueueView::Gallery);
    }
}
