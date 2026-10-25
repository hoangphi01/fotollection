// SPDX-License-Identifier: GPL-3.0-only
//! Camera RAW input (FR-016, ADR-0023): which RAW a file is, from its bytes alone, and its
//! two decode paths through the operating system's codecs. The embedded preview feeds
//! thumbnails and best-shot analysis; the full develop feeds the print preview and export.

pub use fotollection_oscodec::RawKind;

use crate::codec::{CodecError, Decoded, MAX_EDGE, MAX_PIXELS};
use crate::image::RgbImage;

/// How many bytes `kind` may need: the TIFF header, IFD0 and its Make string. Import reads
/// this much of a file's head.
pub const HEAD_BYTES: usize = 64 * 1024;

const LIMITS: fotollection_oscodec::Limits = fotollection_oscodec::Limits {
    max_edge: MAX_EDGE,
    max_pixels: MAX_PIXELS,
};

/// The RAW format of `bytes` (the head of a file is enough), or `None` for anything else,
/// including a plain TIFF. Reads only inside `bytes`, so a truncated or hostile file cannot
/// make it panic.
pub fn kind(bytes: &[u8]) -> Option<RawKind> {
    if bytes.starts_with(b"IIU\0") {
        return Some(RawKind::Rw2);
    }
    if [&b"IIRO"[..], b"IIRS", b"MMOR"]
        .iter()
        .any(|m| bytes.starts_with(m))
    {
        return Some(RawKind::Orf);
    }
    if bytes.starts_with(b"FUJIFILMCCD-RAW") {
        return Some(RawKind::Raf);
    }
    if bytes.get(4..12) == Some(b"ftypcrx ") {
        return Some(RawKind::Cr3);
    }
    let little = if bytes.starts_with(b"II*\0") {
        true
    } else if bytes.starts_with(b"MM\0*") {
        false
    } else {
        return None;
    };
    if bytes.get(8..10) == Some(b"CR") {
        return Some(RawKind::Cr2);
    }
    tiff_kind(bytes, little)
}

