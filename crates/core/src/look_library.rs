// SPDX-License-Identifier: GPL-3.0-only
//! The looks a user picks from (FR-079): the built-in recipes, made by this project from
//! its own sliders under its own names (no third-party LUT, no film or camera trademark,
//! ADR-0038), and the user's own, saved with *Save as look* in `settings.json` (DATABASE
//! §6, v18). Picking a look copies its values into the photo's look with the library
//! reference in `base`, so a later change of the library never changes a photo.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::looks::{Adjust, LookSettings, Mono, Tint, clean_name, valid_base};

/// The most looks a user can keep.
pub const MAX_SAVED: usize = 200;

/// Where a look of the library comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LookKind {
    BuiltIn,
    User,
}

/// One look as the Looks card lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LookEntry {
    /// `builtin:<slug>` or `user:<id>`, the value a photo's look keeps in `base`.
    pub id: String,
    /// The English name; the interface shows a built-in look under its own translation
    /// (`look.builtin.<slug>`) and a user's look as typed.
    pub name: String,
    pub kind: LookKind,
    /// The values a pick copies into the photo, `name` and `base` filled.
    pub look: LookSettings,
}

/// A look the user saved: its own id (lowercase hex), its name, its values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedLook {
    pub id: String,
    pub name: String,
    pub look: LookSettings,
}

fn recipe(slug: &str, name: &str, adjust: Adjust) -> LookEntry {
    let id = format!("builtin:{slug}");
    LookEntry {
        look: LookSettings {
            adjust,
            name: Some(name.to_owned()),
            base: Some(id.clone()),
            ..LookSettings::default()
        },
        id,
        name: name.to_owned(),
        kind: LookKind::BuiltIn,
    }
}

/// The built-in looks, in the order the card shows them. Values are a first proposal,
/// checked by `measure_looks` (neutral greys, no new clipping, looks apart) and approved by
/// eye on real photos before release.
#[must_use]
pub fn built_in() -> Vec<LookEntry> {
    vec![
        recipe("original", "Original", Adjust::default()),
        // The owner's GX9 recipe (FR-039): Natural, contrast −1, saturation +1, highlights
        // a little darker, shadows a little brighter, white balance A+3 / M+1.
        recipe(
            "c200-natural",
            "C200 Natural",
            Adjust {
                contrast: -10,
                saturation: 10,
                highlights: -10,
                shadows: 10,
                warmth: 15,
                tint: 5,
                ..Adjust::default()
            },
        ),
        recipe(
            "warm-film",
            "Warm old film",
            Adjust {
                warmth: 35,
                contrast: -10,
                saturation: -10,
                fade: 20,
                highlight_tint: Tint {
                    hue: 40,
                    amount: 15,
                },
                shadow_tint: Tint {
                    hue: 200,
                    amount: 10,
                },
                ..Adjust::default()
            },
        ),
        recipe(
            "clear",
            "Clear",
            Adjust {
                contrast: 45,
                vibrance: 75,
                saturation: 10,
                highlights: -40,
                shadows: 40,
                ..Adjust::default()
            },
        ),
        recipe(
            "cool",
            "Cool",
            Adjust {
                warmth: -75,
                tint: -15,
                contrast: 5,
                saturation: -5,
                ..Adjust::default()
            },
        ),
        recipe(
            "faded",
            "Faded",
            Adjust {
                fade: 45,
                contrast: -25,
                saturation: -40,
                ..Adjust::default()
            },
        ),
        recipe(
            "mono-red",
            "Black and white, red filter",
            Adjust {
                mono: Mono::Red,
                contrast: 20,
                ..Adjust::default()
            },
        ),
        recipe(
            "mono-soft",
            "Soft black and white",
            Adjust {
                mono: Mono::Neutral,
                contrast: -15,
                fade: 15,
                ..Adjust::default()
            },
        ),
    ]
}

fn valid_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// The saved looks as read from a file or sent by the webview: names cleaned, ids valid
/// and distinct, values in range, at most [`MAX_SAVED`]; anything else is dropped.
#[must_use]
pub fn sanitized_saved(list: &[SavedLook]) -> Vec<SavedLook> {
    let mut seen = std::collections::HashSet::new();
    list.iter()
        .filter_map(|s| {
            let name = clean_name(&s.name)?;
            (valid_id(&s.id) && seen.insert(s.id.clone())).then(|| {
                let base = format!("user:{}", s.id);
                SavedLook {
                    id: s.id.clone(),
                    look: LookSettings {
                        name: Some(name.clone()),
                        base: valid_base(&base).then_some(base),
                        ..s.look.clone()
                    }
                    .sanitized(),
                    name,
                }
            })
        })
        .take(MAX_SAVED)
        .collect()
}

