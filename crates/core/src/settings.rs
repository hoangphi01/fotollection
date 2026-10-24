// SPDX-License-Identifier: GPL-3.0-only
//! Render settings shared by preview and export, persisted in `settings.json` (DATABASE.md §6).

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::print_size::PrintSize;

/// Interface language (FR-020). English is the default; text printed on photos never follows it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    En,
    Vi,
}

/// Per-machine interface preferences (FR-020). Never part of a project file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiSettings {
    pub language: Language,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
    /// A built-in `PrintSize.id` (e.g. `"10x15"`) or a `CustomPrintSize.id` (FR-006, FR-043).
    pub print_size: String,
    pub stamp: StampSettings,
    /// User-defined sizes such as 4 × 6 in (FR-043).
    #[serde(default)]
    pub custom_sizes: Vec<CustomPrintSize>,
    /// How place names are written, in the stamp and in the info frame (FR-005).
    #[serde(default)]
    pub place_format: crate::geocode::PlaceFormat,
}

/// A saved custom print size (FR-043). Stored in millimetres, short edge first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomPrintSize {
    /// `custom-<n>`, unique.
    pub id: String,
    pub name: String,
    pub short_mm: u32,
    pub long_mm: u32,
    /// How the user entered it, so labels show `4 × 6 in` rather than `10.2 × 15.2 cm`.
    #[serde(default)]
    pub unit: LengthUnit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LengthUnit {
    #[default]
    Cm,
    In,
}

pub const MAX_CUSTOM_SIZES: usize = 30;
/// Allowed edge range for custom sizes, in mm (2 cm – 1 m).
pub const CUSTOM_EDGE_MM: std::ops::RangeInclusive<u32> = 20..=1000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StampSettings {
    pub enabled: bool,
    pub format: DateFormat,
    pub corner: Corner,
    pub color: StampColor,
    /// The place on the same line as the date (FR-005). The stamp is always set in the
    /// bundled LCD font; user fonts are for the info frame only (FR-042). Settings v4 and
    /// older had a `font` here, which is ignored.
    #[serde(default)]
    pub place: bool,
    /// A line of the user's own text under the date (FR-036), shown in plain upper-case
    /// letters; empty for none.
    #[serde(default)]
    pub caption: String,
}

/// Longest stamp caption, in characters.
pub const MAX_CAPTION_CHARS: usize = 80;

impl StampSettings {
    /// Brings a caption from a file within limits: no control characters, 80 characters.
    pub fn sanitize(&mut self) {
        self.caption = self
            .caption
            .chars()
            .filter(|c| !c.is_control())
            .take(MAX_CAPTION_CHARS)
            .collect();
    }

    /// Checks a stamp coming over IPC (SECURITY.md §2).
    pub fn validate(&self) -> Result<(), SettingsError> {
        let caption = &self.caption;
        if caption.chars().count() > MAX_CAPTION_CHARS || caption.chars().any(char::is_control) {
            return Err(SettingsError::InvalidCaption);
        }
        Ok(())
    }
}

impl RenderSettings {
    /// The settings one photo is rendered with: its own stamp when it has one (FR-044).
    pub fn for_photo(&self, stamp: Option<&StampSettings>) -> std::borrow::Cow<'_, Self> {
        match stamp {
            Some(own) => std::borrow::Cow::Owned(Self {
                stamp: own.clone(),
                ..self.clone()
            }),
            None => std::borrow::Cow::Borrowed(self),
        }
    }
}

/// Date stamp formats (FR-004).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DateFormat {
    /// `2026-10-23`
    Iso,
    /// `23.10.2026`
    Dmy,
    /// `'26 10 23`, as on 1990s film cameras
    Retro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Corner {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StampColor {
    White,
    RetroOrange,
}

impl StampColor {
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::White => [255, 255, 255],
            Self::RetroOrange => [255, 138, 31],
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            print_size: "10x15".to_owned(),
            stamp: StampSettings {
                enabled: true,
                format: DateFormat::Iso,
                corner: Corner::BottomRight,
                color: StampColor::White,
                place: false,
                caption: String::new(),
            },
            custom_sizes: Vec::new(),
            place_format: crate::geocode::PlaceFormat::default(),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("unknown print size: {0}")]
    UnknownPrintSize(String),
    #[error("invalid custom size: {0}")]
    InvalidCustomSize(String),
    #[error("the caption has control characters or more than {MAX_CAPTION_CHARS} characters")]
    InvalidCaption,
}

