// SPDX-License-Identifier: GPL-3.0-only
//! The self-contained project bundle (`.fotollection`, FR-023, ADR-0016): a ZIP holding the
//! project tables, the print settings and copies of the fonts and logos they use, so a
//! project opens on another machine looking the same. Photos are referenced, not embedded.
//!
//! Every bundle is untrusted input (SECURITY.md §3): part names come from an allowlist,
//! sizes and the compression ratio are capped before anything is read, every part is
//! checked against the BLAKE3 sum in the manifest and parsed by its typed reader.

use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::fonts::{self, FontFormat, UserFont};
use crate::frame::LogoKind;
use crate::logo;
use crate::project::{self, ProjectFile};
use crate::settings::UiSettings;
use crate::settings_store::{self, StoredSettings};

pub const FORMAT: &str = "fotollection-bundle";
pub const SCHEMA_VERSION: u32 = 1;

/// Limits checked before reading (zip-bomb guard). A bundle holds at most two fonts and one
/// logo per camera, so these are generous.
pub const MAX_ENTRIES: usize = 128;
pub const MAX_TOTAL_BYTES: u64 = 160 * 1024 * 1024;
pub const MAX_RATIO: u64 = 100;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const ZIP_MAGIC: &[u8] = b"PK\x03\x04";

const MANIFEST: &str = "manifest.json";
const PROJECT: &str = "project.json";
const SETTINGS: &str = "settings.json";

/// One part listed in `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartEntry {
    pub path: String,
    pub bytes: u64,
    /// Lowercase hex BLAKE3 of the part.
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub parts: Vec<PartEntry>,
}

/// Everything a bundle carries. `settings.ui` is never stored: the interface language is
/// a choice of each machine (FR-020).
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    pub project: ProjectFile,
    pub settings: StoredSettings,
    /// Font files by `UserFont.id`; exactly the fonts listed in `settings.fonts`.
    pub fonts: BTreeMap<u32, Vec<u8>>,
    /// Logo PNGs by camera profile id; only cameras with `logo: "image"`.
    pub logos: BTreeMap<u32, Vec<u8>>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BundleError {
    #[error("not a Fotollection project")]
    NotAProject,
    #[error("the project was saved by a newer version of Fotollection (format {0})")]
    NewerVersion(u32),
    #[error("the project file is damaged: {0}")]
    Corrupt(String),
    #[error("the project file is too large")]
    TooLarge,
    #[error("the project could not be written: {0}")]
    Write(String),
    #[error(
        "the project's fonts would exceed the limit of {} fonts",
        fonts::MAX_USER_FONTS
    )]
    TooManyFonts,
}

impl BundleError {
    /// Stable machine-readable code (SECURITY.md §8).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotAProject => "project.not_a_project",
            Self::NewerVersion(_) => "project.newer_version",
            Self::Corrupt(_) => "project.corrupt",
            Self::TooLarge => "project.too_large",
            Self::Write(_) => "project.write_failed",
            Self::TooManyFonts => "font.too_many",
        }
    }
}

fn corrupt(why: impl Into<String>) -> BundleError {
    BundleError::Corrupt(why.into())
}

/// The kind of a part, from its name. Anything else is rejected (allowlist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Manifest,
    Project,
    Settings,
    Font(u32, FontFormat),
    Logo(u32),
}

impl Part {
    fn parse(name: &str) -> Option<Self> {
        let numbered = |rest: &str, ext: &str| -> Option<u32> {
            let id = rest.strip_suffix(ext)?;
            // Digits only, no leading zeros: one canonical name per id.
            if id.is_empty()
                || id.len() > 9
                || id.starts_with('0')
                || !id.bytes().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            id.parse().ok()
        };
        match name {
            MANIFEST => Some(Self::Manifest),
            PROJECT => Some(Self::Project),
            SETTINGS => Some(Self::Settings),
            _ => {
                if let Some(rest) = name.strip_prefix("fonts/") {
                    numbered(rest, ".ttf")
                        .map(|id| Self::Font(id, FontFormat::Ttf))
                        .or_else(|| {
                            numbered(rest, ".otf").map(|id| Self::Font(id, FontFormat::Otf))
                        })
                } else {
                    name.strip_prefix("logos/")
                        .and_then(|rest| numbered(rest, ".png"))
                        .map(Self::Logo)
                }
            }
        }
    }

