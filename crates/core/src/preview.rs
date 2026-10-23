// SPDX-License-Identifier: GPL-3.0-only
//! Interactive previews (FR-007). Each photo is decoded once into an upright T2 image
//! (~2 MP) kept in a small LRU; re-rendering for new settings then only crops, resizes
//! and stamps that buffer (NFR-002).

use std::sync::{Arc, Mutex};

use crate::codec::{EncodeOptions, JpegCodec, decode_image};
use crate::fonts::LoadedFont;
use crate::image::{RgbImage, apply_orientation};
use crate::library::{PhotoId, PhotoRecord, flags};
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

#[derive(Default)]
pub struct PreviewCache {
    entries: Mutex<Vec<(PhotoId, Arc<RgbImage>)>>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn source(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<Arc<RgbImage>, ThumbError> {
        if let Ok(mut entries) = self.entries.lock()
            && let Some(pos) = entries.iter().position(|(id, _)| *id == record.id)
        {
            let entry = entries.remove(pos);
            let image = Arc::clone(&entry.1);
            entries.push(entry); // most recently used last
            return Ok(image);
        }
        let bytes = std::fs::read(&record.path)?;
        let decoded = decode_image(codec, &bytes)?;
        let upright = apply_orientation(decoded.image, record.orientation);
        let image = Arc::new(fit_long_edge(&upright, T2_LONG_EDGE)?);
        if let Ok(mut entries) = self.entries.lock() {
            if entries.len() >= CAPACITY {
                entries.remove(0);
            }
            entries.push((record.id, Arc::clone(&image)));
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

    /// Renders the preview JPEG for `record` with the current settings.
    pub fn render_jpeg(
        &self,
        record: &PhotoRecord,
        settings: &RenderSettings,
        frame: Option<&FrameRequest>,
        stamp_font: Option<&LoadedFont>,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let source = self.source(record, codec)?;
        let print = settings
            .print_for(source.width, source.height)
            .unwrap_or(crate::PrintSize::ALL[1]);
        let capture = (record.flags & flags::DATE_FROM_FILE == 0).then_some(record.capture);
        let p = plan(&PlanInput {
            width: source.width,
            height: source.height,
            capture,
            settings,
            print,
            max_long_edge: Some(PREVIEW_LONG_EDGE),
            frame,
            crop: record.edits.crop,
            stamp_font,
        });
        let image = render(&source, &p)?;
        Ok(codec.encode(&image, EncodeOptions::screen(PREVIEW_QUALITY), None)?)
    }
}
