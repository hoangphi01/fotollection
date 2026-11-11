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
use crate::codec::{CodecError, EncodeOptions, JpegCodec, decode_photo};
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

/// The default JPEG quality of a print (FR-008, FR-092): 90, a mean ΔE of 0.94 against
/// quality 100 on a 20 × 30 cm print, under a visible step, at a third less than 95
/// (RESULT §30; owner, 2026-11-06).
pub const EXPORT_QUALITY: u8 = 90;
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
    /// Photos not written because a file of their name was there and the rule is *Skip*
    /// (FR-009).
    #[serde(default)]
    pub skipped: u32,
    /// Files still over the size limit at the lowest quality (FR-105).
    #[serde(default)]
    pub over_limit: u32,
}

/// The rules for the files an export writes, kept per machine (settings v16): Lightroom's
/// *Export location*, *File naming*, *Limit file size* and *Metadata* (FR-009, FR-010,
/// FR-104, FR-105).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportFiles {
    /// One cleaned folder name under the destination (and the source folder's, when apart).
    pub subfolder: Option<String>,
    pub naming: crate::naming::ExportNaming,
    pub existing: crate::settings::ExistingFiles,
    pub metadata: crate::settings::ExportMetadata,
    /// The largest file, in bytes.
    pub limit: Option<u64>,
}

impl ExportFiles {
    /// The rules as the user's settings give them.
    pub fn from_ui(ui: &crate::settings::UiSettings) -> Self {
        Self {
            subfolder: ui.export_subfolder.clone(),
            naming: ui.export_naming.clone(),
            existing: ui.export_existing,
            metadata: ui.export_metadata,
            limit: ui.export_limit_kb.map(|kb| u64::from(kb) * 1000),
        }
    }
}

/// What writing one file came to.
struct Written {
    bytes: u64,
    skipped: bool,
    over_limit: bool,
}

/// The faces found in a photo, for the print's credit to keep off (FR-121).
pub type CreditFor<'a> = dyn Fn(&PhotoRecord) -> Vec<crate::culling::FaceSummary> + Sync + 'a;

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
    /// A `YYYY-MM-DD` subfolder for each day (FR-009); `false` puts the files straight into
    /// the destination (or its group, or `Share/`).
    pub by_day: bool,
    /// The user's font for postcard text (FR-073, FR-042); `None` uses the bundled face.
    pub postcard_font: Option<ab_glyph::FontArc>,
    /// Subfolder, names, existing files, metadata and size limit (FR-009, FR-010, FR-104,
    /// FR-105).
    pub files: &'a ExportFiles,
    /// The small "fotollection.org" on each print (FR-121): `Some` when this machine prints
    /// it, giving the faces found in a photo to keep it off (empty when unknown); never used
    /// on a share export.
    pub credit_for: Option<&'a CreditFor<'a>>,
}

/// A sensible default for `max_inflight`: one per core, capped at 6. A photo is decoded and
/// encoded on one core, so throughput follows the photos in flight; past 6 it stops paying
/// on a real folder (ADR-0039, measured 2026-11-05: 26 MP JPEGs 9.4, 11.5, 12.5 photos/s at
/// 4, 6, 8; the Prague trip with its RAW files 9.54 at 6 and 9.21–9.31 at 8, with 0.2 GB
/// more memory). The memory budget below decides how many fit.
pub fn default_inflight() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().clamp(1, 6))
}

/// The memory an export's photos in flight may hold together, under NFR-004's 2 GB with room
/// for the rest of the app (previews, thumbnails, the library). 1.6 GB since 2026-11-05: the
/// per-photo estimate is about twice what a photo holds (106 MB measured for a 26 MP JPEG
/// estimated at 240 MB), so the whole process stayed at 1.21 GB with eight in flight.
pub const EXPORT_MEMORY_BUDGET: u64 = 1_600_000_000;

/// What one photo in flight holds, estimated from its file before it is read: a JPEG's
/// decoded and cropped images and its output are about 14 bytes per byte of a camera file
/// (a 20 MP, 11 MB JPEG holds about 125 MB, measured 2026-11-04); a RAW or HEIC file packs
/// fewer pixels per byte, about 6.
pub fn inflight_bytes(record: &PhotoRecord) -> u64 {
    let jpeg = record
        .path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"));
    record.size.saturating_mul(if jpeg { 14 } else { 6 })
}

