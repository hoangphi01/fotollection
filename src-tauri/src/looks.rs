// SPDX-License-Identifier: GPL-3.0-only
//! The library of looks the Looks card lists (FR-079): the built-in recipes and the looks
//! saved on this machine with *Save as look*. Picking one goes through `set_photo_look`.
// Tauri hands commands their state by value.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::library::PhotoId;
use fotollection_core::look_library::{self, LookEntry, LookSource, SaveLookError};
use fotollection_core::looks::LookSettings;
use tauri::State;

use crate::state::{AppState, IpcError};

fn poisoned<E: std::fmt::Display>(e: E) -> IpcError {
    IpcError::internal(e.to_string())
}
fn save_error(e: SaveLookError) -> IpcError {
    match e {
        SaveLookError::NoName => IpcError::new("look.no_name", e.to_string()),
        SaveLookError::Full => IpcError::new("look.full", e.to_string()),
    }
}

/// Changes one saved look (`user:<id>`) and returns the library as the card lists it.
fn edit_saved(
    state: &AppState,
    id: &str,
    edit: impl FnOnce(&mut look_library::SavedLook),
) -> Result<Vec<LookEntry>, IpcError> {
    let list = {
        let mut saved = state.saved_looks.lock().map_err(poisoned)?;
        let own = id.strip_prefix("user:").unwrap_or_default();
        let found = saved
            .iter_mut()
            .find(|s| s.id == own)
            .ok_or_else(|| IpcError::new("look.unknown", "no saved look with this id"))?;
        edit(found);
        let clean = look_library::sanitized_saved(&saved);
        *saved = clean;
        look_library::entries(&saved)
    };
    state.save_settings();
    Ok(list)
}

