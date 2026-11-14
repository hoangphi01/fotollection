// SPDX-License-Identifier: GPL-3.0-only
//! `AddAssist`'s commands (FR-133): the countries with data, a code's places and a search,
//! all offline from the data bundled in the core. Every input is checked here: the query or
//! code to `addassist::MAX_QUERY_CHARS` characters, the results to `MAX_RESULTS`.

// Tauri injects arguments by value.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::addassist::{self, Country, CountryInfo, Found, MAX_QUERY_CHARS};

use crate::state::IpcError;

const MAX_RESULTS: u32 = 50;

fn checked(text: &str) -> Result<(), IpcError> {
    if text.chars().count() > MAX_QUERY_CHARS {
        return Err(IpcError::new(
            "addassist.query_too_long",
            "The postal code or search is too long",
        ));
    }
    Ok(())
}

/// The countries `AddAssist` has data for, with the data's date.
#[tauri::command]
#[specta::specta]
pub fn addassist_countries() -> Result<Vec<CountryInfo>, IpcError> {
    addassist::countries().map_err(|e| IpcError::internal(e.to_string()))
}

/// Every place a full postal code covers: one, several to pick from, or none.
#[tauri::command]
#[specta::specta]
pub fn addassist_lookup(country: Country, code: String) -> Result<Vec<Found>, IpcError> {
    checked(&code)?;
    addassist::lookup(country, &code).map_err(|e| IpcError::internal(e.to_string()))
}

/// Places whose name starts with `query` (kanji, kana or romaji for Japan), or whose code
/// starts with its digits.
#[tauri::command]
#[specta::specta]
pub fn addassist_search(
    country: Country,
    query: String,
    limit: u32,
) -> Result<Vec<Found>, IpcError> {
    checked(&query)?;
    addassist::search(country, &query, limit.min(MAX_RESULTS) as usize)
        .map_err(|e| IpcError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_133_a_long_query_is_refused() {
        let long = "1".repeat(MAX_QUERY_CHARS + 1);
        assert!(addassist_search(Country::Jp, long.clone(), 5).is_err());
        assert!(addassist_lookup(Country::Jp, long).is_err());
        assert!(addassist_lookup(Country::Jp, "248-0024".into()).is_ok_and(|f| f.len() == 1));
    }
}
