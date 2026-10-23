// SPDX-License-Identifier: GPL-3.0-only
//! `settings.json`: app-wide settings and camera profiles (DATABASE.md §6). Versioned,
//! validated on load, written atomically, and never overwritten by an older app.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::frame::{CameraProfile, FrameSettings};
use crate::settings::RenderSettings;

pub const FORMAT: &str = "fotollection-settings";
pub const SCHEMA_VERSION: u32 = 1;
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Everything persisted in `settings.json`. `cameras` is a table keyed by `id`, with
/// `(make, model)` unique (normalization rules N1–N3).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSettings {
    pub render: RenderSettings,
    pub frame: FrameSettings,
    pub cameras: Vec<CameraProfile>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileV1 {
    format: String,
    schema_version: u32,
    render: RenderSettings,
    frame: FrameSettings,
    cameras: Vec<CameraProfile>,
}

#[derive(Debug, PartialEq)]
pub enum Loaded {
    /// The file was read and validated.
    Ok(StoredSettings),
    /// No file yet: defaults.
    Missing,
    /// Written by a newer app. Defaults are used and the file must not be overwritten.
    NewerVersion(u32),
    /// Unreadable or invalid. Defaults are used; the bad file is kept as `.corrupt`.
    Corrupt(String),
}

/// Just enough of the file to check its format and version.
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
        return Loaded::Corrupt("settings file is too large".into());
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    // Check the version before the strict parse, so newer files are recognised.
    let header: Header = match serde_json::from_slice(&bytes) {
        Ok(h) => h,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    if header.format != FORMAT {
        return Loaded::Corrupt("not a Fotollection settings file".into());
    }
    if header.schema_version > SCHEMA_VERSION {
        return Loaded::NewerVersion(header.schema_version);
    }
    match serde_json::from_slice::<FileV1>(&bytes) {
        Ok(file) => match validate(file) {
            Ok(settings) => Loaded::Ok(settings),
            Err(e) => Loaded::Corrupt(e),
        },
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

fn validate(file: FileV1) -> Result<StoredSettings, String> {
    file.render.validate().map_err(|e| e.to_string())?;
    let mut ids = std::collections::HashSet::new();
    let mut keys = std::collections::HashSet::new();
    let mut cameras = Vec::with_capacity(file.cameras.len());
    for camera in file.cameras {
        if !ids.insert(camera.id) || !keys.insert((camera.make.clone(), camera.model.clone())) {
            return Err(format!("duplicate camera {}", camera.id));
        }
        cameras.push(camera.sanitized());
    }
    Ok(StoredSettings {
        render: file.render,
        frame: file.frame,
        cameras,
    })
}

/// Moves an unreadable file aside so it is never silently lost.
pub fn quarantine(path: &Path) -> std::io::Result<PathBuf> {
    let target = path.with_extension("json.corrupt");
    std::fs::rename(path, &target)?;
    Ok(target)
}

/// Writes atomically: temp file + fsync + rename, keeping the previous file as `.bak`.
pub fn save(path: &Path, settings: &StoredSettings) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = FileV1 {
        format: FORMAT.to_owned(),
        schema_version: SCHEMA_VERSION,
        render: settings.render.clone(),
        frame: settings.frame.clone(),
        cameras: settings.cameras.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(std::io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    if path.exists() {
        std::fs::copy(path, path.with_extension("json.bak"))?;
    }
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> StoredSettings {
        let mut profile = CameraProfile::new(1, Some("Panasonic".into()), Some("DC-GX9".into()));
        profile.lens = Some("Industar-50-2".into());
        profile.focal_mm = Some(50.0);
        StoredSettings {
            cameras: vec![profile],
            ..StoredSettings::default()
        }
    }

    #[test]
    fn round_trip_and_backup() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        assert_eq!(load(&path), Loaded::Missing);
        save(&path, &sample())?;
        assert_eq!(load(&path), Loaded::Ok(sample()));
        save(&path, &StoredSettings::default())?;
        assert!(path.with_extension("json.bak").exists());
        assert!(!path.with_extension("json.tmp").exists());
        Ok(())
    }

    #[test]
    fn refuses_newer_schema_and_flags_corruption() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            br#"{"format":"fotollection-settings","schema_version":99,"future":true}"#,
        )?;
        assert_eq!(load(&path), Loaded::NewerVersion(99));

        std::fs::write(&path, b"{ not json")?;
        assert!(matches!(load(&path), Loaded::Corrupt(_)));

        std::fs::write(
            &path,
            br#"{"format":"fotollection-settings","schema_version":1,"surprise":1}"#,
        )?;
        assert!(
            matches!(load(&path), Loaded::Corrupt(_)),
            "unknown fields rejected"
        );
        Ok(())
    }

    #[test]
    fn duplicate_camera_rows_are_rejected() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        let mut s = sample();
        s.cameras.push(s.cameras[0].clone());
        save(&path, &s)?;
        assert!(matches!(load(&path), Loaded::Corrupt(_)));
        Ok(())
    }
}
