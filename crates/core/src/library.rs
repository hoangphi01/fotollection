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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CullFilter {
    pub flag: FlagFilter,
    /// Minimum stars, 0–5. Values above 5 are treated as 5.
    pub min_rating: u8,
}

impl CullFilter {
    pub fn matches(&self, edits: &PhotoEdits) -> bool {
        let flag_ok = match self.flag {
            FlagFilter::All => true,
            FlagFilter::Picks => edits.flag == Some(Flag::Pick),
            FlagFilter::Unflagged => edits.flag.is_none(),
            FlagFilter::Rejects => edits.flag == Some(Flag::Reject),
        };
        flag_ok && edits.rating >= self.min_rating.min(5)
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
    pub folder_name: String,
    /// Camera-local capture time, `YYYY-MM-DDTHH:MM:SS`.
    pub capture: String,
    pub has_gps: bool,
    pub date_from_file: bool,
    pub camera: Option<String>,
    /// 0–5 stars (FR-022).
    pub rating: u8,
    pub flag: Option<Flag>,
}

#[derive(Default)]
pub struct Library {
    folders: Vec<PathBuf>,
    strings: StrArena,
    /// Distinct `(make, model)` pairs; photos reference them by index (N2/N3).
    cameras: Vec<(Option<String>, Option<String>)>,
    /// Distinct lens names, referenced by index.
    lenses: Vec<String>,
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
    lens: Vec<u16>,
    focal_mm: Vec<f32>,
    f_number: Vec<f32>,
    exposure_s: Vec<f32>,
    iso: Vec<u32>,
}

const NONE_U16: u16 = u16::MAX;

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
        match (make, model) {
            (_, None) => make.map(str::to_owned),
            (Some(make), Some(model)) if !model.starts_with(make) => {
                Some(format!("{make} {model}"))
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
            capture: DateTime::from_timestamp(self.capture[i], 0)?.naive_utc(),
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
            self.capture[a].cmp(&self.capture[b]).then_with(|| {
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
            folder_name,
            capture: record.capture.format("%Y-%m-%dT%H:%M:%S").to_string(),
            has_gps: record.flags & flags::HAS_GPS != 0,
            date_from_file: record.flags & flags::DATE_FROM_FILE != 0,
            camera: self
                .cameras
                .get(self.camera[i] as usize)
                .and_then(|(make, model)| Self::camera_label(make.as_deref(), model.as_deref())),
            rating: record.edits.rating,
            flag: record.edits.flag,
        })
    }

    /// Photos kept by `filter`, in capture order (FR-003, FR-022).
    pub fn filtered_ids(&self, filter: &CullFilter) -> Vec<PhotoId> {
        let mut ids = self.sorted_ids();
        ids.retain(|&id| filter.matches(&self.edits(id)));
        ids
    }

    pub fn summaries(&self, filter: &CullFilter) -> Vec<PhotoSummary> {
        self.filtered_ids(filter)
            .into_iter()
            .filter_map(|id| self.summary(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn fr_022_filter_matches_flags_and_rating() {
        let edits = |rating, flag| PhotoEdits {
            rating,
            flag,
            ..PhotoEdits::default()
        };
        let pick3 = edits(3, Some(Flag::Pick));
        let reject = edits(0, Some(Flag::Reject));
        let plain = edits(1, None);
        let only = |flag, min_rating| CullFilter { flag, min_rating };

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
            min_rating: 0,
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
}