/// Renames a look of the user's (FR-079); the name is cleaned in Rust.
#[tauri::command]
#[specta::specta]
pub fn rename_look(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Vec<LookEntry>, IpcError> {
    let name = fotollection_core::looks::clean_name(&name)
        .ok_or_else(|| IpcError::new("look.no_name", "a look needs a name"))?;
    edit_saved(&state, &id, |s| s.name = name)
}

/// *Update with current settings*, as Lightroom's: the look of the user's takes `look`'s
/// values (the photo's look now), keeping its name and group.
#[tauri::command]
#[specta::specta]
pub fn update_look(
    state: State<'_, AppState>,
    id: String,
    look: LookSettings,
) -> Result<Vec<LookEntry>, IpcError> {
    edit_saved(&state, &id, |s| s.look = look.sanitized())
}

/// Moves a look of the user's to another group of *Presets & LUTs*; `None` is no group (the
/// user's own looks).
#[tauri::command]
#[specta::specta]
pub fn move_look(
    state: State<'_, AppState>,
    id: String,
    group: Option<String>,
) -> Result<Vec<LookEntry>, IpcError> {
    let group = group
        .as_deref()
        .and_then(fotollection_core::looks::clean_name);
    edit_saved(&state, &id, |s| s.group = group)
}

/// Every look the card lists: the built-in ones, then this machine's, oldest first.
#[tauri::command]
#[specta::specta]
pub fn list_looks(state: State<'_, AppState>) -> Result<Vec<LookEntry>, IpcError> {
    let saved = state.saved_looks.lock().map_err(poisoned)?;
    Ok(look_library::entries(&saved))
}

/// Saves `look` under `name` (FR-079) and returns it as the card lists it. The name is
/// cleaned and the values clamped in Rust, whatever the webview sent.
#[tauri::command]
#[specta::specta]
pub fn save_look(
    state: State<'_, AppState>,
    name: String,
    look: LookSettings,
) -> Result<LookEntry, IpcError> {
    let entry = {
        let mut saved = state.saved_looks.lock().map_err(poisoned)?;
        let new = look_library::new_saved(&name, &look, &saved).map_err(save_error)?;
        saved.push(new);
        look_library::entries(&saved).pop()
    };
    state.save_settings();
    entry.ok_or_else(|| IpcError::internal("the saved look is missing"))
}

/// Forgets a saved look (`user:<id>`) and returns the list left. Photos that used it keep
/// their values: a look is copied into a photo, never linked.
#[tauri::command]
#[specta::specta]
pub fn delete_look(state: State<'_, AppState>, id: String) -> Result<Vec<LookEntry>, IpcError> {
    let left = {
        let mut saved = state.saved_looks.lock().map_err(poisoned)?;
        let own = id.strip_prefix("user:").unwrap_or_default();
        let before = saved.len();
        saved.retain(|s| s.id != own);
        if saved.len() == before {
            return Err(IpcError::new("look.unknown", "no saved look with this id"));
        }
        look_library::entries(&saved)
    };
    state.save_settings();
    Ok(left)
}

/// Bytes for `fotothumb://localhost/looktile/{id}?look=<library id>`: the photo through that
/// look of the library, small and square (FR-079).
pub fn look_tile_bytes(state: &AppState, id: PhotoId, look: &str) -> Result<Vec<u8>, IpcError> {
    let entry = {
        let saved = state.saved_looks.lock().map_err(poisoned)?;
        look_library::entries(&saved)
            .into_iter()
            .find(|e| e.id == look)
            .ok_or_else(|| IpcError::new("look.unknown", "no look with this id"))?
    };
    let record = state
        .library
        .read()
        .map_err(poisoned)?
        .record(id)
        .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
    let thumbnail =
        |r: &fotollection_core::library::PhotoRecord| crate::commands::thumbnail_bytes(state, r.id);
    state
        .previews
        .look_tile_jpeg(&record, &entry.look, &thumbnail, state.codec.as_ref())
        .map_err(|e| crate::commands::shown_error(&e, &record))
}

/// Imports a LUT (FR-070): a `.cube` or a `HaldCLUT` PNG chosen in a Rust-side dialog, parsed
/// and kept by its content hash, then saved as a look of the user's named by its title or file
/// name. `None` when the dialog is cancelled; a refused file says why (`look.lut_*`).
#[tauri::command]
#[specta::specta]
pub async fn import_lut(app: tauri::AppHandle) -> Result<Option<LookEntry>, IpcError> {
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let picked = app
            .dialog()
            .file()
            .set_title("Choose a LUT (.cube or HaldCLUT PNG)")
            .add_filter("LUT", &["cube", "png"])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let size = std::fs::metadata(&path)
            .map_err(|e| IpcError::new("look.lut_unreadable", e.to_string()))?
            .len();
        if size > fotollection_core::lut::MAX_BYTES as u64 {
            return Err(IpcError::new(
                "look.lut_too_large",
                "the LUT is larger than 8 MB",
            ));
        }
        let bytes = std::fs::read(&path)
            .map_err(|e| IpcError::new("look.lut_unreadable", e.to_string()))?;
        let (hash, lut) = fotollection_core::lut::import(&bytes)
            .map_err(|e| IpcError::new(e.code(), e.to_string()))?;
        let name = lut
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "LUT".to_owned());
        let look = LookSettings {
            lut: Some(hash),
            ..LookSettings::default()
        };
        let entry = {
            let mut saved = state.saved_looks.lock().map_err(poisoned)?;
            let mut new = look_library::new_saved(&name, &look, &saved).map_err(save_error)?;
            new.group = Some("LUT".to_owned());
            new.source = LookSource::Lut;
            saved.push(new);
            look_library::entries(&saved).pop()
        };
        state.save_settings();
        entry
            .map(Some)
            .ok_or_else(|| IpcError::internal("the imported look is missing"))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// What an import of presets did (FR-089): the looks it made, the files it skipped and why,
/// and how many presets had each setting no look can hold.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub imported: Vec<LookEntry>,
    pub skipped: Vec<Skipped>,
    pub unapplied: Vec<Unapplied>,
}

/// A file an import left out, with the stable code of the reason (`look.preset_*`).
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub file: String,
    pub code: String,
    pub reason: String,
}

/// A setting no look holds, as `Clarity2012: local contrast…`, and how many presets had it.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Unapplied {
    pub setting: String,
    pub count: u32,
}

type Found = (
    String,
    Option<String>,
    Result<fotollection_core::preset::Preset, fotollection_core::preset::PresetError>,
);

