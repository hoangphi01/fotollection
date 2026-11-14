// SPDX-License-Identifier: GPL-3.0-only
//! An address as a Japanese or Korean card writes it (FR-130, FR-132): its postal code, the
//! places `AddAssist` fills (or the user types), the line the user types, and the name. Kept
//! per photo for the recipient and once per machine for the sender; checked here, never
//! trusted from the interface (SECURITY.md §2).

use serde::{Deserialize, Serialize};
use specta::Type;

use super::Country;

/// The most characters of a place field (prefecture, municipality, town, road) and of a name.
pub const MAX_FIELD_CHARS: usize = 60;
/// The most characters of the typed line (block and building number, building, unit), on at
/// most two lines.
pub const MAX_LINE_CHARS: usize = 120;
const MAX_LINE_LINES: usize = 2;

fn yes() -> bool {
    true
}

/// An address on a Japanese or Korean card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub country: Country,
    /// The `AddAssist` version that filled its places ("1.0"), "" when typed by hand, so a card
    /// made under 1.0 is read the same way later (PRD FR-133).
    #[serde(default)]
    pub filled_by: String,
    /// The postal code's digits, 7 in Japan and 5 in Korea; "" when not known.
    #[serde(default)]
    pub code: String,
    /// Prefecture (Japan), 시·도 (Korea).
    #[serde(default)]
    pub region: String,
    /// Municipality, 시·군·구.
    #[serde(default)]
    pub city: String,
    /// Town (Japan), 읍·면 (Korea).
    #[serde(default)]
    pub town: String,
    /// The road (Korea).
    #[serde(default)]
    pub road: String,
    /// What the user types after the places: 1-2-3 and the building in Japan, 175 and the
    /// unit in Korea.
    #[serde(default)]
    pub line: String,
    #[serde(default)]
    pub name: String,
    /// 様 (Japan) or 귀하 (Korea) after the name; Japan always prints it.
    #[serde(default = "yes")]
    pub honorific: bool,
    /// Printed in Latin letters (romaji, Revised Romanization) instead of kanji or Hangul.
    #[serde(default)]
    pub latin: bool,
}

impl Address {
    /// An empty address for a country.
    pub fn new(country: Country) -> Self {
        Self {
            country,
            filled_by: String::new(),
            code: String::new(),
            region: String::new(),
            city: String::new(),
            town: String::new(),
            road: String::new(),
            line: String::new(),
            name: String::new(),
            honorific: true,
            latin: false,
        }
    }

    /// How many digits the country's postal code has.
    pub fn code_digits(&self) -> usize {
        match self.country {
            Country::Jp => 7,
            Country::Kr => 5,
        }
    }

    /// Nothing typed or filled.
    pub fn is_blank(&self) -> bool {
        [
            &self.code,
            &self.region,
            &self.city,
            &self.town,
            &self.road,
            &self.line,
            &self.name,
        ]
        .iter()
        .all(|f| f.trim().is_empty())
    }

    /// Every field within its limits, as read from a file: control characters dropped, the
    /// code to its digits, the line to two lines.
    #[must_use]
    pub fn sanitized(&self) -> Self {
        let field = |s: &str| -> String {
            s.chars()
                .filter(|c| !c.is_control())
                .take(MAX_FIELD_CHARS)
                .collect()
        };
        let line: String = self
            .line
            .chars()
            .filter(|c| *c == '\n' || !c.is_control())
            .take(MAX_LINE_CHARS)
            .collect();
        let filled_by = if self.filled_by.len() <= 8
            && self
                .filled_by
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.')
        {
            self.filled_by.clone()
        } else {
            String::new()
        };
        Self {
            country: self.country,
            filled_by,
            code: self
                .code
                .chars()
                .filter(char::is_ascii_digit)
                .take(self.code_digits())
                .collect(),
            region: field(&self.region),
            city: field(&self.city),
            town: field(&self.town),
            road: field(&self.road),
            line: line
                .split('\n')
                .take(MAX_LINE_LINES)
                .collect::<Vec<_>>()
                .join("\n"),
            name: field(&self.name),
            honorific: self.honorific,
            latin: self.latin,
        }
    }

    /// Checks an address coming over IPC: it must already be what [`Self::sanitized`] makes.
    pub fn is_valid(&self) -> bool {
        *self == self.sanitized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_133_an_address_is_kept_within_its_limits() {
        let mut a = Address::new(Country::Jp);
        a.code = "248-0024".into();
        a.name = format!("鎌倉{}", "\u{7}");
        a.line = "一丁目\n二番\n三号".into();
        a.filled_by = "1.0".into();
        let s = a.sanitized();
        assert_eq!(s.code, "2480024");
        assert_eq!(s.name, "鎌倉");
        assert_eq!(s.line, "一丁目\n二番");
        assert_eq!(s.filled_by, "1.0");
        assert!(!a.is_valid());
        assert!(s.is_valid());
        let mut k = Address::new(Country::Kr);
        k.code = "031720".into();
        assert_eq!(k.sanitized().code, "03172");
        assert!(Address::new(Country::Kr).is_blank());
    }

    #[test]
    fn fr_133_an_address_round_trips_and_reads_old_fields_absent() {
        let mut a = Address::new(Country::Kr);
        a.road = "세종대로".into();
        a.honorific = false;
        let json = serde_json::to_string(&a).unwrap_or_default();
        let back: Option<Address> = serde_json::from_str(&json).ok();
        assert_eq!(back.as_ref(), Some(&a));
        let bare: Option<Address> = serde_json::from_str(r#"{"country":"jp"}"#).ok();
        assert_eq!(bare, Some(Address::new(Country::Jp)));
    }
}
