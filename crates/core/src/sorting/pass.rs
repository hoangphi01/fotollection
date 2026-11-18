// SPDX-License-Identifier: GPL-3.0-only
//! The pixels' pass of the sorting (FR-141 S6, ADR-0056): each photo's difference hash
//! (FR-015) and CLIP embedding, from the cheapest picture it has (its cached T1, else the file
//! decoded for the screen), cached on disk beside the thumbnails (DATABASE.md §8). The
//! embedding is kept, not the labels, so a label added later costs a dot product, not a pass.
//!
//! A cache entry is 2,064 bytes: `FSRT`, a version byte, three zero bytes, the hash (u64 LE)
//! and the 512 floats (f32 LE). An entry of another length, version or magic, or holding a
//! value that is not finite, reads as absent; it is recomputed, never trusted.

use std::io::Read;
use std::path::PathBuf;

use super::clip::{ClipError, DIM, Embedder};
use crate::codec::{CodecError, JpegCodec, decode_for_screen};
use crate::image::apply_orientation;
use crate::library::PhotoRecord;
use crate::thumbs::{ThumbCache, content_key, write_atomic};

const MAGIC: &[u8; 4] = b"FSRT";
// 2: three cuts averaged (owner, 2026-11-18); a centre-only entry is read again.
const VERSION: u8 = 2;
const ENTRY_BYTES: usize = 8 + 8 + DIM * 4;
/// The long edge decoded when no T1 is cached: CLIP needs 224, the hash 9.
const DECODE_EDGE: u32 = 512;

/// What the pass keeps of one photo.
#[derive(Debug, Clone, PartialEq)]
pub struct Pixels {
    pub hash: u64,
    /// Unit length, [`DIM`] floats.
    pub embedding: Vec<f32>,
}

#[derive(Debug, thiserror::Error)]
pub enum SortingError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Clip(#[from] ClipError),
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),
}

impl SortingError {
    /// Stable machine-readable code (SECURITY.md §8), for the `ERR-` code on screen.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Codec(e) => e.code(),
            Self::Clip(e) => e.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

/// The pass's results on disk: `sorting/<aa>/<key>.bin`.
#[derive(Debug, Clone)]
pub struct SortingCache {
    dir: PathBuf,
}

impl SortingCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The cache folder, for its size limit (`cache_limit`).
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    fn path(&self, record: &PhotoRecord) -> PathBuf {
        let key = content_key(record);
        self.dir.join(&key[..2]).join(format!("{key}.bin"))
    }

    /// The cached result, if present and whole.
    pub fn get(&self, record: &PhotoRecord) -> Option<Pixels> {
        let path = self.path(record);
        let file = std::fs::File::open(&path).ok()?;
        let mut bytes = Vec::with_capacity(ENTRY_BYTES);
        file.take(ENTRY_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        let pixels = decode(&bytes)?;
        crate::cache_limit::touch(&path);
        Some(pixels)
    }

    /// Cached, or computed and stored. A failed write only costs a redo later.
    pub fn pass(
        &self,
        record: &PhotoRecord,
        thumbs: &ThumbCache,
        codec: &dyn JpegCodec,
        embedder: &Embedder,
    ) -> Result<Pixels, SortingError> {
        if let Some(cached) = self.get(record) {
            return Ok(cached);
        }
        let upright = picture(record, thumbs, codec)?;
        let pixels = Pixels {
            hash: super::dupes::dhash(&upright),
            embedding: embedder.embed(&upright)?,
        };
        let _ = write_atomic(&self.path(record), &encode(&pixels));
        Ok(pixels)
    }
}

/// The cheapest upright picture of a photo: its cached T1 (already upright and in sRGB),
/// else the file decoded for the screen and turned as shown.
pub fn picture(
    record: &PhotoRecord,
    thumbs: &ThumbCache,
    codec: &dyn JpegCodec,
) -> Result<crate::image::RgbImage, SortingError> {
    if let Some(t1) = thumbs.cached_t1(record).and_then(|t| codec.decode(&t).ok()) {
        return Ok(t1.image);
    }
    let bytes = std::fs::read(&record.path)?;
    let decoded = decode_for_screen(codec, &bytes, DECODE_EDGE)?;
    Ok(apply_orientation(decoded.image, record.orientation))
}

fn encode(p: &Pixels) -> Vec<u8> {
    let mut out = Vec::with_capacity(ENTRY_BYTES);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&[VERSION, 0, 0, 0]);
    out.extend_from_slice(&p.hash.to_le_bytes());
    for v in &p.embedding {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn decode(bytes: &[u8]) -> Option<Pixels> {
    if bytes.len() != ENTRY_BYTES || &bytes[..4] != MAGIC || bytes[4] != VERSION {
        return None;
    }
    let hash = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
    let embedding: Vec<f32> = bytes[16..]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    embedding
        .iter()
        .all(|v| v.is_finite())
        .then_some(Pixels { hash, embedding })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_141_a_sorting_cache_entry_round_trips_and_a_broken_one_is_a_miss() {
        let p = Pixels {
            hash: 0xDEAD_BEEF_0123_4567,
            embedding: (0..DIM)
                .map(|i| f32::from(u16::try_from(i).unwrap_or(0)) / 1000.0)
                .collect(),
        };
        let bytes = encode(&p);
        assert_eq!(bytes.len(), 2_064);
        assert_eq!(decode(&bytes), Some(p));
        for cut in [0, 4, 16, bytes.len() - 1] {
            assert_eq!(decode(&bytes[..cut]), None);
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert_eq!(decode(&longer), None);
        let mut magic = bytes.clone();
        magic[0] = b'X';
        assert_eq!(decode(&magic), None);
        let mut version = bytes.clone();
        version[4] = VERSION + 1;
        assert_eq!(decode(&version), None);
        let mut nan = bytes;
        nan[16..20].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(decode(&nan), None);
    }
}