impl RenderSettings {
    /// Validates input coming over IPC (SECURITY.md §2).
    /// Checks the settings and resolves the selected size. For Original (FR-045) this is a
    /// placeholder: use [`Self::print_for`] wherever the photo is known.
    pub fn validate(&self) -> Result<PrintSize, SettingsError> {
        self.validate_custom_sizes()?;
        self.stamp.validate()?;
        if self.print_size == PrintSize::ORIGINAL_ID {
            return Ok(PrintSize::original(3, 2));
        }
        if let Some(size) = PrintSize::by_id(&self.print_size) {
            return Ok(size);
        }
        self.custom_sizes
            .iter()
            .find(|c| c.id == self.print_size)
            .map(|c| PrintSize::custom(c.short_mm, c.long_mm))
            .ok_or_else(|| SettingsError::UnknownPrintSize(self.print_size.clone()))
    }

    /// The print size for an upright `width` × `height` photo: the selected size, or the
    /// photo's own ratio when Original is selected (FR-045).
    pub fn print_for(&self, width: u32, height: u32) -> Result<PrintSize, SettingsError> {
        let size = self.validate()?;
        Ok(if size.is_original() {
            PrintSize::original(width, height)
        } else {
            size
        })
    }

    /// Display unit of the selected size: inches for custom sizes entered in inches.
    pub fn unit(&self) -> LengthUnit {
        self.custom_sizes
            .iter()
            .find(|c| c.id == self.print_size)
            .map_or(LengthUnit::Cm, |c| c.unit)
    }

    fn validate_custom_sizes(&self) -> Result<(), SettingsError> {
        let bad = |why: &str| Err(SettingsError::InvalidCustomSize(why.to_owned()));
        if self.custom_sizes.len() > MAX_CUSTOM_SIZES {
            return bad("too many custom sizes");
        }
        let mut ids = std::collections::HashSet::new();
        for c in &self.custom_sizes {
            if !c.id.starts_with("custom-") || !ids.insert(c.id.as_str()) {
                return bad("duplicate or malformed id");
            }
            if c.name.chars().count() > 40 || c.name.chars().any(char::is_control) {
                return bad("name");
            }
            if !CUSTOM_EDGE_MM.contains(&c.short_mm)
                || !CUSTOM_EDGE_MM.contains(&c.long_mm)
                || c.short_mm > c.long_mm
            {
                return bad("edges must be 2–100 cm, short edge first");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_custom(sizes: Vec<CustomPrintSize>, selected: &str) -> RenderSettings {
        RenderSettings {
            print_size: selected.to_owned(),
            custom_sizes: sizes,
            ..RenderSettings::default()
        }
    }

    fn size(id: &str, short_mm: u32, long_mm: u32, unit: LengthUnit) -> CustomPrintSize {
        CustomPrintSize {
            id: id.to_owned(),
            name: "Mine".to_owned(),
            short_mm,
            long_mm,
            unit,
        }
    }

    #[test]
    fn fr_043_custom_size_resolves_and_validates() {
        // 4 × 6 in, stored in mm.
        let four_by_six = size("custom-1", 102, 152, LengthUnit::In);
        let s = with_custom(vec![four_by_six.clone()], "custom-1");
        let print = s.validate();
        assert_eq!(print.map(|p| (p.short_mm, p.long_mm)), Ok((102, 152)));
        assert_eq!(s.unit(), LengthUnit::In);
        // Built-ins still resolve, and use cm.
        let builtin = with_custom(vec![four_by_six.clone()], "10x15");
        assert!(builtin.validate().is_ok());
        assert_eq!(builtin.unit(), LengthUnit::Cm);

        for bad in [
            vec![size("custom-1", 10, 152, LengthUnit::Cm)], // edge below 2 cm
            vec![size("custom-1", 152, 102, LengthUnit::Cm)], // long edge first
            vec![size("x-1", 102, 152, LengthUnit::Cm)],     // malformed id
            vec![four_by_six.clone(), four_by_six.clone()],  // duplicate id
        ] {
            assert!(with_custom(bad, "10x15").validate().is_err());
        }
        assert!(
            with_custom(vec![], "custom-9").validate().is_err(),
            "unknown id"
        );
    }
}
