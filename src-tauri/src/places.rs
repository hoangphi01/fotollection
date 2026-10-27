// SPDX-License-Identifier: GPL-3.0-only
//! Dates and places set by hand (FR-032) and renamed places (FR-005), ADR-0027. Every input
//! is checked here: ids and counts in `edit_photos`, dates against 1826–2100, place ids
//! against the bundled places, names to `MAX_PLACE_NAME` characters.

// Tauri injects `State`, `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;

use fotollection_core::culling::similar::Signature;
use fotollection_core::geocode::{self, Place};
use fotollection_core::library::{
    MAX_MANUAL_DATE, MIN_MANUAL_DATE, PhotoEdits, PhotoId, PhotoPlace, PlaceSource,
};
use fotollection_core::places::{self, Borrow, Confidence};
use rayon::prelude::*;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::commands::{HistoryState, changed, edit_photos};
use crate::state::{AppState, IpcError};

/// Longest search text and most results (IPC input bounds, SECURITY.md §2).
const MAX_QUERY: usize = 100;
const MAX_RESULTS: u32 = 50;

/// Places whose name starts with `query`, largest first, from the bundled data (FR-032).
#[tauri::command]
#[specta::specta]
pub fn search_places(query: String, limit: u32) -> Result<Vec<Place>, IpcError> {
    if query.chars().count() > MAX_QUERY {
        return Err(IpcError::new(
            "place.query_too_long",
            "The search is too long",
        ));
    }
    geocode::search(&query, limit.min(MAX_RESULTS) as usize)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// `YYYY-MM-DDTHH:MM` or `YYYY-MM-DDTHH:MM:SS` within 1826–2100, as library seconds.
fn parse_date(text: &str) -> Result<i64, IpcError> {
    let bad = || IpcError::new("photo.invalid_date", "Pick a date between 1826 and 2100");
    let date = chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M"))
        .map_err(|_| bad())?;
    let seconds = date.and_utc().timestamp();
    (MIN_MANUAL_DATE..=MAX_MANUAL_DATE)
        .contains(&seconds)
        .then_some(seconds)
        .ok_or_else(bad)
}

/// Sets a date by hand on photos (FR-032), or (`None`) goes back to the camera's. One
/// undoable step; the photos move in the grid, and stamps and day folders follow.
#[tauri::command]
#[specta::specta]
pub fn set_photo_date(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    date: Option<String>,
) -> Result<HistoryState, IpcError> {
    let date = date.as_deref().map(parse_date).transpose()?;
    edit_photos(&app, &state, &ids, |_, e| e.date = date)
}

/// Sets a place by hand on photos (FR-032), or (`None`) goes back to their own GPS. One
/// undoable step.
#[tauri::command]
#[specta::specta]
pub fn set_photo_place(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<PhotoId>,
    geonames_id: Option<u32>,
) -> Result<HistoryState, IpcError> {
    if let Some(id) = geonames_id
        && geocode::by_id(id).ok().flatten().is_none()
    {
        return Err(IpcError::new("place.unknown", "Unknown place"));
    }
    let place = geonames_id.map(|geonames_id| PhotoPlace {
        geonames_id,
        source: PlaceSource::Manual,
    });
    edit_photos(&app, &state, &ids, |_, e| e.place = place)
}

/// A place used in the library, for the Places card (FR-005).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaceUse {
    pub geonames_id: u32,
    /// The user's name for it, or the dataset's.
    pub name: String,
    /// The dataset's name.
    pub original: String,
    pub country: String,
    pub renamed: bool,
    pub photos: u32,
}

