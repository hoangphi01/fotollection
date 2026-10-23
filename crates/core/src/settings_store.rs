// SPDX-License-Identifier: GPL-3.0-only
//! `settings.json`: app-wide settings and camera profiles (DATABASE.md §6). Versioned,
//! validated on load, written atomically, and never overwritten by an older app.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::fonts::UserFont;
use crate::frame::{CameraProfile, FrameSettings};
use crate::settings::RenderSettings;

pub const FORMAT: &str = "fotollection-settings";
/// v2 (FR-042, FR-043): `fonts` table, custom print sizes, stamp/frame font choice.
pub const SCHEMA_VERSION: u32 = 2;
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Everything persisted in `settings.json`. `cameras` is a table keyed by `id`, with
/// `(make, model)` unique (normalization rules N1–N3).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSettings {
    pub render: RenderSettings,
    pub frame: FrameSettings,
    pub cameras: Vec<CameraProfile>,
    /// User fonts; files live in `config/fonts/<id>.<ext>` (FR-042).
    pub fonts: Vec<UserFont>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileV2 {
    format: String,
    schema_version: u32,
    render: RenderSettings,
    frame: FrameSettings,
    cameras: Vec<CameraProfile>,
    /// Absent in v1 files; every other v2 addition also has a serde default, so
    /// migrating v1 → v2 is reading with defaults (DATABASE.md §6).
    #[serde(default)]
    fonts: Vec<UserFont>,
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
    match serde_json::from_slice::<FileV2>(&bytes) {
        Ok(file) => match validate(file) {
            Ok(settings) => Loaded::Ok(settings),
            Err(e) => Loaded::Corrupt(e),
        },
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

fn validate(file: FileV2) -> Result<StoredSettings, String> {
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
    let mut font_ids = std::collections::HashSet::new();
    if file.fonts.len() > crate::fonts::MAX_USER_FONTS
        || !file.fonts.iter().all(|f| font_ids.insert(f.id))
    {
        return Err("invalid fonts table".into());
    }
    let mut render = file.render;
    let mut frame = file.frame;
    // References to fonts that no longer exist fall back to the built-in font (N7).
    if render.stamp.font.is_some_and(|id| !font_ids.contains(&id)) {
        render.stamp.font = None;
    }
    if frame.font.is_some_and(|id| !font_ids.contains(&id)) {
        frame.font = None;
    }
    Ok(StoredSettings {
        render,
        frame,
        cameras,
        fonts: file.fonts,
    })
}

/// Moves an unreadable file aside so it is never silently lost.
pub fn quarantine(path: &Path) -> std::io::Result<PathBuf> {
    let target = path.with_extension("json.corrupt");
    std::fs::rename(path, &target)?;
    Ok(target)
}

/// Writes atomically, keeping the previous file as `.bak` (DATABASE.md §4.4).
pub fn save(path: &Path, settings: &StoredSettings) -> std::io::Result<()> {
    let file = FileV2 {
        format: FORMAT.to_owned(),
        schema_version: SCHEMA_VERSION,
        render: settings.render.clone(),
        frame: settings.frame.clone(),
        cameras: settings.cameras.clone(),
        fonts: settings.fonts.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(std::io::Error::other)?;
    crate::atomic::write_with_backup(path, &bytes)
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
    fn migrates_v1_files() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        // A file written by schema 1 (before fonts and custom sizes existed).
        std::fs::write(
            &path,
            br#"{"format":"fotollection-settings","schema_version":1,
                "render":{"printSize":"10x15","stamp":{"enabled":true,"format":"iso","corner":"bottomRight","color":"white"}},
                "frame":{"enabled":false,"fit":"crop","color":"white","dateFormat":"slash",
                  "fields":{"logo":true,"camera":true,"lens":true,"focal":true,"aperture":true,"shutter":true,"iso":true,"photoStyle":true,"date":true,"place":true}},
                "cameras":[]}"#,
        )?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v1 should load");
        };
        assert!(s.fonts.is_empty() && s.render.custom_sizes.is_empty());
        assert_eq!(s.render.stamp.font, None);
        Ok(())
    }

    #[test]
    fn dangling_font_reference_falls_back() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        let mut s = sample();
        s.render.stamp.font = Some(7);
        save(&path, &s)?;
        let Loaded::Ok(loaded) = load(&path) else {
            panic!("should load");
        };
        assert_eq!(loaded.render.stamp.font, None);
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
