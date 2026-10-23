// SPDX-License-Identifier: GPL-3.0-only
//! The project file (`.fotollection`, FR-023, ADR-0010): normalized tables of folders and
//! sparse photo rows (DATABASE.md §4). Sources stay read-only; this file holds the work.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::library::{Flag, Library, PhotoEdits};
use crate::plan::CropFrame;

pub const FORMAT: &str = "fotollection-project";
pub const SCHEMA_VERSION: u32 = 1;
const MAX_BYTES: u64 = 64 * 1024 * 1024;

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
    pub path: String,
    pub path_kind: PathKind,
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
    }
}

pub fn save(path: &Path, file: &ProjectFile) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(file).map_err(std::io::Error::other)?;
    crate::atomic::write_with_backup(path, &bytes)
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
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    let header: Header = match serde_json::from_slice(&bytes) {
        Ok(h) => h,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    if header.format != FORMAT {
        return Loaded::Corrupt("not a Fotollection project".into());
    }
    if header.schema_version > SCHEMA_VERSION {
        return Loaded::NewerVersion(header.schema_version);
    }
    match serde_json::from_slice::<ProjectFile>(&bytes) {
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
    }
    Ok(file)
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
    let mut summary = RestoreSummary::default();
    for row in &file.photos {
        let Some(folder) = file.folders.iter().find(|f| f.id == row.folder) else {
            continue;
        };
        let full = Path::new(&folder.path).join(&row.path);
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
        assert_eq!(restored.and_then(|e| e.crop), Some(crop()));
        assert_eq!(restored.map(|e| e.rating), Some(4));
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
}