/// Every preset in the chosen files: a `.zip`'s entries under the zip's name as their pack.
fn read_presets(paths: Vec<std::path::PathBuf>) -> Vec<Found> {
    use fotollection_core::preset::{self, PresetError};
    let mut found = Vec::new();
    for path in paths.into_iter().take(preset::MAX_ZIP_ENTRIES) {
        let file = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stem = path.file_stem().map(|n| n.to_string_lossy().into_owned());
        let zip = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
        let cap = if zip {
            preset::MAX_ZIP_BYTES
        } else {
            preset::MAX_BYTES
        } as u64;
        let read = std::fs::metadata(&path)
            .map_err(|e| PresetError::Invalid(e.to_string()))
            .and_then(|m| {
                if m.len() > cap {
                    Err(PresetError::TooLarge)
                } else {
                    Ok(())
                }
            })
            .and_then(|()| std::fs::read(&path).map_err(|e| PresetError::Invalid(e.to_string())));
        match (zip, read) {
            (_, Err(e)) => found.push((file, None, Err(e))),
            (true, Ok(bytes)) => match preset::parse_zip(&bytes) {
                Ok(entries) => found.extend(entries.into_iter().map(|(f, r)| (f, stem.clone(), r))),
                Err(e) => found.push((file, None, Err(e))),
            },
            (false, Ok(bytes)) => found.push((file, None, preset::parse(&bytes))),
        }
    }
    found
}

/// Imports Lightroom presets (FR-089, ADR-0045): `.xmp`, `.lrtemplate` or `.zip` files chosen
/// in a Rust-side dialog, each read and compiled in Rust, its per-pixel part kept as a LUT,
/// saved as looks of the user's under the preset's group (or its pack's name). `None` when the
/// dialog is cancelled.
#[tauri::command]
#[specta::specta]
pub async fn import_presets(app: tauri::AppHandle) -> Result<Option<ImportReport>, IpcError> {
    use fotollection_core::preset::PresetError;
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let Some(paths) = app
            .dialog()
            .file()
            .set_title("Choose Lightroom presets (.xmp, .lrtemplate or .zip)")
            .add_filter("Lightroom presets", &["xmp", "lrtemplate", "zip"])
            .blocking_pick_files()
        else {
            return Ok(None);
        };
        let found = read_presets(
            paths
                .into_iter()
                .filter_map(|p| p.into_path().ok())
                .collect(),
        );
        let mut report = ImportReport {
            imported: Vec::new(),
            skipped: Vec::new(),
            unapplied: Vec::new(),
        };
        let mut counts: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let skip = |report: &mut ImportReport, file: String, code: &str, reason: String| {
            report.skipped.push(Skipped {
                file,
                code: code.to_owned(),
                reason,
            });
        };
        for (file, pack, result) in found {
            let preset = match result {
                Ok(p) => p,
                Err(e) => {
                    skip(&mut report, file, e.code(), e.to_string());
                    continue;
                }
            };
            let compiled = fotollection_core::preset_look::compile(&preset);
            let mut look = compiled.look;
            if let Some(cube) = &compiled.lut {
                match fotollection_core::lut::import(cube.as_bytes()) {
                    Ok((hash, _)) => look.lut = Some(hash),
                    Err(e) => {
                        skip(&mut report, file, e.code(), e.to_string());
                        continue;
                    }
                }
            }
            if look.lut.is_none() && look.adjust == fotollection_core::looks::Adjust::default() {
                let e = PresetError::Empty;
                skip(&mut report, file, e.code(), e.to_string());
                continue;
            }
            let name = preset
                .name
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| {
                    file.rsplit_once('.')
                        .map_or(file.clone(), |(stem, _)| stem.to_owned())
                });
            let mut saved = state.saved_looks.lock().map_err(poisoned)?;
            let mut new = look_library::new_saved(&name, &look, &saved).map_err(save_error)?;
            new.group = preset
                .group
                .clone()
                .or(pack)
                .or_else(|| Some("Presets".to_owned()));
            new.source = LookSource::Preset;
            new.unapplied.clone_from(&compiled.unapplied);
            for u in &compiled.unapplied {
                *counts.entry(u.clone()).or_default() += 1;
            }
            saved.push(new);
            if let Some(entry) = look_library::entries(&saved).pop() {
                report.imported.push(entry);
            }
        }
        report.unapplied = counts
            .into_iter()
            .map(|(setting, count)| Unapplied { setting, count })
            .collect();
        if !report.imported.is_empty() {
            state.save_settings();
        }
        Ok(Some(report))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Bytes for `fotothumb://localhost/looktable/{id}?look=<library id>`: that look's table for
