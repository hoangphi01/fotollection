// SPDX-License-Identifier: GPL-3.0-only
//! Dates and places set by hand (FR-032) and renamed places (FR-005), ADR-0027. Every input
//! is checked here: ids and counts in `edit_photos`, dates against 1826–2100, place ids
//! against the bundled places, names to `MAX_PLACE_NAME` characters.

// Tauri injects `State`, `AppHandle` and arguments by value.
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;

use fotollection_core::geocode::{self, Place};
use fotollection_core::library::{
    MAX_MANUAL_DATE, MIN_MANUAL_DATE, PhotoId, PhotoPlace, PlaceSource,
};
use fotollection_core::places;
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
