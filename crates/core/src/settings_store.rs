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
/// v9 (FR-065): `frame.border`, the Crop-to-fill border, absent before (off).
/// v10 (FR-067, FR-048): `render.border`, the 3 mm border on every print, where a v9
/// `frame.border` moves; `ui.mode`, absent before (Professional); `render.collage`
/// (FR-035), absent before (the defaults); `render.stamp.size` (FR-004), absent before
/// (medium); `render.borderColor` (FR-067), absent before (white).
/// v11 (FR-073): `render.collage.postcard`, absent before (off).
/// v12 (FR-009, FR-075): `ui.exportFolders`, absent before (one folder); `render.shareFill`,
/// absent before (off).
/// v13 (FR-076, FR-077): `render.collage.postcard.face`, absent before (the typewriter);
/// `recentAddresses`, absent before (none).
/// v14 (FR-079): `render.look`.
/// v15 (FR-086, FR-091): `ui.previewMemoryPercent` (25), `ui.cacheLimitGb` (5) and
/// `ui.preparePreviews` (on), each absent before.
/// v16 (FR-092, FR-093): `ui.exportQuality` (90), `ui.exportSubfolder`, `ui.exportNaming`, `ui.exportExisting`, `ui.exportMetadata`, `ui.exportLimitKb` (FR-009, FR-010, FR-104, FR-105), `ui.checkUpdatesAtStart` (FR-103, on), `ui.exportAsk` (on), `ui.exportSeparate`
/// (off) and `ui.exportDestination` (none), each absent before.
pub const SCHEMA_VERSION: u32 = 17;
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
    /// The addresses last typed on postcards, newest first, for the next cards (FR-077).
    /// This machine's only: never in a project bundle.
    pub recent_addresses: Vec<String>,
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
    /// Absent before v13: none (FR-077).
    #[serde(default)]
    recent_addresses: Vec<String>,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // read once at start-up, never stored in bulk
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
    let mut render = file.render;
    render.collage = render.collage.sanitized();
    render.share_fill = render.share_fill.sanitized();
    render.look = render.look.sanitized();
    let mut frame = file.frame;
    // v9 → v10: FR-065's Crop-to-fill border becomes the border on every print (FR-067),
    // only where it showed: with the info frame on, in Crop to fill.
    if version < 10 && frame.border && frame.enabled && frame.fit == crate::frame::FrameFit::Crop {
        render.border = true;
    }
    frame.border = false;
    // References to fonts that no longer exist fall back to the built-in font (N7).
    if frame.font.is_some_and(|id| !font_ids.contains(&id)) {
        frame.font = None;
    }
    Ok(StoredSettings {
        render,
        frame,
        cameras,
        fonts: file.fonts,
        ui: file.ui.sanitized(),
        recent_addresses: crate::settings::sanitized_addresses(&file.recent_addresses),
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
        ui: settings.ui.clone(),
        recent_addresses: settings.recent_addresses.clone(),
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
    fn fr_065_v8_files_have_no_border_and_the_border_round_trips() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v8.json"))?;
        let Loaded::Ok(mut s) = load(&path) else {
            panic!("v8 should load");
        };
        assert!(!s.render.border);
        s.render.border = true;
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v10 should load");
        };
        assert!(back.render.border);
        Ok(())
    }

    #[test]
    fn fr_067_a_v9_border_moves_to_every_print_and_round_trips() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v9.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v9 should load");
        };
        assert!(s.render.border);
        assert!(!s.frame.border);
        assert_eq!(s.ui.mode, crate::settings::AppMode::Professional);
        save(&path, &s)?;
        // The old key is never on again.
        let written: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(written["frame"]["border"], serde_json::Value::Bool(false));
        assert_eq!(written["render"]["border"], serde_json::Value::Bool(true));
        let Loaded::Ok(back) = load(&path) else {
            panic!("v10 should load");
        };
        assert!(back.render.border);
        Ok(())
    }

    /// FR-073: a v10 file has no postcard: it loads with the postcard off and four lines,
    /// and the postcard settings round-trip.
    #[test]
    fn fr_073_v10_files_have_no_postcard_and_it_round_trips() -> std::io::Result<()> {
        use crate::settings::{PostcardOrientation, PostcardSettings};
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v10.json"))?;
        let Loaded::Ok(mut s) = load(&path) else {
            panic!("v10 should load");
        };
        assert_eq!(s.render.collage.postcard, PostcardSettings::default());
        s.render.collage.postcard = PostcardSettings {
            enabled: true,
            orientation: PostcardOrientation::Portrait,
            lines: 3,
            font: Some(7),
            face: crate::settings::PostcardFace::Handwriting,
            credit: "Photo © Anna Schmidt".into(),
            ink: crate::settings::PostcardInk::Blue,
            text_size: crate::settings::StampSize::Large,
            message_style: crate::settings::TextStyle {
                italic: true,
                ..crate::settings::TextStyle::default()
            },
            address_style: crate::settings::TextStyle {
                bold: true,
                underline: true,
                ..crate::settings::TextStyle::default()
            },
        };
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v11 should load");
        };
        assert_eq!(back.render.collage.postcard, s.render.collage.postcard);
        Ok(())
    }

    /// FR-009: a v11 file has no export folders choice: it loads as one folder, the
    /// choice of a new install, and a folder per day round-trips.
    #[test]
    fn fr_009_v11_files_export_into_one_folder_and_by_day_round_trips() -> std::io::Result<()> {
        use crate::settings::ExportFolders;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v11.json"))?;
        let Loaded::Ok(mut s) = load(&path) else {
            panic!("v11 should load");
        };
        assert_eq!(s.ui.export_folders, ExportFolders::OneFolder);
        assert!(s.render.collage.postcard.enabled);
        s.ui.export_folders = ExportFolders::ByDay;
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v12 should load");
        };
        assert_eq!(back.ui.export_folders, ExportFolders::ByDay);
        Ok(())
    }

    /// FR-076, FR-077: a v12 file has no postcard face and no remembered addresses: it loads
    /// with the typewriter, keeps what v12 added, and both round-trip.
    #[test]
    fn fr_076_v12_files_type_in_the_typewriter_and_the_face_round_trips() -> std::io::Result<()> {
        use crate::settings::{ExportFolders, PostcardFace};
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v12.json"))?;
        let Loaded::Ok(mut s) = load(&path) else {
            panic!("v12 should load");
        };
        assert_eq!(s.render.collage.postcard.face, PostcardFace::Typewriter);
        assert_eq!(s.ui.export_folders, ExportFolders::ByDay);
        assert_eq!(s.render.share_fill.ratio, [4, 5]);
        assert_eq!(s.recent_addresses, Vec::<String>::new());
        s.render.collage.postcard.face = PostcardFace::Handwriting;
        s.recent_addresses = vec!["Národní muzeum\n110 00 Praha 1".into()];
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v13 should load");
        };
        assert_eq!(back.render.collage.postcard.face, PostcardFace::Handwriting);
        assert_eq!(back.recent_addresses, s.recent_addresses);
        Ok(())
    }

    /// FR-079: a v13 file has no look: it loads with none, and a v14 file keeps its sliders,
    /// the ones it leaves out at 0, and round-trips.
    #[test]
    fn fr_079_v13_files_have_no_look_and_v14_keeps_it() -> std::io::Result<()> {
        use crate::looks::{LookSettings, Mono};
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v13.json"))?;
        let loaded = load(&path);
        let Loaded::Ok(old) = loaded else {
            panic!("v13 should load: {loaded:?}");
        };
        assert_eq!(old.render.look, LookSettings::default());
        assert_eq!(old.recent_addresses.len(), 1);
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v14.json"))?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v14 should load");
        };
        let a = s.render.look.adjust;
        assert_eq!(
            (a.brightness, a.contrast, a.warmth, a.fade),
            (20, -10, 30, 25)
        );
        assert_eq!(
            (a.shadow_tint.hue, a.shadow_tint.amount, a.mono),
            (200, 30, Mono::Off)
        );
        assert_eq!((a.tint, a.highlights, s.render.look.amount), (0, 0, 80));
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v14 should load again");
        };
        assert_eq!(back.render.look, s.render.look);
        Ok(())
    }

    /// FR-086, FR-091: a v14 file has no memory settings: 25 % of RAM, a 5 GB cache and
    /// previews prepared. A v15 file keeps its own, snapped: 99 % becomes 40, 7 GB the
    /// FR-109: a v16 file has never shown the welcome screen, so it shows once and not at
    /// every start; a v17 file keeps both, and they round-trip.
    #[test]
    fn fr_109_the_welcome_shows_once_unless_asked_every_start() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v16.json"))?;
        let Loaded::Ok(old) = load(&path) else {
            panic!("v16 should load");
        };
        assert_eq!(
            (old.ui.welcome_seen, old.ui.welcome_every_start),
            (false, false)
        );
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v17.json"))?;
        let Loaded::Ok(new) = load(&path) else {
            panic!("v17 should load");
        };
        assert_eq!(
            (new.ui.welcome_seen, new.ui.welcome_every_start),
            (true, true)
        );
        save(&path, &new)?;
        let Loaded::Ok(again) = load(&path) else {
            panic!("a saved v17 should load");
        };
        assert_eq!(
            (again.ui.welcome_seen, again.ui.welcome_every_start),
            (true, true)
        );
        Ok(())
    }

    /// FR-092, FR-093: a v15 file has no export settings: quality 90, asked each time,
    /// together, no folder. A v16 file keeps its own, the quality clamped to 60–100, and
    /// they round-trip.
    #[test]
    fn fr_093_export_settings_default_clamp_and_round_trip() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v15.json"))?;
        let Loaded::Ok(old) = load(&path) else {
            panic!("v15 should load");
        };
        assert_eq!(
            (
                old.ui.export_quality,
                old.ui.export_ask,
                old.ui.export_separate,
                old.ui.export_destination
            ),
            (90, true, false, None)
        );
        // FR-009, FR-010, FR-104, FR-105, FR-103: as exports always were, and the check on.
        assert_eq!(
            (
                old.ui.export_subfolder.clone(),
                old.ui.export_naming.clone(),
                old.ui.export_existing,
                old.ui.export_metadata,
                old.ui.export_limit_kb,
                old.ui.check_updates_at_start
            ),
            (
                None,
                crate::naming::ExportNaming::default(),
                crate::settings::ExistingFiles::AddNumber,
                crate::settings::ExportMetadata::Camera,
                None,
                true
            )
        );
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v16.json"))?;
        let loaded = load(&path);
        let Loaded::Ok(s) = loaded else {
            panic!("v16 should load: {loaded:?}");
        };
        assert_eq!(
            (s.ui.export_quality, s.ui.export_ask, s.ui.export_separate),
            (100, false, true)
        );
        assert_eq!(
            s.ui.export_destination.as_deref(),
            Some("/Volumes/Prints/Prague")
        );
        // A path in a name is cleaned, the start and the limit clamped (SECURITY.md §2).
        assert_eq!(s.ui.export_subfolder.as_deref(), Some("etc"));
        assert_eq!(
            s.ui.export_naming,
            crate::naming::ExportNaming {
                template: crate::naming::NameTemplate::TextSequence,
                text: "Praguein".into(),
                start: 1,
            }
        );
        assert_eq!(s.ui.export_existing, crate::settings::ExistingFiles::Skip);
        assert_eq!(
            s.ui.export_metadata,
            crate::settings::ExportMetadata::DateOnly
        );
        assert_eq!(s.ui.export_limit_kb, Some(100));
        assert!(!s.ui.check_updates_at_start);
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v16 should load again");
        };
        assert_eq!(back.ui, s.ui);
        Ok(())
    }

    /// nearest limit offered (5), and they round-trip.
    #[test]
    fn fr_086_memory_settings_default_clamp_and_round_trip() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v14.json"))?;
        let Loaded::Ok(old) = load(&path) else {
            panic!("v14 should load");
        };
        assert_eq!(
            (
                old.ui.preview_memory_percent,
                old.ui.cache_limit_gb,
                old.ui.prepare_previews
            ),
            (25, 5, true)
        );
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v15.json"))?;
        let loaded = load(&path);
        let Loaded::Ok(s) = loaded else {
            panic!("v15 should load: {loaded:?}");
        };
        assert_eq!(
            (
                s.ui.preview_memory_percent,
                s.ui.cache_limit_gb,
                s.ui.prepare_previews
            ),
            (40, 5, false)
        );
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v15 should load again");
        };
        assert_eq!(back.ui, s.ui);
        Ok(())
    }

    /// FR-075: a v11 file has no fill: it loads off, a fill round-trips, and a ratio out of
    /// range is kept to 4:1 at most.
    #[test]
    fn fr_075_v11_files_have_no_fill_and_it_round_trips() -> std::io::Result<()> {
        use crate::settings::ShareFill;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        std::fs::write(&path, include_str!("../tests/fixtures/settings/v11.json"))?;
        let Loaded::Ok(mut s) = load(&path) else {
            panic!("v11 should load");
        };
        assert_eq!(s.render.share_fill, ShareFill::default());
        s.render.share_fill = ShareFill {
            enabled: true,
            ratio: [4, 5],
            color: [20, 40, 60],
        };
        save(&path, &s)?;
        let Loaded::Ok(back) = load(&path) else {
            panic!("v12 should load");
        };
        assert_eq!(back.render.share_fill, s.render.share_fill);
        let wild = ShareFill {
            ratio: [900, 0],
            ..ShareFill::default()
        };
        assert_eq!(wild.sanitized().ratio, [4, 1]);
        Ok(())
    }

    #[test]
    fn fr_067_a_v9_border_without_the_frame_stays_off() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("settings.json");
        // The border switch was on, but the info frame was off: no print had a border.
        let text = include_str!("../tests/fixtures/settings/v9.json").replacen(
            "\"enabled\": true",
            "\"enabled\": false",
            1,
        );
        std::fs::write(&path, text)?;
        let Loaded::Ok(s) = load(&path) else {
            panic!("v9 should load");
        };
        assert!(!s.render.border);
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
