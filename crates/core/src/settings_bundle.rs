// SPDX-License-Identifier: GPL-3.0-only
//! The app's brain in one file (`.fotosettings`, FR-082, FR-116): this machine's settings
//! (the interface, the defaults for new projects, cameras, fonts, saved looks, custom sizes,
//! the recent addresses only when asked) with the fonts, logos and LUT files they refer to,
//! to move to another computer. The same ZIP container as a project (`zip_parts`), the same
//! readers for every part; never a photo, a project or a cache. Untrusted on import.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bundle::{self, BundleError, Installed, Part};
use crate::fonts::UserFont;
use crate::frame::LogoKind;
use crate::settings_store::{self, StoredSettings};
use crate::zip_parts::{self, Limits, PartKind};

pub const FORMAT: &str = "fotollection-settings-bundle";
pub const SCHEMA_VERSION: u32 = 1;
pub const EXTENSION: &str = "fotosettings";
const SETTINGS: &str = "settings.json";
const LIMITS: Limits = Limits {
    entries: bundle::MAX_ENTRIES,
    total: bundle::MAX_TOTAL_BYTES,
    ratio: bundle::MAX_RATIO,
};

/// A project's allowlist less its tables: settings, fonts, logos and LUTs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SettingsPart(Part);

impl PartKind for SettingsPart {
    fn parse(name: &str) -> Option<Self> {
        match Part::parse(name)? {
            Part::Project => None,
            part => Some(Self(part)),
        }
    }
    fn limit(self) -> u64 {
        self.0.limit()
    }
}

/// What a `.fotosettings` file holds.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsBundle {
    pub settings: StoredSettings,
    pub fonts: BTreeMap<u32, Vec<u8>>,
    pub logos: BTreeMap<u32, Vec<u8>>,
    pub luts: BTreeMap<String, Vec<u8>>,
}

/// What an import would bring, shown before anything changes (FR-116).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub looks: u32,
    pub luts: u32,
    pub fonts: u32,
    pub logos: u32,
    pub cameras: u32,
    pub sizes: u32,
    pub addresses: u32,
}

/// How an import meets this machine's settings (owner, 2026-11-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ImportMode {
    /// Adds what this machine lacks (looks, LUTs, fonts, logos, cameras, sizes, addresses);
    /// its interface and its defaults for new projects stay.
    Merge,
    /// The file's settings whole, its fonts given this machine's ids.
    Replace,
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl SettingsBundle {
    pub fn summary(&self) -> ImportSummary {
        let s = &self.settings;
        ImportSummary {
            looks: count(s.looks.len()),
            luts: count(self.luts.len()),
            fonts: count(s.fonts.len()),
            logos: count(self.logos.len()),
            cameras: count(s.cameras.len()),
            sizes: count(s.render.custom_sizes.len()),
            addresses: count(s.recent_addresses.len()),
        }
    }
}

/// The LUTs this machine's settings use: its saved looks' and its default look's.
pub fn used_luts(settings: &StoredSettings) -> std::collections::BTreeSet<String> {
    settings
        .looks
        .iter()
        .map(|l| &l.look)
        .chain(std::iter::once(&settings.render.look))
        .filter_map(|look| look.lut.clone())
        .collect()
}

