// SPDX-License-Identifier: GPL-3.0-only
//! Batch export (FR-008, FR-009, FR-011): bounded parallelism, cooperative cancellation,
//! per-photo error isolation, atomic writes, and never overwriting existing files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::cancel::CancellationToken;
use crate::codec::{CodecError, EncodeOptions, JpegCodec, decode_image};
use crate::image::apply_orientation;
use crate::library::{PhotoId, PhotoRecord, flags};
use crate::plan::{FrameRequest, PlanInput, plan};
use crate::render::{RenderError, render};
use crate::settings::RenderSettings;
use crate::share::{ExportTarget, SharePreset, target_size};
use crate::{colour, exif_out, sharpen};

/// Unsharp amount after a share downscale (FR-053): 0.2 scored the best SSIM against the
/// resized original (0.35 over-sharpens, none looks soft), RESULT §10.
const SHARE_SHARPEN: f32 = 0.2;
/// Shared photos go here, beside the dated print folders.
const SHARE_FOLDER: &str = "Share";

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
    /// Bytes of the originals that were exported, and of the files written (f64: TypeScript
    /// has no u64), so the UI can say how much lighter the export is (FR-053).
    pub source_bytes: f64,
    pub output_bytes: f64,
}

pub struct ExportOptions<'a> {
    pub destination: &'a Path,
    pub settings: &'a RenderSettings,
    pub quality: u8,
    /// Photos processed at once; bounds peak memory (NFR-004).
    pub max_inflight: usize,
    /// The place written in the stamp and the frame for one photo, if any (FR-005).
    pub place_for: &'a (dyn Fn(&PhotoRecord) -> Option<String> + Sync),
    /// The info frame for one photo, if any (FR-040).
    pub frame_for: &'a (dyn Fn(&PhotoRecord) -> Option<FrameRequest> + Sync),
    /// A subfolder of the destination for one photo, when folders are exported separately
    /// (FR-056); `None` puts every photo straight into the destination.
    pub group_for: &'a (dyn Fn(&PhotoRecord) -> Option<String> + Sync),
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
) -> Result<u64, ExportError> {
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }
    let bytes = std::fs::read(&record.path)?;
    let decoded = decode_image(codec, &bytes)?;
    let upright = apply_orientation(decoded.image, record.display_orientation());
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }

    let share = match options.settings.export_target.sanitized() {
        ExportTarget::Share(preset) => Some(preset),
        ExportTarget::Print => None,
    };
    // Shared photos keep their own ratio: no print crop (FR-053).
    let share_settings;
    let base = if share.is_some() {
        share_settings = RenderSettings {
            print_size: crate::PrintSize::ORIGINAL_ID.to_owned(),
            ..options.settings.clone()
        };
        &share_settings
    } else {
        options.settings
    };
    let max_long_edge = share.and_then(|preset| {
        let (w, h) = target_size(upright.width, upright.height, preset);
        let long = w.max(h);
        (long < upright.width.max(upright.height)).then_some(long)
    });
    let print = base
        .print_for(upright.width, upright.height)
        .unwrap_or(crate::PrintSize::ALL[1]);
    let capture = record.stamp_date();
    let frame = (options.frame_for)(record);
    let place = (options.place_for)(record);
    // The photo's own stamp, when it has one (FR-044).
    let settings = base.for_photo(record.edits.stamp.as_ref());
    let p = plan(&PlanInput {
        width: upright.width,
        height: upright.height,
        capture,
        settings: &settings,
        print,
        max_long_edge,
        frame: frame.as_ref(),
        crop: record.edits.crop,
        place: place.as_deref(),
        straighten: record.edits.straighten,
    });
    let mut image = render(&upright, &p)?;
    drop(upright);
    let jpeg = match share {
        Some(preset) => share_jpeg(
            &mut image,
            preset,
            max_long_edge.is_some(),
            decoded.icc.as_deref(),
            record,
            capture,
            codec,
        )?,
        // Prints carry no EXIF, so GPS never leaves the machine (FR-010).
        None => codec.encode(
            &image,
            EncodeOptions::print(options.quality),
            decoded.icc.as_deref(),
        )?,
    };
    drop(image);

    let base = match (options.group_for)(record) {
        Some(group) => options.destination.join(group),
        None => options.destination.to_path_buf(),
    };
    let dir = if share.is_some() {
        base.join(SHARE_FOLDER).join(day_folder(record))
    } else {
        base.join(day_folder(record))
    };
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
    Ok(jpeg.len() as u64)
}

/// A shared photo's JPEG (FR-053): sRGB, lightly sharpened after a downscale, the share
/// quality, and EXIF with the camera and Fotollection's name but never GPS.
fn share_jpeg(
    image: &mut crate::image::RgbImage,
    preset: SharePreset,
    downscaled: bool,
    icc: Option<&[u8]>,
    record: &PhotoRecord,
    capture: Option<chrono::NaiveDateTime>,
    codec: &dyn JpegCodec,
) -> Result<Vec<u8>, ExportError> {
    let conversion = colour::to_srgb(image, icc);
    if downscaled {
        sharpen::unsharp(image, SHARE_SHARPEN);
    }
    // sRGB needs no profile; one qcms could not read is kept so the colours stay right.
    let keep = (conversion == colour::Conversion::Unreadable)
        .then_some(icc)
        .flatten();
    let jpeg = codec.encode(image, preset.encode_options(), keep)?;
    let software = format!("Fotollection {}", crate::VERSION);
    let app1 = exif_out::exif_app1(&record.shot, capture, &software);
    Ok(exif_out::insert_app1(&jpeg, &app1).unwrap_or(jpeg))
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
    let (source_bytes, output_bytes) = (AtomicU64::new(0), AtomicU64::new(0));
    let failed = Mutex::new(Vec::new());
    let reserved = Mutex::new(HashSet::new());

    let work = || {
        records.par_iter().for_each(|record| {
            match export_one(record, options, codec, cancel, &reserved) {
                Ok(written) => {
                    exported.fetch_add(1, Ordering::Relaxed);
                    source_bytes.fetch_add(record.size, Ordering::Relaxed);
                    output_bytes.fetch_add(written, Ordering::Relaxed);
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
        #[allow(clippy::cast_precision_loss)] // exact far beyond any real export
        source_bytes: source_bytes.into_inner() as f64,
        #[allow(clippy::cast_precision_loss)]
        output_bytes: output_bytes.into_inner() as f64,
    }
}
