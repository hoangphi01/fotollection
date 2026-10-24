// SPDX-License-Identifier: GPL-3.0-only
//! The project file (`.fotollection`, FR-023, ADR-0010): normalized tables of folders and
//! sparse photo rows (DATABASE.md §4). Sources stay read-only; this file holds the work.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::library::{Flag, Library, PhotoEdits};
use crate::plan::CropFrame;

pub const FORMAT: &str = "fotollection-project";
/// v2: `folders[].absolute`, the fallback for folders stored relative to a bundle (ADR-0016).
/// v3: `photos[].stamp`, a photo's own date stamp set in the preview (FR-044).
/// v4: `photos[].rotation`, the user's quarter turns clockwise (FR-006).
/// v5: `photos[].straighten`, degrees within ±15 (FR-006).
pub const SCHEMA_VERSION: u32 = 5;
pub const MAX_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PathKind {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FolderRow {
    pub id: u32,
    /// Absolute, or relative to the project file's directory with `/` separators.
    pub path: String,
    pub path_kind: PathKind,
    /// For a relative row, the absolute path when it was saved: used when the project file
    /// moved without its photos. Absent in v1 files (read with this default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absolute: Option<String>,
}

/// A photo with user state. Only edited photos have rows (N5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhotoRow {
    pub id: u32,
    pub folder: u32,
    /// Relative to its folder, `/`-separated.
    pub path: String,
    pub fp_size: u64,
    pub fp_mtime: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<CropFrame>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<Flag>,
    /// The photo's own stamp (v3); absent when it follows the settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp: Option<crate::settings::StampSettings>,
    /// Quarter turns clockwise, 1–3 (v4); absent when the photo is not turned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<u8>,
    /// Straightening in degrees, ±15 (v5); absent when level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub straighten: Option<f32>,
}

/// The document a session belongs to (FR-023). Only written into the session autosave, so
/// a restart reopens "Trip.fotollection — Edited" instead of an untitled session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRef {
    pub path: String,
    /// True when the session holds changes that were not saved into the document.
    pub edited: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFile {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub modified_at: String,
    pub folders: Vec<FolderRow>,
    pub photos: Vec<PhotoRow>,
    /// Session autosave only (schema v2); never inside a bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentRef>,
}

#[derive(Debug, PartialEq)]
pub enum Loaded {
    Ok(ProjectFile),
    Missing,
    /// Written by a newer app: open nothing and never overwrite it (NFR-013).
    NewerVersion(u32),
    Corrupt(String),
}

/// Builds the file content from the library: every folder, and rows for edited photos.
pub fn snapshot(library: &Library) -> ProjectFile {
    let folders = library
        .folders()
        .iter()
        .enumerate()
        .map(|(i, p)| FolderRow {
            id: u32::try_from(i + 1).unwrap_or(u32::MAX),
            path: p.display().to_string(),
            path_kind: PathKind::Absolute,
            absolute: None,
        })
        .collect();
    let mut photos: Vec<PhotoRow> = library
        .edited()
        .filter_map(|(id, edits)| {
            let (folder, rel) = library.location(id)?;
            let record = library.record(id)?;
            Some(PhotoRow {
                id: 0,
                folder: u32::from(folder) + 1,
                path: rel.replace('\\', "/"),
                fp_size: record.size,
                fp_mtime: record.mtime,
                crop: edits.crop,
                rating: (edits.rating > 0).then_some(edits.rating),
                flag: edits.flag,
                stamp: edits.stamp,
                rotation: (edits.rotation % 4 != 0).then_some(edits.rotation % 4),
                straighten: crate::straighten::is_straightened(edits.straighten)
                    .then(|| crate::straighten::sanitize(edits.straighten)),
            })
        })
        .collect();
    // Stable order keeps diffs small (DATABASE.md §4.4).
    photos.sort_by(|a, b| (a.folder, &a.path).cmp(&(b.folder, &b.path)));
    for (i, row) in photos.iter_mut().enumerate() {
        row.id = u32::try_from(i + 1).unwrap_or(u32::MAX);
    }
    ProjectFile {
        format: FORMAT.to_owned(),
        schema_version: SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        modified_at: chrono::Utc::now().to_rfc3339(),
        folders,
        photos,
        document: None,
    }
}

