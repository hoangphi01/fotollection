// SPDX-License-Identifier: GPL-3.0-only
//! Analysis results on disk (DATABASE.md §8): derived data, keyed by the source file's
//! content key, never in the project file. Safe to delete at any time.

use std::io::Read;
use std::path::PathBuf;

use super::{ANALYSIS_VERSION, CullingError, PhotoAnalysis, analyze};
use crate::codec::{CodecError, JpegCodec, decode_image};
use crate::image::apply_orientation;
use crate::library::PhotoRecord;
use crate::thumbs::{content_key, write_atomic};

/// Cached files above this size are treated as corrupt (a real one is about 3 KB).
const MAX_ENTRY_BYTES: u64 = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Culling(#[from] CullingError),
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),
}

impl AnalyzeError {
    /// Stable error code for the UI (FR-020).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Codec(e) => e.code(),
            Self::Culling(e) => e.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

pub struct AnalysisCache {
    dir: PathBuf,
}

impl AnalysisCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// `analysis/<aa>/<key>.json`
    fn path(&self, record: &PhotoRecord) -> PathBuf {
        let key = content_key(record);
        self.dir.join(&key[..2]).join(format!("{key}.json"))
    }

    /// The cached analysis, if present, readable and made by this analysis version.
    pub fn get(&self, record: &PhotoRecord) -> Option<PhotoAnalysis> {
        let file = std::fs::File::open(self.path(record)).ok()?;
        let mut bytes = Vec::new();
        file.take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        if bytes.len() as u64 > MAX_ENTRY_BYTES {
            return None;
        }
        let analysis: PhotoAnalysis = serde_json::from_slice(&bytes).ok()?;
        (analysis.version == ANALYSIS_VERSION && analysis.is_plausible()).then_some(analysis)
    }

    /// Cached, or computed from the file and stored. A failed write only costs a redo later.
    pub fn analyze(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<PhotoAnalysis, AnalyzeError> {
        if let Some(cached) = self.get(record) {
            return Ok(cached);
        }
        let bytes = std::fs::read(&record.path)?;
        let upright = apply_orientation(decode_image(codec, &bytes)?.image, record.orientation);
        let analysis = analyze(&upright)?;
        if let Ok(json) = serde_json::to_vec(&analysis) {
            let _ = write_atomic(&self.path(record), &json);
        }
        Ok(analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::culling::FaceSummary;
    use crate::culling::metrics::Metrics;

    fn record(dir: &std::path::Path) -> PhotoRecord {
        let path = dir.join("a.jpg");
        let image = crate::image::RgbImage::new(64, 48);
        let codec = crate::codec::default_codec();
        let jpeg = codec
            .encode(&image, crate::codec::EncodeOptions::print(90), None)
            .unwrap_or_default();
        std::fs::write(&path, &jpeg).ok();
        PhotoRecord {
            id: crate::library::PhotoId(0),
            path,
            file_name: "a.jpg".into(),
            size: jpeg.len() as u64,
            mtime: 0,
            capture: chrono::NaiveDateTime::default(),
            orientation: 1,
            flags: 0,
            gps: None,
            shot: crate::library::ShotInfo::default(),
            edits: crate::library::PhotoEdits::default(),
        }
    }

    #[test]
    fn fr_041_analysis_is_cached_and_versioned() {
        let dir = tempfile::tempdir().map_err(|e| e.to_string());
        let Ok(dir) = dir else { return };
        let cache = AnalysisCache::new(dir.path().join("analysis"));
        let photo = record(dir.path());
        assert!(cache.get(&photo).is_none());
        let codec = crate::codec::default_codec();
        let first = cache
            .analyze(&photo, codec.as_ref())
            .map_err(|e| e.to_string());
        assert!(first.is_ok(), "{first:?}");
        assert_eq!(cache.get(&photo), first.ok());

        // Another analysis version, or garbage, is a miss rather than an error.
        let stale = PhotoAnalysis {
            version: ANALYSIS_VERSION + 1,
            metrics: Metrics::default(),
            signature: crate::culling::similar::Signature::default(),
            faces: Vec::new(),
        };
        let path = cache.path(&photo);
        std::fs::write(&path, serde_json::to_vec(&stale).unwrap_or_default()).ok();
        assert!(cache.get(&photo).is_none());
        std::fs::write(&path, b"{not json").ok();
        assert!(cache.get(&photo).is_none());
        let absurd = PhotoAnalysis {
            version: ANALYSIS_VERSION,
            faces: vec![
                FaceSummary {
                    x: 0.0,
                    y: 0.0,
                    w: 9.0,
                    h: 0.1,
                    ear: None,
                    sharpness: 1.0,
                };
                3
            ],
            ..stale
        };
        std::fs::write(&path, serde_json::to_vec(&absurd).unwrap_or_default()).ok();
        assert!(
            cache.get(&photo).is_none(),
            "out-of-range boxes are rejected"
        );
    }
}