    fn limit(self) -> u64 {
        match self {
            Self::Manifest => MAX_MANIFEST_BYTES,
            Self::Project => project::MAX_BYTES,
            Self::Settings => settings_store::MAX_BYTES,
            Self::Font(..) => fonts::MAX_FONT_BYTES as u64,
            Self::Logo(_) => logo::MAX_LOGO_BYTES as u64,
        }
    }
}

fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Content fingerprint used to recognise a font the user already has (same BLAKE3).
pub fn fingerprint(bytes: &[u8]) -> String {
    hash(bytes)
}

/// True when the bytes look like a bundle (as opposed to a v1 JSON project file).
pub fn is_bundle(bytes: &[u8]) -> bool {
    bytes.starts_with(ZIP_MAGIC)
}

/// Serializes a bundle. Parts are written in a fixed order with the fixed ZIP epoch, so the
/// same project always produces the same bytes.
pub fn to_bytes(bundle: &Bundle) -> Result<Vec<u8>, BundleError> {
    let write_err = |e: &dyn std::fmt::Display| BundleError::Write(e.to_string());
    let settings = StoredSettings {
        ui: UiSettings::default(),
        ..bundle.settings.clone()
    };
    let project = ProjectFile {
        document: None,
        ..bundle.project.clone()
    };
    let mut parts: Vec<(String, Vec<u8>)> = vec![
        (
            PROJECT.into(),
            project::to_bytes(&project).map_err(|e| write_err(&e))?,
        ),
        (
            SETTINGS.into(),
            settings_store::to_bytes(&settings).map_err(|e| write_err(&e))?,
        ),
    ];
    for font in &bundle.settings.fonts {
        let bytes = bundle
            .fonts
            .get(&font.id)
            .ok_or_else(|| BundleError::Write(format!("font {} has no file", font.id)))?;
        parts.push((
            format!("fonts/{}.{}", font.id, font.format.extension()),
            bytes.clone(),
        ));
    }
    for (id, bytes) in &bundle.logos {
        parts.push((format!("logos/{id}.png"), bytes.clone()));
    }
    let manifest = Manifest {
        format: FORMAT.into(),
        schema_version: SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION").into(),
        parts: parts
            .iter()
            .map(|(path, bytes)| PartEntry {
                path: path.clone(),
                bytes: bytes.len() as u64,
                blake3: hash(bytes),
            })
            .collect(),
    };
    let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| write_err(&e))?;

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    // Font and PNG data are already compressed.
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    zip.start_file(MANIFEST, deflated)
        .map_err(|e| write_err(&e))?;
    zip.write_all(&manifest).map_err(|e| write_err(&e))?;
    for (path, bytes) in &parts {
        let options = if path == PROJECT || path == SETTINGS {
            deflated
        } else {
            stored
        };
        zip.start_file(path.as_str(), options)
            .map_err(|e| write_err(&e))?;
        zip.write_all(bytes).map_err(|e| write_err(&e))?;
    }
    Ok(zip.finish().map_err(|e| write_err(&e))?.into_inner())
}

/// Writes atomically, keeping the previous file as `.bak` (DATABASE.md §4.4).
pub fn save(path: &Path, bundle: &Bundle) -> Result<(), BundleError> {
    let bytes = to_bytes(bundle)?;
    crate::atomic::write_with_backup(path, &bytes).map_err(|e| BundleError::Write(e.to_string()))
}