/// DNG by its `DNGVersion` tag, NEF and ARW by the maker in IFD0's `Make` tag.
fn tiff_kind(bytes: &[u8], little: bool) -> Option<RawKind> {
    let u16_at = |at: usize| {
        let b = bytes.get(at..at + 2)?;
        Some(if little {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        })
    };
    let u32_at = |at: usize| {
        let b = bytes.get(at..at + 4)?;
        let b = [b[0], b[1], b[2], b[3]];
        Some(if little {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    };
    let ifd = usize::try_from(u32_at(4)?).ok()?;
    let entries = usize::from(u16_at(ifd)?);
    let mut maker = None;
    for i in 0..entries.min(512) {
        let entry = ifd + 2 + i * 12;
        match u16_at(entry)? {
            0xC612 => return Some(RawKind::Dng),
            0x010F => {
                let count = usize::try_from(u32_at(entry + 4)?).ok()?;
                let at = if count <= 4 {
                    entry + 8
                } else {
                    usize::try_from(u32_at(entry + 8)?).ok()?
                };
                maker = bytes.get(at..at + count.min(32));
            }
            _ => {}
        }
    }
    let maker = maker?;
    let starts =
        |name: &[u8]| maker.len() >= name.len() && maker[..name.len()].eq_ignore_ascii_case(name);
    if starts(b"NIKON") {
        Some(RawKind::Nef)
    } else if starts(b"SONY") {
        Some(RawKind::Arw)
    } else {
        None
    }
}

fn error(e: fotollection_oscodec::OsCodecError) -> CodecError {
    use fotollection_oscodec::OsCodecError as E;
    match e {
        E::Unsupported => CodecError::Unsupported,
        E::CodecMissing => CodecError::RawCodecMissing,
        E::TooLarge(w, h) => CodecError::TooLarge(w, h),
        E::Decode(why) => CodecError::Decode(why),
    }
}

fn decoded(image: fotollection_oscodec::OsImage) -> Result<Decoded, CodecError> {
    let Some(rgb) = RgbImage::from_raw(image.width, image.height, image.rgb) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    Ok(Decoded {
        image: rgb,
        icc: image.icc,
    })
}

/// One develop at a time across the app: a single develop can hold more than 1 GB
/// (RESULT.md §12), so a batch export of RAW stays within the 2 GB ceiling (NFR-004).
static DEVELOP: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The full develop, for the print preview, zoom and export.
pub fn decode(bytes: &[u8], kind: RawKind) -> Result<Decoded, CodecError> {
    let format = fotollection_oscodec::Format::Raw(kind);
    // A poisoned lock only means another develop panicked; the lock itself guards nothing.
    let _one = DEVELOP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    decoded(fotollection_oscodec::decode(bytes, format, LIMITS).map_err(error)?)
}

/// The developed size, the only one to use for geometry (crops are fractions of it).
pub fn dimensions(bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError> {
    fotollection_oscodec::dimensions(bytes, fotollection_oscodec::Format::Raw(kind)).map_err(error)
}

/// The preview the RAW carries, at most `max_px` on its long edge: 30 to 100 times faster
/// than a develop (RESULT.md §12), for thumbnails and best-shot analysis. Its size can
/// differ from `dimensions`.
pub fn preview(bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
    let format = fotollection_oscodec::Format::Raw(kind);
    decoded(fotollection_oscodec::preview(bytes, format, max_px, LIMITS).map_err(error)?)
}

/// Whether this machine can develop RAW, asked once per run.
pub fn available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        fotollection_oscodec::available(fotollection_oscodec::Format::Raw(RawKind::Rw2))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A little-endian TIFF head with one IFD0 holding `entries` (tag, type, count, value),
    /// then `tail` at offset 8 + 2 + 12 × n + 4.
    fn tiff(entries: &[(u16, u16, u32, u32)], tail: &[u8]) -> Vec<u8> {
        let mut out = b"II*\0".to_vec();
        out.extend_from_slice(&8u32.to_le_bytes());
        out.extend_from_slice(&u16::try_from(entries.len()).unwrap_or(0).to_le_bytes());
        for &(tag, kind, count, value) in entries {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&kind.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(tail);
        out
    }

    #[test]
    fn fr_016_recognises_raw_by_bytes() {
        assert_eq!(kind(b"IIU\0\x18\0\0\0"), Some(RawKind::Rw2));
        assert_eq!(kind(b"IIRO\x08\0\0\0"), Some(RawKind::Orf));
        assert_eq!(kind(b"MMOR\0\0\0\x08"), Some(RawKind::Orf));
        assert_eq!(kind(b"FUJIFILMCCD-RAW 0201"), Some(RawKind::Raf));
        assert_eq!(kind(b"\0\0\0\x18ftypcrx \0\0\0\x01"), Some(RawKind::Cr3));
        assert_eq!(kind(b"II*\0\x10\0\0\0CR\x02\0"), Some(RawKind::Cr2));
        // Make in the tail: offset = 8 + 2 + 12 + 4 = 26.
        let nikon = tiff(&[(0x010F, 2, 18, 26)], b"NIKON CORPORATION\0");
        assert_eq!(kind(&nikon), Some(RawKind::Nef));
        let sony = tiff(&[(0x010F, 2, 5, 26)], b"SONY\0");
        assert_eq!(kind(&sony), Some(RawKind::Arw));
        let dng = tiff(&[(0xC612, 1, 4, 0x0001_0401)], b"");
        assert_eq!(kind(&dng), Some(RawKind::Dng));
    }

    #[test]
    fn fr_016_other_files_are_not_raw() {
        // HEIC keeps its own path; a plain TIFF, a JPEG and noise are not RAW.
        assert_eq!(kind(b"\0\0\0\x18ftypheic\0\0\0\0"), None);
        assert_eq!(kind(&tiff(&[(0x010F, 2, 6, 26)], b"Apple\0")), None);
        assert_eq!(kind(&[0xFF, 0xD8, 0xFF, 0xE0]), None);
        assert_eq!(kind(b""), None);
        // A truncated or hostile IFD stays inside the buffer.
        assert_eq!(kind(b"II*\0\xFF\xFF\xFF\x7F"), None);
        assert_eq!(kind(&tiff(&[(0x010F, 2, 1_000_000, 26)], b"NI")), None);
        let mut many = tiff(&[], b"");
        many[8] = 0xFF;
        many[9] = 0xFF;
        assert_eq!(kind(&many), None);
    }

    /// The CC0 samples from `scripts/fetch-raw-samples.py`, when `FOTOLLECTION_RAW_SAMPLES`
    /// names their folder: each is recognised from its head and, on macOS, develops at the
    /// size its header gives.
    #[test]
    fn fr_016_samples_are_recognised_and_develop() -> Result<(), Box<dyn std::error::Error>> {
        let Some(dir) = std::env::var_os("FOTOLLECTION_RAW_SAMPLES") else {
            return Ok(());
        };
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let Some(expected) = RawKind::from_extension(&ext) else {
                continue;
            };
            let bytes = std::fs::read(&path)?;
            let head = &bytes[..bytes.len().min(HEAD_BYTES)];
            assert_eq!(kind(head), Some(expected), "{}", path.display());
            if cfg!(target_os = "macos") {
                let (w, h) = dimensions(&bytes, expected)?;
                let preview = preview(&bytes, expected, 400)?;
                assert!(preview.image.width.max(preview.image.height) <= 400);
                assert!(w > 2000 && h > 1000, "{}: {w}×{h}", path.display());
            }
        }
        Ok(())
    }
}