/// Photos exported at once for `records` (NFR-003, NFR-004): one per core up to 6, fewer when
/// the largest photo would push them past [`EXPORT_MEMORY_BUDGET`]; at least one.
pub fn inflight_for<'a>(records: impl IntoIterator<Item = &'a PhotoRecord>) -> usize {
    let largest = records.into_iter().map(inflight_bytes).max().unwrap_or(0);
    inflight_within(largest, default_inflight())
}

/// How many photos holding `largest` bytes each fit the budget, from one to `cores`.
fn inflight_within(largest: u64, cores: usize) -> usize {
    let fit = usize::try_from(EXPORT_MEMORY_BUDGET / largest.max(1)).unwrap_or(usize::MAX);
    fit.clamp(1, cores.max(1))
}

/// `YYYY-MM-DD` folder for a photo (FR-009).
pub fn day_folder(record: &PhotoRecord) -> String {
    if record.flags & flags::DATE_FROM_FILE != 0 && record.mtime == 0 {
        return UNDATED_FOLDER.to_owned();
    }
    record.capture.format("%Y-%m-%d").to_string()
}

/// The file this photo goes to, by the rule for existing files (FR-009): its own name when
/// free; a file there from before is skipped (`None`) or replaced, as the rule says; a name
/// another photo of this run took gets `_1`, `_2`, whatever the rule, so a run never writes
/// over itself.
fn reserve_name(
    dir: &Path,
    stem: &str,
    reserved: &Mutex<HashSet<PathBuf>>,
    existing: crate::settings::ExistingFiles,
) -> Option<PathBuf> {
    use crate::settings::ExistingFiles;
    let own = dir.join(format!("{stem}.jpg"));
    if existing != ExistingFiles::AddNumber
        && let Ok(mut set) = reserved.lock()
        && !set.contains(&own)
    {
        if own.exists() && existing == ExistingFiles::Skip {
            return None;
        }
        set.insert(own.clone());
        return Some(own);
    }
    Some(number_name(dir, stem, reserved))
}

