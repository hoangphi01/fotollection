// SPDX-License-Identifier: GPL-3.0-only
//! `AddAssist` (FR-133): a postal code or a search fills an address from the post's own data,
//! bundled with the app and looked up offline. One module owns every country's data behind
//! this one interface; the postcard and any later form only call it. Its version is
//! [`VERSION`]: a minor one adds a country, a field or a newer edition of a country's data,
//! a major one changes this interface (PRD FR-133).

use serde::{Deserialize, Serialize};
use specta::Type;

mod jp;
pub mod romaji;

/// `AddAssist`'s version, shown with its countries' data dates and kept with an address it
/// filled.
pub const VERSION: &str = "1.0";

/// The longest query or code accepted from the interface.
pub const MAX_QUERY_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AddAssistError {
    #[error("bundled postal data does not match its checksum")]
    Checksum,
    #[error("bundled postal data is malformed")]
    Malformed,
}

/// A country `AddAssist` has data for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Country {
    /// Japan Post's postal-code data.
    Jp,
}

/// A country and its data, for Settings and the manual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CountryInfo {
    pub country: Country,
    /// How many digits its postal code has.
    pub code_digits: u8,
    /// The date of the post's data, `YYYY-MM-DD`.
    pub data_date: String,
}

/// What a code or a search found: the places the post's data knows. The street number, the
/// building and the name are the user's to type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Found {
    /// The code's digits (`2480024`).
    pub code: String,
    /// Prefecture (Japan).
    pub region: String,
    /// Municipality.
    pub city: String,
    /// Town, or "" where the data leaves it to the address.
    pub town: String,
    /// The data's note on the town (`１〜１９丁目`), never printed by itself.
    pub note: String,
    pub region_latin: String,
    pub city_latin: String,
    pub town_latin: String,
}

/// The countries with data, in a fixed order.
pub fn countries() -> Result<Vec<CountryInfo>, AddAssistError> {
    Ok(vec![CountryInfo {
        country: Country::Jp,
        code_digits: 7,
        data_date: jp::snapshot()?.to_owned(),
    }])
}

/// Every place a full code covers: one, several to pick from, or none.
pub fn lookup(country: Country, code: &str) -> Result<Vec<Found>, AddAssistError> {
    match country {
        Country::Jp => jp::lookup(code),
    }
}

/// Places whose name starts with the query, or whose code starts with its digits.
pub fn search(country: Country, query: &str, limit: usize) -> Result<Vec<Found>, AddAssistError> {
    match country {
        Country::Jp => jp::search(query, limit),
    }
}
