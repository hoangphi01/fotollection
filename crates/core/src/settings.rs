// SPDX-License-Identifier: GPL-3.0-only
//! Render settings shared by preview and export (in memory until projects land, FR-023).

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::print_size::PrintSize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
    /// `PrintSize.id`, e.g. `"10x15"` (FR-006).
    pub print_size: String,
    pub stamp: StampSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StampSettings {
    pub enabled: bool,
    pub format: DateFormat,
    pub corner: Corner,
    pub color: StampColor,
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
            },
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("unknown print size: {0}")]
    UnknownPrintSize(String),
}

impl RenderSettings {
    /// Validates input coming over IPC (SECURITY.md §2).
    pub fn validate(&self) -> Result<PrintSize, SettingsError> {
        PrintSize::by_id(&self.print_size)
            .ok_or_else(|| SettingsError::UnknownPrintSize(self.print_size.clone()))
    }
}