/// `dir/stem.jpg`, or `stem_1.jpg`, `stem_2.jpg`… so nothing existing is overwritten and
/// parallel workers never collide.
fn number_name(dir: &Path, stem: &str, reserved: &Mutex<HashSet<PathBuf>>) -> PathBuf {
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

#[allow(clippy::too_many_arguments)]
fn export_one(
    record: &PhotoRecord,
    index: usize,
    total: usize,
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
    reserved: &Mutex<HashSet<PathBuf>>,
) -> Result<Written, ExportError> {
    let (jpeg, share, fits) = encode_photo(record, options, codec, cancel)?;
    let name = export_name(options.files, &file_stem(record), record, index, total);
    let mut written = write_photo(record, options, &jpeg, share, cancel, reserved, &name)?;
    written.over_limit = !fits;
    Ok(written)
}

fn file_stem(record: &PhotoRecord) -> String {
    crate::naming::original_stem(record)
}

/// The `index`-th of `total` files' name, without `.jpg`: the template applied to `own`
/// (FR-104), the day as its day folder says.
pub fn export_name(
    files: &ExportFiles,
    own: &str,
    record: &PhotoRecord,
    index: usize,
    total: usize,
) -> String {
    crate::naming::file_name(&files.naming, own, &day_folder(record), index, total)
}

/// Two photos on one print (FR-035): the first and, unless the last photo was odd, the
/// second.
pub type SheetRecords = (PhotoRecord, Option<PhotoRecord>);

/// One collage sheet as a print JPEG, in memory (FR-035, ADR-0032). Sheets are always
/// prints: a share target does not apply to them.
fn encode_sheet(
    sheet: &SheetRecords,
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
) -> Result<(Vec<u8>, bool), ExportError> {
    let (first, second) = (&sheet.0, sheet.1.as_ref());
    let mut decoded = Vec::with_capacity(2);
    let mut profiles = Vec::with_capacity(2);
    // A RAW cell's five linear sliders go to its develop, the rest to the table (FR-088).
    let mut looks = Vec::with_capacity(2);
    for record in std::iter::once(first).chain(second) {
        if cancel.is_cancelled() {
            return Err(ExportError::Cancelled);
        }
        let (raw, table_look) = crate::preview::split_look(record, options.settings);
        looks.push(table_look);
        let bytes = std::fs::read(&record.path)?;
        let d = decode_photo(codec, &bytes, record.edits.raw_look(), &raw.adjust())?;
        profiles.push(d.icc);
        decoded.push((
            // As the photo sits on the sheet, its own turns in the cell included (FR-035).
            apply_orientation(d.image, record.sheet_orientation()),
            (options.place_for)(record),
        ));
    }
    // One canvas carries one profile: two photos in the same colour space keep it; two in
    // different spaces both become sRGB, so neither is read in the other's space.
    let icc = if profiles.windows(2).all(|w| w[0] == w[1]) {
        profiles.into_iter().next().flatten()
    } else {
        for ((image, _), profile) in decoded.iter_mut().zip(&profiles) {
            colour::to_srgb(image, profile.as_deref());
        }
        None
    };
    let print = crate::sheet::sheet_print(options.settings);
    let records = [Some(first), second];
    let mut sources = [None, None];
    for ((slot, ((image, place), record)), table_look) in sources
        .iter_mut()
        .zip(decoded.iter().zip(records.into_iter().flatten()))
        .zip(&looks)
    {
        *slot = Some(crate::sheet::CellSource {
            image,
            capture: record.stamp_date(),
            crop: record.edits.sheet_crop(),
            straighten: record.edits.sheet_straighten(),
            place: place.as_deref(),
            stamp: record.edits.stamp.as_ref(),
            caption: record.edits.caption.as_deref(),
            look: Some(table_look),
            postcard: record.edits.postcard.as_ref(),
        });
    }
    let turned = decoded
        .first()
        .map_or(first.edits.sheet_turned, |(image, _)| {
            crate::sheet::layout_turned(
                first.edits.sheet_turned,
                first.edits.sheet_turns(),
                (image.width, image.height),
            )
        });
    let image = crate::sheet::render_sheet_with(
        options.settings,
        print,
        sources,
        turned,
        None,
        options.postcard_font.as_ref(),
        [crate::sheet::CellMark::Normal; 2],
        options.credit_for.is_some(),
    )?;
    drop(decoded);
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }
    let (jpeg, fits) = encode_within(options.files.limit, options.quality, |q| {
        codec.encode(&image, EncodeOptions::print(q), icc.as_deref())
    })?;
    Ok((
        with_exif(jpeg, first, first.stamp_date(), options.files.metadata),
        fits,
    ))
}

/// `<first>+<second>`, the file name of a sheet; `<photo>-postcard` for a postcard.
fn sheet_stem(sheet: &SheetRecords, postcard: bool) -> String {
    match &sheet.1 {
        Some(second) => format!("{}+{}", file_stem(&sheet.0), file_stem(second)),
        // `<photo>-postcard`, so it never takes the name of the photo's own print (FR-073).
        None if postcard => format!("{}-postcard", file_stem(&sheet.0)),
        None => file_stem(&sheet.0),
    }
}

