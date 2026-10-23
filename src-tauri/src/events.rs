// SPDX-License-Identifier: GPL-3.0-only
//! Typed events (core → UI). Contract: docs/ARCHITECTURE.md §6.

use fotollection_core::export::ExportSummary;
use fotollection_core::library::PhotoId;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// Photos were added; the UI refetches the list.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LibraryChanged {
    pub count: u32,
}

/// A sharper thumbnail (T1) is now cached for this photo.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ThumbReady {
    pub id: PhotoId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub job_id: u32,
    pub done: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ExportFinished {
    pub job_id: u32,
    pub summary: ExportSummary,
}