/// Reads and fully validates a bundle. Nothing in it is trusted until this returns.
pub fn from_bytes(bytes: &[u8]) -> Result<Bundle, BundleError> {
    if !is_bundle(bytes) {
        return Err(BundleError::NotAProject);
    }
    if bytes.len() as u64 > MAX_TOTAL_BYTES {
        return Err(BundleError::TooLarge);
    }
    let mut data = read_parts(bytes)?;
    verify_manifest(&mut data)?;

    let project = match data.remove(PROJECT).map(|(_, b)| project::from_bytes(&b)) {
        Some(project::Loaded::Ok(p)) => p,
        Some(project::Loaded::NewerVersion(v)) => return Err(BundleError::NewerVersion(v)),
        Some(project::Loaded::Corrupt(why)) => return Err(corrupt(why)),
        Some(project::Loaded::Missing) | None => return Err(corrupt("no project tables")),
    };
    let mut settings = match data
        .remove(SETTINGS)
        .map(|(_, b)| settings_store::from_bytes(&b))
    {
        Some(settings_store::Loaded::Ok(s)) => s,
        Some(settings_store::Loaded::NewerVersion(v)) => return Err(BundleError::NewerVersion(v)),
        Some(settings_store::Loaded::Corrupt(why)) => return Err(corrupt(why)),
        Some(settings_store::Loaded::Missing) | None => return Err(corrupt("no settings")),
    };
    settings.ui = UiSettings::default();

    let mut font_files = BTreeMap::new();
    let mut logos = BTreeMap::new();
    for (name, (part, bytes)) in data {
        match part {
            Part::Font(id, format) => {
                let listed = settings.fonts.iter().find(|f| f.id == id);
                if listed.map(|f| f.format) != Some(format) {
                    return Err(corrupt(format!("{name:?} is not in the fonts table")));
                }
                // Same validation as importing a font by hand (FR-042).
                let (sniffed, _) =
                    fonts::load(bytes.clone()).map_err(|e| corrupt(format!("{name:?}: {e}")))?;
                if sniffed != format {
                    return Err(corrupt(format!("{name:?} has the wrong extension")));
                }
                font_files.insert(id, bytes);
            }
            Part::Logo(id) => {
                let camera = settings.cameras.iter().find(|c| c.id == id);
                if camera.map(|c| c.logo) != Some(LogoKind::Image) {
                    return Err(corrupt(format!("{name:?} belongs to no camera")));
                }
                logo::decode_png(&bytes).map_err(|e| corrupt(format!("{name:?}: {e}")))?;
                logos.insert(id, bytes);
            }
            Part::Manifest | Part::Project | Part::Settings => {}
        }
    }
    if let Some(font) = settings
        .fonts
        .iter()
        .find(|f| !font_files.contains_key(&f.id))
    {
        return Err(corrupt(format!("font {} is listed but missing", font.id)));
    }
    Ok(Bundle {
        project,
        settings,
        fonts: font_files,
        logos,
    })
}

type Parts = BTreeMap<String, (Part, Vec<u8>)>;