/// The exported JPEG of one photo, in memory, whether it is a share export, and whether it
/// fits the size limit (FR-105).
fn encode_photo(
    record: &PhotoRecord,
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
) -> Result<(Vec<u8>, bool, bool), ExportError> {
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }
    let bytes = std::fs::read(&record.path)?;
    let (raw, table_look) = crate::preview::split_look(record, options.settings);
    let decoded = decode_photo(codec, &bytes, record.edits.raw_look(), &raw.adjust())?;
    // The file's bytes are not needed past the decode (NFR-004).
    drop(bytes);
    let upright = apply_orientation(decoded.image, record.display_orientation());
    if cancel.is_cancelled() {
        return Err(ExportError::Cancelled);
    }

    let share = match options.settings.export_target.sanitized() {
        ExportTarget::Share(preset) => Some(preset),
        ExportTarget::Print => None,
    };
    // Shared photos are not prints: no print crop and no print border (FR-053, FR-067).
    let base = options.settings.for_target();
    let base = base.as_ref();
    let max_long_edge = share.and_then(|preset| {
        // A photo with a fill is sized by its canvas (FR-075); the plan never enlarges it.
        let fill = base.share_fill.sanitized();
        if fill.enabled {
            return crate::share::fill_long_edge(preset, fill.ratio);
        }
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
    let settings = base.for_photo(
        record.edits.stamp.as_ref(),
        record.edits.caption.as_deref(),
        Some(&table_look),
    );
    let faces = options.credit_for.map(|faces| faces(record));
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
        ratio: None,
        credit: faces.as_deref(),
        turned: record.edits.print_turned,
    });
    let p = crate::plan::RenderPlan {
        linear_light: share.is_some(),
        ..p
    };
    let downscaled = p.out_width.max(p.out_height) < p.crop.width.max(p.crop.height);
    let mut image = render(&upright, &p)?;
    drop(upright);
    let (jpeg, fits) = if let Some(preset) = share {
        share_jpeg(
            &mut image,
            preset,
            options,
            downscaled,
            decoded.icc.as_deref(),
            record,
            capture,
            codec,
        )?
    } else {
        // Prints keep the camera details from an allowlist, never GPS (FR-010).
        let (jpeg, fits) = encode_within(options.files.limit, options.quality, |q| {
            codec.encode(&image, EncodeOptions::print(q), decoded.icc.as_deref())
        })?;
        (
            with_exif(jpeg, record, capture, options.files.metadata),
            fits,
        )
    };
    drop(image);
    Ok((jpeg, share.is_some(), fits))
}

/// The lowest quality a size limit may go down to (FR-105): the slider's own floor.
pub const LIMIT_FLOOR: u8 = 60;

/// A JPEG within `limit` bytes, when one is asked (FR-105, Lightroom's *Limit file size*):
/// `encode` at `quality` first; over the limit, the highest quality between [`LIMIT_FLOOR`]
/// and `quality` that fits, by bisection (RESULT §31: 3.9 encodes at 2 MB). Returns the
/// JPEG and whether it fits; one that cannot fit even at the floor is the floor's.
pub fn encode_within<E>(
    limit: Option<u64>,
    quality: u8,
    mut encode: impl FnMut(u8) -> Result<Vec<u8>, E>,
) -> Result<(Vec<u8>, bool), E> {
    let first = encode(quality)?;
    let Some(limit) = limit else {
        return Ok((first, true));
    };
    if first.len() as u64 <= limit || quality <= LIMIT_FLOOR {
        let fits = first.len() as u64 <= limit;
        return Ok((first, fits));
    }
    // `best` fits; the answer lies in (best, high).
    let floor = encode(LIMIT_FLOOR)?;
    if floor.len() as u64 > limit {
        return Ok((floor, false));
    }
    // Bisection: interpolating from the sizes either side took more encodes (4.3 against
    // 3.9 at 2 MB, RESULT §31), as a file grows fastest near the top of the scale.
    let (mut best, mut best_q, mut high) = (floor, LIMIT_FLOOR, quality);
    while high - best_q > 1 {
        let mid = best_q + (high - best_q) / 2;
        let jpeg = encode(mid)?;
        if jpeg.len() as u64 <= limit {
            best = jpeg;
            best_q = mid;
        } else {
            high = mid;
        }
    }
    Ok((best, true))
}