/// Every place the library's photos have (set, borrowed or from GPS), most photos first.
#[tauri::command]
#[specta::specta]
pub async fn list_places(app: AppHandle) -> Result<Vec<PlaceUse>, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        let names = library.place_names();
        let mut counts: HashMap<u32, u32> = HashMap::new();
        for id in library.sorted_ids() {
            let Some(record) = library.record(id) else {
                continue;
            };
            let place = places::effective(&record, names, geocode::PlaceFormat::City);
            if let Some(p) = place {
                *counts.entry(p.geonames_id).or_default() += 1;
            }
        }
        let mut list: Vec<PlaceUse> = counts
            .into_iter()
            .filter_map(|(id, photos)| {
                let place = geocode::by_id(id).ok().flatten()?;
                let renamed = names.get(&id);
                Some(PlaceUse {
                    geonames_id: id,
                    name: renamed.cloned().unwrap_or_else(|| place.name.clone()),
                    original: place.name,
                    country: place.country,
                    renamed: renamed.is_some(),
                    photos,
                })
            })
            .collect();
        list.sort_by(|a, b| b.photos.cmp(&a.photos).then_with(|| a.name.cmp(&b.name)));
        Ok(list)
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

/// Renames a place everywhere it is printed (FR-005), or (`None`) back to the dataset's name.
#[tauri::command]
#[specta::specta]
pub fn rename_place(
    app: AppHandle,
    state: State<'_, AppState>,
    geonames_id: u32,
    name: Option<String>,
) -> Result<(), IpcError> {
    if geocode::by_id(geonames_id).ok().flatten().is_none() {
        return Err(IpcError::new("place.unknown", "Unknown place"));
    }
    state
        .library
        .write()
        .map_err(|e| IpcError::internal(e.to_string()))?
        .rename_place(geonames_id, name.as_deref());
    changed(&app, &state);
    Ok(())
}

/// What a search for borrowed places found (FR-031).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BorrowReport {
    /// Photos that got a place by themselves (high confidence), as one undoable step.
    pub applied: u32,
    /// Places to accept or skip (medium confidence).
    pub suggestions: Vec<Borrow>,
}

/// Time windows a search may use, in minutes (FR-031; 10 is the default).
const MAX_WINDOW_MIN: u32 = 60;

