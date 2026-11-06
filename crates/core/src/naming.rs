// SPDX-License-Identifier: GPL-3.0-only
//! The names an export writes (FR-104, Lightroom's *File naming*), and the cleaning of any
//! name part that comes from the user, so no file or folder ever leaves the destination
//! (SECURITY.md §2).

use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::library::PhotoRecord;

/// How exported files are named, as Lightroom's templates (FR-104).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NameTemplate {
    /// `P1057694.jpg`, the photo's own name.
    #[default]
    Original,
    /// `2026-07-14_P1057694.jpg`.
    DateOriginal,
    /// `Prague-001.jpg`.
    TextSequence,
    /// `Prague-2026-07-14-001.jpg`.
    TextDateSequence,
    /// `2026-07-14-001.jpg`.
    DateSequence,
}

impl NameTemplate {
    /// Whether the template writes the custom text.
    pub fn uses_text(self) -> bool {
        matches!(self, Self::TextSequence | Self::TextDateSequence)
    }

    /// Whether the template writes a sequence number.
    pub fn uses_sequence(self) -> bool {
        matches!(
            self,
            Self::TextSequence | Self::TextDateSequence | Self::DateSequence
        )
    }
}

/// The export's naming, kept per machine (settings v16, FR-104).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportNaming {
    #[serde(default)]
    pub template: NameTemplate,
    /// The custom text of the `Text…` templates, cleaned (`sanitize_component`).
    #[serde(default)]
    pub text: String,
    /// The first sequence number.
    #[serde(default = "default_start")]
    pub start: u32,
}

const fn default_start() -> u32 {
    1
}

/// The largest first sequence number.
pub const MAX_START: u32 = 999_999;

impl Default for ExportNaming {
    fn default() -> Self {
        Self {
            template: NameTemplate::Original,
            text: String::new(),
            start: default_start(),
        }
    }
}

impl ExportNaming {
    /// Within what the app allows: clean text, a start from 1; a text template left without
    /// text falls back to the one without it, so no name starts with a dash.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        self.text = sanitize_component(&self.text).unwrap_or_default();
        self.start = self.start.clamp(1, MAX_START);
        if self.text.is_empty() {
            self.template = match self.template {
                NameTemplate::TextSequence => NameTemplate::Original,
                NameTemplate::TextDateSequence => NameTemplate::DateSequence,
                other => other,
            };
        }
        self
    }
}

/// The longest name part kept, in characters.
pub const MAX_COMPONENT_CHARS: usize = 100;

/// `raw` as one safe part of a path (a file name's text, a subfolder), or `None` when
/// nothing usable is left: no separators, no characters Windows refuses, no control
/// characters, no leading or trailing dots and spaces, not `.` or `..`, not a name Windows
/// reserves (`CON`, `COM1`…), at most [`MAX_COMPONENT_CHARS`]. Vietnamese letters stay.
pub fn sanitize_component(raw: &str) -> Option<String> {
    let kept: String = raw
        .chars()
        .filter(|c| !c.is_control() && !r#"/\:*?"<>|"#.contains(*c))
        .take(MAX_COMPONENT_CHARS)
        .collect();
    let name = kept.trim_matches(|c: char| c == '.' || c.is_whitespace());
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit());
    (!name.is_empty() && !reserved).then(|| name.to_owned())
}

/// The photo's own file name without its extension.
pub fn original_stem(record: &PhotoRecord) -> String {
    Path::new(&record.file_name)
        .file_stem()
        .map_or_else(|| "photo".to_owned(), |s| s.to_string_lossy().into_owned())
}

/// The sequence number of the `index`-th file of `total`, padded to the width of the last
/// one, at least 3 digits (`001`).
fn sequence(naming: &ExportNaming, index: usize, total: usize) -> String {
    let start = u64::from(naming.start.max(1));
    let number = start + index as u64;
    let last = start + total.saturating_sub(1) as u64;
    let width = last.to_string().len().max(3);
    format!("{number:0width$}")
}

