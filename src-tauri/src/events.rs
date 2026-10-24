// SPDX-License-Identifier: GPL-3.0-only
//! Typed events (core → UI). Contract: docs/ARCHITECTURE.md §6.

use fotollection_core::export::ExportSummary;
use fotollection_core::library::PhotoId;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// Render-affecting state changed (settings, frame, crops, undo); previews refresh.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RevisionChanged {
    pub revision: u32,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Photos were added, or their ratings/flags changed; the UI refetches the list.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaskKind {
    /// Reading files and EXIF during import.
    Import,
    /// Generating sharp (T1) thumbnails in the background.
    Thumbnails,
    /// Best-shot analysis of the library (FR-041).
    Analyze,
}

/// A best-shot analysis finished (or was cancelled); the UI refetches the groups.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisFinished {
    pub job_id: u32,
    pub analyzed: u32,
    pub failed: u32,
    pub groups: u32,
    pub cancelled: bool,
}

/// Progress of a background task, for the progress bar (coalesced to ≤ 30 per second).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    pub task: TaskKind,
    pub done: u32,
    pub total: u32,
}

/// The project document changed: name, unsaved changes, or a new library (FR-023).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChanged {
    pub document: crate::document::DocumentState,
}

/// A native menu item was chosen. The UI runs the same flow as its own buttons, including
/// the unsaved-changes question (FR-023).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct MenuAction {
    pub action: crate::menu::Action,
}