pub fn to_bytes(file: &ProjectFile) -> std::io::Result<Vec<u8>> {
    serde_json::to_vec_pretty(file).map_err(std::io::Error::other)
}

pub fn save(path: &Path, file: &ProjectFile) -> std::io::Result<()> {
    crate::atomic::write_with_backup(path, &to_bytes(file)?)
}

#[derive(Deserialize)]
struct Header {
    format: String,
    schema_version: u32,
}

pub fn load(path: &Path) -> Loaded {
    let Ok(meta) = std::fs::metadata(path) else {
        return Loaded::Missing;
    };
    if meta.len() > MAX_BYTES {
        return Loaded::Corrupt("project file is too large".into());
    }
    match std::fs::read(path) {
        Ok(bytes) => from_bytes(&bytes),
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

/// Parses and validates a project file already in memory (also a bundle part, ADR-0016).
pub fn from_bytes(bytes: &[u8]) -> Loaded {
    if bytes.len() as u64 > MAX_BYTES {
        return Loaded::Corrupt("project file is too large".into());
    }
    let header: Header = match serde_json::from_slice(bytes) {
        Ok(h) => h,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    if header.format != FORMAT {
        return Loaded::Corrupt("not a Fotollection project".into());
    }
    if header.schema_version > SCHEMA_VERSION {
        return Loaded::NewerVersion(header.schema_version);
    }
    match serde_json::from_slice::<ProjectFile>(bytes) {
        Ok(file) => validate(file).map_or_else(Loaded::Corrupt, Loaded::Ok),
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

/// Keys unique, foreign keys resolvable (N7), values in range.
fn validate(mut file: ProjectFile) -> Result<ProjectFile, String> {
    let mut folder_ids = HashSet::new();
    for f in &file.folders {
        if !folder_ids.insert(f.id) {
            return Err(format!("duplicate folder {}", f.id));
        }
    }
    let mut ids = HashSet::new();
    let mut keys = HashSet::new();
    for row in &mut file.photos {
        if !ids.insert(row.id) || !keys.insert((row.folder, row.path.clone())) {
            return Err(format!("duplicate photo row {}", row.id));
        }
        if !folder_ids.contains(&row.folder) {
            return Err(format!(
                "photo {} points to missing folder {}",
                row.id, row.folder
            ));
        }
        if row.path.split('/').any(|seg| seg == "..") {
            return Err(format!("photo {} has an invalid path", row.id));
        }
        row.crop = row.crop.map(CropFrame::sanitized);
        row.rating = row.rating.map(|r| r.min(5));
        if let Some(stamp) = &mut row.stamp {
            stamp.sanitize();
        }
        row.rotation = row.rotation.map(|r| r % 4).filter(|&r| r != 0);
        row.straighten = row
            .straighten
            .map(crate::straighten::sanitize)
            .filter(|&d| crate::straighten::is_straightened(d));
    }
    Ok(file)
}

/// `target` relative to the directory `base`, with `/` separators (`../Photos/Trip`).
/// `None` when they share no root, for example two Windows drives.
pub fn relative_path(base: &Path, target: &Path) -> Option<String> {
    use std::path::Component;
    let b: Vec<Component> = base.components().collect();
    let t: Vec<Component> = target.components().collect();
    if b.first() != t.first()
        || !matches!(b.first(), Some(Component::Prefix(_) | Component::RootDir))
    {
        return None;
    }
    let common = b.iter().zip(&t).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = std::iter::repeat_n("..".to_owned(), b.len() - common).collect();
    parts.extend(
        t[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    Some(if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    })
}

/// Stores every folder relative to `base` (the directory of the project file) when
/// possible, keeping the absolute path as a fallback, so a project copied together with
/// its photos opens anywhere (FR-023).
pub fn relocate(file: &mut ProjectFile, base: &Path) {
    for folder in &mut file.folders {
        if folder.path_kind != PathKind::Absolute {
            continue;
        }
        if let Some(rel) = relative_path(base, Path::new(&folder.path)) {
            folder.absolute = Some(std::mem::replace(&mut folder.path, rel));
            folder.path_kind = PathKind::Relative;
        }
    }
}

/// Where a folder of the project is on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFolder {
    pub id: u32,
    /// The folder as it was saved, for messages and the Relink dialog.
    pub saved: String,
    /// `None` when neither the relative nor the absolute path exists here.
    pub path: Option<PathBuf>,
}

/// Finds every folder: relative to `base` first, then the absolute path (FR-023).
pub fn resolve_folders(file: &ProjectFile, base: Option<&Path>) -> Vec<ResolvedFolder> {
    file.folders
        .iter()
        .map(|f| {
            let mut candidates = Vec::new();
            match f.path_kind {
                PathKind::Relative => {
                    if let Some(base) = base {
                        candidates.push(base.join(&f.path));
                    }
                    candidates.extend(f.absolute.as_ref().map(PathBuf::from));
                }
                PathKind::Absolute => candidates.push(PathBuf::from(&f.path)),
            }
            ResolvedFolder {
                id: f.id,
                saved: f.absolute.clone().unwrap_or_else(|| f.path.clone()),
                path: candidates.into_iter().find(|p| p.is_dir()),
            }
        })
        .collect()
}

/// Folder paths to re-scan when the project opens.
pub fn folder_paths(file: &ProjectFile) -> Vec<PathBuf> {
    file.folders
        .iter()
        .map(|f| PathBuf::from(&f.path))
        .collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RestoreSummary {
    pub restored: u32,
    /// Rows whose file is gone.
    pub missing: u32,
    /// Rows whose file changed size or time since the project was saved.
    pub changed: u32,
}

/// Re-applies saved edits to a freshly scanned library, matching by folder + path.
pub fn restore(library: &mut Library, file: &ProjectFile) -> RestoreSummary {
    let folders: Vec<(u32, PathBuf)> = file
        .folders
        .iter()
        .map(|f| (f.id, PathBuf::from(&f.path)))
        .collect();
    restore_into(library, file, &folders)
}

/// Like [`restore`], with each folder id mapped to where it is on this machine (after
/// [`resolve_folders`] or a relink). Rows of folders not in `folders` are skipped.
pub fn restore_into(
    library: &mut Library,
    file: &ProjectFile,
    folders: &[(u32, PathBuf)],
) -> RestoreSummary {
    let mut summary = RestoreSummary::default();
    for row in &file.photos {
        let Some((_, folder)) = folders.iter().find(|(id, _)| *id == row.folder) else {
            continue;
        };
        let full = folder.join(&row.path);
        let Some(id) = library.find_path(&full) else {
            summary.missing += 1;
            continue;
        };
        if let Some(record) = library.record(id)
            && (record.size != row.fp_size || record.mtime != row.fp_mtime)
        {
            summary.changed += 1;
        }
        library.set_edits(
            id,
            PhotoEdits {
                crop: row.crop,
                rating: row.rating.unwrap_or(0),
                flag: row.flag,
                stamp: row.stamp.clone(),
                rotation: row.rotation.unwrap_or(0),
                straighten: row.straighten.unwrap_or(0.0),
            },
        );
        summary.restored += 1;
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;

    fn library_with(paths: &[&str]) -> Library {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/trip"));
        for p in paths {
            let file = ImportedFile {
                path: PathBuf::from(p),
                size: 100,
                mtime: 1_700_000_000,
            };
            lib.push(folder, &file, &PhotoMeta::default());
        }
        lib
    }

    fn crop() -> CropFrame {
        CropFrame {
            cx: 0.3,
            cy: 0.6,
            zoom: 1.5,
        }
    }

    #[test]
    fn fr_023_round_trip_restores_edits_by_path() -> std::io::Result<()> {
        let mut lib = library_with(&["/trip/a.jpg", "/trip/b.jpg"]);
        let b = lib.find_path(Path::new("/trip/b.jpg"));
        if let Some(b) = b {
            lib.set_edits(
                b,
                PhotoEdits {
                    crop: Some(crop()),
                    rating: 4,
                    flag: Some(Flag::Pick),
                    stamp: Some(crate::settings::StampSettings {
                        caption: "Lisbon".into(),
                        ..crate::settings::RenderSettings::default().stamp
                    }),
                    rotation: 1,
                    straighten: -3.5,
                },
            );
        }
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("session.fotollection");
        save(&path, &snapshot(&lib))?;
        save(&path, &snapshot(&lib))?;
        assert!(dir.path().join("session.fotollection.bak").exists());

        let Loaded::Ok(file) = load(&path) else {
            panic!("project should load");
        };
        assert_eq!(file.photos.len(), 1, "only edited photos have rows");
        // A new session: same files scanned again, possibly in another order.
        let mut fresh = library_with(&["/trip/b.jpg", "/trip/a.jpg"]);
        let summary = restore(&mut fresh, &file);
        assert_eq!(
            (summary.restored, summary.missing, summary.changed),
            (1, 0, 0)
        );
        let restored = fresh
            .find_path(Path::new("/trip/b.jpg"))
            .map(|id| fresh.edits(id));
        assert_eq!(restored.as_ref().and_then(|e| e.crop), Some(crop()));
        assert_eq!(restored.as_ref().map(|e| e.rating), Some(4));
        assert_eq!(
            restored.and_then(|e| e.stamp).map(|s| s.caption),
            Some("Lisbon".into()),
            "a photo's own stamp is kept (FR-044)"
        );
        let turned = fresh
            .find_path(Path::new("/trip/b.jpg"))
            .map(|id| fresh.edits(id).rotation);
        assert_eq!(turned, Some(1), "the quarter turn is kept (FR-006)");
        let level = fresh
            .find_path(Path::new("/trip/b.jpg"))
            .map(|id| fresh.edits(id).straighten);
        assert_eq!(level, Some(-3.5), "the straightening is kept (FR-006)");
        Ok(())
    }

    #[test]
    fn fr_023_missing_and_changed_files_are_reported() {
        let lib = {
            let mut lib = library_with(&["/trip/a.jpg"]);
            if let Some(a) = lib.find_path(Path::new("/trip/a.jpg")) {
                lib.set_edits(
                    a,
                    PhotoEdits {
                        rating: 2,
                        ..PhotoEdits::default()
                    },
                );
            }
            lib
        };
        let mut file = snapshot(&lib);
        file.photos[0].fp_size = 999;
        let mut extra = file.photos[0].clone();
        extra.id = 2;
        extra.path = "gone.jpg".into();
        file.photos.push(extra);
        let mut fresh = library_with(&["/trip/a.jpg"]);
        let summary = restore(&mut fresh, &file);
        assert_eq!(
            (summary.restored, summary.missing, summary.changed),
            (1, 1, 1)
        );
    }

    #[test]
    fn fr_023_rejects_newer_and_invalid_files() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("p.fotollection");
        std::fs::write(
            &path,
            br#"{"format":"fotollection-project","schema_version":7}"#,
        )?;
        assert_eq!(load(&path), Loaded::NewerVersion(7));

        let mut file = snapshot(&library_with(&["/trip/a.jpg"]));
        file.photos.push(PhotoRow {
            id: 1,
            folder: 42,
            path: "x.jpg".into(),
            fp_size: 1,
            fp_mtime: 1,
            crop: None,
            rating: None,
            flag: None,
            stamp: None,
            rotation: None,
            straighten: None,
        });
        save(&path, &file)?;
        assert!(
            matches!(load(&path), Loaded::Corrupt(_)),
            "dangling folder FK"
        );

        file.photos[0].folder = 1;
        file.photos[0].path = "../../etc/passwd".into();
        save(&path, &file)?;
        assert!(matches!(load(&path), Loaded::Corrupt(_)), "path traversal");
        Ok(())
    }

    #[test]
    fn fr_044_v3_fixture_keeps_a_photos_own_stamp() {
        let Loaded::Ok(file) =
            from_bytes(include_bytes!("../tests/fixtures/project/v3.fotollection"))
        else {
            panic!("v3 project should load");
        };
        let Some(stamp) = &file.photos[0].stamp else {
            panic!("photo 1 has its own stamp");
        };
        assert!(stamp.place);
        // Control characters from a file are dropped on load.
        assert_eq!(stamp.caption, "Summer by the lake");
        assert_eq!(file.photos[1].stamp, None);
    }

    #[test]
    fn fr_006_v5_fixture_keeps_straightening_within_limits() {
        let Loaded::Ok(file) =
            from_bytes(include_bytes!("../tests/fixtures/project/v5.fotollection"))
        else {
            panic!("v5 project should load");
        };
        let degrees: Vec<Option<f32>> = file.photos.iter().map(|p| p.straighten).collect();
        // 40° from a file is clamped to 15°.
        assert_eq!(degrees, [Some(2.5), None, Some(15.0)]);
    }

    #[test]
    fn fr_006_v4_fixture_keeps_quarter_turns() {
        let Loaded::Ok(file) =
            from_bytes(include_bytes!("../tests/fixtures/project/v4.fotollection"))
        else {
            panic!("v4 project should load");
        };
        let turns: Vec<Option<u8>> = file.photos.iter().map(|p| p.rotation).collect();
        // 7 quarter turns from a file are the same as 3.
        assert_eq!(turns, [None, Some(2), Some(3)]);
    }

    #[test]
    fn fr_023_v1_fixture_still_opens() {
        let Loaded::Ok(file) =
            from_bytes(include_bytes!("../tests/fixtures/project/v1.fotollection"))
        else {
            panic!("v1 project should load");
        };
        assert_eq!(
            file.folders[0].absolute, None,
            "v1 rows have no absolute hint"
        );
        assert_eq!(file.photos.len(), 2);
        assert_eq!(file.photos[0].rating, Some(4));
        assert_eq!(file.photos[1].flag, Some(Flag::Reject));
    }

    #[test]
    fn fr_023_relative_paths() {
        let rel = |b: &str, t: &str| relative_path(Path::new(b), Path::new(t));
        assert_eq!(rel("/a/b", "/a/b/c/d").as_deref(), Some("c/d"));
        assert_eq!(
            rel("/a/b/projects", "/a/photos/trip").as_deref(),
            Some("../../photos/trip")
        );
        assert_eq!(rel("/a/b", "/a/b").as_deref(), Some("."));
        assert_eq!(rel("relative/base", "/a").as_deref(), None);
        #[cfg(windows)]
        assert_eq!(rel(r"C:\Users\me", r"D:\Photos").as_deref(), None);
    }

    #[test]
    fn fr_023_relocated_folders_resolve_relative_first_then_absolute() -> std::io::Result<()> {
        let root = tempfile::tempdir()?;
        let photos = root.path().join("photos/trip");
        std::fs::create_dir_all(&photos)?;
        let mut lib = Library::new();
        lib.add_folder(&photos);
        let mut file = snapshot(&lib);
        relocate(&mut file, root.path());
        assert_eq!(file.folders[0].path_kind, PathKind::Relative);
        assert_eq!(file.folders[0].path, "photos/trip");
        assert_eq!(file.folders[0].absolute.as_deref(), photos.to_str());

        // Same machine: found relative to the project.
        let found = resolve_folders(&file, Some(root.path()));
        assert_eq!(found[0].path.as_deref(), Some(photos.as_path()));

        // The project and its photos copied elsewhere together: still relative.
        let other = tempfile::tempdir()?;
        std::fs::create_dir_all(other.path().join("photos/trip"))?;
        let found = resolve_folders(&file, Some(other.path()));
        assert_eq!(found[0].path, Some(other.path().join("photos/trip")));

        // Only the project moved: falls back to the absolute path.
        let lonely = tempfile::tempdir()?;
        let found = resolve_folders(&file, Some(lonely.path()));
        assert_eq!(found[0].path.as_deref(), Some(photos.as_path()));

        // Neither exists: reported for the Relink dialog.
        std::fs::remove_dir_all(&photos)?;
        let found = resolve_folders(&file, Some(lonely.path()));
        assert_eq!(found[0].path, None);
        assert_eq!(Some(found[0].saved.as_str()), photos.to_str());
        Ok(())
    }
}