/// the photo, as the shader reads it (RGBA8, the route `/look/`'s format), for the hover
/// preview of *Presets & LUTs* (FR-089). A RAW's linear sliders are left out as when a slider
/// is dragged (FR-088); a look that changes nothing gives the identity, so the photo shows
/// without its own.
pub fn look_table_bytes(state: &AppState, id: PhotoId, look: &str) -> Result<Vec<u8>, IpcError> {
    let entry = {
        let saved = state.saved_looks.lock().map_err(poisoned)?;
        look_library::entries(&saved)
            .into_iter()
            .find(|e| e.id == look)
            .ok_or_else(|| IpcError::new("look.unknown", "no look with this id"))?
    };
    let mut record = state
        .library
        .read()
        .map_err(poisoned)?
        .record(id)
        .ok_or_else(|| IpcError::new("photo.unknown", "Unknown photo"))?;
    record.edits.look = Some(entry.look);
    let settings = state.settings.read().map_err(poisoned)?.0.clone();
    Ok(
        fotollection_core::preview::PreviewCache::look_table(&record, &settings)
            .unwrap_or_else(|| fotollection_core::looks::Table::identity().to_rgba8()),
    )
}

/// What a look applied to many photos covers (FR-114): how many photos, and how many of them
/// have a colour of their own that it would replace, so the panel can ask once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LookBatchPlan {
    pub total: u32,
    pub edited: u32,
}

/// What a look applied to many photos did (FR-114): the photos it was given to, and those
/// kept as they were because they had a colour of their own and the user chose to keep it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LookBatchReport {
    pub applied: u32,
    pub kept: u32,
}

/// A look given to many photos (FR-114): which photos (the ticked ones, else every photo the
/// filter shows), which look of the library, at what amount and under what name, and whether
/// photos with a colour of their own keep it.
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LookBatchRequest {
    pub filter: fotollection_core::library::CullFilter,
    pub ticked: Vec<PhotoId>,
    pub look: String,
    pub amount: u8,
    pub name: Option<String>,
    pub keep_edited: bool,
}

/// The look a batch gives each photo: the library's entry as it is, at `amount` (0–100) and
/// under the name the interface shows (a built-in's translation), cleaned as any look is.
fn batch_look(entry: &LookEntry, amount: u8, name: Option<String>) -> LookSettings {
    LookSettings {
        amount: amount.min(100),
        name: name.or_else(|| Some(entry.name.clone())),
        ..entry.look.clone()
    }
    .sanitized()
}

/// A colour of its own that a batch would replace: a look that does something and is not
/// already the one being given (a photo that follows the shared look has none).
fn has_own_colour(current: Option<&LookSettings>, next: &LookSettings) -> bool {
    current.is_some_and(|l| !l.is_identity() && l != next)
}

fn library_entry(state: &AppState, look: &str) -> Result<LookEntry, IpcError> {
    let saved = state.saved_looks.lock().map_err(poisoned)?;
    look_library::entries(&saved)
        .into_iter()
        .find(|e| e.id == look)
        .ok_or_else(|| IpcError::new("look.unknown", "no look with this id"))
}

/// The photos a batch covers, as an export's (FR-050): the ticked ones, else every photo the
/// filter shows; at most 10,000.
fn batch_ids(
    state: &AppState,
    filter: &fotollection_core::library::CullFilter,
    ticked: &[PhotoId],
) -> Result<Vec<PhotoId>, IpcError> {
    crate::commands::check_ticked(ticked)?;
    let library = state.library.read().map_err(poisoned)?;
    let ids = crate::commands::export_ids(None, ticked, || library.filtered_ids(filter));
    if ids.is_empty() || ids.len() > crate::commands::MAX_EDIT_IDS {
        return Err(IpcError::new(
            "photo.invalid_selection",
            "Select between 1 and 10,000 photos",
        ));
    }
    Ok(ids)
}