/// Why a look could not be saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SaveLookError {
    #[error("a look needs a name")]
    NoName,
    #[error("no room for another look")]
    Full,
}

/// A new saved look named `name` with `look`'s values, its id unused in `existing`.
pub fn new_saved(
    name: &str,
    look: &LookSettings,
    existing: &[SavedLook],
) -> Result<SavedLook, SaveLookError> {
    let name = clean_name(name).ok_or(SaveLookError::NoName)?;
    if existing.len() >= MAX_SAVED {
        return Err(SaveLookError::Full);
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut salt = 0u32;
    let id = loop {
        let hash = blake3::hash(format!("{name}\u{0}{nanos}\u{0}{salt}").as_bytes());
        let id = hash.to_hex()[..12].to_owned();
        if !existing.iter().any(|s| s.id == id) {
            break id;
        }
        salt += 1;
    };
    let base = format!("user:{id}");
    Ok(SavedLook {
        look: LookSettings {
            name: Some(name.clone()),
            base: Some(base),
            ..look.clone()
        }
        .sanitized(),
        id,
        name,
    })
}

/// Every look the card lists: the built-in ones, then the user's, oldest first.
#[must_use]
pub fn entries(saved: &[SavedLook]) -> Vec<LookEntry> {
    let mut all = built_in();
    all.extend(saved.iter().map(|s| LookEntry {
        id: format!("user:{}", s.id),
        name: s.name.clone(),
        kind: LookKind::User,
        look: s.look.clone(),
    }));
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_079_built_in_looks_are_distinct_valid_and_original_is_identity() {
        let all = built_in();
        let ids: std::collections::HashSet<_> = all.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids.len(), all.len(), "ids are distinct");
        for e in &all {
            assert!(valid_base(&e.id), "{}", e.id);
            assert_eq!(e.look, e.look.clone().sanitized(), "{} in range", e.id);
            assert_eq!(e.look.base.as_deref(), Some(e.id.as_str()));
        }
        assert!(all[0].look.table().is_none(), "Original changes nothing");
        assert!(all[1..].iter().all(|e| e.look.table().is_some()));
    }

    #[test]
    fn fr_079_a_saved_look_keeps_its_values_under_its_own_reference() -> Result<(), SaveLookError> {
        let mut look = LookSettings::default();
        look.adjust.warmth = 40;
        look.base = Some("builtin:clear".into());
        let saved = new_saved("  My summer ", &look, &[])?;
        assert_eq!(saved.name, "My summer");
        assert_eq!(saved.look.adjust.warmth, 40);
        assert_eq!(saved.look.base, Some(format!("user:{}", saved.id)));
        assert_eq!(new_saved(" \n ", &look, &[]), Err(SaveLookError::NoName));
        let full = vec![saved.clone(); MAX_SAVED];
        assert_eq!(
            new_saved("One more", &look, &full),
            Err(SaveLookError::Full)
        );
        let listed = entries(std::slice::from_ref(&saved));
        assert_eq!(listed.len(), built_in().len() + 1);
        assert_eq!(listed.last().map(|e| e.kind), Some(LookKind::User));
        Ok(())
    }

    #[test]
    fn fr_079_saved_looks_from_a_file_are_cleaned() {
        let mut wild = LookSettings::default();
        wild.adjust.contrast = 120;
        let good = SavedLook {
            id: "0a1b2c".into(),
            name: "Fine".into(),
            look: wild,
        };
        let list = vec![
            good.clone(),
            SavedLook {
                name: "Twin".into(),
                ..good.clone()
            },
            SavedLook {
                id: "../x".into(),
                ..good.clone()
            },
            SavedLook {
                id: "ffff".into(),
                name: "\u{7}".into(),
                ..good.clone()
            },
        ];
        let clean = sanitized_saved(&list);
        assert_eq!(clean.len(), 1, "duplicate, bad id and empty name dropped");
        assert_eq!(clean[0].look.adjust.contrast, 100);
        assert_eq!(clean[0].look.base.as_deref(), Some("user:0a1b2c"));
        assert_eq!(clean[0].look.name.as_deref(), Some("Fine"));
    }
}
