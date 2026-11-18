// SPDX-License-Identifier: GPL-3.0-only
//! The user's changes to RollSense's trips (FR-012, FR-141 S7c; owner 2026-11-18): rename,
//! split after a day, merge with the trip before, and a trip's days for its page. Kept in the
//! project (schema v17); every input is checked in the core against the trips found.

use fotollection_core::library::TripEditError;
use fotollection_core::sorting::facets::TripDay;
use tauri::{AppHandle, State};

use crate::commands::changed;
use crate::document::poisoned;
use crate::state::{AppState, IpcError};

fn refused(e: &TripEditError) -> IpcError {
    IpcError::new(e.code(), e.to_string())
}

/// Names the trip that starts on `first_day` (days since 1970); `None` brings back the
/// suggested name. The core trims it and cuts it to 80 characters.
#[tauri::command]
#[specta::specta]
pub fn rename_trip(
    app: AppHandle,
    state: State<'_, AppState>,
    first_day: i32,
    name: Option<String>,
) -> Result<(), IpcError> {
    state
        .library
        .write()
        .map_err(poisoned)?
        .rename_trip(i64::from(first_day), name.as_deref())
        .map_err(|e| refused(&e))?;
    changed(&app, &state);
    Ok(())
}

/// Splits the trip that starts on `first_day` after the day `after_day`.
#[tauri::command]
#[specta::specta]
pub fn split_trip(
    app: AppHandle,
    state: State<'_, AppState>,
    first_day: i32,
    after_day: i32,
) -> Result<(), IpcError> {
    state
        .library
        .write()
        .map_err(poisoned)?
        .split_trip(i64::from(first_day), i64::from(after_day))
        .map_err(|e| refused(&e))?;
    changed(&app, &state);
    Ok(())
}

/// Merges the trip that starts on `first_day` into the trip before it.
#[tauri::command]
#[specta::specta]
pub fn merge_trip(app: AppHandle, state: State<'_, AppState>, first_day: i32) -> Result<(), IpcError> {
    state
        .library
        .write()
        .map_err(poisoned)?
        .merge_trip(i64::from(first_day))
        .map_err(|e| refused(&e))?;
    changed(&app, &state);
    Ok(())
}

/// The days of the trip that starts on `first_day`, for its page.
#[tauri::command]
#[specta::specta]
pub fn trip_days(state: State<'_, AppState>, first_day: i32) -> Result<Vec<TripDay>, IpcError> {
    state
        .library
        .read()
        .map_err(poisoned)?
        .trip_days(i64::from(first_day))
        .map_err(|e| refused(&e))
}
