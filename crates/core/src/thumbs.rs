// SPDX-License-Identifier: GPL-3.0-only
//! Thumbnail tiers and the on-disk cache (DATABASE.md §8, OPTIMIZATION.md §4.1).
//!
//! - **T0**: the JPEG thumbnail embedded in EXIF, re-oriented. Instant.
//! - **T1**: ~400 px from a full decode, cached on disk by content key.

use std::path::{Path, PathBuf};

use crate::codec::{CodecError, EncodeOptions, JpegCodec, decode_for_screen};
use crate::image::apply_orientation;
use crate::library::PhotoRecord;
use crate::metadata;
use crate::render::{RenderError, fit_long_edge};

pub const T1_LONG_EDGE: u32 = 400;
const THUMB_QUALITY: u8 = 82;

#[derive(Debug, thiserror::Error)]
pub enum ThumbError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),
}

impl ThumbError {
    /// Stable machine-readable code (SECURITY.md §8), for the `ERR-` code on screen.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Codec(e) => e.code(),
            Self::Render(_) => "render.failed",
            Self::Io(_) => "io.read_failed",
        }
    }
}

pub struct ThumbCache {
    dir: PathBuf,
}

impl ThumbCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The cache folder, for its size limit (`cache_limit`).
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `thumbs/<aa>/<blake3(path ‖ size ‖ mtime)>.t1.jpg`
    fn t1_path(&self, record: &PhotoRecord) -> PathBuf {
        let key = content_key(record);
        self.dir.join(&key[..2]).join(format!("{key}.t1.jpg"))
    }

    pub fn cached_t1(&self, record: &PhotoRecord) -> Option<Vec<u8>> {
        let path = self.t1_path(record);
        let bytes = std::fs::read(&path).ok()?;
        // Browsed thumbnails stay when the cache is pruned (DATABASE.md §8).
        crate::cache_limit::touch(&path);
        Some(bytes)
    }

    /// Generates T1 and stores it atomically in the cache.
    pub fn make_t1(
        &self,
        record: &PhotoRecord,
        codec: &dyn JpegCodec,
    ) -> Result<Vec<u8>, ThumbError> {
        let bytes = std::fs::read(&record.path)?;
        // A RAW's embedded preview, not a one-second develop (ADR-0023).
        let decoded = decode_for_screen(codec, &bytes, T1_LONG_EDGE)?;
        let upright = apply_orientation(decoded.image, record.orientation);
        let mut small = fit_long_edge(&upright, T1_LONG_EDGE)?;
        // To sRGB after the resize, on a 25th of the pixels (FR-072).
        crate::colour::to_srgb(&mut small, decoded.icc.as_deref());
        let jpeg = codec.encode(&small, EncodeOptions::screen(THUMB_QUALITY), None)?;
        let path = self.t1_path(record);
        write_atomic(&path, &jpeg)?;
        Ok(jpeg)
    }
}

/// Bumped when a decoder fix changes the pixels of files already seen, so every cache made
/// from them is made again (ADR-0033). 2: v0.8.1, one-scan-per-component JPEGs were green
/// (version 1 had no salt). 3: v0.9.1, 16-bit grey JPEGs were noise and JPEGs cut short
/// were half photos. 4: v0.10.0, Display P3 and Adobe RGB photos were shown unconverted,
/// washed out (FR-072). Old files are left to the cache's size limit.
pub const CACHE_VERSION: u8 = 4;

/// Cache key of a source file: `blake3(version ‖ path ‖ size ‖ mtime)` in hex, so an edited or
/// replaced file, or a decoder fix, misses the cache (DATABASE.md §8).
pub fn content_key(record: &PhotoRecord) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[CACHE_VERSION]);
    hasher.update(record.path.as_os_str().as_encoded_bytes());
    hasher.update(&record.size.to_le_bytes());
    hasher.update(&record.mtime.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

/// T0 from the EXIF thumbnail, rotated upright (embedded thumbnails carry no orientation).
pub fn t0(record: &PhotoRecord, codec: &dyn JpegCodec) -> Option<Vec<u8>> {
    // A camera's EXIF thumbnail is in the photo's own space, untagged: only an sRGB photo's
    // can be shown as it is (FR-072); T1 converts the others.
    if !matches!(
        record.facts.space,
        None | Some(crate::facts::Space::Srgb | crate::facts::Space::Untagged)
    ) {
        return None;
    }
    let raw = metadata::embedded_thumbnail(&record.path)?;
    if record.orientation <= 1 {
        return Some(raw);
    }
    let decoded = codec.decode(&raw).ok()?;
    let upright = apply_orientation(decoded.image, record.orientation);
    codec
        .encode(&upright, EncodeOptions::screen(THUMB_QUALITY), None)
        .ok()
}

/// Writes `bytes` to a temporary sibling, then renames it into place.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}
