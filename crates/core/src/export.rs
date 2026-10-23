// SPDX-License-Identifier: GPL-3.0-only
//! Batch export (FR-008, FR-009, FR-011): bounded parallelism, cooperative cancellation,
//! per-photo error isolation, atomic writes, and never overwriting existing files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::cancel::CancellationToken;
use crate::codec::{CodecError, JpegCodec};
use crate::image::apply_orientation;
use crate::library::{PhotoId, PhotoRecord, flags};
use crate::plan::{FrameRequest, PlanInput, plan};
use crate::render::{RenderError, render};
use crate::settings::RenderSettings;

pub const EXPORT_QUALITY: u8 = 95;
/// Folder for photos without any capture date.
pub const UNDATED_FOLDER: &str = "undated";

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("could not write the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("cancelled")]
    Cancelled,
}

impl ExportError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Codec(e) => e.code(),
            Self::Render(_) => "render.failed",
            Self::Io(_) => "export.write_failed",
            Self::Cancelled => "export.cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportFailure {
    pub id: PhotoId,
    pub file_name: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    pub exported: u32,
    pub failed: Vec<ExportFailure>,
    pub cancelled: bool,
    pub destination: String,
}

pub struct ExportOptions<'a> {
    pub destination: &'a Path,
    pub settings: &'a RenderSettings,
    pub quality: u8,
    /// Photos processed at once; bounds peak memory (NFR-004).
    pub max_inflight: usize,
    /// A user font for the date stamp (FR-042).
    pub stamp_font: Option<crate::fonts::LoadedFont>,
    /// The info frame for one photo, if any (FR-040).
    pub frame_for: &'a (dyn Fn(&PhotoRecord) -> Option<FrameRequest> + Sync),
}

/// A sensible default for `max_inflight`: one per core, capped so decoded 24 MP buffers
/// (~160 MB each, including the crop and encode buffers) stay within the 2 GB budget.
pub fn default_inflight() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().clamp(1, 6))
}

/// `YYYY-MM-DD` folder for a photo (FR-009).
pub fn day_folder(record: &PhotoRecord) -> String {
    if record.flags & flags::DATE_FROM_FILE != 0 && record.mtime == 0 {
        return UNDATED_FOLDER.to_owned();
    }
    record.capture.format("%Y-%m-%d").to_string()
}

/// Picks `dir/stem.jpg`, or `stem_1.jpg`, `stem_2.jpg`… so nothing existing is overwritten
/// and parallel workers never collide.
fn reserve_name(dir: &Path, stem: &str, reserved: &Mutex<HashSet<PathBuf>>) -> PathBuf {
    let mut n = 0u32;
    loop {
        let name = if n == 0 {
            format!("{stem}.jpg")
        } else {
            format!("{stem}_{n}.jpg")
        };
        let candidate = dir.join(name);
        if let Ok(mut set) = reserved.lock()
            && !candidate.exists()
            && set.insert(candidate.clone())
        {
            return candidate;
        }
        n += 1;
    }
}

fn export_one(
    record: &PhotoRecord,
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
    reserved: &Mutex<HashSet<PathBuf>>,
) -> Result<(), ExportError> {
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }
    let bytes = std::fs::read(&record.path)?;
    let decoded = codec.decode(&bytes)?;
    let upright = apply_orientation(decoded.image, record.orientation);
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }

    let print = options
        .settings
        .validate()
        .unwrap_or(crate::PrintSize::ALL[1]);
    let capture = (record.flags & flags::DATE_FROM_FILE == 0).then_some(record.capture);
    let frame = (options.frame_for)(record);
    let p = plan(&PlanInput {
        width: upright.width,
        height: upright.height,
        capture,
        settings: options.settings,
        print,
        max_long_edge: None,
        frame: frame.as_ref(),
        crop: record.edits.crop,
        stamp_font: options.stamp_font.as_ref(),
    });
    let image = render(&upright, &p)?;
    drop(upright);
    // No EXIF is written, so GPS never leaves the machine by default (FR-010).
    let jpeg = codec.encode(&image, options.quality, decoded.icc.as_deref())?;
    drop(image);

    let dir = options.destination.join(day_folder(record));
    std::fs::create_dir_all(&dir)?;
    let stem = Path::new(&record.file_name)
        .file_stem()
        .map_or_else(|| "photo".to_owned(), |s| s.to_string_lossy().into_owned());
    let target = reserve_name(&dir, &stem, reserved);
    let tmp = dir.join(format!(".{stem}.{}.part", record.id.0));
    std::fs::write(&tmp, &jpeg)?;
    if cancel.is_cancelled() {
        let _ = std::fs::remove_file(&tmp);
        return Err(ExportError::Cancelled);
    }
    std::fs::rename(&tmp, &target).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// Exports `records`. `on_progress(done, total)` is called after each photo.
pub fn run_export(
    records: &[PhotoRecord],
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(u32, u32) + Sync),
) -> ExportSummary {
    let total = u32::try_from(records.len()).unwrap_or(u32::MAX);
    let done = AtomicU32::new(0);
    let exported = AtomicU32::new(0);
    let failed = Mutex::new(Vec::new());
    let reserved = Mutex::new(HashSet::new());

    let work = || {
        records.par_iter().for_each(|record| {
            match export_one(record, options, codec, cancel, &reserved) {
                Ok(()) => {
                    exported.fetch_add(1, Ordering::Relaxed);
                }
                Err(ExportError::Cancelled) => {}
                Err(e) => {
                    if let Ok(mut f) = failed.lock() {
                        f.push(ExportFailure {
                            id: record.id,
                            file_name: record.file_name.clone(),
                            code: e.code().to_owned(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            on_progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
        });
    };
    match rayon::ThreadPoolBuilder::new()
        .num_threads(options.max_inflight.max(1))
        .build()
    {
        Ok(pool) => pool.install(work),
        Err(_) => work(),
    }

    ExportSummary {
        exported: exported.into_inner(),
        failed: failed.into_inner().unwrap_or_default(),
        cancelled: cancel.is_cancelled(),
        destination: options.destination.display().to_string(),
    }
}