/// Finds places for photos without one, from other devices' photos of the same scene
/// (FR-031). Visual signatures come from the best-shot analysis, computed and cached for the
/// photos that need one. High-confidence places are applied at once, as one undoable step;
/// the others come back as suggestions.
#[tauri::command]
#[specta::specta]
pub async fn find_borrowed_places(
    app: AppHandle,
    window_min: u32,
) -> Result<BorrowReport, IpcError> {
    if window_min == 0 || window_min > MAX_WINDOW_MIN {
        return Err(IpcError::new(
            "place.bad_window",
            "Pick a window of 1 to 60 minutes",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || borrow_and_apply(&app, i64::from(window_min) * 60))
        .await
        .map_err(|e| IpcError::internal(e.to_string()))?
}

/// The search behind `find_borrowed_places`, also run after every best-shot analysis.
pub fn borrow_and_apply(app: &AppHandle, window_s: i64) -> Result<BorrowReport, IpcError> {
    let state = app.state::<AppState>();
    let poisoned = |e: &dyn std::fmt::Display| IpcError::internal(e.to_string());
    // Photos that could be compared: every one with a candidate, and its donors.
    let (needed, known) = {
        let library = state.library.read().map_err(|e| poisoned(&e))?;
        let mut needed: Vec<PhotoId> = Vec::new();
        for (photo, donors) in library.borrow_candidates(window_s, places::DONORS_PER_PHOTO) {
            needed.push(photo);
            needed.extend(donors.into_iter().map(|(d, _)| d));
        }
        needed.sort_unstable_by_key(|id| id.0);
        needed.dedup();
        let analyses = state.analyses.lock().map_err(|e| poisoned(&e))?;
        let known: HashMap<PhotoId, Signature> = needed
            .iter()
            .filter_map(|id| Some((*id, analyses.by_photo.get(id)?.signature.clone())))
            .collect();
        let records: Vec<_> = needed
            .iter()
            .filter(|id| !known.contains_key(id))
            .filter_map(|&id| library.record(id))
            .collect();
        (records, known)
    };
    // The rest are analysed once and cached on disk, as the best-shot analysis does.
    let computed: Vec<(PhotoId, Signature)> = state.background_pool.install(|| {
        needed
            .par_iter()
            .filter_map(|record| {
                let analysis = state
                    .analysis_cache
                    .analyze(record, state.codec.as_ref())
                    .ok()?;
                Some((record.id, analysis.signature))
            })
            .collect()
    });
    let mut signatures = known;
    signatures.extend(computed);
    let mut library = state.library.write().map_err(|e| poisoned(&e))?;
    let found = places::borrow(&library, window_s, |id| signatures.get(&id));
    let (high, suggestions): (Vec<Borrow>, Vec<Borrow>) = found
        .into_iter()
        .partition(|b| b.confidence == Confidence::High);
    let changes: Vec<(PhotoId, PhotoEdits)> = high
        .iter()
        .map(|b| {
            let mut edits = library.edits(b.photo);
            edits.place = Some(PhotoPlace {
                geonames_id: b.geonames_id,
                source: PlaceSource::Borrowed,
            });
            (b.photo, edits)
        })
        .collect();
    let applied = u32::try_from(changes.len()).unwrap_or(u32::MAX);
    if !changes.is_empty() {
        state
            .history
            .lock()
            .map_err(|e| poisoned(&e))?
            .apply(&mut library, &changes);
    }
    drop(library);
    if applied > 0 {
        changed(app, &state);
    }
    Ok(BorrowReport {
        applied,
        suggestions,
    })
}

/// One suggestion the user accepted.
#[derive(Debug, Clone, serde::Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BorrowAccept {
    pub photo: PhotoId,
    pub geonames_id: u32,
}

/// Gives photos the places the user accepted from the suggestions (FR-031), as one undoable
/// step. A photo that has its own GPS or a place set by hand meanwhile keeps it.
#[tauri::command]
#[specta::specta]
pub fn accept_borrowed_places(
    app: AppHandle,
    state: State<'_, AppState>,
    accepted: Vec<BorrowAccept>,
) -> Result<HistoryState, IpcError> {
    if accepted
        .iter()
        .any(|a| geocode::by_id(a.geonames_id).ok().flatten().is_none())
    {
        return Err(IpcError::new("place.unknown", "Unknown place"));
    }
    let places: HashMap<PhotoId, u32> = accepted.iter().map(|a| (a.photo, a.geonames_id)).collect();
    let ids: Vec<PhotoId> = places.keys().copied().collect();
    let has_gps: Vec<PhotoId> = {
        let library = state
            .library
            .read()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        ids.iter()
            .copied()
            .filter(|&id| library.record(id).is_some_and(|r| r.gps.is_some()))
            .collect()
    };
    edit_photos(&app, &state, &ids, |id, e| {
        let manual = e.place.is_some_and(|p| p.source == PlaceSource::Manual);
        if let (Some(&geonames_id), false, false) = (places.get(&id), manual, has_gps.contains(&id))
        {
            e.place = Some(PhotoPlace {
                geonames_id,
                source: PlaceSource::Borrowed,
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_032_dates_are_checked_in_rust() {
        assert!(parse_date("1994-07-14T15:30").is_ok());
        assert!(parse_date("1994-07-14T15:30:05").is_ok());
        assert!(parse_date("1700-01-01T00:00").is_err());
        assert!(parse_date("2101-01-01T00:00").is_err());
        assert!(parse_date("14/07/1994").is_err());
        assert!(parse_date("").is_err());
    }

    #[test]
    fn fr_032_search_is_bounded() {
        assert!(search_places("a".repeat(101), 10).is_err());
        assert!(search_places("debre".into(), 1000).is_ok_and(|v| v.len() <= 50));
    }
}