/// Both passes over the archive: names and declared sizes first (no decompression), then
/// each part read up to its own limit, whatever the header claims.
fn read_parts(bytes: &[u8]) -> Result<Parts, BundleError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| corrupt(e.to_string()))?;
    if zip.len() > MAX_ENTRIES {
        return Err(corrupt("too many parts"));
    }

    // Pass 1: names and declared sizes only, before any decompression.
    let mut total = 0u64;
    let mut seen = HashSet::new();
    let mut kinds = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|e| corrupt(e.to_string()))?;
        let name = entry
            .name()
            .map_err(|_| corrupt("a part name is not valid UTF-8"))?
            .into_owned();
        let part =
            Part::parse(&name).ok_or_else(|| corrupt(format!("unexpected part {name:?}")))?;
        if entry.encrypted() || !seen.insert(name.clone()) {
            return Err(corrupt(format!("invalid part {name:?}")));
        }
        let (size, packed) = (entry.size(), entry.compressed_size());
        if size > part.limit() {
            return Err(BundleError::TooLarge);
        }
        if size > packed.max(1).saturating_mul(MAX_RATIO) {
            return Err(corrupt(format!("{name:?} is compressed suspiciously well")));
        }
        total = total.saturating_add(size);
        if total > MAX_TOTAL_BYTES {
            return Err(BundleError::TooLarge);
        }
        kinds.push((i, name, part));
    }

    // Pass 2: read each part, never more than its limit, whatever the header claims.
    let mut data = Parts::new();
    for (i, name, part) in kinds {
        let entry = zip.by_index(i).map_err(|e| corrupt(e.to_string()))?;
        let mut buf = Vec::new();
        entry
            .take(part.limit() + 1)
            .read_to_end(&mut buf)
            .map_err(|e| corrupt(e.to_string()))?;
        if buf.len() as u64 > part.limit() {
            return Err(BundleError::TooLarge);
        }
        data.insert(name, (part, buf));
    }
    Ok(data)
}

/// The manifest lists exactly the other parts, each with the right size and BLAKE3 sum.
/// Removes the manifest from `data`.
fn verify_manifest(data: &mut Parts) -> Result<(), BundleError> {
    let (_, manifest) = data
        .remove(MANIFEST)
        .ok_or_else(|| corrupt("no manifest"))?;
    let manifest = parse_manifest(&manifest)?;
    let listed: HashSet<&str> = manifest.parts.iter().map(|p| p.path.as_str()).collect();
    if listed.len() != manifest.parts.len() || listed.len() != data.len() {
        return Err(corrupt("the manifest does not match the parts"));
    }
    for entry in &manifest.parts {
        let (_, bytes) = data
            .get(&entry.path)
            .ok_or_else(|| corrupt(format!("{:?} is listed but missing", entry.path)))?;
        if bytes.len() as u64 != entry.bytes || hash(bytes) != entry.blake3 {
            return Err(corrupt(format!("{:?} is damaged", entry.path)));
        }
    }
    Ok(())
}

fn parse_manifest(bytes: &[u8]) -> Result<Manifest, BundleError> {
    #[derive(Deserialize)]
    struct Header {
        format: String,
        schema_version: u32,
    }
    let header: Header = serde_json::from_slice(bytes).map_err(|e| corrupt(e.to_string()))?;
    if header.format != FORMAT {
        return Err(BundleError::NotAProject);
    }
    if header.schema_version > SCHEMA_VERSION {
        return Err(BundleError::NewerVersion(header.schema_version));
    }
    serde_json::from_slice(bytes).map_err(|e| corrupt(e.to_string()))
}

/// What a `.fotollection` file turned out to be.
// Returned once per Open and consumed at once, so boxing the bundle would buy nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq)]
pub enum Opened {
    Bundle(Bundle),
    /// A v1/v2 JSON project file (before bundles): tables only, settings stay as they are.
    Legacy(ProjectFile),
}

/// Opens either container, recognised by content, never by extension (EXTENSION.md §1).
pub fn open(path: &Path) -> Result<Opened, BundleError> {
    let size = std::fs::metadata(path)
        .map_err(|e| corrupt(e.to_string()))?
        .len();
    if size > MAX_TOTAL_BYTES {
        return Err(BundleError::TooLarge);
    }
    let bytes = std::fs::read(path).map_err(|e| corrupt(e.to_string()))?;
    if is_bundle(&bytes) {
        return from_bytes(&bytes).map(Opened::Bundle);
    }
    match project::from_bytes(&bytes) {
        project::Loaded::Ok(file) => Ok(Opened::Legacy(file)),
        project::Loaded::NewerVersion(v) => Err(BundleError::NewerVersion(v)),
        project::Loaded::Corrupt(_) | project::Loaded::Missing => Err(BundleError::NotAProject),
    }
}