/// The file's bytes; the recent addresses stay out unless `include_addresses` (owner: a
/// box ticked at export, off by default, since they are other people's).
pub fn to_bytes(bundle: &SettingsBundle, include_addresses: bool) -> Result<Vec<u8>, BundleError> {
    let write_err = |e: &dyn std::fmt::Display| BundleError::Write(e.to_string());
    let settings = StoredSettings {
        recent_addresses: if include_addresses {
            bundle.settings.recent_addresses.clone()
        } else {
            Vec::new()
        },
        ..bundle.settings.clone()
    };
    let mut parts = vec![(
        SETTINGS.to_owned(),
        settings_store::to_bytes(&settings).map_err(|e| write_err(&e))?,
    )];
    for font in &settings.fonts {
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
    for (hash, bytes) in &bundle.luts {
        let ext = crate::lut::check(bytes).map_err(|e| write_err(&e))?;
        parts.push((format!("looks/{hash}.{ext}"), bytes.clone()));
    }
    let deflate = |path: &str| path == SETTINGS || path.to_ascii_lowercase().ends_with(".cube");
    zip_parts::write(FORMAT, SCHEMA_VERSION, &parts, deflate).map_err(BundleError::Write)
}

/// Reads and fully checks a `.fotosettings`: nothing in it is trusted until this returns.
pub fn from_bytes(bytes: &[u8]) -> Result<SettingsBundle, BundleError> {
    if !bundle::is_bundle(bytes) {
        return Err(BundleError::NotAProject);
    }
    if bytes.len() as u64 > bundle::MAX_TOTAL_BYTES {
        return Err(BundleError::TooLarge);
    }
    let (_, data) = zip_parts::read::<SettingsPart>(bytes, LIMITS, FORMAT, SCHEMA_VERSION)
        .map_err(bundle::from_zip)?;
    let mut data: zip_parts::Parts<Part> =
        data.into_iter().map(|(n, (p, b))| (n, (p.0, b))).collect();
    let settings = match data
        .remove(SETTINGS)
        .map(|(_, b)| settings_store::from_bytes(&b))
    {
        Some(settings_store::Loaded::Ok(s)) => s,
        Some(settings_store::Loaded::NewerVersion(v)) => return Err(BundleError::NewerVersion(v)),
        Some(settings_store::Loaded::Corrupt(why)) => return Err(BundleError::Corrupt(why)),
        Some(settings_store::Loaded::Missing | settings_store::Loaded::Recovered(_)) | None => {
            return Err(BundleError::Corrupt("no settings".into()));
        }
    };
    let (fonts, logos, luts) = bundle::check_assets(data, &settings)?;
    Ok(SettingsBundle {
        settings,
        fonts,
        logos,
        luts,
    })
}

/// `incoming` met with `local` in `mode`; returns the fonts and logos to write (LUTs are the
/// bundle's own, installed by hash). Pure: `local` is changed, nothing else.
pub fn apply(
    local: &mut StoredSettings,
    incoming: &SettingsBundle,
    mode: ImportMode,
    local_font: &dyn Fn(&UserFont) -> Option<Vec<u8>>,
) -> Result<Installed, BundleError> {
    let theirs = &incoming.settings;
    let (installed, font_ids) =
        bundle::merge_library(local, theirs, &incoming.fonts, &incoming.logos, local_font)?;
    let remap = |id: Option<u32>| id.and_then(|id| font_ids.get(&id).copied());
    match mode {
        ImportMode::Merge => {
            for look in &theirs.looks {
                if !local.looks.iter().any(|l| l.id == look.id) {
                    local.looks.push(look.clone());
                }
            }
            for size in &theirs.render.custom_sizes {
                let same = |s: &crate::settings::CustomPrintSize| {
                    s.short_mm == size.short_mm && s.long_mm == size.long_mm && s.name == size.name
                };
                if local.render.custom_sizes.iter().any(same) {
                    continue;
                }
                let mut size = size.clone();
                if local.render.custom_sizes.iter().any(|s| s.id == size.id) {
                    let next = local
                        .render
                        .custom_sizes
                        .iter()
                        .filter_map(|s| s.id.strip_prefix("custom-")?.parse::<u32>().ok())
                        .max()
                        .unwrap_or(0)
                        + 1;
                    size.id = format!("custom-{next}");
                }
                local.render.custom_sizes.push(size);
            }
            let mut addresses = local.recent_addresses.clone();
            addresses.extend(theirs.recent_addresses.iter().cloned());
            local.recent_addresses = crate::settings::sanitized_addresses(&addresses);
        }
        ImportMode::Replace => {
            // The folder to export into is a path of the other computer: this one's stays.
            let destination = local.ui.export_destination.take();
            local.ui = theirs.ui.clone();
            local.ui.export_destination = destination;
            local.render = theirs.render.clone();
            local.frame = theirs.frame.clone();
            // Their font choices, in this machine's ids (Merge keeps this machine's own).
            local.frame.font = remap(theirs.frame.font);
            local.render.collage.postcard.font = remap(theirs.render.collage.postcard.font);
            local.looks.clone_from(&theirs.looks);
            if !theirs.recent_addresses.is_empty() {
                local.recent_addresses.clone_from(&theirs.recent_addresses);
            }
            // A camera both know takes the file's profile; its logo stays this machine's.
            for camera in &theirs.cameras {
                if let Some(mine) = local
                    .cameras
                    .iter_mut()
                    .find(|c| c.make == camera.make && c.model == camera.model)
                {
                    let logo = if mine.logo == LogoKind::Image {
                        LogoKind::Image
                    } else {
                        camera.logo
                    };
                    *mine = crate::frame::CameraProfile {
                        id: mine.id,
                        logo,
                        ..camera.clone()
                    };
                }
            }
        }
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look_library::SavedLook;

    const CUBE: &[u8] = b"LUT_3D_SIZE 2\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";

    fn sample() -> SettingsBundle {
        let hash = blake3::hash(CUBE).to_hex().to_string();
        let mut settings = StoredSettings::default();
        let look = crate::looks::LookSettings {
            lut: Some(hash.clone()),
            ..crate::looks::LookSettings::default()
        };
        settings.looks.push(SavedLook {
            id: "a1b2c3d4e5f6".into(),
            name: "Sapa".into(),
            look,
            group: None,
            source: crate::look_library::LookSource::default(),
            unapplied: Vec::new(),
        });
        settings.recent_addresses = vec!["Váci utca 1, Budapest".into()];
        settings.ui.language = crate::settings::Language::Vi;
        SettingsBundle {
            settings,
            fonts: BTreeMap::new(),
            logos: BTreeMap::new(),
            luts: BTreeMap::from([(hash, CUBE.to_vec())]),
        }
    }

    /// FR-082: a settings bundle reads back as written; the addresses only when asked.
    #[test]
    fn fr_082_a_settings_bundle_reads_back() {
        let bundle = sample();
        let Ok(Ok(with)) = to_bytes(&bundle, true).map(|b| from_bytes(&b)) else {
            panic!("reads back with addresses");
        };
        // Saved looks are sanitized on reading (their name and base filled in): what must
        // travel is there.
        assert_eq!(with.settings.looks.len(), 1);
        assert_eq!(
            with.settings.looks[0].look.lut,
            bundle.settings.looks[0].look.lut
        );
        assert_eq!(
            with.settings.recent_addresses,
            bundle.settings.recent_addresses
        );
        assert_eq!(with.luts, bundle.luts);
        let Ok(Ok(without)) = to_bytes(&bundle, false).map(|b| from_bytes(&b)) else {
            panic!("reads back without addresses");
        };
        assert_eq!(without.settings.recent_addresses, Vec::<String>::new());
        assert_eq!(without.luts, bundle.luts);
    }

    /// A project is not a settings bundle, and a newer settings bundle is refused as newer.
    #[test]
    fn fr_082_other_files_are_refused() {
        assert_eq!(from_bytes(b"not a zip"), Err(BundleError::NotAProject));
        let newer = zip_parts::write(FORMAT, SCHEMA_VERSION + 1, &[], |_| true).unwrap_or_default();
        assert_eq!(
            from_bytes(&newer),
            Err(BundleError::NewerVersion(SCHEMA_VERSION + 1))
        );
        let project = zip_parts::write(bundle::FORMAT, 1, &[], |_| true).unwrap_or_default();
        assert_eq!(from_bytes(&project), Err(BundleError::NotAProject));
    }

    /// Merge adds the file's looks and addresses, keeps this machine's interface; Replace takes
    /// the file's interface and looks.
    #[test]
    fn fr_082_merge_adds_and_replace_takes() {
        let incoming = sample();
        let mut local = StoredSettings::default();
        let Ok(_) = apply(&mut local, &incoming, ImportMode::Merge, &|_| None) else {
            panic!("merges");
        };
        assert_eq!(local.looks.len(), 1);
        assert_eq!(local.recent_addresses, incoming.settings.recent_addresses);
        assert_eq!(
            local.ui.language,
            crate::settings::Language::En,
            "the interface stays"
        );
        let Ok(_) = apply(&mut local, &incoming, ImportMode::Merge, &|_| None) else {
            panic!("merges again");
        };
        assert_eq!(
            local.looks.len(),
            1,
            "a look already here is not added twice"
        );
        let mut fresh = StoredSettings::default();
        let Ok(_) = apply(&mut fresh, &incoming, ImportMode::Replace, &|_| None) else {
            panic!("replaces");
        };
        assert_eq!(fresh.ui.language, crate::settings::Language::Vi);
        assert_eq!(fresh.looks, incoming.settings.looks);
    }

    /// The LUTs a machine's settings use: its saved looks' and its default look's.
    #[test]
    fn fr_082_used_luts_come_from_the_looks() {
        let b = sample();
        assert_eq!(used_luts(&b.settings).len(), 1);
    }
}