/// Writes one exported JPEG into the destination, under its day's folder when exported by
/// day; returns its size.
fn write_photo(
    record: &PhotoRecord,
    options: &ExportOptions<'_>,
    jpeg: &[u8],
    share: bool,
    cancel: &CancellationToken,
    reserved: &Mutex<HashSet<PathBuf>>,
    stem: &str,
) -> Result<Written, ExportError> {
    let base = match (options.group_for)(record) {
        Some(group) => options.destination.join(group),
        None => options.destination.to_path_buf(),
    };
    // Lightroom's *Put in subfolder* (FR-009): one cleaned name, before Share and the days.
    let base = match &options.files.subfolder {
        Some(sub) => base.join(sub),
        None => base,
    };
    let base = if share { base.join(SHARE_FOLDER) } else { base };
    let dir = if options.by_day {
        base.join(day_folder(record))
    } else {
        base
    };
    std::fs::create_dir_all(&dir)?;
    let Some(target) = reserve_name(&dir, stem, reserved, options.files.existing) else {
        return Ok(Written {
            bytes: 0,
            skipped: true,
            over_limit: false,
        });
    };
    let tmp = dir.join(format!(".{stem}.{}.part", record.id.0));
    std::fs::write(&tmp, jpeg)?;
    if cancel.is_cancelled() {
        let _ = std::fs::remove_file(&tmp);
        return Err(ExportError::Cancelled);
    }
    // Over a file being replaced, the rename swaps it in whole (FR-009).
    std::fs::rename(&tmp, &target).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(Written {
        bytes: jpeg.len() as u64,
        skipped: false,
        over_limit: false,
    })
}

/// A shared photo's JPEG (FR-053): sRGB, lightly sharpened after a downscale, the share
/// quality, and EXIF with the camera and Fotollection's name but never GPS.
#[allow(clippy::too_many_arguments)]
fn share_jpeg(
    image: &mut crate::image::RgbImage,
    preset: SharePreset,
    options: &ExportOptions<'_>,
    downscaled: bool,
    icc: Option<&[u8]>,
    record: &PhotoRecord,
    capture: Option<chrono::NaiveDateTime>,
    codec: &dyn JpegCodec,
) -> Result<(Vec<u8>, bool), ExportError> {
    let conversion = colour::to_srgb(image, icc);
    if downscaled {
        sharpen::unsharp(image, SHARE_SHARPEN);
    }
    // sRGB needs no profile; one qcms could not read is kept so the colours stay right.
    let keep = (conversion == colour::Conversion::Unreadable)
        .then_some(icc)
        .flatten();
    // The preset's settings, at the quality chosen for this export (FR-092; a share preset
    // sets its own default, 88 or 80, when chosen).
    let image = &*image;
    let (jpeg, fits) = encode_within(options.files.limit, options.quality, |quality| {
        codec.encode(
            image,
            EncodeOptions {
                quality,
                ..preset.encode_options()
            },
            keep,
        )
    })?;
    Ok((
        with_exif(jpeg, record, capture, options.files.metadata),
        fits,
    ))
}

/// `jpeg` with what `metadata` keeps (FR-010): the camera details and Fotollection's name
/// (the EXIF allowlist of `exif_out`), the capture time and the name only, or nothing. GPS is
/// never written (FR-053). Pixels are written upright, so no orientation tag is needed.
fn with_exif(
    jpeg: Vec<u8>,
    record: &PhotoRecord,
    capture: Option<chrono::NaiveDateTime>,
    metadata: crate::settings::ExportMetadata,
) -> Vec<u8> {
    use crate::settings::ExportMetadata;
    let software = format!("Fotollection {}", crate::VERSION);
    let app1 = match metadata {
        ExportMetadata::Camera => exif_out::exif_app1(&record.shot, capture, &software),
        ExportMetadata::DateOnly => {
            exif_out::exif_app1(&crate::library::ShotInfo::default(), capture, &software)
        }
        ExportMetadata::Nothing => return jpeg,
    };
    exif_out::insert_app1(&jpeg, &app1).unwrap_or(jpeg)
}

/// What an export will weigh, before it runs (FR-053): the originals' total and an estimate
/// of the files, from a few photos encoded in memory with the same settings.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportEstimate {
    pub photos: u32,
    /// Bytes as f64: TypeScript has no u64.
    pub source_bytes: f64,
    pub estimated_bytes: f64,
    /// Photos actually encoded for the estimate; when it equals `photos`, it is exact.
    pub sampled: u32,
}

/// Photos encoded to estimate an export; a few spread over the set is close enough.
pub const ESTIMATE_SAMPLES: usize = 4;

