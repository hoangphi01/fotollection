// SPDX-License-Identifier: GPL-3.0-only
//! Interactive previews (FR-007). Each photo is decoded once into an upright T2 image
//! (~2 MP) kept in a small LRU; re-rendering for new settings then only crops, resizes
//! and stamps that buffer (NFR-002).
//!
//! Zooming past the preview's pixels (FR-057) asks for one region at full resolution: the
//! original is decoded only then, into a second cache bounded by bytes, rendered exactly as
//! export renders it, and only the visible part is encoded.

use std::sync::{Arc, Mutex};

use crate::codec::{EncodeOptions, JpegCodec, decode_image};
use crate::image::{RgbImage, apply_orientation};
use crate::library::{PhotoId, PhotoRecord};
use crate::plan::{FrameRequest, PlanInput, plan};
use crate::render::{fit_long_edge, render};
use crate::settings::RenderSettings;
use crate::thumbs::ThumbError;

/// Long edge of the cached T2 source.
pub const T2_LONG_EDGE: u32 = 2048;
/// Long edge of the preview sent to the UI.
pub const PREVIEW_LONG_EDGE: u32 = 1600;
const PREVIEW_QUALITY: u8 = 85;
/// About 12.5 MB each at 2048×1365, so the cache stays near 100 MB.
const CAPACITY: usize = 8;
/// Full-resolution sources kept for zoom: two 32 MP photos (97 MB each, RESULT §11). The
/// photo being looked at always stays, whatever its size.
const FULL_BUDGET_BYTES: usize = 200_000_000;
/// Widest region the UI may ask for, in pixels: a 4K screen at 1:1.
pub const REGION_MAX_PX: u32 = 4096;

/// A photo and the orientation it was decoded in, so a rotated photo is decoded again.
type SourceKey = (PhotoId, u8);

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

#[derive(Default)]
pub struct PreviewCache {
    entries: Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
    /// Upright originals, decoded only for zoom; bounded by `FULL_BUDGET_BYTES`.
    full: Mutex<Vec<(SourceKey, Arc<RgbImage>)>>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every cached image, e.g. when another project replaces the library and
    /// photo ids start again from 0.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
        if let Ok(mut full) = self.full.lock() {
            full.clear();
        }
    }

    fn source(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        if let Some(image) = take_cached(&self.entries, (record.id, record.display_orientation())) {
            return Ok(image);
        }
        let bytes = std::fs::read(&record.path)?;
        let decoded = decode_image(codec, &bytes)?;
        let upright = apply_orientation(decoded.image, record.display_orientation());
        let image = Arc::new(fit_long_edge(&upright, T2_LONG_EDGE)?);
        if let Ok(mut entries) = self.entries.lock() {
            if entries.len() >= CAPACITY {
                entries.remove(0);
            }
            entries.push((
                (record.id, record.display_orientation()),
                Arc::clone(&image),
            ));
        }
        Ok(image)
    }

    /// The upright, uncropped T2 image as JPEG: the background of the crop editor.
    pub fn source_jpeg(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let source = self.source(record, codec)?;
        Ok(codec.encode(&source, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }

    /// The upright original at full resolution, for zoom.
    fn full_source(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        let key = (record.id, record.display_orientation());
        if let Some(image) = take_cached(&self.full, key) {
            return Ok(image);
        }
        let bytes = std::fs::read(&record.path)?;
        let decoded = decode_image(codec, &bytes)?;
        let image = Arc::new(apply_orientation(decoded.image, key.1));
        if let Ok(mut full) = self.full.lock() {
            full.push((key, Arc::clone(&image)));
            let mut bytes: usize = full.iter().map(|(_, i)| i.data.len()).sum();
            while bytes > FULL_BUDGET_BYTES && full.len() > 1 {
                bytes -= full.remove(0).1.data.len();
            }
        }
        Ok(image)
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
        let print = settings
            .print_for(source.width, source.height)
            .unwrap_or(crate::PrintSize::ALL[1]);
        let capture = record.stamp_date();
        let settings = settings.for_photo(record.edits.stamp.as_ref());
        let p = plan(&PlanInput {
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
        });
        Ok(render(source, &p)?)
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
        let source = self.source(record, codec)?;
        let image = Self::render_from(
            &source,
            record,
            settings,
            frame,
            place,
            Some(PREVIEW_LONG_EDGE),
        )?;
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
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
        let source = self.full_source(record, codec)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::default_codec;
    use crate::import::ImportedFile;
    use crate::library::Library;
    use crate::metadata::PhotoMeta;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    /// A `width`×`height` JPEG with a gradient, as a library record.
    #[allow(clippy::cast_possible_truncation)]
    fn photo(dir: &std::path::Path, width: u32, height: u32) -> TestResult<PhotoRecord> {
        let mut image = RgbImage::new(width, height);
        for (i, px) in image.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % width, i as u32 / width);
            *px = [(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8];
        }
        let codec = default_codec();
        let path = dir.join(format!("{width}x{height}.jpg"));
        let bytes = codec.encode(&image, EncodeOptions::print(95), None)?;
        std::fs::write(&path, &bytes)?;
        let mut lib = Library::new();
        let folder = lib.add_folder(dir);
        let file = ImportedFile {
            path,
            size: bytes.len() as u64,
            mtime: 0,
        };
        let id = lib
            .push(folder, &file, &PhotoMeta::default())
            .ok_or("photo")?;
        Ok(lib.record(id).ok_or("record")?)
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
            &*cache.full_source(&record, codec.as_ref())?,
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
            cache.full_source(&record, codec.as_ref())?;
            let full = cache.full.lock().map_err(|_| "lock")?;
            let bytes: usize = full.iter().map(|(_, i)| i.data.len()).sum();
            assert!(bytes <= FULL_BUDGET_BYTES, "{bytes} bytes after photo {i}");
            assert_eq!(full.last().map(|(k, _)| k.0), Some(PhotoId(i)));
        }
        Ok(())
    }
}