/// What opening a bundle adds to this machine, after [`merge_settings`].
#[derive(Debug, Default, PartialEq)]
pub struct Installed {
    /// Fonts that were not here yet, with their new local ids: write and load them.
    pub new_fonts: Vec<(UserFont, Vec<u8>)>,
    /// Logo PNGs by local camera profile id: write them.
    pub logos: Vec<(u32, Vec<u8>)>,
}

/// Merges a bundle's settings into this machine's (FR-023, ADR-0016):
///
/// - fonts with the same content as one already here are reused, the others get new ids;
/// - camera profiles are matched by make and model, and the project's values win;
/// - the print and frame settings become the project's, with font ids remapped;
/// - `local.ui` (the interface language) is left alone.
///
/// `local_font` returns the bytes of a local font, to compare contents.
pub fn merge_settings(
    local: &mut StoredSettings,
    bundle: &Bundle,
    local_font: &dyn Fn(&UserFont) -> Option<Vec<u8>>,
) -> Result<Installed, BundleError> {
    let mut installed = Installed::default();
    let fingerprints: Vec<(u32, String)> = local
        .fonts
        .iter()
        .filter_map(|f| local_font(f).map(|b| (f.id, hash(&b))))
        .collect();
    let mut font_ids = BTreeMap::new();
    for font in &bundle.settings.fonts {
        let Some(bytes) = bundle.fonts.get(&font.id) else {
            continue;
        };
        let sum = hash(bytes);
        let id = if let Some((id, _)) = fingerprints.iter().find(|(_, s)| *s == sum) {
            *id
        } else {
            if local.fonts.len() >= fonts::MAX_USER_FONTS {
                return Err(BundleError::TooManyFonts);
            }
            let id = local.fonts.iter().map(|f| f.id).max().unwrap_or(0) + 1;
            let row = UserFont { id, ..font.clone() };
            local.fonts.push(row.clone());
            installed.new_fonts.push((row, bytes.clone()));
            id
        };
        font_ids.insert(font.id, id);
    }
    let remap = |id: Option<u32>| id.and_then(|id| font_ids.get(&id).copied());

    let mut camera_ids = BTreeMap::new();
    for incoming in &bundle.settings.cameras {
        let found = local
            .cameras
            .iter()
            .position(|c| c.make == incoming.make && c.model == incoming.model);
        let id = if let Some(i) = found {
            local.cameras[i].id
        } else {
            local.cameras.iter().map(|c| c.id).max().unwrap_or(0) + 1
        };
        let profile = crate::frame::CameraProfile {
            id,
            ..incoming.clone()
        };
        match found {
            Some(i) => local.cameras[i] = profile,
            None => local.cameras.push(profile),
        }
        camera_ids.insert(incoming.id, id);
    }
    for (incoming, bytes) in &bundle.logos {
        if let Some(&id) = camera_ids.get(incoming) {
            installed.logos.push((id, bytes.clone()));
        }
    }

    local.render = bundle.settings.render.clone();
    local.frame = bundle.settings.frame.clone();
    local.frame.font = remap(bundle.settings.frame.font);
    Ok(installed)
}

