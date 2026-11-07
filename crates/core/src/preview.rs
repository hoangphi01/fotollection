// SPDX-License-Identifier: GPL-3.0-only
//! Interactive previews (FR-007). Each photo is decoded once into an upright T2 image
//! (~2 MP) kept in a small LRU; re-rendering for new settings then only crops, resizes
//! and stamps that buffer (NFR-002).
//!
//! Zooming past the preview's pixels (FR-057) asks for one region at full resolution. The
//! full image the preview was made from is kept in a second cache bounded by bytes, so the
//! first region needs no second decode; it is rendered exactly as export renders it, and
//! only the visible part is encoded.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::codec::{EncodeOptions, JpegCodec, decode_image, decode_photo};
use crate::image::{RgbImage, apply_orientation};
use crate::library::{PhotoId, PhotoRecord};
use crate::looks::{LookSettings, RawSliders};
use crate::plan::{CropRect, FrameRequest, PlanInput, RenderPlan, plan};
use crate::render::{crop_and_resize, finish, fit_long_edge, render};
use crate::settings::RenderSettings;
use crate::thumbs::ThumbError;

/// Long edge of the cached T2 source.
pub const T2_LONG_EDGE: u32 = 2048;
/// Long edge of the preview sent to the UI.
pub const PREVIEW_LONG_EDGE: u32 = 1600;
/// The histogram counts every second pixel of every second row: 1.0 ms p95 on a 1600 px
/// preview against 3.7 ms for every pixel, its curves within 0.9 per mille of the full count
/// (`examples/measure_histogram.rs`, OPTIMIZATION §5, 2026-11-07).
pub const HISTOGRAM_STEP: u32 = 2;
const PREVIEW_QUALITY: u8 = 85;
/// What the preview may keep in memory, in bytes (FR-091, ADR-0041): the app gives it a
/// quarter of the machine's RAM, split between the 2048 px sources (about 12.5 MB each), the
/// kept RAW develops (about 50 MB each) and the full images for zoom. The image on screen
/// is always kept, whatever its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub sources: usize,
    pub prepared: usize,
    pub full: usize,
}

impl Budget {
    /// `total` bytes, split 60 / 25 / 15 between sources, RAW develops and zoom.
    pub fn of(total: u64) -> Self {
        let total = usize::try_from(total).unwrap_or(usize::MAX);
        Self {
            sources: total / 100 * 60,
            prepared: total / 100 * 25,
            full: total / 100 * 15,
        }
    }

    /// A tenth of each part: what the preview keeps while the machine is short of memory.
    #[must_use]
    pub const fn lowered(self) -> Self {
        Self {
            sources: self.sources / 10,
            prepared: self.prepared / 10,
            full: self.full / 10,
        }
    }
}

/// Quality of a standard preview on disk: the screen's 85 would add its own loss to the
/// preview's, so a little higher (about 0.7 MB at 2048 px, RESULT §29).
const STANDARD_QUALITY: u8 = 90;

/// The standard previews' own version, in their file name (DATABASE.md §8). Bump it when the
/// pixels of a 2048 px source change without `thumbs::CACHE_VERSION` changing: a RAW develop
/// or finish (`raw`, `raw_finish`), the colour conversion, the resize. Files of an older
/// version are never read again and go with the cache limit. v0.11.0 wrote version 1
/// without a number (`.t2-o…`). 3: v0.14.0, the RAW tone curve interpolated, shadows
/// without steps (RESULT §33).
pub const STANDARD_VERSION: u8 = 3;

/// Writes a standard preview atomically (a `.part` of its own, then a rename), as T1 is
/// written; a failed write only means it is made again next time. Two writers of one photo
/// never share a `.part` (audit 2026-11-06): the last rename wins, whole.
fn write_standard(path: &std::path::Path, image: &RgbImage, codec: &dyn JpegCodec) -> bool {
    static PART: AtomicU64 = AtomicU64::new(0);
    let Ok(bytes) = codec.encode(image, EncodeOptions::screen(STANDARD_QUALITY), None) else {
        return false;
    };
    let Some(dir) = path.parent() else {
        return false;
    };
    let part = path.with_extension(format!(
        "{}-{}.part",
        std::process::id(),
        PART.fetch_add(1, Ordering::Relaxed)
    ));
    let written = std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&part, &bytes))
        .and_then(|()| std::fs::rename(&part, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    written.is_ok()
}

/// Standard previews of photos on screen waiting to be written: a few, at about 12.5 MB each.
const WRITES_WAITING: usize = 4;

/// Writes a standard preview of the photo on screen on a writer thread of its own, so its
/// first view does not wait for the encode and the write (about 15 ms of 290 for a JPEG,
/// OPTIMIZATION §5). When the writer is behind, the write is dropped: the background pass makes
/// it later. The writer encodes with the default codec, the one the app uses.
fn write_standard_later(path: std::path::PathBuf, image: Arc<RgbImage>) {
    type Job = (std::path::PathBuf, Arc<RgbImage>);
    static WRITER: std::sync::OnceLock<Option<std::sync::mpsc::SyncSender<Job>>> =
        std::sync::OnceLock::new();
    let writer = WRITER.get_or_init(|| {
        let (send, jobs) = std::sync::mpsc::sync_channel::<Job>(WRITES_WAITING);
        std::thread::Builder::new()
            .name("fotollection-standard-writer".into())
            .spawn(move || {
                let codec = crate::codec::default_codec();
                for (path, image) in jobs {
                    write_standard(&path, &image, codec.as_ref());
                }
            })
            .ok()
            .map(|_| send)
    });
    if let Some(writer) = writer {
        let _ = writer.try_send((path, image));
    }
}

/// Under this share of memory available, the preview gives back what it keeps (FR-091).
pub const SHORT_OF_MEMORY_PERCENT: u8 = 10;

/// Whether the machine is short of memory, from the share still available: the watcher
/// lowers the preview's budget then, for as long as it lasts (ADR-0041).
pub fn short_of_memory(available_percent: Option<u8>) -> bool {
    available_percent.is_some_and(|p| p < SHORT_OF_MEMORY_PERCENT)
}

/// Bytes free on the disk that holds `path` (FR-086's *Auto*).
pub fn free_disk(path: &std::path::Path) -> Option<u64> {
    fotollection_oscodec::free_disk(path)
}

/// The share of the machine's memory still available, in percent (FR-091).
pub fn available_memory_percent() -> Option<u8> {
    fotollection_oscodec::available_memory_percent()
}

/// The machine's RAM in bytes, 8 GB when the system does not say. Asked once: it does not
/// change while the app runs, and on macOS each ask starts `sysctl` (audit 2026-11-06).
pub fn machine_memory() -> u64 {
    const UNKNOWN: u64 = 8 * 1024 * 1024 * 1024;
    static TOTAL: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *TOTAL.get_or_init(|| fotollection_oscodec::total_memory().unwrap_or(UNKNOWN))
}

/// `percent` of the machine's RAM for the preview (owner, 2026-11-05: use the memory for
/// instant previews, ADR-0041; 25 by default, set in the Settings window, FR-086).
pub fn machine_budget(percent: u8) -> Budget {
    Budget::of(machine_memory() / 100 * u64::from(percent))
}

impl Default for Budget {
    /// The sizes before FR-091: eight sources, three RAW develops, two 32 MP photos.
    fn default() -> Self {
        Self {
            sources: 100_000_000,
            prepared: 150_000_000,
            full: 200_000_000,
        }
    }
}

/// Whether, and where, an image just read or made stays in memory (FR-091).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    /// Not at all: the background builder, so it never pushes out what the user looked at.
    No,
    /// Just below the newest: a prefetch, so the photo on screen stays the one always kept
    /// (audit 2026-11-06). Its full image is not kept for zoom either.
    Behind,
    /// As the newest: the photo on screen.
    Shown,
}

/// The bytes the preview holds, by kind (FR-086).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub sources: usize,
    pub prepared: usize,
    pub full: usize,
    /// The collage grid's small sources and the last resized preview, outside the budget.
    pub others: usize,
}

impl Usage {
    pub fn total(&self) -> usize {
        self.sources + self.prepared + self.full + self.others
    }
}

/// Lets the oldest of `entries` go until the rest fit in `budget` bytes; the newest stays.
fn shrink_to<K, V>(entries: &Mutex<Vec<(K, V)>>, bytes: impl Fn(&V) -> usize, budget: usize) {
    if let Ok(mut entries) = entries.lock() {
        let mut total: usize = entries.iter().map(|(_, v)| bytes(v)).sum();
        while total > budget && entries.len() > 1 {
            total -= bytes(&entries.remove(0).1);
        }
    }
}

/// Keeps `value` under `key` (as the newest, or just below it when `place` is `Behind`),
/// then lets the oldest go until the rest fit in `budget` bytes; the newest always stays.
/// Nothing is kept when `still` says the cache was cleared since the value was asked for:
/// checked under the lock `clear` takes, so a photo of the old library never lands in the
/// new one (audit 2026-11-06).
fn keep_bounded<K: PartialEq, V>(
    entries: &Mutex<Vec<(K, V)>>,
    key: K,
    value: V,
    bytes: impl Fn(&V) -> usize,
    budget: usize,
    place: Keep,
    still: impl Fn() -> bool,
) {
    if place == Keep::No {
        return;
    }
    if let Ok(mut entries) = entries.lock() {
        if !still() {
            return;
        }
        entries.retain(|(k, _)| *k != key);
        if place == Keep::Behind && !entries.is_empty() {
            let below_newest = entries.len() - 1;
            entries.insert(below_newest, (key, value));
        } else {
            entries.push((key, value));
        }
    }
    shrink_to(entries, bytes, budget);
}

/// Small sources kept for the collage grid: about 0.3 MB each, so 64 photos (32 sheets, more
/// than a grid shows) cost about 20 MB (FR-035, NFR-021).
const SMALL_CAPACITY: usize = 64;
/// RAW tile sources kept, about 0.3 MB each (FR-079): a Looks card and Simple mode's strip
/// show one photo at a time.
const TILE_CAPACITY: usize = 16;
/// The long edge of a small source made from the full photo when it has no thumbnail yet.
const SMALL_LONG_EDGE: u32 = crate::thumbs::T1_LONG_EDGE;
/// A look tile's side in pixels (FR-079): the Looks card draws tiles about 80 px wide, so
/// twice that for sharp screens.
pub const LOOK_TILE_EDGE: u32 = 192;
/// A look tile's JPEG quality: small, seen small.
const LOOK_TILE_QUALITY: u8 = 85;

/// A look tile's pixels (FR-079): the centre square of `small`, [`LOOK_TILE_EDGE`] wide,
/// through `look`'s table. `measure_look_tiles` times this.
pub fn look_tile(
    small: &RgbImage,
    look: &crate::looks::LookSettings,
) -> Result<RgbImage, crate::render::RenderError> {
    let mut tile = fit_long_edge(&centre_square(small), LOOK_TILE_EDGE)?;
    if let Some(table) = look.table() {
        crate::looks::apply(&mut tile, &table);
    }
    Ok(tile)
}