/// Estimates [`run_export`] for `records` by encoding up to `samples` of them, spread
/// evenly, and scaling their output-to-source ratio to the whole set. Nothing is written.
pub fn estimate_export(
    records: &[PhotoRecord],
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    samples: usize,
) -> ExportEstimate {
    let cancel = CancellationToken::new();
    estimate(
        records,
        samples,
        |r| r.size,
        |r| encode_photo(r, options, codec, &cancel).map(|(jpeg, _, _)| jpeg),
    )
}

/// Estimates an export of collage sheets (FR-035), as [`estimate_export`] does for photos.
pub fn estimate_sheets(
    sheets: &[SheetRecords],
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    samples: usize,
) -> ExportEstimate {
    let cancel = CancellationToken::new();
    // Two decoded photos per sample: half as many at once (NFR-004).
    estimate(sheets, samples.div_ceil(2), sheet_size, |s| {
        encode_sheet(s, options, codec, &cancel).map(|(jpeg, _)| jpeg)
    })
}

fn sheet_size(sheet: &SheetRecords) -> u64 {
    sheet.0.size + sheet.1.as_ref().map_or(0, |r| r.size)
}

#[allow(clippy::cast_precision_loss)]
fn estimate<T: Sync>(
    items: &[T],
    samples: usize,
    size: impl Fn(&T) -> u64 + Sync,
    encode: impl Fn(&T) -> Result<Vec<u8>, ExportError> + Sync,
) -> ExportEstimate {
    let total: u64 = items.iter().map(&size).sum();
    let n = items.len();
    let k = samples.clamp(1, n.max(1)).min(n);
    let picks: Vec<&T> = (0..k).map(|i| &items[i * n / k.max(1)]).collect();
    let sizes: Vec<(u64, u64)> = picks
        .par_iter()
        .filter_map(|item| {
            // Behind the photo on screen: a RAW sample develops in full (FR-091).
            crate::raw::with_priority(crate::raw::Priority::Background, || encode(item))
                .ok()
                .map(|jpeg| (size(item), jpeg.len() as u64))
        })
        .collect();
    let (src, out) = sizes
        .iter()
        .fold((0u64, 0u64), |(s, o), (a, b)| (s + a, o + b));
    let ratio = if src == 0 {
        0.0
    } else {
        out as f64 / src as f64
    };
    ExportEstimate {
        photos: u32::try_from(n).unwrap_or(u32::MAX),
        source_bytes: total as f64,
        estimated_bytes: (total as f64 * ratio).round(),
        sampled: u32::try_from(sizes.len()).unwrap_or(u32::MAX),
    }
}

/// Exports `records`. `on_progress(done, total)` is called after each photo.
pub fn run_export(
    records: &[PhotoRecord],
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(u32, u32) + Sync),
) -> ExportSummary {
    let reserved = Mutex::new(HashSet::new());
    run(
        records,
        options,
        on_progress,
        |r| r,
        |r| r.size,
        |r, index| export_one(r, index, records.len(), options, codec, cancel, &reserved),
        cancel,
    )
}

/// Exports collage sheets, one file per sheet (FR-035). `on_progress(done, total)` counts
/// sheets; a failure is reported for the sheet's first photo.
pub fn run_export_sheets(
    sheets: &[SheetRecords],
    options: &ExportOptions<'_>,
    codec: &dyn JpegCodec,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(u32, u32) + Sync),
) -> ExportSummary {
    let reserved = Mutex::new(HashSet::new());
    // A sheet holds two decoded photos at once: half as many in flight keeps the peak memory
    // of single photos (NFR-004).
    let options = &ExportOptions {
        max_inflight: options.max_inflight.div_ceil(2),
        postcard_font: options.postcard_font.clone(),
        ..*options
    };
    run(
        sheets,
        options,
        on_progress,
        |s| &s.0,
        sheet_size,
        |sheet, index| {
            let (jpeg, fits) = encode_sheet(sheet, options, codec, cancel)?;
            let own = sheet_stem(sheet, options.settings.collage.postcard.enabled);
            let name = export_name(options.files, &own, &sheet.0, index, sheets.len());
            let mut written =
                write_photo(&sheet.0, options, &jpeg, false, cancel, &reserved, &name)?;
            written.over_limit = !fits;
            Ok(written)
        },
        cancel,
    )
}