/// The fonts a project actually uses: the frame's (FR-042; the stamp has its own font).
pub fn used_fonts(settings: &StoredSettings) -> Vec<UserFont> {
    let used = [settings.frame.font];
    settings
        .fonts
        .iter()
        .filter(|f| used.contains(&Some(f.id)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::CameraProfile;
    use crate::library::Flag;
    use crate::plan::CropFrame;
    use crate::project::{FolderRow, PathKind, PhotoRow};

    static INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, width, height);
            encoder.set_color(png::ColorType::Rgba);
            if let Ok(mut writer) = encoder.write_header() {
                let _ = writer.write_image_data(&vec![200; (width * height * 4) as usize]);
            }
        }
        out
    }

    fn project() -> ProjectFile {
        ProjectFile {
            format: project::FORMAT.into(),
            schema_version: project::SCHEMA_VERSION,
            app_version: "0.0.1".into(),
            modified_at: "2026-10-24T08:00:00+00:00".into(),
            folders: vec![FolderRow {
                id: 1,
                path: "photos/lisbon".into(),
                path_kind: PathKind::Relative,
                absolute: Some("/Users/someone/trip/photos/lisbon".into()),
            }],
            photos: vec![PhotoRow {
                id: 1,
                folder: 1,
                path: "IMG_0416.JPG".into(),
                fp_size: 3_012_345,
                fp_mtime: 1_726_000_000,
                crop: Some(CropFrame {
                    cx: 0.4,
                    cy: 0.5,
                    zoom: 1.25,
                }),
                rating: Some(4),
                flag: Some(Flag::Pick),
                stamp: None,
                rotation: None,
            }],
            document: None,
        }
    }

    /// A bundle with one font (used by the stamp) and one camera logo.
    fn sample(with_font: bool) -> Bundle {
        let mut settings = StoredSettings::default();
        let mut camera = CameraProfile::new(3, Some("Panasonic".into()), Some("DC-GX9".into()));
        camera.logo = LogoKind::Image;
        camera.lens = Some("Industar-50-2".into());
        settings.cameras = vec![camera];
        settings.render.print_size = "original".into();
        let mut fonts = BTreeMap::new();
        if with_font {
            settings.fonts = vec![UserFont {
                id: 7,
                name: "Inter copy".into(),
                format: FontFormat::Ttf,
            }];
            settings.frame.font = Some(7);
            fonts.insert(7, INTER.to_vec());
        }
        Bundle {
            project: project(),
            settings,
            fonts,
            logos: BTreeMap::from([(3, png(8, 4))]),
        }
    }

    /// A ZIP with exactly these entries, plus a manifest that lists them correctly.
    fn zip_with(entries: &[(&str, Vec<u8>)], method: CompressionMethod) -> Vec<u8> {
        let manifest = Manifest {
            format: FORMAT.into(),
            schema_version: SCHEMA_VERSION,
            app_version: "test".into(),
            parts: entries
                .iter()
                .map(|(path, bytes)| PartEntry {
                    path: (*path).into(),
                    bytes: bytes.len() as u64,
                    blake3: hash(bytes),
                })
                .collect(),
        };
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(method);
        let _ = zip.start_file(MANIFEST, options);
        let _ = zip.write_all(&serde_json::to_vec(&manifest).unwrap_or_default());
        for (path, bytes) in entries {
            let _ = zip.start_file(*path, options);
            let _ = zip.write_all(bytes);
        }
        zip.finish().map(Cursor::into_inner).unwrap_or_default()
    }

    fn valid_parts(bundle: &Bundle) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                PROJECT,
                project::to_bytes(&bundle.project).unwrap_or_default(),
            ),
            (
                SETTINGS,
                settings_store::to_bytes(&bundle.settings).unwrap_or_default(),
            ),
            ("logos/3.png", bundle.logos[&3].clone()),
        ]
    }

    #[test]
    fn fr_023_bundle_round_trip_is_exact_and_deterministic() {
        let mut bundle = sample(true);
        bundle.settings.ui.language = crate::settings::Language::Vi;
        let bytes = to_bytes(&bundle);
        assert!(bytes.as_ref().is_ok_and(|b| is_bundle(b)));
        let bytes = bytes.unwrap_or_default();
        assert_eq!(
            to_bytes(&bundle).ok(),
            Some(bytes.clone()),
            "same input, same bytes"
        );

        let back = from_bytes(&bytes);
        let mut expected = bundle;
        expected.settings.ui = UiSettings::default(); // the UI language stays on each machine
        assert_eq!(back, Ok(expected));
    }

    #[test]
    fn fr_023_bundle_rejects_names_outside_the_allowlist() {
        let bundle = sample(false);
        for evil in [
            "../evil.json",
            "/etc/passwd",
            "C:\\\\Windows\\\\x.png",
            "fonts/../../x.ttf",
            "fonts/01.ttf",
            "fonts/7.woff",
            "logos/3.png.exe",
            "notes.txt",
            "Fonts/1.ttf",
        ] {
            let mut parts = valid_parts(&bundle);
            parts.push((evil, b"x".to_vec()));
            let result = from_bytes(&zip_with(&parts, CompressionMethod::Stored));
            assert!(
                matches!(result, Err(BundleError::Corrupt(_))),
                "{evil} must be rejected"
            );
        }
        // The same parts without the stray entry are fine.
        assert!(from_bytes(&zip_with(&valid_parts(&bundle), CompressionMethod::Stored)).is_ok());
    }

    #[test]
    fn fr_023_bundle_rejects_zip_bombs_before_reading() {
        let bundle = sample(false);
        let mut parts = valid_parts(&bundle);
        // 3 MB of zeros deflates to a few KB: far beyond the 100:1 ratio, and over the
        // 2 MB logo limit too.
        parts[2].1 = vec![0; 3 * 1024 * 1024];
        let result = from_bytes(&zip_with(&parts, CompressionMethod::Deflated));
        assert!(matches!(
            result,
            Err(BundleError::TooLarge | BundleError::Corrupt(_))
        ));

        // Within the size limit but still a bomb by ratio.
        let mut parts = valid_parts(&bundle);
        parts[0].1 = vec![b' '; 1024 * 1024];
        let result = from_bytes(&zip_with(&parts, CompressionMethod::Deflated));
        assert!(matches!(result, Err(BundleError::Corrupt(why)) if why.contains("compressed")));
    }

    #[test]
    fn fr_023_bundle_rejects_damaged_parts_and_bad_content() {
        let bundle = sample(true);
        let good = to_bytes(&bundle).unwrap_or_default();
        // Flip one byte inside a stored part: the checksum no longer matches.
        let mut damaged = good.clone();
        if let Some(pos) = damaged.windows(4).position(|w| w == b"IHDR") {
            damaged[pos + 6] ^= 0xFF;
        }
        assert!(from_bytes(&damaged).is_err());

        // A garbage font, correctly listed and checksummed, still fails font validation.
        let mut parts = valid_parts(&bundle);
        parts.push(("fonts/7.ttf", b"\0\x01\0\0 not really a font".to_vec()));
        let result = from_bytes(&zip_with(&parts, CompressionMethod::Stored));
        assert!(matches!(result, Err(BundleError::Corrupt(why)) if why.contains("fonts/7.ttf")));

        // A logo for a camera that does not use an image logo.
        let mut parts = valid_parts(&sample(false));
        parts.push(("logos/9.png", png(2, 2)));
        assert!(from_bytes(&zip_with(&parts, CompressionMethod::Stored)).is_err());

        // A listed font whose file is missing.
        let mut parts = valid_parts(&bundle);
        parts.retain(|(p, _)| !p.starts_with("fonts/"));
        assert!(from_bytes(&zip_with(&parts, CompressionMethod::Stored)).is_err());
    }

    #[test]
    fn fr_023_bundle_from_a_newer_app_is_refused() {
        let mut parts = valid_parts(&sample(false));
        parts.retain(|_| true);
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        let _ = zip.start_file(MANIFEST, options);
        let _ = zip.write_all(
            br#"{"format":"fotollection-bundle","schema_version":9,"app_version":"9","parts":[]}"#,
        );
        let bytes = zip.finish().map(Cursor::into_inner).unwrap_or_default();
        assert_eq!(from_bytes(&bytes), Err(BundleError::NewerVersion(9)));
        assert_eq!(
            from_bytes(b"{\"format\":\"x\"}"),
            Err(BundleError::NotAProject)
        );
    }

    #[test]
    fn fr_023_open_recognises_both_containers_by_content() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let legacy = dir.path().join("old.fotollection");
        std::fs::write(
            &legacy,
            include_bytes!("../tests/fixtures/project/v1.fotollection"),
        )?;
        assert!(matches!(open(&legacy), Ok(Opened::Legacy(p)) if p.photos.len() == 2));

        let fixture = dir.path().join("trip.fotollection");
        std::fs::write(
            &fixture,
            include_bytes!("../tests/fixtures/bundle/v1.fotollection"),
        )?;
        let Ok(Opened::Bundle(b)) = open(&fixture) else {
            panic!("the v1 bundle fixture must open");
        };
        assert_eq!(b.project.photos[0].rating, Some(4));
        assert_eq!(b.settings.cameras[0].lens.as_deref(), Some("Industar-50-2"));
        assert_eq!(b.logos.len(), 1);
        Ok(())
    }

    #[test]
    fn used_fonts_are_only_the_ones_referenced() {
        let mut s = sample(true).settings;
        s.fonts.push(UserFont {
            id: 8,
            name: "unused".into(),
            format: FontFormat::Otf,
        });
        assert_eq!(used_fonts(&s).iter().map(|f| f.id).collect::<Vec<_>>(), [7]);
    }

    /// `cargo test -p fotollection-core regenerate_bundle_fixture -- --ignored` rewrites the
    /// committed fixture. Only do that when the format version changes (DATABASE.md §9).
    #[test]
    #[ignore = "writes the committed fixture"]
    fn regenerate_bundle_fixture() -> std::io::Result<()> {
        let bytes = to_bytes(&sample(false)).map_err(std::io::Error::other)?;
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/bundle/v1.fotollection"
            ),
            bytes,
        )
    }

    #[test]
    fn fr_023_open_on_another_machine_installs_fonts_once_and_remaps_ids() {
        let incoming = from_bytes(&to_bytes(&sample(true)).unwrap_or_default());
        let Ok(incoming) = incoming else {
            panic!("sample bundle must read back");
        };

        // Machine B already has two fonts (ids 1 and 2) and a GX9 profile with id 9.
        let mut local = StoredSettings {
            fonts: vec![
                UserFont {
                    id: 1,
                    name: "Other".into(),
                    format: FontFormat::Otf,
                },
                UserFont {
                    id: 2,
                    name: "Other 2".into(),
                    format: FontFormat::Ttf,
                },
            ],
            cameras: vec![CameraProfile::new(
                9,
                Some("Panasonic".into()),
                Some("DC-GX9".into()),
            )],
            ui: UiSettings {
                language: crate::settings::Language::Vi,
            },
            ..StoredSettings::default()
        };
        let no_match = |_: &UserFont| Some(b"different bytes".to_vec());
        let installed = merge_settings(&mut local, &incoming, &no_match);
        let Ok(installed) = installed else {
            panic!("merge must succeed");
        };
        // The font is new here: id 3, and the frame now points at it.
        assert_eq!(installed.new_fonts.len(), 1);
        assert_eq!(installed.new_fonts[0].0.id, 3);
        assert_eq!(local.frame.font, Some(3));
        // The camera matched by make + model keeps its local id, takes the project's values.
        assert_eq!(local.cameras.len(), 1);
        assert_eq!(local.cameras[0].id, 9);
        assert_eq!(local.cameras[0].lens.as_deref(), Some("Industar-50-2"));
        assert_eq!(
            installed
                .logos
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            [9]
        );
        assert_eq!(local.render.print_size, "original");
        assert_eq!(
            local.ui.language,
            crate::settings::Language::Vi,
            "language stays"
        );

        // Opening the same project again installs nothing new: the font is recognised.
        let font_three = |f: &UserFont| (f.id == 3).then(|| INTER.to_vec());
        let again = merge_settings(&mut local, &incoming, &font_three);
        assert!(again.is_ok_and(|i| i.new_fonts.is_empty()));
        assert_eq!(local.fonts.len(), 3);
        assert_eq!(local.frame.font, Some(3));
    }
}
