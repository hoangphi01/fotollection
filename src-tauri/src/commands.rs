// SPDX-License-Identifier: GPL-3.0-only
//! IPC commands. Contract documented in docs/ARCHITECTURE.md §6.

use fotollection_core::PrintSize;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub app_version: String,
    pub core_version: String,
}

/// Versions shown in the About section.
#[tauri::command]
#[specta::specta]
pub fn app_info() -> AppInfo {
    AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        core_version: fotollection_core::VERSION.to_owned(),
    }
}

/// Supported print sizes for the settings panel (FR-006).
#[tauri::command]
#[specta::specta]
pub fn list_print_sizes() -> Vec<PrintSize> {
    PrintSize::ALL.to_vec()
}