/// How many photos a look given to the ticked ones (else to every photo shown) covers, and
/// how many have a colour of their own (FR-114). Nothing changes.
#[tauri::command]
#[specta::specta]
pub fn look_batch_plan(
    state: State<'_, AppState>,
    filter: fotollection_core::library::CullFilter,
    ticked: Vec<PhotoId>,
    look: String,
) -> Result<LookBatchPlan, IpcError> {
    let next = batch_look(&library_entry(&state, &look)?, 100, None);
    let ids = batch_ids(&state, &filter, &ticked)?;
    let library = state.library.read().map_err(poisoned)?;
    let edited = ids
        .iter()
        .filter(|&&id| has_own_colour(library.edits(id).look.as_ref(), &next))
        .count();
    Ok(LookBatchPlan {
        total: u32::try_from(ids.len()).unwrap_or(u32::MAX),
        edited: u32::try_from(edited).unwrap_or(u32::MAX),
    })
}

/// Gives a look of the library to the ticked photos, else to every photo shown (FR-114), as
/// Lightroom applies a preset only at import: one undo step for all of them. A photo's crop,
/// stamp and RAW look stay; with `keepEdited`, a photo with a colour of its own is left as it
/// is. The look is built here from the library, never taken from the interface.
#[tauri::command]
#[specta::specta]
pub fn apply_look_to_photos(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: LookBatchRequest,
) -> Result<LookBatchReport, IpcError> {
    let LookBatchRequest {
        filter,
        ticked,
        look,
        amount,
        name,
        keep_edited,
    } = request;
    let next = batch_look(&library_entry(&state, &look)?, amount, name);
    let ids = batch_ids(&state, &filter, &ticked)?;
    let (give, keep): (Vec<PhotoId>, Vec<PhotoId>) = {
        let library = state.library.read().map_err(poisoned)?;
        ids.iter().partition(|&&id| {
            !keep_edited || !has_own_colour(library.edits(id).look.as_ref(), &next)
        })
    };
    let report = LookBatchReport {
        applied: u32::try_from(give.len()).unwrap_or(u32::MAX),
        kept: u32::try_from(keep.len()).unwrap_or(u32::MAX),
    };
    if !give.is_empty() {
        crate::commands::edit_photos_as(&app, &state, &give, None, |_, edits| {
            edits.look = Some(next.clone());
        })?;
        // Their thumbnails coloured in the background, the import's bar showing it.
        crate::commands::warm_look_thumbnails(&app, give);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fotollection_core::look_library::built_in;

    fn entry(id: &str) -> LookEntry {
        let Some(e) = built_in().into_iter().find(|e| e.id == id) else {
            panic!("no built-in look {id}");
        };
        e
    }

    /// FR-114: the look a batch gives is the library's, at the amount asked (at most 100),
    /// under the name the interface shows, its base kept so the Looks card marks it.
    #[test]
    fn fr_114_a_batch_gives_the_librarys_look() {
        let c200 = entry("builtin:c200-natural");
        let look = batch_look(&c200, 250, Some("C200 Tự nhiên".into()));
        assert_eq!(look.amount, 100);
        assert_eq!(look.adjust, c200.look.adjust);
        assert_eq!(look.base.as_deref(), Some("builtin:c200-natural"));
        assert_eq!(look.name.as_deref(), Some("C200 Tự nhiên"));
        assert_eq!(
            batch_look(&c200, 40, None).name.as_deref(),
            Some("C200 Natural")
        );
    }

    /// FR-114: only a look that does something and is not the one given counts as a photo's
    /// own colour; the same look with its sliders moved does.
    #[test]
    fn fr_114_a_colour_of_its_own() {
        let next = batch_look(&entry("builtin:c200-natural"), 100, None);
        assert!(!has_own_colour(None, &next));
        assert!(!has_own_colour(Some(&LookSettings::default()), &next));
        assert!(!has_own_colour(Some(&next), &next));
        let warm = batch_look(&entry("builtin:warm-film"), 100, None);
        assert!(has_own_colour(Some(&warm), &next));
        let mut moved = next.clone();
        moved.adjust.contrast += 20;
        assert!(has_own_colour(Some(&moved), &next));
    }
}