/// The largest square at the centre of `image`.
fn centre_square(image: &RgbImage) -> RgbImage {
    let side = image.width.min(image.height);
    let (x0, y0) = ((image.width - side) / 2, (image.height - side) / 2);
    let row = image.width as usize * 3;
    let mut data = Vec::with_capacity(side as usize * side as usize * 3);
    for y in y0..y0 + side {
        let start = y as usize * row + x0 as usize * 3;
        data.extend_from_slice(&image.data[start..start + side as usize * 3]);
    }
    RgbImage {
        width: side,
        height: side,
        data,
    }
}

/// Widest region the UI may ask for, in pixels: a 4K screen at 1:1.
pub const REGION_MAX_PX: u32 = 4096;

/// A photo and the orientation it was decoded in, so a rotated photo is decoded again.
/// A photo, its orientation and how it is finished if it is a RAW (ADR-0040): either change
/// is another image, so it misses the cache and the other stays for a quick comparison.
/// The RAW sliders made in linear light (FR-088) are part of it too: each step of one is
/// another image, re-finished from the kept develop (`prepared`).
type SourceKey = (PhotoId, u8, crate::raw::RawLook, RawSliders);

fn source_key(record: &PhotoRecord, raw: RawSliders) -> SourceKey {
    (
        record.id,
        record.display_orientation(),
        record.edits.raw_look(),
        raw,
    )
}

/// For `record` with `settings`: the sliders its RAW takes in linear light and the look left
/// for the table (FR-088); any other file keeps its whole look in the table.
pub fn split_look(record: &PhotoRecord, settings: &RenderSettings) -> (RawSliders, LookSettings) {
    let look = record.edits.look.as_ref().unwrap_or(&settings.look);
    if crate::raw::is_raw_name(&record.file_name) {
        look.split_for_raw()
    } else {
        (RawSliders::default(), look.clone())
    }
}

/// A part of the rendered photo, in fractions of its width and height, wanted at most
/// `px` pixels wide (FR-057).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    px: u32,
}

impl Region {
    /// Checks a region from the UI: finite fractions that start inside the photo and a
    /// positive size that ends inside it; `px` is clamped to `1..=REGION_MAX_PX`.
    pub fn new(x: f64, y: f64, width: f64, height: f64, px: u32) -> Option<Self> {
        let inside = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
        (inside(x) && inside(y) && inside(width) && inside(height))
            .then_some(())
            .filter(|()| x < 1.0 && y < 1.0 && width > 0.0 && height > 0.0)
            .filter(|()| x + width <= 1.0 + 1e-6)
            .filter(|()| y + height <= 1.0 + 1e-6)
            .map(|()| Self {
                x,
                y,
                width,
                height,
                px: px.clamp(1, REGION_MAX_PX),
            })
    }

    /// The region in pixels of a `width`×`height` image: at least one pixel, never outside.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn rect(&self, width: u32, height: u32) -> (u32, u32, u32, u32) {
        let (w, h) = (f64::from(width), f64::from(height));
        let x0 = ((self.x * w).floor() as u32).min(width - 1);
        let y0 = ((self.y * h).floor() as u32).min(height - 1);
        let x1 = (((self.x + self.width) * w).ceil() as u32).clamp(x0 + 1, width);
        let y1 = (((self.y + self.height) * h).ceil() as u32).clamp(y0 + 1, height);
        (x0, y0, x1 - x0, y1 - y0)
    }
}

/// `entries` in least-recently-used order.
fn take_cached(
    entries: &Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    key: SourceKey,
) -> Option<Arc<RgbImage>> {
    let mut entries = entries.lock().ok()?;
    let pos = entries.iter().position(|(k, _)| *k == key)?;
    let entry = entries.remove(pos);
    let image = Arc::clone(&entry.1);
    entries.push(entry); // most recently used last
    Some(image)
}

/// The image kept under `key` without marking it used: a prefetch or the background builder
/// must not move a neighbour above the photo on screen.
fn peek_cached(
    entries: &Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    key: SourceKey,
) -> Option<Arc<RgbImage>> {
    let entries = entries.lock().ok()?;
    entries
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, image)| Arc::clone(image))
}

/// A full-resolution upright image, and the profile its pixels are still in: converting a
/// 24 MP photo to sRGB costs about 20 ms, so it waits until a zoom needs the full pixels;
/// the preview converts its 2048 px copy instead (FR-072, OPTIMIZATION §5).
#[derive(Clone)]
struct Full {
    image: Arc<RgbImage>,
    /// `Some` until converted to sRGB.
    icc: Option<Arc<[u8]>>,
}

/// Gives a photo's T1 thumbnail JPEG, when it has one (the thumbnail cache).
pub type Thumbnail<'a> = dyn Fn(&PhotoRecord) -> Option<Vec<u8>> + 'a;

#[derive(Default)]
pub struct PreviewCache {
    entries: Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    /// Upright originals, decoded only for zoom; bounded by `budget.full`.
    full: Mutex<Vec<(SourceKey, Full)>>,
    /// Small upright sources for the collage grid's sheets, from the photos' thumbnails.
    small: Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    /// The last preview photo cropped and resized, before its look and stamp.
    resized: Mutex<Option<Resized>>,
    /// Small sources of camera RAWs' look tiles (FR-079), cut from their standard previews (our
    /// develop) rather than their thumbnails (the camera's JPEG, ΔE2000 2–9 apart, RESULT §34).
    /// At most `TILE_CAPACITY` (about 0.3 MB each, outside the budget); emptied with the small
    /// sources (project switch, memory short, *Clear cache*).
    tiles: Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    /// RAW develops at 2048 px with their fits, so a RAW slider's step only re-finishes
    /// (FR-088).
    prepared: Mutex<Vec<(PhotoId, Arc<Prepared>)>>,
    /// Changed live from the Settings window (FR-086); read, never held across work.
    budget: Mutex<Budget>,
    /// The thumbnail cache's folder, where standard previews are kept on disk (FR-091).
    disk: Option<std::path::PathBuf>,
    /// Bumped by [`Self::clear`]: an image asked for before it is not kept after it, so a
    /// develop that ends after a project switch does not land under an id the new library
    /// reuses (audit 2026-11-06).
    epoch: AtomicU64,
    /// The sources being made right now. Background work skips them, so a photo is not made
    /// twice at once; the photo on screen never waits for them (it may run on a pool, where a
    /// wait can hang the pool, LESSONS 2026-11-06).
    making: Mutex<Vec<SourceKey>>,
}

/// A source marked as being made, until dropped.
struct Making<'a> {
    cache: &'a PreviewCache,
    key: SourceKey,
}

impl Drop for Making<'_> {
    fn drop(&mut self) {
        let mut making = self
            .cache
            .making
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(i) = making.iter().position(|k| *k == self.key) {
            making.swap_remove(i);
        }
    }
}

/// What asking for a RAW's kept develop came to.
enum Prep {
    /// Not a RAW after all (a JPEG renamed).
    NotRaw,
    /// The caller no longer wanted it when the develop's turn came.
    NotWanted,
    Made(Arc<Prepared>),
}

/// A kept RAW develop and the profile of what it finishes to.
struct Prepared {
    raw: crate::raw_finish::Prepared,
    icc: Option<Vec<u8>>,
}

/// A preview photo cropped and resized, and what it was made from: a slider's drag changes
/// only the look, so each step reuses it and skips the resize (FR-079, OPTIMIZATION §5).
struct Resized {
    source: Arc<RgbImage>,
    crop: CropRect,
    size: (u32, u32),
    straighten: u32,
    linear_light: bool,
    photo: Arc<RgbImage>,
}

impl Resized {
    fn fits(&self, source: &Arc<RgbImage>, p: &RenderPlan) -> bool {
        Arc::ptr_eq(&self.source, source)
            && self.crop == p.crop
            && self.size == (p.out_width, p.out_height)
            && self.straighten == p.straighten.to_bits()
            && self.linear_light == p.linear_light
    }
}