/// The name, without `.jpg`, of the `index`-th of `total` files whose photo is `record` and
/// whose own name would be `own` (a photo's stem, or a sheet's `<first>+<second>`); `date`
/// is its day (`2026-07-14`, or `undated`, as its day folder).
pub fn file_name(
    naming: &ExportNaming,
    own: &str,
    date: &str,
    index: usize,
    total: usize,
) -> String {
    let text = &naming.text;
    match naming.template {
        NameTemplate::Original => own.to_owned(),
        NameTemplate::DateOriginal => format!("{date}_{own}"),
        NameTemplate::TextSequence => format!("{text}-{}", sequence(naming, index, total)),
        NameTemplate::TextDateSequence => {
            format!("{text}-{date}-{}", sequence(naming, index, total))
        }
        NameTemplate::DateSequence => format!("{date}-{}", sequence(naming, index, total)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naming(template: NameTemplate, text: &str, start: u32) -> ExportNaming {
        ExportNaming {
            template,
            text: text.to_owned(),
            start,
        }
        .sanitized()
    }

    #[test]
    fn fr_104_each_template_names_as_lightroom() {
        let (own, date) = ("P1057694", "2026-07-14");
        let name = |n: &ExportNaming, i, total| file_name(n, own, date, i, total);
        assert_eq!(
            name(&naming(NameTemplate::Original, "", 1), 0, 9),
            "P1057694"
        );
        assert_eq!(
            name(&naming(NameTemplate::DateOriginal, "", 1), 0, 9),
            "2026-07-14_P1057694"
        );
        assert_eq!(
            name(&naming(NameTemplate::TextSequence, "Prague", 1), 0, 9),
            "Prague-001"
        );
        assert_eq!(
            name(&naming(NameTemplate::TextDateSequence, "Prague", 1), 4, 9),
            "Prague-2026-07-14-005"
        );
        assert_eq!(
            name(&naming(NameTemplate::DateSequence, "", 1), 0, 9),
            "2026-07-14-001"
        );
        // Padded to the last number: 1 to 1,200 is four digits; a start of 98 runs 098–107.
        assert_eq!(
            name(&naming(NameTemplate::TextSequence, "Trip", 1), 0, 1200),
            "Trip-0001"
        );
        assert_eq!(
            name(&naming(NameTemplate::TextSequence, "Trip", 98), 9, 10),
            "Trip-107"
        );
        assert_eq!(
            name(&naming(NameTemplate::TextSequence, "Trip", 98), 0, 10),
            "Trip-098"
        );
        // Vietnamese text is kept as typed.
        assert_eq!(
            name(&naming(NameTemplate::TextSequence, "Hà Nội · Tết", 1), 0, 2),
            "Hà Nội · Tết-001"
        );
        // A text template without text falls back, so no name starts with a dash.
        assert_eq!(
            naming(NameTemplate::TextSequence, " / ", 1).template,
            NameTemplate::Original
        );
        assert_eq!(
            naming(NameTemplate::TextDateSequence, "", 1).template,
            NameTemplate::DateSequence
        );
        // The start number is clamped.
        assert_eq!(naming(NameTemplate::TextSequence, "a", 0).start, 1);
        assert_eq!(
            naming(NameTemplate::TextSequence, "a", u32::MAX).start,
            MAX_START
        );
    }

    #[test]
    fn fr_104_a_name_part_never_leaves_the_destination() {
        assert_eq!(sanitize_component("../etc").as_deref(), Some("etc"));
        assert_eq!(sanitize_component("a/b").as_deref(), Some("ab"));
        assert_eq!(
            sanitize_component(r"C:\Windows").as_deref(),
            Some("CWindows")
        );
        assert_eq!(sanitize_component(".."), None);
        assert_eq!(sanitize_component("  .  "), None);
        assert_eq!(sanitize_component("con"), None);
        assert_eq!(sanitize_component("COM1.txt"), None);
        assert_eq!(
            sanitize_component("Prague in\u{0}\n").as_deref(),
            Some("Prague in")
        );
        assert_eq!(sanitize_component("Sài Gòn ").as_deref(), Some("Sài Gòn"));
        let long = "x".repeat(300);
        assert_eq!(
            sanitize_component(&long).map(|s| s.chars().count()),
            Some(MAX_COMPONENT_CHARS)
        );
    }
}