/// Runs `export` over `items` in a pool of `max_inflight` threads, isolating failures.
fn run<T: Sync>(
    items: &[T],
    options: &ExportOptions<'_>,
    on_progress: &(dyn Fn(u32, u32) + Sync),
    record: impl Fn(&T) -> &PhotoRecord + Sync,
    size: impl Fn(&T) -> u64 + Sync,
    export: impl Fn(&T, usize) -> Result<Written, ExportError> + Sync,
    cancel: &CancellationToken,
) -> ExportSummary {
    let total = u32::try_from(items.len()).unwrap_or(u32::MAX);
    let done = AtomicU32::new(0);
    let exported = AtomicU32::new(0);
    let (source_bytes, output_bytes) = (AtomicU64::new(0), AtomicU64::new(0));
    let failed = Mutex::new(Vec::new());
    let (skipped, over_limit) = (AtomicU32::new(0), AtomicU32::new(0));

    let work = || {
        // Each item's place in the export is fixed before any thread starts, so a sequence
        // number in its name (FR-104) never depends on which photo finishes first.
        items.par_iter().enumerate().for_each(|(index, item)| {
            // A photo the user opens meanwhile develops first (FR-091).
            match crate::raw::with_priority(crate::raw::Priority::Background, || {
                export(item, index)
            }) {
                Ok(written) if written.skipped => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                }
                Ok(written) => {
                    exported.fetch_add(1, Ordering::Relaxed);
                    source_bytes.fetch_add(size(item), Ordering::Relaxed);
                    output_bytes.fetch_add(written.bytes, Ordering::Relaxed);
                    if written.over_limit {
                        over_limit.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(ExportError::Cancelled) => {}
                Err(e) => {
                    let r = record(item);
                    if let Ok(mut f) = failed.lock() {
                        f.push(ExportFailure {
                            id: r.id,
                            file_name: r.file_name.clone(),
                            code: e.code().to_owned(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            on_progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
        });
    };
    match crate::pool::contained(Some(options.max_inflight), "export") {
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
        skipped: skipped.into_inner(),
        over_limit: over_limit.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fr_105_a_limit_finds_the_highest_quality_that_fits() -> Result<(), ()> {
        // A file of `q * 10` bytes: the limit 700 fits quality 70 at most.
        let mut asked = Vec::new();
        let encode = |q: u8| -> Result<Vec<u8>, ()> { Ok(vec![0; usize::from(q) * 10]) };
        let (jpeg, fits) = super::encode_within(Some(700), 90, |q| {
            asked.push(q);
            encode(q)
        })?;
        assert!(fits);
        assert_eq!(jpeg.len(), 700);
        assert!(asked.len() <= 8, "{asked:?}");
        // No limit, or already under it: one encode.
        let (_, fits) = super::encode_within(None, 90, encode)?;
        assert!(fits);
        let mut once = 0;
        super::encode_within(Some(10_000), 90, |q| {
            once += 1;
            encode(q)
        })?;
        assert_eq!(once, 1);
        // Too big even at the floor: the floor's file, marked as not fitting.
        let (jpeg, fits) = super::encode_within(Some(100), 90, encode)?;
        assert!(!fits);
        assert_eq!(jpeg.len(), 600);
        Ok(())
    }

    use super::*;

    /// NFR-003, NFR-004: 20 MP camera JPEGs (11 MB) run one per core up to 6; 60 MP ones
    /// (about 30 MB) only two at once, inside the budget; a photo larger than the budget
    /// still runs, alone.
    #[test]
    fn nfr_004_photos_in_flight_follow_the_largest_photo() {
        let jpeg = |mb: u64| mb * 1_000_000 * 14;
        assert_eq!(inflight_within(jpeg(11), 6), 6);
        assert_eq!(inflight_within(jpeg(11), 4), 4);
        assert_eq!(inflight_within(jpeg(30), 6), 3);
        assert!(inflight_within(jpeg(30), 6) as u64 * jpeg(30) <= EXPORT_MEMORY_BUDGET);
        assert_eq!(inflight_within(jpeg(200), 6), 1);
        assert_eq!(inflight_within(0, 6), 6);
    }
}
