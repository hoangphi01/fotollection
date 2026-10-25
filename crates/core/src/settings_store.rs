// SPDX-License-Identifier: GPL-3.0-only
//! `settings.json`: app-wide settings and camera profiles (DATABASE.md §6). Versioned,
//! validated on load, written atomically, and never overwritten by an older app.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::fonts::UserFont;
use crate::frame::{CameraProfile, FrameSettings, LogoKind};
use crate::settings::{RenderSettings, UiSettings};

pub const FORMAT: &str = "fotollection-settings";
/// v2 (FR-042, FR-043): `fonts` table, custom print sizes, stamp/frame font choice.
/// v3 (FR-020): `ui.language`.
/// v4 (FR-040, ADR-0020): bundled brand logos (`logo: "brand"`) and `frame.logoStyle`.
/// v5 (FR-005, FR-036, FR-033): `render.stamp.place`, `render.stamp.caption`,
/// `render.placeFormat`; `render.stamp.font` is dropped (the stamp has its own font).
/// v6 (FR-052): `ui.theme`, absent before (follow the system).
/// v7 (FR-040): `frame.logoScale`, absent before (1.0).
/// v8 (FR-053): `render.exportTarget` (prints or a share preset), absent before (prints).
pub const SCHEMA_VERSION: u32 = 8;
pub const MAX_BYTES: u64 = 4 * 1024 * 1024;

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
    /// Interface preferences of this machine (FR-020).
    pub ui: UiSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileV4 {
    format: String,
    schema_version: u32,
    render: RenderSettings,
    frame: FrameSettings,
    cameras: Vec<CameraProfile>,
    /// Absent in v1 files; every other v2 addition also has a serde default, so
    /// migrating v1 → v2 is reading with defaults (DATABASE.md §6).
    #[serde(default)]
    fonts: Vec<UserFont>,
    /// Absent before v3: English (FR-020).
    #[serde(default)]
    ui: UiSettings,
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
    match std::fs::read(path) {
        Ok(bytes) => from_bytes(&bytes),
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

/// Parses and validates settings already in memory (also a bundle part, ADR-0016).
pub fn from_bytes(bytes: &[u8]) -> Loaded {
    if bytes.len() as u64 > MAX_BYTES {
        return Loaded::Corrupt("settings file is too large".into());
    }
    // Check the version before the strict parse, so newer files are recognised.
    let header: Header = match serde_json::from_slice(bytes) {
        Ok(h) => h,
        Err(e) => return Loaded::Corrupt(e.to_string()),
    };
    if header.format != FORMAT {
        return Loaded::Corrupt("not a Fotollection settings file".into());
    }
    if header.schema_version > SCHEMA_VERSION {
        return Loaded::NewerVersion(header.schema_version);
    }
    match serde_json::from_slice::<FileV4>(bytes) {
        Ok(file) => match validate(file, header.schema_version) {
            Ok(settings) => Loaded::Ok(settings),
            Err(e) => Loaded::Corrupt(e),
        },
        Err(e) => Loaded::Corrupt(e.to_string()),
    }
}

fn validate(file: FileV4, version: u32) -> Result<StoredSettings, String> {
    file.render.validate().map_err(|e| e.to_string())?;
    let mut ids = std::collections::HashSet::new();
    let mut keys = std::collections::HashSet::new();
    let mut cameras = Vec::with_capacity(file.cameras.len());
    for camera in file.cameras {
        if !ids.insert(camera.id) || !keys.insert((camera.make.clone(), camera.model.clone())) {
            return Err(format!("duplicate camera {}", camera.id));
        }
        let mut camera = camera.sanitized();
        // v3 → v4: cameras still on the default typeset name get their bundled brand logo;
        // a name the user typed is kept.
        if version < 4
            && camera.logo == LogoKind::Text
            && camera.brand_text.is_none()
            && camera.brand().is_some()
        {
            camera.logo = LogoKind::Brand;
        }
        cameras.push(camera);
    }
    let mut font_ids = std::collections::HashSet::new();
    if file.fonts.len() > crate::fonts::MAX_USER_FONTS
        || !file.fonts.iter().all(|f| font_ids.insert(f.id))
    {
        return Err("invalid fonts table".into());
    }
    let render = file.render;
    let mut frame = file.frame;
    // References to fonts that no longer exist fall back to the built-in font (N7).
    if frame.font.is_some_and(|id| !font_ids.contains(&id)) {
        frame.font = None;
    }
    Ok(StoredSettings {
        render,
        frame,
        cameras,
        fonts: file.fonts,
        ui: file.ui,
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
    crate::atomic::write_with_backup(path, &to_bytes(settings)?)
}

pub fn to_bytes(settings: &StoredSettings) -> std::io::Result<Vec<u8>> {
    let file = FileV4 {
        format: FORMAT.to_owned(),
        schema_version: SCHEMA_VERSION,
        render: settings.render.clone(),
        frame: settings.frame.clone(),
        cameras: settings.cameras.clone(),
        fonts: settings.fonts.clone(),
        ui: settings.ui,
    };
    serde_json::to_vec_pretty(&file).map_err(std::io::Error::other)
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
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v1.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v1 should load");
        };
        assert!(s.fonts.is_empty() && s.render.custom_sizes.is_empty());
        assert_eq!(s.ui, UiSettings::default());
        Ok(())
    }

    #[test]
    fn fr_020_migrates_v2_files_to_english() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v2.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v2 should load");
        };
        assert_eq!(s.ui.language, crate::settings::Language::En);
        assert_eq!(s.render.custom_sizes.len(), 1);
        assert_eq!(s.fonts.len(), 1);
        // v2 had a stamp font; the stamp now always uses its LCD face, so it is ignored.
        assert!(!s.render.stamp.place && s.render.stamp.caption.is_empty());
        Ok(())
    }

    #[test]
    fn fr_040_migrates_v3_cameras_to_bundled_logos() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v3.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v3 should load");
        };
        let logos: Vec<LogoKind> = s.cameras.iter().map(|c| c.logo).collect();
        // Canon gets its logo; a typed "LUMIX" and an unknown brand keep the typeset name.
        assert_eq!(logos, [LogoKind::Brand, LogoKind::Text, LogoKind::Text]);
        assert_eq!(s.frame.logo_style, crate::brands::LogoStyle::Classic);
        assert_eq!(s.ui.language, crate::settings::Language::Vi);
        Ok(())
    }

    #[test]
    fn fr_036_migrates_v4_files_without_the_stamp_font() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v4.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v4 should load");
        };
        // The stamp font is gone (the stamp has its own LCD face); place and caption start off.
        assert!(!s.render.stamp.place && s.render.stamp.caption.is_empty());
        assert_eq!(s.frame.logo_style, crate::brands::LogoStyle::Mono);
        assert_eq!(s.fonts.len(), 1, "the font stays installed for the frame");
        Ok(())
    }

    #[test]
    fn fr_052_v5_files_follow_the_system_theme() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v5.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v5 should load");
        };
        assert_eq!(s.ui.theme, crate::settings::ThemePreference::System);
        assert_eq!(s.ui.language, crate::settings::Language::Vi);
        // A chosen theme is written and read back.
        let dark = StoredSettings {
            ui: UiSettings {
                theme: crate::settings::ThemePreference::Dark,
                ..s.ui
            },
            ..StoredSettings::default()
        };
        save(&path, &dark)?;
        assert!(std::fs::read_to_string(&path)?.contains(r#""theme": "dark""#));
        let Loaded::Ok(back) = load(&path) else {
            panic!("v6 should load");
        };
        assert_eq!(back.ui.theme, crate::settings::ThemePreference::Dark);
        Ok(())
    }

    #[test]
    fn fr_040_v6_files_get_the_default_logo_scale() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v6.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v6 should load");
        };
        assert!((s.frame.logo_scale - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.ui.theme, crate::settings::ThemePreference::Dark);
        Ok(())
    }

    #[test]
    fn fr_053_v7_files_default_to_print_and_a_share_target_round_trips() -> std::io::Result<()> {
        use crate::share::{ExportTarget, SharePreset};
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v7.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v7 should load");
        };
        assert_eq!(s.render.export_target, ExportTarget::Print);
        assert!((s.frame.logo_scale - 1.5).abs() < f32::EPSILON);
        let mut shared = s;
        shared.render.export_target = ExportTarget::Share(SharePreset::Custom(1600));
        save(&path, &shared)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v8 should load");
        };
        assert_eq!(
            back.render.export_target,
            ExportTarget::Share(SharePreset::Custom(1600))
        );
        Ok(())
    }

    #[test]
    fn fr_020_language_survives_a_round_trip() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        let s = StoredSettings {
            ui: UiSettings {
                language: crate::settings::Language::Vi,
                ..UiSettings::default()
            },
            ..StoredSettings::default()
        };
        save(&path, &s)?;
        assert!(std::fs::read_to_string(&path)?.contains(r#""language": "vi""#));
        assert_eq!(load(&path), Loaded::Ok(s));
        Ok(())
    }

    #[test]
    fn dangling_font_reference_falls_back() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        let mut s = sample();
        s.frame.font = Some(7);
        save(&path, &s)?;
        let Loaded::Ok(loaded) = load(&path) else {
            panic!("should load");
        };
        assert_eq!(loaded.frame.font, None);
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