impl PreviewCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// What the cache may keep now.
    pub fn budget(&self) -> Budget {
        self.budget
            .lock()
            .map_or_else(|_| Budget::default(), |b| *b)
    }

    /// A new memory budget from the Settings window (FR-086): what is over it goes at once,
    /// the oldest first.
    pub fn set_budget(&self, budget: Budget) {
        if let Ok(mut b) = self.budget.lock() {
            *b = budget;
        }
        shrink_to(&self.entries, |i| i.data.len(), budget.sources);
        shrink_to(
            &self.prepared,
            |p| p.raw.bytes() + p.icc.as_ref().map_or(0, Vec::len),
            budget.prepared,
        );
        shrink_to(&self.full, |f| f.image.data.len(), budget.full);
    }

    /// Follows the machine's memory (FR-091, ADR-0041), from the share still available:
    /// while it is short, the preview keeps a tenth of `full`, the user's budget, so it cannot
    /// fill up again until memory comes back; then `full` again. Called by the watcher every
    /// few seconds; returns the bytes let go (audit 2026-11-06: one trim, then a refill to
    /// the whole budget).
    pub fn follow_memory(&self, full: Budget, available_percent: Option<u8>) -> usize {
        let short = short_of_memory(available_percent);
        let wanted = if short { full.lowered() } else { full };
        if self.budget() == wanted {
            return 0;
        }
        let before = self.usage().total();
        self.set_budget(wanted);
        if short && let Ok(mut tiles) = self.tiles.lock() {
            tiles.clear();
        }
        if short && let Ok(mut small) = self.small.lock() {
            small.clear();
        }
        before.saturating_sub(self.usage().total())
    }

    /// The bytes held now, by kind (FR-086).
    pub fn usage(&self) -> Usage {
        Usage {
            sources: self
                .entries
                .lock()
                .map_or(0, |e| e.iter().map(|(_, i)| i.data.len()).sum()),
            prepared: self.prepared.lock().map_or(0, |e| {
                e.iter()
                    .map(|(_, p)| p.raw.bytes() + p.icc.as_ref().map_or(0, Vec::len))
                    .sum()
            }),
            full: self
                .full
                .lock()
                .map_or(0, |e| e.iter().map(|(_, f)| f.image.data.len()).sum()),
            others: self
                .small
                .lock()
                .map_or(0, |e| e.iter().map(|(_, i)| i.data.len()).sum::<usize>())
                + self
                    .tiles
                    .lock()
                    .map_or(0, |e| e.iter().map(|(_, i)| i.data.len()).sum::<usize>())
                + self
                    .resized
                    .lock()
                    .map_or(0, |r| r.as_ref().map_or(0, |r| r.photo.data.len())),
        }
    }

    /// A cache keeping up to `budget` in memory and its standard previews in `disk`, the
    /// thumbnail cache's folder (FR-091).
    pub fn with(budget: Budget, disk: Option<std::path::PathBuf>) -> Self {
        Self {
            budget: Mutex::new(budget),
            disk,
            ..Self::default()
        }
    }

    /// Forgets every cached image, e.g. when another project replaces the library and
    /// photo ids start again from 0.
    pub fn clear(&self) {
        // First, so anything asked before this is refused under the locks below.
        self.epoch.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
        if let Ok(mut full) = self.full.lock() {
            full.clear();
        }
        if let Ok(mut small) = self.small.lock() {
            small.clear();
        }
        if let Ok(mut tiles) = self.tiles.lock() {
            tiles.clear();
        }
        if let Ok(mut resized) = self.resized.lock() {
            *resized = None;
        }
        if let Ok(mut prepared) = self.prepared.lock() {
            prepared.clear();
        }
    }

    /// A small upright source for a sheet in the collage grid (FR-035): the photo's thumbnail
    /// (`thumbnail` gives its T1 JPEG, upright by EXIF), turned by the user's quarter turns;
    /// without one, the preview source made small. Kept for `SMALL_CAPACITY` photos, so a
    /// change of setting renders the grid again without decoding a single photo.
    fn small_source(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        thumbnail: &Thumbnail<'_>,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let key = source_key(record, raw);
        if let Some(image) = take_cached(&self.small, key) {
            return Ok(image);
        }
        let image = match thumbnail(record) {
            Some(bytes) => {
                let decoded = decode_image(codec, &bytes)?;
                let turns = crate::image::compose_orientation(1, record.edits.rotation);
                apply_orientation(decoded.image, turns)
            }
            None => fit_long_edge(self.source(record, raw, codec)?.as_ref(), SMALL_LONG_EDGE)?,
        };
        let image = Arc::new(image);
        if let Ok(mut small) = self.small.lock()
            && self.epoch.load(Ordering::SeqCst) == epoch
        {
            if small.len() >= SMALL_CAPACITY {
                small.remove(0);
            }
            small.push((key, Arc::clone(&image)));
        }
        Ok(image)
    }

    /// The kept develop of a RAW (FR-088), made once per photo and kept as `keep` says.
    fn prepared(
        &self,
        record: &PhotoRecord,
        keep: Keep,
        epoch: u64,
        wanted: &(dyn Fn() -> bool + Sync),
    ) -> Result<Prep, ThumbError> {
        if let Ok(mut kept) = self.prepared.lock()
            && let Some(i) = kept.iter().position(|(id, _)| *id == record.id)
        {
            let found = Arc::clone(&kept[i].1);
            if keep == Keep::Shown {
                let entry = kept.remove(i);
                kept.push(entry);
            }
            return Ok(Prep::Made(found));
        }
        let bytes = std::fs::read(&record.path)?;
        let Some(kind) = crate::raw::kind(&bytes[..bytes.len().min(crate::raw::HEAD_BYTES)]) else {
            return Ok(Prep::NotRaw);
        };
        let Some((raw, icc)) = crate::raw::prepare_while(&bytes, kind, wanted)? else {
            return Ok(Prep::NotWanted);
        };
        let prepared = Arc::new(Prepared { raw, icc });
        keep_bounded(
            &self.prepared,
            record.id,
            Arc::clone(&prepared),
            |p| p.raw.bytes() + p.icc.as_ref().map_or(0, Vec::len),
            self.budget().prepared,
            keep,
            || self.epoch.load(Ordering::SeqCst) == epoch,
        );
        Ok(Prep::Made(prepared))
    }

    /// Where `record`'s standard preview for `key` is kept on disk (FR-091), or `None` without
    /// a disk cache, or for a RAW with linear sliders set (each step of one is another image,
    /// made from the kept develop instead).
    fn standard_path(&self, record: &PhotoRecord, key: SourceKey) -> Option<std::path::PathBuf> {
        let dir = self.disk.as_ref()?;
        if key.3 != RawSliders::default() {
            return None;
        }
        let look = match key.2 {
            crate::raw::RawLook::Neutral => "neutral",
            crate::raw::RawLook::Camera => "camera",
        };
        let name = crate::thumbs::content_key(record);
        Some(dir.join(&name[..2]).join(format!(
            "{name}.t2v{STANDARD_VERSION}-o{}-{look}.jpg",
            key.1
        )))
    }

    /// The standard preview at `path`, read and decoded, when there is one.
    fn read_standard(
        path: Option<&std::path::Path>,
        codec: &dyn JpegCodec,
    ) -> Option<Arc<RgbImage>> {
        let path = path?;
        let bytes = std::fs::read(path).ok()?;
        let decoded = decode_image(codec, &bytes).ok()?;
        crate::cache_limit::touch(path);
        Some(Arc::new(decoded.image))
    }

    /// Marks `key` as being made; `None` when another thread already makes it.
    fn begin(&self, key: SourceKey) -> Option<Making<'_>> {
        let mut making = self
            .making
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if making.contains(&key) {
            return None;
        }
        making.push(key);
        Some(Making { cache: self, key })
    }

    /// The 2048 px upright sRGB source of `record`: from memory, from its standard preview on
    /// disk, or made from the file, in that order (FR-091), and kept as `keep` says.
    ///
    /// `None` only for background work (`keep` not `Shown`): when another thread is making the
    /// same source, or when `wanted` says no once a RAW's develop turn comes. The photo on
    /// screen never waits for another thread's make; when its develop turn comes and the
    /// standard preview has appeared meanwhile, it reads that instead of developing again.
    fn source_kept(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        codec: &dyn JpegCodec,
        keep: Keep,
        wanted: &(dyn Fn() -> bool + Sync),
    ) -> Result<Option<Arc<RgbImage>>, ThumbError> {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let key = source_key(record, raw);
        let cached = if keep == Keep::Shown {
            take_cached(&self.entries, key)
        } else {
            peek_cached(&self.entries, key)
        };
        if let Some(image) = cached {
            return Ok(Some(image));
        }
        let standard = self.standard_path(record, key);
        if let Some(image) = Self::read_standard(standard.as_deref(), codec) {
            self.keep_source(key, &image, keep, epoch);
            return Ok(Some(image));
        }
        let background = keep != Keep::Shown;
        let making = self.begin(key);
        if background && making.is_none() {
            return Ok(None);
        }
        let not_on_disk = || standard.as_ref().is_none_or(|p| !p.exists());
        let check = || wanted() && not_on_disk();
        let mut made = self.make_source(record, raw, key, codec, keep, epoch, &check)?;
        if made.is_none() {
            if let Some(image) = Self::read_standard(standard.as_deref(), codec) {
                self.keep_source(key, &image, keep, epoch);
                return Ok(Some(image));
            }
            if background {
                return Ok(None);
            }
            // The file appeared but could not be read: make it after all.
            made = self.make_source(record, raw, key, codec, keep, epoch, &|| true)?;
        }
        let Some(mut image) = made else {
            return Err(crate::codec::CodecError::Decode("the RAW develop failed".into()).into());
        };
        image.data.shrink_to_fit();
        let image = Arc::new(image);
        if let Some(path) = standard {
            if keep == Keep::Shown {
                write_standard_later(path, Arc::clone(&image));
            } else {
                write_standard(&path, &image, codec);
            }
        }
        drop(making);
        self.keep_source(key, &image, keep, epoch);
        Ok(Some(image))
    }

    /// The source of the photo on screen (or another view the user waits for): kept as the
    /// newest.
    fn source(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        self.source_kept(record, raw, codec, Keep::Shown, &|| true)?
            .ok_or_else(|| {
                crate::codec::CodecError::Decode("the preview was not made".into()).into()
            })
    }

    /// Decodes or develops `record` into its 2048 px source; `None` when `wanted` said no at
    /// a RAW's develop turn.
    #[allow(clippy::too_many_arguments)]
    fn make_source(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        key: SourceKey,
        codec: &dyn JpegCodec,
        keep: Keep,
        epoch: u64,
        wanted: &(dyn Fn() -> bool + Sync),
    ) -> Result<Option<RgbImage>, ThumbError> {
        let prepared = if crate::raw::is_raw_name(&record.file_name) {
            self.prepared(record, keep, epoch, wanted)?
        } else {
            Prep::NotRaw
        };
        // A RAW: its kept develop finished with the sliders made in linear light, so a step of
        // one costs a finish at 2048 px, not a develop (FR-088).
        let prepared = match prepared {
            Prep::NotWanted => return Ok(None),
            Prep::Made(prepared) => prepared,
            Prep::NotRaw => {
                return self
                    .decode_source(record, raw, key, codec, keep, epoch)
                    .map(Some);
            }
        };
        let finished = prepared
            .raw
            .finish(key.2, &raw.adjust())
            .ok_or(crate::render::RenderError::BadCrop)?;
        let mut image = fit_long_edge(&apply_orientation(finished, key.1), T2_LONG_EDGE)?;
        crate::colour::to_srgb(&mut image, prepared.icc.as_deref());
        Ok(Some(image))
    }

    /// Any file but a RAW decoded into its 2048 px source; its full image kept for zoom when
    /// it is the photo on screen.
    fn decode_source(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        key: SourceKey,
        codec: &dyn JpegCodec,
        keep: Keep,
        epoch: u64,
    ) -> Result<RgbImage, ThumbError> {
        // Zoomed before: the full image is already here.
        let full = if let Some(full) = self.take_full(key) {
            full
        } else {
            let decoded = decode_upright(record, key, raw, codec)?;
            let full = Full {
                image: Arc::new(decoded.image),
                icc: decoded.icc.map(Arc::from),
            };
            // Kept for zoom, so its first sharp region does not decode it again; a prefetch
            // keeps none, so it never pushes out the zoom of the photo on screen.
            if keep == Keep::Shown {
                self.keep_full(key, full.clone(), epoch);
            }
            full
        };
        let mut image = fit_long_edge(&full.image, T2_LONG_EDGE)?;
        // Every view of the photo is made from this sRGB copy (FR-072).
        crate::colour::to_srgb(&mut image, full.icc.as_deref());
        Ok(image)
    }

    /// Makes `record`'s standard preview on disk if it has none, keeping nothing in memory
    /// (FR-091); `true` when one was made. A RAW with linear sliders set has none to make; a
    /// photo another thread is making, or one `wanted` drops at its develop turn, is skipped.
    pub fn ensure_standard(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        codec: &dyn JpegCodec,
        wanted: &(dyn Fn() -> bool + Sync),
    ) -> Result<bool, ThumbError> {
        let raw = split_look(record, settings).0;
        let Some(path) = self.standard_path(record, source_key(record, raw)) else {
            return Ok(false);
        };
        if path.exists() || peek_cached(&self.entries, source_key(record, raw)).is_some() {
            return Ok(false);
        }
        Ok(self
            .source_kept(record, raw, codec, Keep::No, wanted)?
            .is_some())
    }

    /// Loads `record`'s source into memory before it is clicked (FR-091): from its standard
    /// preview when there is one. Kept just below the newest, so it never pushes out the photo
    /// on screen; skipped when another thread makes it, or `wanted` drops it at its develop turn.
    pub fn prefetch(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        codec: &dyn JpegCodec,
        wanted: &(dyn Fn() -> bool + Sync),
    ) -> Result<(), ThumbError> {
        self.source_kept(
            record,
            split_look(record, settings).0,
            codec,
            Keep::Behind,
            wanted,
        )
        .map(|_| ())
    }

    /// The upright, uncropped T2 image as JPEG: the background of the crop editor. With
    /// `sheet`, as the photo sits in its collage cell, its own turns included (FR-035).
    pub fn source_jpeg(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        sheet: bool,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let mut source = self.source(record, split_look(record, settings).0, codec)?;
        if sheet {
            source = turned_in_cell(source, record);
        }
        Ok(codec.encode(&source, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }

    /// The upright original at full resolution, for zoom.
    fn full_source(
        &self,
        record: &PhotoRecord,
        raw: RawSliders,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let key = source_key(record, raw);
        let cached = match self.take_full(key) {
            Some(Full { image, icc: None }) => return Ok(image),
            // Out of the cache while it converts, so the pixels are converted in place.
            Some(_) => self.remove_full(key),
            None => None,
        };
        let image = match cached {
            Some(Full { image, icc: None }) => return Ok(image),
            // Kept by the preview, still in the photo's profile: converted now, once.
            Some(Full {
                image,
                icc: Some(icc),
            }) => {
                let mut image = Arc::unwrap_or_clone(image);
                crate::colour::to_srgb(&mut image, Some(&icc));
                image
            }
            // Since v0.11.0 the photo on screen often comes from its standard preview, so zoom
            // decodes here; `decode_upright` turns it as the preview's own decode does.
            None => crate::colour::screen(decode_upright(record, key, raw, codec)?),
        };
        let image = Arc::new(image);
        self.keep_full(
            key,
            Full {
                image: Arc::clone(&image),
                icc: None,
            },
            epoch,
        );
        Ok(image)
    }

    /// The cached full image of `key`, marked most recently used.
    fn take_full(&self, key: SourceKey) -> Option<Full> {
        let mut full = self.full.lock().ok()?;
        let pos = full.iter().position(|(k, _)| *k == key)?;
        let entry = full.remove(pos);
        let found = entry.1.clone();
        full.push(entry);
        Some(found)
    }

    fn remove_full(&self, key: SourceKey) -> Option<Full> {
        let mut full = self.full.lock().ok()?;
        let pos = full.iter().position(|(k, _)| *k == key)?;
        Some(full.remove(pos).1)
    }

    /// Keeps a full-resolution image of the photo on screen within `budget.full`, the newest
    /// always, unless the cache was cleared since `epoch`.
    fn keep_full(&self, key: SourceKey, image: Full, epoch: u64) {
        keep_bounded(
            &self.full,
            key,
            image,
            |f| f.image.data.len(),
            self.budget().full,
            Keep::Shown,
            || self.epoch.load(Ordering::SeqCst) == epoch,
        );
    }

    /// Keeps a 2048 px source within `budget.sources` as `keep` says, unless the cache was
    /// cleared since `epoch`.
    fn keep_source(&self, key: SourceKey, image: &Arc<RgbImage>, keep: Keep, epoch: u64) {
        keep_bounded(
            &self.entries,
            key,
            Arc::clone(image),
            |i| i.data.len(),
            self.budget().sources,
            keep,
            || self.epoch.load(Ordering::SeqCst) == epoch,
        );
    }

    /// Renders `record` from `source` with the current settings, downsized to
    /// `max_long_edge` or, with `None`, at full resolution as export does.
    fn render_from(
        source: &RgbImage,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        max_long_edge: Option<u32>,
    ) -> Result<RgbImage, ThumbError> {
        let p = Self::plan_from(source, record, settings, frame, place, max_long_edge);
        Ok(render(source, &p)?)
    }

    /// The plan [`Self::render_from`] renders with.
    fn plan_from(
        source: &RgbImage,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        max_long_edge: Option<u32>,
    ) -> RenderPlan {
        // The preview shows what is exported, for prints or for social media (FR-053).
        let settings = settings.for_target();
        let settings = settings.as_ref();
        let print = settings
            .print_for(source.width, source.height)
            .unwrap_or(crate::PrintSize::ALL[1]);
        let capture = record.stamp_date();
        // A RAW's five linear sliders are in its source already (FR-088): the table gets the rest.
        let (_, table_look) = split_look(record, settings);
        let settings = settings.for_photo(
            record.edits.stamp.as_ref(),
            record.edits.caption.as_deref(),
            Some(&table_look),
        );
        plan(&PlanInput {
            width: source.width,
            height: source.height,
            capture,
            settings: &settings,
            print,
            max_long_edge,
            frame,
            crop: record.edits.crop,
            place,
            straighten: record.edits.straighten,
            ratio: None,
        })
    }

    /// The cropped and resized photo for `p`, kept from the last preview when nothing but
    /// the look or the stamp changed since. Never held while the resize runs.
    fn resized(&self, source: &Arc<RgbImage>, p: &RenderPlan) -> Result<Arc<RgbImage>, ThumbError> {
        if let Ok(kept) = self.resized.lock()
            && let Some(kept) = kept.as_ref().filter(|r| r.fits(source, p))
        {
            return Ok(Arc::clone(&kept.photo));
        }
        let photo = Arc::new(crop_and_resize(source, p)?);
        if let Ok(mut kept) = self.resized.lock() {
            *kept = Some(Resized {
                source: Arc::clone(source),
                crop: p.crop,
                size: (p.out_width, p.out_height),
                straighten: p.straighten.to_bits(),
                linear_light: p.linear_light,
                photo: Arc::clone(&photo),
            });
        }
        Ok(photo)
    }

    /// Where the preview shows the photo itself (255) and where the stamp, the frame, the
    /// border or the fill cover it (0), as a grey PNG the preview's size: the webview tints
    /// only the photo while a colour slider is dragged (ADR-0039). Found without geometry:
    /// the photo goes through the plan once as it is and once inverted; where nothing is
    /// drawn over it, the second is exactly the inverse of the first.
    pub fn render_mask_png(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let source = self.source(record, split_look(record, settings).0, codec)?;
        let mut plain = record.clone();
        plain.edits.look = None;
        let mut settings = settings.clone();
        settings.look = crate::looks::LookSettings::default();
        let p = Self::plan_from(
            &source,
            &plain,
            &settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        );
        let photo = self.resized(&source, &p)?;
        let mut inverted = photo.as_ref().clone();
        inverted.data.iter_mut().for_each(|v| *v = 255 - *v);
        let (a, b) = (finish(photo.as_ref().clone(), &p), finish(inverted, &p));
        let mask: Vec<u8> = a
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .zip(b.data.as_chunks::<3>().0)
            // Untouched photo: the inverted run is exactly its inverse. Anything drawn over
            // it, even partly (a stamp's soft edge), breaks that and stays out of the tint.
            .map(|(x, y)| if *y == x.map(|v| 255 - v) { u8::MAX } else { 0 })
            .collect();
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, a.width, a.height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().map_err(|e| {
            ThumbError::Codec(crate::codec::CodecError::Encode(format!("mask PNG: {e}")))
        })?;
        writer.write_image_data(&mask).map_err(|e| {
            ThumbError::Codec(crate::codec::CodecError::Encode(format!("mask PNG: {e}")))
        })?;
        drop(writer);
        Ok(out)
    }

    /// The colour look's table for `record` as the webview's shader reads it (RGBA8,
    /// [`crate::looks::Table::to_rgba8`]); `None` when its look changes nothing.
    pub fn look_table(record: &PhotoRecord, settings: &RenderSettings) -> Option<Vec<u8>> {
        split_look(record, settings).1.table().map(|t| t.to_rgba8())
    }

    /// The photo through `look`, small and square, for a tile of the Looks card (FR-079):
    /// the centre of its small source (its thumbnail, kept for the collage grid too), then
    /// the whole look through its table. A RAW's tile shows its thumbnail through the table,
    /// linear sliders included (RESULT §34 measures how close that is to its preview).
    pub fn look_tile_jpeg(
        &self,
        record: &PhotoRecord,
        look: &crate::looks::LookSettings,
        thumbnail: &Thumbnail<'_>,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let raw = crate::raw::is_raw_name(&record.path.to_string_lossy())
            || record.facts.sensor.is_some();
        let small = match raw.then(|| self.raw_tile_source(record, codec)).flatten() {
            Some(developed) => developed,
            None => self.small_source(record, RawSliders::default(), thumbnail, codec)?,
        };
        let tile = look_tile(&small, look)?;
        Ok(codec.encode(&tile, EncodeOptions::screen(LOOK_TILE_QUALITY), None)?)
    }

    /// A camera RAW's tile source from our develop, without developing: its standard preview
    /// from memory or from disk, made small; `None` until one exists (the thumbnail is used).
    fn raw_tile_source(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Option<Arc<RgbImage>> {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let key = source_key(record, RawSliders::default());
        if let Some(kept) = take_cached(&self.tiles, key) {
            return Some(kept);
        }
        let source = match take_cached(&self.entries, key) {
            Some(source) => source,
            None => Self::read_standard(self.standard_path(record, key).as_deref(), codec)?,
        };
        let small = Arc::new(fit_long_edge(&source, SMALL_LONG_EDGE).ok()?);
        if let Ok(mut tiles) = self.tiles.lock()
            && self.epoch.load(Ordering::SeqCst) == epoch
        {
            if tiles.len() >= TILE_CAPACITY {
                tiles.remove(0);
            }
            tiles.push((key, Arc::clone(&small)));
        }
        Some(small)
    }

    /// Renders the preview JPEG for `record` with the current settings.
    pub fn render_jpeg(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let image = self.render_rgb(record, settings, frame, place, codec)?;
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }

    /// The photo's colour before its look around the point (`fx`, `fy`) of its preview, as
    /// fractions of the preview's width and height, for the white-balance picker (FR-098, `W`):
    /// the mean of the photo's own pixels in the 5 × 5 around it, or `None` when the point
    /// falls on the border, the frame, the stamp or outside. A RAW's colour carries the warmth
    /// and tint it was developed with (FR-088); the caller adds them back.
    pub fn neutral_sample(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        (fx, fy): (f32, f32),
        codec: &dyn JpegCodec,
    ) -> Result<Option<[f32; 3]>, ThumbError> {
        let source = self.source(record, split_look(record, settings).0, codec)?;
        let mut plain = record.clone();
        plain.edits.look = None;
        let mut settings = settings.clone();
        settings.look = crate::looks::LookSettings::default();
        let p = Self::plan_from(
            &source,
            &plain,
            &settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        );
        let photo = self.resized(&source, &p)?;
        let mut inverted = photo.as_ref().clone();
        inverted.data.iter_mut().for_each(|v| *v = 255 - *v);
        // The same test as the webview's mask: a pixel of the photo is the exact inverse in
        // the inverted run; anything drawn over it is not.
        let (a, b) = (finish(photo.as_ref().clone(), &p), finish(inverted, &p));
        Ok(sample_where_inverse(&a, &b, (fx, fy)))
    }

    /// The preview JPEG with the photo's clipped pixels marked (FR-098, `J`): blown in red,
    /// blocked in blue, the stamp, frame and border drawn as usual over them.
    pub fn render_clipping_jpeg(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let source = self.source(record, split_look(record, settings).0, codec)?;
        let p = Self::plan_from(
            &source,
            record,
            settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        );
        let image =
            crate::render::finish_marked(self.resized(&source, &p)?.as_ref().clone(), &p, true);
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }

    /// The preview's pixels, before the JPEG: what [`Self::render_jpeg`] encodes.
    pub fn render_rgb(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        codec: &dyn JpegCodec,
    ) -> Result<RgbImage, ThumbError> {
        let source = self.source(record, split_look(record, settings).0, codec)?;
        let p = Self::plan_from(
            &source,
            record,
            settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        );
        Ok(finish(self.resized(&source, &p)?.as_ref().clone(), &p))
    }

    /// The histogram of `record`'s preview (FR-098): the cropped, resized photo kept from the
    /// last preview, seen through its look on every [`HISTOGRAM_STEP`]th pixel; the stamp,
    /// frame and border are drawn later and never counted. Within 3 ms (OPTIMIZATION §3).
    pub fn histogram(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        codec: &dyn JpegCodec,
    ) -> Result<crate::histogram::Histogram, ThumbError> {
        let source = self.source(record, split_look(record, settings).0, codec)?;
        let p = Self::plan_from(
            &source,
            record,
            settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        );
        let photo = self.resized(&source, &p)?;
        Ok(crate::histogram::measure(
            &photo,
            p.look.as_deref(),
            HISTOGRAM_STEP,
        ))
    }

    /// A collage sheet's preview JPEG (FR-035): both photos from their cached sources,
    /// the sheet's long edge at most `long_edge`. Each photo comes with its place. With
    /// `thumbnail`, for the grid's small sheets, the photos come from their thumbnails.
    #[allow(clippy::too_many_arguments)]
    pub fn render_sheet_jpeg(
        &self,
        photos: [Option<(&PhotoRecord, Option<&str>)>; 2],
        settings: &RenderSettings,
        long_edge: u32,
        thumbnail: Option<&Thumbnail<'_>>,
        card_font: Option<&ab_glyph::FontArc>,
        clipping: bool,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let mark = if clipping {
            crate::sheet::CellMark::Clipping
        } else {
            crate::sheet::CellMark::Normal
        };
        let image = self.render_sheet_rgb(
            photos, settings, long_edge, thumbnail, card_font, [mark; 2], None, codec,
        )?;
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }

    /// A sheet's pixels, each cell drawn as `marks` asks; the cell `plain`, if any, without
    /// its look (FR-098: the white-balance picker reads the photo before its look).
    #[allow(clippy::too_many_arguments)]
    fn render_sheet_rgb(
        &self,
        photos: [Option<(&PhotoRecord, Option<&str>)>; 2],
        settings: &RenderSettings,
        long_edge: u32,
        thumbnail: Option<&Thumbnail<'_>>,
        card_font: Option<&ab_glyph::FontArc>,
        marks: [crate::sheet::CellMark; 2],
        plain: Option<usize>,
        codec: &dyn JpegCodec,
    ) -> Result<RgbImage, ThumbError> {
        let none = crate::looks::LookSettings::default();
        let mut images = [None, None];
        // Each RAW cell's five linear sliders go to its source, the rest to the table (FR-088).
        let looks = photos.map(|p| p.map(|(record, _)| split_look(record, settings)));
        for ((slot, photo), look) in images.iter_mut().zip(&photos).zip(&looks) {
            if let (Some((record, _)), Some((raw, _))) = (photo, look) {
                let upright = match thumbnail {
                    Some(thumbnail) => self.small_source(record, *raw, thumbnail, codec)?,
                    None => self.source(record, *raw, codec)?,
                };
                *slot = Some(turned_in_cell(upright, record));
            }
        }
        let mut sources = [None, None];
        for (cell, (((slot, image), photo), look)) in sources
            .iter_mut()
            .zip(&images)
            .zip(&photos)
            .zip(&looks)
            .enumerate()
        {
            if let (Some(image), Some((record, place)), Some((_, table_look))) =
                (image, photo, look)
            {
                *slot = Some(crate::sheet::CellSource {
                    image,
                    capture: record.stamp_date(),
                    crop: record.edits.sheet_crop(),
                    straighten: record.edits.sheet_straighten(),
                    place: *place,
                    stamp: record.edits.stamp.as_ref(),
                    caption: record.edits.caption.as_deref(),
                    look: Some(if plain == Some(cell) {
                        &none
                    } else {
                        table_look
                    }),
                    postcard: record.edits.postcard.as_ref(),
                });
            }
        }
        let print = crate::sheet::sheet_print(settings);
        let turned = match (photos[0], &images[0]) {
            (Some((r, _)), Some(image)) => crate::sheet::layout_turned(
                r.edits.sheet_turned,
                r.edits.sheet_turns(),
                (image.width, image.height),
            ),
            _ => false,
        };
        Ok(crate::sheet::render_sheet_with(
            settings,
            print,
            sources,
            turned,
            Some(long_edge),
            card_font,
            marks,
        )?)
    }

    /// The colour before its look around (`fx`, `fy`) of a sheet's preview, as fractions of
    /// the sheet, read only from the photo in `cell` (FR-098, `W` on a sheet): the sheet drawn
    /// twice, that photo plain and then inverted before its stamp; a pixel that is the exact
    /// inverse is the photo's own. The other photo, the card, the paper, the stamp and the
    /// cut line are the same in both runs, so they never count: `None` there.
    #[allow(clippy::too_many_arguments)]
    pub fn neutral_sample_on_sheet(
        &self,
        photos: [Option<(&PhotoRecord, Option<&str>)>; 2],
        settings: &RenderSettings,
        card_font: Option<&ab_glyph::FontArc>,
        cell: usize,
        (fx, fy): (f32, f32),
        codec: &dyn JpegCodec,
    ) -> Result<Option<[f32; 3]>, ThumbError> {
        use crate::sheet::CellMark;
        let mut inverted = [CellMark::Normal; 2];
        if let Some(slot) = inverted.get_mut(cell) {
            *slot = CellMark::Inverted;
        }
        let a = self.render_sheet_rgb(
            photos,
            settings,
            PREVIEW_LONG_EDGE,
            None,
            card_font,
            [CellMark::Normal; 2],
            Some(cell),
            codec,
        )?;
        let b = self.render_sheet_rgb(
            photos,
            settings,
            PREVIEW_LONG_EDGE,
            None,
            card_font,
            inverted,
            Some(cell),
            codec,
        )?;
        Ok(sample_where_inverse(&a, &b, (fx, fy)))
    }

    /// One region of the full-resolution render (FR-057): the pixels export would write,
    /// downsized only when wider than `region.px`.
    pub fn render_region(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        region: Region,
        codec: &dyn JpegCodec,
    ) -> Result<RgbImage, ThumbError> {
        let source = self.full_source(record, split_look(record, settings).0, codec)?;
        let full = Self::render_from(&source, record, settings, frame, place, None)?;
        let (x, y, w, h) = region.rect(full.width, full.height);
        let row = full.width as usize * 3;
        let mut data = Vec::with_capacity(w as usize * h as usize * 3);
        for line in y..y + h {
            let start = line as usize * row + x as usize * 3;
            data.extend_from_slice(&full.data[start..start + w as usize * 3]);
        }
        let part = RgbImage::from_raw(w, h, data).ok_or(crate::render::RenderError::BadCrop)?;
        if w <= region.px {
            return Ok(part);
        }
        // Keep the aspect: the long edge shrinks by the same factor as the width.
        let long = u64::from(part.long_edge()) * u64::from(region.px) / u64::from(w);
        Ok(fit_long_edge(
            &part,
            u32::try_from(long).unwrap_or(region.px).max(1),
        )?)
    }

    /// `render_region` as a screen JPEG.
    pub fn render_region_jpeg(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        place: Option<&str>,
        region: Region,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let image = self.render_region(record, settings, frame, place, region, codec)?;
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }
}

/// `record`'s file decoded at full size and turned upright (its EXIF orientation and the
/// user's turns, `key`): the one way to the full image, for the preview's source and for
/// zoom alike. Two ways, one of which forgot the turn, showed zoomed portraits sideways
/// (LESSONS 2026-11-06, v0.11.2).
fn decode_upright(
    record: &PhotoRecord,
    key: SourceKey,
    raw: RawSliders,
    codec: &dyn JpegCodec,
) -> Result<crate::codec::Decoded, ThumbError> {
    let bytes = std::fs::read(&record.path)?;
    let decoded = decode_photo(codec, &bytes, key.2, &raw.adjust())?;
    Ok(crate::codec::Decoded {
        image: apply_orientation(decoded.image, key.1),
        icc: decoded.icc,
    })
}

/// The upright image turned as the photo sits in its collage cell (FR-035); the cached image
/// itself when the photo has no turns there.
fn turned_in_cell(upright: Arc<RgbImage>, record: &PhotoRecord) -> Arc<RgbImage> {
    match record.edits.sheet_turns() {
        0 => upright,
        turns => Arc::new(apply_orientation(
            upright.as_ref().clone(),
            crate::image::compose_orientation(1, turns),
        )),
    }
}

/// The mean colour of `a`'s pixels in the 5 × 5 around (`fx`, `fy`), fractions of its size,
/// counting only those whose pixel in `b` is their exact inverse: the photo's own, in a render
/// where only the photo was inverted (FR-098, the white-balance picker). `None` when the point
/// itself is not one of them.
fn sample_where_inverse(a: &RgbImage, b: &RgbImage, (fx, fy): (f32, f32)) -> Option<[f32; 3]> {
    let (w, h) = (a.width as usize, a.height as usize);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let (cx, cy) = (
        (fx.clamp(0.0, 1.0) * w as f32) as usize,
        (fy.clamp(0.0, 1.0) * h as f32) as usize,
    );
    let at = |x: usize, y: usize| -> Option<[u8; 3]> {
        let i = (y * w + x) * 3;
        let (pa, pb) = (a.data.get(i..i + 3)?, b.data.get(i..i + 3)?);
        (pa.iter().zip(pb).all(|(x, y)| *y == 255 - *x)).then(|| [pa[0], pa[1], pa[2]])
    };
    if cx >= w || cy >= h || at(cx, cy).is_none() {
        return None;
    }
    let (mut sum, mut n) = ([0f32; 3], 0f32);
    for y in cy.saturating_sub(2)..(cy + 3).min(h) {
        for x in cx.saturating_sub(2)..(cx + 3).min(w) {
            if let Some(px) = at(x, y) {
                for c in 0..3 {
                    sum[c] += f32::from(px[c]);
                }
                n += 1.0;
            }
        }
    }
    Some(sum.map(|s| s / n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::default_codec;
    use crate::import::ImportedFile;
    use crate::library::Library;
    use crate::metadata::PhotoMeta;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    /// A `width`×`height` JPEG with a gradient, as a library record.
    fn photo(dir: &std::path::Path, width: u32, height: u32) -> TestResult<PhotoRecord> {
        photos(dir, &[(width, height)])?
            .pop()
            .ok_or_else(|| "photo".into())
    }

    /// JPEGs of these sizes with a gradient, as records of one library (ids 0, 1, 2…).
    #[allow(clippy::cast_possible_truncation)]
    fn photos(dir: &std::path::Path, sizes: &[(u32, u32)]) -> TestResult<Vec<PhotoRecord>> {
        let codec = default_codec();
        let mut lib = Library::new();
        let folder = lib.add_folder(dir);
        let mut out = Vec::new();
        for &(width, height) in sizes {
            let mut image = RgbImage::new(width, height);
            for (i, px) in image.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                let (x, y) = (i as u32 % width, i as u32 / width);
                *px = [(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8];
            }
            let path = dir.join(format!("{width}x{height}.jpg"));
            let bytes = codec.encode(&image, EncodeOptions::print(95), None)?;
            std::fs::write(&path, &bytes)?;
            let file = ImportedFile {
                path,
                size: bytes.len() as u64,
                mtime: 0,
            };
            let id = lib
                .push(folder, &file, &PhotoMeta::default())
                .ok_or("photo")?;
            out.push(lib.record(id).ok_or("record")?);
        }
        Ok(out)
    }

    /// A codec that stops in `decode` until told to go, so a test can act while a photo is
    /// being made.
    struct Held {
        started: Mutex<std::sync::mpsc::Sender<()>>,
        go: Mutex<std::sync::mpsc::Receiver<()>>,
    }

    impl JpegCodec for Held {
        fn dimensions(&self, bytes: &[u8]) -> Result<(u32, u32), crate::codec::CodecError> {
            default_codec().dimensions(bytes)
        }
        fn decode(&self, bytes: &[u8]) -> Result<crate::codec::Decoded, crate::codec::CodecError> {
            if let Ok(started) = self.started.lock() {
                let _ = started.send(());
            }
            if let Ok(go) = self.go.lock() {
                let _ = go.recv();
            }
            default_codec().decode(bytes)
        }
        fn encode(
            &self,
            image: &RgbImage,
            options: EncodeOptions,
            icc: Option<&[u8]>,
        ) -> Result<Vec<u8>, crate::codec::CodecError> {
            default_codec().encode(image, options, icc)
        }
    }

    /// Audit 2026-11-06: a photo still being made when the project changes (`clear`) is not
    /// kept: its id belongs to another photo in the new library.
    #[test]
    fn fr_091_a_photo_made_across_a_clear_is_not_kept() -> TestResult {
        let dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 600, 400)?;
        let (started_tx, started) = std::sync::mpsc::channel();
        let (go, go_rx) = std::sync::mpsc::channel();
        let codec = Arc::new(Held {
            started: Mutex::new(started_tx),
            go: Mutex::new(go_rx),
        });
        let cache = Arc::new(PreviewCache::new());
        let making = {
            let (cache, codec, record) = (Arc::clone(&cache), Arc::clone(&codec), record.clone());
            std::thread::spawn(move || {
                cache
                    .render_rgb(
                        &record,
                        &RenderSettings::default(),
                        None,
                        None,
                        codec.as_ref(),
                    )
                    .is_ok()
            })
        };
        started.recv_timeout(std::time::Duration::from_secs(10))?;
        cache.clear();
        go.send(())?;
        assert!(making.join().unwrap_or(false));
        assert!(
            cache.entries.lock().map_err(|_| "lock")?.is_empty(),
            "source kept"
        );
        assert!(
            cache.full.lock().map_err(|_| "lock")?.is_empty(),
            "full image kept"
        );
        Ok(())
    }

    /// Audit 2026-11-06: prefetching the neighbours never pushes out the photo on screen,
    /// even when the budget holds only two, and keeps no full image for zoom.
    #[test]
    fn fr_091_a_prefetch_never_pushes_out_the_photo_on_screen() -> TestResult {
        let dir = tempfile::tempdir()?;
        let records = photos(dir.path(), &[(300, 200), (301, 200), (302, 200)])?;
        let codec = default_codec();
        let settings = RenderSettings::default();
        let budget = Budget {
            sources: 2 * 302 * 200 * 3,
            ..Budget::default()
        };
        let cache = PreviewCache::with(budget, None);
        cache.render_rgb(&records[0], &settings, None, None, codec.as_ref())?;
        for record in &records[1..] {
            cache.prefetch(record, &settings, codec.as_ref(), &|| true)?;
        }
        let kept: Vec<PhotoId> = cache
            .entries
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| k.0)
            .collect();
        assert!(
            kept.contains(&records[0].id),
            "the photo on screen went: {kept:?}"
        );
        assert_eq!(kept.last(), Some(&records[0].id), "it stays the newest");
        let full: Vec<PhotoId> = cache
            .full
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| k.0)
            .collect();
        assert_eq!(
            full,
            [records[0].id],
            "only the photo on screen keeps its full image"
        );
        Ok(())
    }

    /// Audit 2026-11-06: background work skips a photo another thread is making, so it is
    /// not made twice at once.
    #[test]
    fn fr_091_background_work_skips_a_photo_being_made() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache_dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 600, 400)?;
        let codec = default_codec();
        let settings = RenderSettings::default();
        let cache = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        let key = source_key(&record, RawSliders::default());
        let held = cache.begin(key).ok_or("free")?;
        assert!(!cache.ensure_standard(&record, &settings, codec.as_ref(), &|| true)?);
        cache.prefetch(&record, &settings, codec.as_ref(), &|| true)?;
        assert!(cache.entries.lock().map_err(|_| "lock")?.is_empty());
        drop(held);
        assert!(cache.ensure_standard(&record, &settings, codec.as_ref(), &|| true)?);
        Ok(())
    }

    /// Audit 2026-11-06: writers of one standard preview at once never share a `.part`; the
    /// file left is whole and no `.part` stays behind.
    #[test]
    fn fr_091_standard_previews_written_at_once_stay_whole() -> TestResult {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("ab").join("x.t2v2-o1-neutral.jpg");
        let codec = default_codec();
        let image = crate::codec::decode_image(
            codec.as_ref(),
            &std::fs::read(&photo(dir.path(), 2048, 1365)?.path)?,
        )?
        .image;
        let written = std::thread::scope(|scope| {
            let writers: Vec<_> = (0..32)
                .map(|_| scope.spawn(|| write_standard(&path, &image, codec.as_ref())))
                .collect();
            writers
                .into_iter()
                .filter_map(|w| w.join().ok())
                .filter(|ok| *ok)
                .count()
        });
        assert_eq!(written, 32, "a writer lost its file");
        let back = crate::codec::decode_image(codec.as_ref(), &std::fs::read(&path)?)?.image;
        assert_eq!((back.width, back.height), (2048, 1365));
        let left: Vec<_> = std::fs::read_dir(dir.path().join("ab"))?
            .filter_map(Result::ok)
            .map(|e| e.file_name())
            .collect();
        assert_eq!(left.len(), 1, "{left:?}");
        Ok(())
    }

    /// Audit 2026-11-06: the photo on screen gets its standard preview too, written off its
    /// own path by the writer thread.
    #[test]
    fn fr_091_the_photo_on_screen_writes_its_standard_preview_later() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache_dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 640, 480)?;
        let codec = default_codec();
        let cache = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        cache.render_rgb(
            &record,
            &RenderSettings::default(),
            None,
            None,
            codec.as_ref(),
        )?;
        let path = cache
            .standard_path(&record, source_key(&record, RawSliders::default()))
            .ok_or("path")?;
        let start = std::time::Instant::now();
        while !path.exists() {
            assert!(start.elapsed().as_secs() < 10, "never written");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok(())
    }

    /// Audit 2026-11-06: standard previews carry their own version: one written under
    /// v0.11.0's name is not read, and is made again.
    #[test]
    fn fr_091_standard_previews_of_an_older_version_are_made_again() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache_dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 600, 400)?;
        let codec = default_codec();
        let settings = RenderSettings::default();
        let cache = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        let key = source_key(&record, RawSliders::default());
        let path = cache.standard_path(&record, key).ok_or("path")?;
        let name = path.file_name().and_then(|n| n.to_str()).ok_or("name")?;
        assert!(name.contains(&format!(".t2v{STANDARD_VERSION}-")), "{name}");
        let old = path.with_file_name(name.replace(&format!(".t2v{STANDARD_VERSION}-"), ".t2-"));
        std::fs::create_dir_all(old.parent().ok_or("dir")?)?;
        std::fs::copy(&record.path, &old)?;
        assert!(cache.ensure_standard(&record, &settings, codec.as_ref(), &|| true)?);
        assert!(path.exists());
        Ok(())
    }

    /// ADR-0039: the mask is the photo itself, 255, and 0 where the 3 mm border or the date
    /// stamp cover it; it has the preview's size.
    #[test]
    fn fr_079_the_mask_marks_the_photo_not_the_border_or_the_stamp() -> TestResult {
        let dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 3000, 2000)?;
        let codec = default_codec();
        let settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        let cache = PreviewCache::new();
        let png = cache.render_mask_png(&record, &settings, None, None, codec.as_ref())?;
        let jpeg = cache.render_jpeg(&record, &settings, None, None, codec.as_ref())?;
        let preview = crate::codec::decode_image(codec.as_ref(), &jpeg)?.image;
        let mut reader = png::Decoder::new(std::io::Cursor::new(png)).read_info()?;
        let mut mask = vec![0; reader.output_buffer_size().ok_or("size")?];
        let info = reader.next_frame(&mut mask)?;
        assert_eq!((info.width, info.height), (preview.width, preview.height));
        let at = |x: u32, y: u32| mask[(y * info.width + x) as usize];
        assert_eq!(at(1, 1), 0, "the border");
        assert_eq!(at(info.width / 2, info.height / 2), 255, "the photo");
        // The stamp sits at the bottom right inside the border: some of that corner is 0.
        let corner = (info.height * 3 / 4..info.height - 20)
            .flat_map(|y| (info.width * 3 / 4..info.width - 20).map(move |x| (x, y)))
            .filter(|&(x, y)| at(x, y) == 0)
            .count();
        assert!(corner > 50, "the stamp is left out: {corner}");
        Ok(())
    }

    /// FR-098: the histogram counts the photo only: the same with the stamp drawn as without
    /// it, the border's colour never counted (the photo inside it is smaller, so fewer samples,
    /// none of them white border), and a look moves it.
    #[test]
    fn fr_098_the_histogram_counts_the_photo_not_the_stamp_or_border() -> TestResult {
        let dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 3000, 2000)?;
        let codec = default_codec();
        let mut plain = RenderSettings::default();
        plain.stamp.enabled = false;
        let mut stamped = plain.clone();
        stamped.stamp.enabled = true;
        let cache = PreviewCache::new();
        let a = cache.histogram(&record, &plain, None, None, codec.as_ref())?;
        let b = cache.histogram(&record, &stamped, None, None, codec.as_ref())?;
        assert!(a.samples > 0);
        assert_eq!(a, b, "the stamp is never counted");
        let mut bordered = plain.clone();
        bordered.border = true;
        let c = cache.histogram(&record, &bordered, None, None, codec.as_ref())?;
        assert!(
            c.samples < a.samples,
            "the photo is smaller inside its border"
        );
        let mut warm = plain.clone();
        warm.look.adjust.brightness = 60;
        assert_ne!(
            cache.histogram(&record, &warm, None, None, codec.as_ref())?,
            a,
            "the look is counted"
        );
        Ok(())
    }

    /// A flat-coloured JPEG per colour, as records of one library (ids 0, 1…).
    fn flat_photos(dir: &std::path::Path, colours: &[[u8; 3]]) -> TestResult<Vec<PhotoRecord>> {
        let codec = default_codec();
        let mut lib = Library::new();
        let folder = lib.add_folder(dir);
        let mut out = Vec::new();
        for (i, colour) in colours.iter().enumerate() {
            let mut image = RgbImage::new(600, 400);
            for px in image.data.as_chunks_mut::<3>().0 {
                *px = *colour;
            }
            let path = dir.join(format!("flat{i}.jpg"));
            let bytes = codec.encode(&image, EncodeOptions::print(100), None)?;
            std::fs::write(&path, &bytes)?;
            let file = ImportedFile {
                path,
                size: bytes.len() as u64,
                mtime: 0,
            };
            let id = lib
                .push(folder, &file, &PhotoMeta::default())
                .ok_or("photo")?;
            out.push(lib.record(id).ok_or("record")?);
        }
        Ok(out)
    }

    /// FR-098 (`J` on a sheet): only the photos' clipped pixels turn red or blue; the paper
    /// round them is left alone.
    #[test]
    fn fr_098_a_sheet_marks_clipping_only_in_the_photos() -> TestResult {
        use crate::sheet::CellMark;
        let dir = tempfile::tempdir()?;
        let records = photos(dir.path(), &[(600, 400), (601, 400)])?;
        let codec = default_codec();
        let settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        let cache = PreviewCache::new();
        let pair = [Some((&records[0], None)), Some((&records[1], None))];
        let draw = |mark| {
            cache.render_sheet_rgb(
                pair,
                &settings,
                800,
                None,
                None,
                [mark; 2],
                None,
                codec.as_ref(),
            )
        };
        let (plain, marked) = (draw(CellMark::Normal)?, draw(CellMark::Clipping)?);
        // The paper's own colour, from the sheet's corner (an off-white, not 255).
        let paper = *plain.data.as_chunks::<3>().0.first().ok_or("empty")?;
        let (mut changed, mut kept) = (0, 0);
        for (a, b) in plain
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .zip(marked.data.as_chunks::<3>().0)
        {
            if a == b {
                kept += usize::from(*a == paper);
            } else {
                changed += 1;
                assert!(
                    *b == [255, 0, 0] || *b == [0, 64, 255],
                    "only marks change: {a:?} -> {b:?}"
                );
            }
        }
        assert!(changed > 0, "the gradients have clipped pixels");
        assert!(kept > 1000, "the paper is never marked: {kept}");
        Ok(())
    }

    /// FR-098 (`W` on a sheet): the picker reads the picked photo only, before its look; the
    /// other photo and the border give nothing.
    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn fr_098_a_sheet_pick_reads_only_the_picked_photo() -> TestResult {
        let dir = tempfile::tempdir()?;
        let (warm, grey) = ([150, 128, 110], [100, 100, 100]);
        let mut records = flat_photos(dir.path(), &[warm, grey])?;
        records[0].edits.look = Some(crate::looks::LookSettings {
            adjust: crate::looks::Adjust {
                brightness: 50,
                ..crate::looks::Adjust::default()
            },
            ..crate::looks::LookSettings::default()
        });
        let codec = default_codec();
        let settings = RenderSettings {
            border: true,
            ..RenderSettings::default()
        };
        let cache = PreviewCache::new();
        let pair = [Some((&records[0], None)), Some((&records[1], None))];
        let near =
            |s: [f32; 3], c: [u8; 3]| s.iter().zip(c).all(|(s, c)| (s - f32::from(c)).abs() < 4.0);
        for (cell, colour, other) in [(0, warm, grey), (1, grey, warm)] {
            let mut found = 0;
            for i in 1..20 {
                for j in 1..20 {
                    let at = (i as f32 / 20.0, j as f32 / 20.0);
                    let sample = cache.neutral_sample_on_sheet(
                        pair,
                        &settings,
                        None,
                        cell,
                        at,
                        codec.as_ref(),
                    )?;
                    if let Some(s) = sample {
                        assert!(near(s, colour), "cell {cell} read {s:?}, not {colour:?}");
                        assert!(!near(s, other), "never the other photo");
                        found += 1;
                    }
                }
            }
            assert!(found > 20, "cell {cell} is found: {found}");
            let edge = cache.neutral_sample_on_sheet(
                pair,
                &settings,
                None,
                cell,
                (0.001, 0.001),
                codec.as_ref(),
            )?;
            assert_eq!(edge, None, "the border is no photo");
        }
        Ok(())
    }

    /// FR-079: a look tile is the centre of the photo, square and small, through the look;
    /// Original leaves it as the photo is, a warm look makes it warmer.
    #[test]
    fn fr_079_a_look_tile_is_the_photo_through_that_look() -> TestResult {
        let dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 600, 400)?;
        let codec = default_codec();
        let cache = PreviewCache::default();
        let no_thumb = |_: &PhotoRecord| None;
        let mean = |jpeg: &[u8]| -> TestResult<[f64; 3]> {
            let img = decode_image(codec.as_ref(), jpeg)?.image;
            assert_eq!((img.width, img.height), (LOOK_TILE_EDGE, LOOK_TILE_EDGE));
            let mut sum = [0f64; 3];
            for p in img.data.as_chunks::<3>().0 {
                for c in 0..3 {
                    sum[c] += f64::from(p[c]);
                }
            }
            let n = f64::from(img.width * img.height);
            Ok(sum.map(|v| v / n))
        };
        let plain = mean(&cache.look_tile_jpeg(
            &record,
            &crate::looks::LookSettings::default(),
            &no_thumb,
            codec.as_ref(),
        )?)?;
        let mut warm = crate::looks::LookSettings::default();
        warm.adjust.warmth = 80;
        let warm = mean(&cache.look_tile_jpeg(&record, &warm, &no_thumb, codec.as_ref())?)?;
        assert!(
            warm[0] - warm[2] > plain[0] - plain[2] + 5.0,
            "warmer: {warm:?} against {plain:?}"
        );
        let wide = RgbImage {
            width: 5,
            height: 2,
            data: (0..30).collect(),
        };
        let square = centre_square(&wide);
        assert_eq!((square.width, square.height), (2, 2));
        assert_eq!(
            square.data[..6],
            [3, 4, 5, 6, 7, 8],
            "columns 1 and 2 of row 0"
        );
        Ok(())
    }

    /// ADR-0039: the shader's table is the CPU's, in RGBA8; no table without a look.
    #[test]
    fn fr_079_the_look_table_goes_to_the_webview_as_rgba8() -> TestResult {
        let dir = tempfile::tempdir()?;
        let mut record = photo(dir.path(), 300, 200)?;
        let mut settings = RenderSettings::default();
        assert!(PreviewCache::look_table(&record, &settings).is_none());
        settings.look.adjust.saturation = 40;
        let table = PreviewCache::look_table(&record, &settings).ok_or("a table")?;
        let n = crate::looks::GRID;
        assert_eq!(table.len(), n * n * n * 4);
        // A photo's own look wins: black and white on this one.
        record.edits.look = Some(crate::looks::LookSettings {
            adjust: crate::looks::Adjust {
                mono: crate::looks::Mono::Neutral,
                ..crate::looks::Adjust::default()
            },
            ..crate::looks::LookSettings::default()
        });
        let mono = PreviewCache::look_table(&record, &settings).ok_or("its own")?;
        // Pure red at grid (32, 0, 0): grey, alpha 255.
        let red = &mono[32 * 4..32 * 4 + 4];
        assert!(
            red[0] == red[1] && red[1] == red[2] && red[3] == 255,
            "{red:?}"
        );
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn region_is_the_export_render_cropped() -> TestResult {
        let dir = tempfile::tempdir()?;
        let record = photo(dir.path(), 3000, 2000)?;
        let codec = default_codec();
        let settings = RenderSettings::default();
        let cache = PreviewCache::new();
        let full = PreviewCache::render_from(
            &*cache.full_source(&record, RawSliders::default(), codec.as_ref())?,
            &record,
            &settings,
            None,
            None,
            None,
        )?;
        let region = Region::new(0.25, 0.5, 0.25, 0.25, REGION_MAX_PX).ok_or("region")?;
        let part = cache.render_region(&record, &settings, None, None, region, codec.as_ref())?;
        let (x, y, w, h) = region.rect(full.width, full.height);
        assert_eq!((part.width, part.height), (w, h));
        let row = full.width as usize * 3;
        for line in 0..h as usize {
            let start = (y as usize + line) * row + x as usize * 3;
            let want = &full.data[start..start + w as usize * 3];
            let got = &part.data[line * w as usize * 3..(line + 1) * w as usize * 3];
            assert_eq!(want, got, "row {line}");
        }

        // Wider than the screen asks for: downsized, aspect kept.
        let small = Region::new(0.0, 0.0, 1.0, 1.0, 600).ok_or("region")?;
        let part = cache.render_region(&record, &settings, None, None, small, codec.as_ref())?;
        assert_eq!(part.width, 600);
        let want = f64::from(full.height) * 600.0 / f64::from(full.width);
        assert!((f64::from(part.height) - want).abs() <= 1.0);
        Ok(())
    }

    #[test]
    fn bad_regions_are_refused_or_clamped() -> TestResult {
        assert_eq!(Region::new(f64::NAN, 0.0, 0.5, 0.5, 100), None);
        assert_eq!(Region::new(0.0, 0.0, 0.0, 0.5, 100), None);
        assert_eq!(Region::new(-0.1, 0.0, 0.5, 0.5, 100), None);
        assert_eq!(Region::new(0.75, 0.0, 0.5, 0.5, 100), None);
        assert_eq!(Region::new(0.0, 0.0, 1.0, f64::INFINITY, 100), None);
        let huge = Region::new(0.0, 0.0, 1.0, 1.0, u32::MAX).ok_or("region")?;
        assert_eq!(huge.px, REGION_MAX_PX);
        let zero = Region::new(0.0, 0.0, 1.0, 1.0, 0).ok_or("region")?;
        assert_eq!(zero.px, 1);
        // A sliver at the edge still has one pixel inside the image.
        let edge = Region::new(1.0, 1.0, 1e-9, 1e-9, 10);
        assert_eq!(edge, None);
        let tiny = Region::new(0.999_999, 0.0, 0.000_001, 0.1, 10).ok_or("region")?;
        let (x, _, w, _) = tiny.rect(100, 100);
        assert_eq!((x, w), (99, 1));
        Ok(())
    }

    /// FR-091: once its standard preview is on disk, a photo opens from it in a new session
    /// (here with the file itself gone, so nothing else could answer), with the pixels of a
    /// source made from the file, within the JPEG's loss; the builder keeps nothing in memory.
    #[test]
    fn fr_091_a_standard_preview_on_disk_opens_the_photo() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache_dir = tempfile::tempdir()?;
        let codec = default_codec();
        let record = photo(dir.path(), 3000, 2000)?;
        let settings = RenderSettings::default();
        let fresh =
            PreviewCache::new().render_rgb(&record, &settings, None, None, codec.as_ref())?;

        let builder = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        assert!(builder.ensure_standard(&record, &settings, codec.as_ref(), &|| true)?);
        assert!(!builder.ensure_standard(&record, &settings, codec.as_ref(), &|| true)?);
        assert!(builder.entries.lock().map_err(|_| "lock")?.is_empty());

        std::fs::remove_file(&record.path)?;
        let session = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        let shown = session.render_rgb(&record, &settings, None, None, codec.as_ref())?;
        assert_eq!((shown.width, shown.height), (fresh.width, fresh.height));
        let error: f64 = shown
            .data
            .iter()
            .zip(&fresh.data)
            .map(|(a, b)| f64::from(a.abs_diff(*b)))
            .sum::<f64>()
            / f64::from(u32::try_from(shown.data.len())?);
        // This test pattern is all hard edges, JPEG's worst case (2.4 levels at q90); real
        // photos differ by 1.35 (RESULT §29).
        assert!(error < 3.0, "mean error {error}");
        Ok(())
    }

    /// FR-079: a camera RAW's look tile comes from its standard preview (our develop) once it
    /// exists, never from the thumbnail (the camera's JPEG, RESULT §34); before, the thumbnail.
    #[test]
    fn fr_079_a_raw_tile_comes_from_the_develop_once_there_is_one() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache_dir = tempfile::tempdir()?;
        let codec = default_codec();
        let mut record = photo(dir.path(), 600, 400)?;
        // A RAW by its name; the bytes stay a JPEG, which the decoders read by content.
        let raw_path = record.path.with_extension("rw2");
        std::fs::rename(&record.path, &raw_path)?;
        record.path = raw_path;
        let mut blue = RgbImage::new(64, 64);
        for p in blue.data.as_chunks_mut::<3>().0 {
            *p = [20, 40, 230];
        }
        let thumb = codec.encode(&blue, EncodeOptions::screen(90), None)?;
        let thumbnail = |_: &PhotoRecord| Some(thumb.clone());
        let mean_blue = |jpeg: &[u8]| -> TestResult<f64> {
            let img = decode_image(codec.as_ref(), jpeg)?.image;
            let sum: f64 = img
                .data
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| f64::from(p[2]) - f64::from(p[0]))
                .sum();
            Ok(sum / f64::from(img.width * img.height))
        };
        let cache = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        let look = crate::looks::LookSettings::default();
        let before =
            mean_blue(&cache.look_tile_jpeg(&record, &look, &thumbnail, codec.as_ref())?)?;
        assert!(
            before > 150.0,
            "no standard preview yet: the thumbnail ({before})"
        );
        assert!(cache.ensure_standard(
            &record,
            &RenderSettings::default(),
            codec.as_ref(),
            &|| true
        )?);
        let fresh = PreviewCache::with(Budget::default(), Some(cache_dir.path().to_path_buf()));
        let after =
            mean_blue(&fresh.look_tile_jpeg(&record, &look, &thumbnail, codec.as_ref())?)?;
        assert!(after < 100.0, "the develop, not the thumbnail ({after})");
        Ok(())
    }

    /// Zoom on a photo turned upright (EXIF or the user's turns), with nothing kept in
    /// memory, as when the photo opened from its standard preview: the full-resolution region
    /// is cut from the upright photo, so a portrait print stays portrait (v0.11.2; before,
    /// it came out sideways and stretched).
    #[test]
    fn fr_057_zoom_turns_the_photo_upright_when_nothing_is_kept() -> TestResult {
        let dir = tempfile::tempdir()?;
        let codec = default_codec();
        let mut record = photo(dir.path(), 1200, 800)?;
        record.edits.rotation = 1;
        let settings = RenderSettings::default();
        let cache = PreviewCache::new();
        let whole = Region::new(0.0, 0.0, 1.0, 1.0, 4096).ok_or("region")?;
        let part = cache.render_region(&record, &settings, None, None, whole, codec.as_ref())?;
        assert!(part.height > part.width, "{}×{}", part.width, part.height);
        let shown = cache.render_rgb(&record, &settings, None, None, codec.as_ref())?;
        let ratio = |w: u32, h: u32| f64::from(w) / f64::from(h);
        assert!((ratio(part.width, part.height) - ratio(shown.width, shown.height)).abs() < 0.02);
        Ok(())
    }

    /// FR-091, ADR-0041: short of memory under a tenth available, never when the system
    /// does not say; a trim keeps only the photo on screen and says what it let go; a lower
    /// budget lets the oldest go at once.
    #[test]
    fn fr_091_memory_is_given_back_when_short_or_lowered() -> TestResult {
        assert!(short_of_memory(Some(9)) && !short_of_memory(Some(10)) && !short_of_memory(None));
        let dir = tempfile::tempdir()?;
        let codec = default_codec();
        let cache = PreviewCache::with(Budget::default(), None);
        let settings = RenderSettings::default();
        let mut records = Vec::new();
        for i in 0..3 {
            let sub = dir.path().join(i.to_string());
            std::fs::create_dir(&sub)?;
            let mut record = photo(&sub, 1200, 800)?;
            record.id = PhotoId(i);
            cache.render_jpeg(&record, &settings, None, None, codec.as_ref())?;
            records.push(record);
        }
        let held = cache.usage();
        assert_eq!(cache.entries.lock().map_err(|_| "lock")?.len(), 3);
        let one = held.sources / 3;
        cache.set_budget(Budget {
            sources: one * 2,
            ..Budget::default()
        });
        let left: Vec<u32> = cache
            .entries
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| k.0.0)
            .collect();
        assert_eq!(left, [1, 2]);
        let full = cache.budget();
        let freed = cache.follow_memory(full, Some(5));
        assert!(freed > 0);
        let left: Vec<u32> = cache
            .entries
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| k.0.0)
            .collect();
        assert_eq!(left, [2]);
        Ok(())
    }

    /// Audit 2026-11-06: while the machine stays short, photos opened after the first
    /// warning do not fill the preview back to its whole budget (v0.12.0's watcher trimmed
    /// once, then the cache refilled); when memory comes back, the budget is the user's again.
    #[test]
    fn fr_091_memory_stays_low_while_short_and_comes_back_after() -> TestResult {
        let dir = tempfile::tempdir()?;
        let codec = default_codec();
        let settings = RenderSettings::default();
        let one = 1200 * 800 * 3;
        let full = Budget {
            sources: one * 3,
            ..Budget::default()
        };
        let cache = PreviewCache::with(full, None);
        let open = |i: u32| -> TestResult {
            let sub = dir.path().join(i.to_string());
            std::fs::create_dir(&sub)?;
            let mut record = photo(&sub, 1200, 800)?;
            record.id = PhotoId(i);
            cache.render_jpeg(&record, &settings, None, None, codec.as_ref())?;
            Ok(())
        };
        let kept = || cache.entries.lock().map_or(usize::MAX, |e| e.len());
        for i in 0..3 {
            open(i)?;
        }
        assert_eq!(kept(), 3);
        assert!(cache.follow_memory(full, Some(5)) > 0);
        for i in 3..6 {
            open(i)?;
            // Each watcher tick while short changes nothing more.
            assert_eq!(cache.follow_memory(full, Some(5)), 0);
            assert_eq!(kept(), 1, "photo {i} opened while short");
        }
        assert_eq!(cache.follow_memory(full, Some(40)), 0);
        assert_eq!(cache.budget(), full);
        for i in 6..9 {
            open(i)?;
        }
        assert_eq!(kept(), 3);
        Ok(())
    }

    /// Audit 2026-11-06: what the preview holds counts the collage grid's small sources
    /// and the last resized photo too.
    #[test]
    fn fr_086_usage_counts_every_cache() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cache = PreviewCache::new();
        let record = photo(dir.path(), 600, 400)?;
        let before = cache.usage();
        cache.small.lock().map_err(|_| "lock")?.push((
            source_key(&record, RawSliders::default()),
            Arc::new(RgbImage::new(60, 40)),
        ));
        let after = cache.usage();
        assert!(after.others > before.others, "{after:?}");
        assert_eq!(
            after.total(),
            after.sources + after.prepared + after.full + after.others
        );
        Ok(())
    }

    /// FR-091: a cache keeps what fits in its bytes, letting the oldest go, and always keeps
    /// the newest, even when it alone is over.
    #[test]
    fn fr_091_byte_budgets_let_the_oldest_go_and_keep_the_newest() -> TestResult {
        let kept: Mutex<Vec<(u32, usize)>> = Mutex::new(Vec::new());
        for (key, bytes) in [(1, 40), (2, 40), (3, 40), (2, 40)] {
            keep_bounded(&kept, key, bytes, |b| *b, 100, Keep::Shown, || true);
        }
        let keys: Vec<u32> = kept
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| *k)
            .collect();
        assert_eq!(keys, [3, 2]);
        keep_bounded(&kept, 9, 500, |b| *b, 100, Keep::Shown, || true);
        let keys: Vec<u32> = kept
            .lock()
            .map_err(|_| "lock")?
            .iter()
            .map(|(k, _)| *k)
            .collect();
        assert_eq!(keys, [9]);
        Ok(())
    }

    #[test]
    fn full_cache_stays_within_budget() -> TestResult {
        let dir = tempfile::tempdir()?;
        let codec = default_codec();
        let cache = PreviewCache::new();
        // 5000×4000 is 60 MB decoded: four would pass 200 MB.
        for i in 0..4 {
            let sub = dir.path().join(i.to_string());
            std::fs::create_dir(&sub)?;
            let mut record = photo(&sub, 5000, 4000)?;
            record.id = PhotoId(i);
            cache.full_source(&record, RawSliders::default(), codec.as_ref())?;
            let full = cache.full.lock().map_err(|_| "lock")?;
            let bytes: usize = full.iter().map(|(_, f)| f.image.data.len()).sum();
            assert!(
                bytes <= Budget::default().full,
                "{bytes} bytes after photo {i}"
            );
            assert_eq!(full.last().map(|(k, _)| k.0), Some(PhotoId(i)));
        }
        Ok(())
    }
}
