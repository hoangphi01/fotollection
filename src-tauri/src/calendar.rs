// SPDX-License-Identifier: GPL-3.0-only
//! A month's grid for the calendar of RollSense's Time card (FR-141 C1), laid out by the
//! core's `calendar` module; the inputs are checked here.

use chrono::Weekday;
use fotollection_core::calendar::{self, CalDay};

use crate::state::IpcError;

/// The weeks of `month` (1…12) in `year` (1900…2100), each starting on `first_weekday`
/// (0 Monday … 6 Sunday): 4 to 6 rows of 7 days, the neighbour months' days marked.
#[tauri::command]
#[specta::specta]
pub fn month_grid(year: i32, month: u8, first_weekday: u8) -> Result<Vec<[CalDay; 7]>, IpcError> {
    let bad = || {
        IpcError::new(
            "calendar.bad_month",
            "No such month or first day of the week",
        )
    };
    let first = Weekday::try_from(first_weekday).map_err(|_| bad())?;
    calendar::month_grid(year, u32::from(month), first).ok_or_else(bad)
}
