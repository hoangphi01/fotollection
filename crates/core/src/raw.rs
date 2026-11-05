// SPDX-License-Identifier: GPL-3.0-only
//! Camera RAW input (FR-016, ADR-0023, ADR-0029, ADR-0030): which RAW a file is, from its
//! bytes alone, and its decoders, tried in the order of [`decoders`]: our own DNG reader,
//! rawler, and the operating system's codecs (first after our own on macOS, the last fallback
//! elsewhere). The embedded preview feeds thumbnails and best-shot analysis; the full develop
//! feeds the print preview and export.

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
/// File names of the camera RAW formats the app reads, badged RAW in the grid and the
/// preview (owner, 2026-11-05). A TIFF may come from a scanner, so it is not one.
pub fn is_raw_name(name: &str) -> bool {
    const RAW: &[&str] = &["dng", "rw2", "cr2", "cr3", "nef", "arw", "raf", "orf"];
    std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| RAW.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

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

/// One way to read camera RAW (ADR-0029). Fotollection tries them in [`decoders`] order,
/// so our own decoder can take over format by format without changing any caller.
pub trait RawDecoder: Sync {
    /// The full develop, finished to the camera's look.
    fn decode(&self, bytes: &[u8], kind: RawKind) -> Result<Decoded, CodecError>;
    /// The developed size, the only one to use for geometry.
    fn dimensions(&self, bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError>;
    /// An image the camera stored in the file, at most `max_px` on its long edge.
    fn preview(&self, bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError>;
}

/// The operating system's codecs: `ImageIO` on macOS, WIC with the Raw Image Extension on
/// Windows (ADR-0023). They correct lens distortion, which rawler does not.
pub struct Os;

/// The system's codecs develop a RAW file cut short without an error, the missing part as
/// coloured stripes (`ImageIO`, LESSONS 2026-11-05). Before one is asked, the sensor data is
/// read through by our own DNG reader or rawler, and a file either finds damaged is refused
/// like the same file through them. A format neither unpacks (X-Trans until v0.11.0's
/// session 18, some DNG kinds) is left to the system.
fn intact(bytes: &[u8], kind: RawKind) -> Result<(), CodecError> {
    let read = if kind == RawKind::Dng {
        fotollection_rawcore::dng::parse(bytes)
            .map(|_| ())
            .map_err(own_error)
    } else {
        fotollection_rawlgpl::unpack(bytes, LIMITS.max_pixels)
            .map(|_| ())
            .map_err(rawler_error)
    };
    match read {
        Ok(()) | Err(CodecError::Unsupported | CodecError::TooLarge(..)) => Ok(()),
        Err(_) => Err(CodecError::Decode(
            "the RAW file is damaged or cut short".into(),
        )),
    }
}

impl RawDecoder for Os {
    fn decode(&self, bytes: &[u8], kind: RawKind) -> Result<Decoded, CodecError> {
        intact(bytes, kind)?;
        let format = fotollection_oscodec::Format::Raw(kind);
        decoded(fotollection_oscodec::decode(bytes, format, LIMITS).map_err(error)?)
    }

    fn dimensions(&self, bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError> {
        fotollection_oscodec::dimensions(bytes, fotollection_oscodec::Format::Raw(kind))
            .map_err(error)
    }

    fn preview(&self, bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
        let format = fotollection_oscodec::Format::Raw(kind);
        decoded(fotollection_oscodec::preview(bytes, format, max_px, LIMITS).map_err(error)?)
    }
}

/// rawler (LGPL-2.1), built in, so RAW opens with nothing to install; its develop is
/// finished to the look of the camera's own embedded JPEG (`raw_finish`).
pub struct Rawler;

fn rawler_error(e: fotollection_rawlgpl::RawError) -> CodecError {
    use fotollection_rawlgpl::RawError as E;
    match e {
        E::Unsupported(_) => CodecError::Unsupported,
        E::TooLarge(_) => CodecError::TooLarge(LIMITS.max_edge, LIMITS.max_edge),
        E::Decode(why) => CodecError::Decode(why),
        E::Panicked => CodecError::Decode("the RAW decoder failed on this file".into()),
    }
}

fn rgb8(image: fotollection_rawlgpl::Rgb8) -> Option<RgbImage> {
    RgbImage::from_raw(image.width, image.height, image.rgb)
}

impl RawDecoder for Rawler {
    fn decode(&self, bytes: &[u8], _kind: RawKind) -> Result<Decoded, CodecError> {
        let developed =
            fotollection_rawlgpl::develop(bytes, LIMITS.max_pixels).map_err(rawler_error)?;
        let reference = fotollection_rawlgpl::preview(bytes)
            .ok()
            .flatten()
            .and_then(rgb8);
        let curves = reference
            .as_ref()
            .and_then(|r| crate::raw_finish::match_reference(&developed.rgb, r))
            .unwrap_or_else(|| crate::raw_finish::default_curves(&developed.rgb));
        Ok(Decoded {
            image: crate::raw_finish::apply(
                &developed.rgb,
                developed.width,
                developed.height,
                &curves,
            ),
            icc: None,
        })
    }

    fn dimensions(&self, bytes: &[u8], _kind: RawKind) -> Result<(u32, u32), CodecError> {
        fotollection_rawlgpl::dimensions(bytes).map_err(rawler_error)
    }

    fn preview(&self, bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
        let image = match fotollection_rawlgpl::preview(bytes).map_err(rawler_error)? {
            Some(embedded) => rgb8(embedded)
                .ok_or_else(|| CodecError::Decode("unexpected pixel layout".into()))?,
            // No image in the file (some ORF): develop it, once, for the thumbnail.
            None => self.decode(bytes, kind)?.image,
        };
        let small = crate::render::fit_long_edge(&image, max_px)
            .map_err(|e| CodecError::Decode(e.to_string()))?;
        Ok(Decoded {
            image: small,
            icc: None,
        })
    }
}

/// Fotollection's own develop (ADR-0030, ADR-0040): DNG through our own reader, every other
/// format unpacked by rawler and developed here, for the Bayer sensors (X-Trans follows in
/// v0.11.0's session 18). Its develop is finished to the camera's embedded JPEG (tone, lens
/// shading and colour look); anything else it answers `Unsupported`, so the next decoder
/// takes it.
pub struct Own;

fn own_error(e: fotollection_rawcore::RawError) -> CodecError {
    use fotollection_rawcore::RawError as E;
    match e {
        E::Unsupported(_) => CodecError::Unsupported,
        E::TooLarge(_) => CodecError::TooLarge(LIMITS.max_edge, LIMITS.max_edge),
        E::Truncated => CodecError::Decode("the RAW file is cut short".into()),
        E::Corrupt(why) => CodecError::Decode(format!("the RAW file is damaged: {why}")),
    }
}

impl Own {
    fn supports(bytes: &[u8], kind: RawKind) -> Result<(), CodecError> {
        let ours = if kind == RawKind::Dng {
            fotollection_rawcore::supports(bytes)
        } else {
            matches!(
                fotollection_rawlgpl::sensor(bytes),
                Ok((fotollection_rawlgpl::Filter::Bayer(_), _))
            )
        };
        if ours {
            Ok(())
        } else {
            Err(CodecError::Unsupported)
        }
    }

    /// The camera's own JPEG inside the file, decoded and in sRGB, like the develop it is
    /// matched to and the thumbnails made from it (a camera set to Adobe RGB tags it so).
    fn embedded(bytes: &[u8], kind: RawKind) -> Option<RgbImage> {
        if kind != RawKind::Dng {
            return fotollection_rawlgpl::preview(bytes)
                .ok()
                .flatten()
                .and_then(rgb8);
        }
        let jpeg = fotollection_rawcore::dng::embedded_jpeg(bytes)?;
        crate::codec::default_codec()
            .decode(jpeg)
            .ok()
            .map(crate::colour::screen)
    }

    /// The scene-linear develop: a DNG read by our own reader, any other file's sensor data
    /// unpacked by rawler, with the camera's matrices and white balance.
    fn linear(bytes: &[u8], kind: RawKind) -> Result<fotollection_rawcore::Linear, CodecError> {
        if kind == RawKind::Dng {
            Self::supports(bytes, kind)?;
            return fotollection_rawcore::develop(bytes).map_err(own_error);
        }
        let u = fotollection_rawlgpl::unpack(bytes, LIMITS.max_pixels).map_err(rawler_error)?;
        let tags =
            fotollection_rawcore::color::ColorTags::from_camera(&u.matrices, u.white_balance);
        let mosaic = fotollection_rawcore::cfa::Mosaic::new(
            u.width, u.height, u.data, u.pattern, u.black, u.white, u.active, u.crop,
        )
        .map_err(own_error)?;
        fotollection_rawcore::develop_mosaic(&mosaic, &tags).map_err(own_error)
    }
}

impl RawDecoder for Own {
    fn decode(&self, bytes: &[u8], kind: RawKind) -> Result<Decoded, CodecError> {
        let linear = Self::linear(bytes, kind)?;
        let look = Self::embedded(bytes, kind).and_then(|jpeg| {
            fotollection_rawcore::match_look(
                &linear,
                &fotollection_rawcore::Reference {
                    width: jpeg.width as usize,
                    height: jpeg.height as usize,
                    rgb: &jpeg.data,
                },
            )
        });
        let picture = match look {
            Some(look) => fotollection_rawcore::finish_look(&linear, &look),
            None => {
                fotollection_rawcore::finish(&linear, &fotollection_rawcore::default_curve(&linear))
            }
        };
        let (Ok(width), Ok(height)) = (u32::try_from(picture.width), u32::try_from(picture.height))
        else {
            return Err(CodecError::TooLarge(LIMITS.max_edge, LIMITS.max_edge));
        };
        let image = RgbImage::from_raw(width, height, picture.rgb)
            .ok_or_else(|| CodecError::Decode("unexpected pixel layout".into()))?;
        Ok(Decoded { image, icc: None })
    }

    fn dimensions(&self, bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError> {
        Self::supports(bytes, kind)?;
        let (w, h) = if kind == RawKind::Dng {
            fotollection_rawcore::dimensions(bytes).map_err(own_error)?
        } else {
            let (w, h) = fotollection_rawlgpl::dimensions(bytes).map_err(rawler_error)?;
            (w as usize, h as usize)
        };
        match (u32::try_from(w), u32::try_from(h)) {
            (Ok(w), Ok(h)) => Ok((w, h)),
            _ => Err(CodecError::TooLarge(LIMITS.max_edge, LIMITS.max_edge)),
        }
    }

    fn preview(&self, bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
        Self::supports(bytes, kind)?;
        let image = match Self::embedded(bytes, kind) {
            Some(image) => image,
            None => self.decode(bytes, kind)?.image,
        };
        let small = crate::render::fit_long_edge(&image, max_px)
            .map_err(|e| CodecError::Decode(e.to_string()))?;
        Ok(Decoded {
            image: small,
            icc: None,
        })
    }
}

/// The sensor's filter, its Bayer pattern and bit depth, from rawler without developing
/// (FR-071); `None` when rawler cannot read the file.
pub fn sensor(bytes: &[u8]) -> Option<(crate::facts::Sensor, Option<crate::facts::Cfa>, u8)> {
    use crate::facts::Sensor;
    use fotollection_rawlgpl::Filter;
    let (filter, bits) = fotollection_rawlgpl::sensor(bytes).ok()?;
    Some(match filter {
        Filter::Bayer(name) => {
            let mut sites = [3u8; 4];
            for (site, c) in sites.iter_mut().zip(name.chars()) {
                *site = match c {
                    'R' => 0,
                    'G' => 1,
                    _ => 2,
                };
            }
            (Sensor::Bayer, crate::facts::cfa_of(sites), bits)
        }
        Filter::XTrans => (Sensor::XTrans, None, bits),
        Filter::Other(_) => (Sensor::OtherFilter, None, bits),
        Filter::Linear => (Sensor::Linear, None, bits),
        Filter::Mono => (Sensor::Monochrome, None, bits),
    })
}

/// The order RAW decoders are tried in (ADR-0029, ADR-0030, ADR-0040). Our own develop goes
/// first; rawler's develop and the system's codecs are fallbacks for what it does not read
/// yet. On macOS, Panasonic and Olympus RAW keep `ImageIO` first, because it corrects their
/// lenses' distortion, which Micro Four Thirds rely on and our core does not do yet (owner,
/// 2026-11-05; FR-085).
pub fn decoders(kind: RawKind) -> [&'static dyn RawDecoder; 3] {
    if cfg!(target_os = "macos") {
        if os_first(kind) {
            [&Os, &Own, &Rawler]
        } else {
            [&Own, &Os, &Rawler]
        }
    } else {
        [&Own, &Rawler, &Os]
    }
}

/// The formats macOS's own codec reads first (`decoders`): lens distortion corrected.
pub fn os_first(kind: RawKind) -> bool {
    cfg!(target_os = "macos") && matches!(kind, RawKind::Rw2 | RawKind::Orf)
}

/// The first decoder's answer that works; otherwise the most telling error (a missing
/// codec only when nothing else said more: every reader passed on the file and Windows has
/// no codec for it, ERR-EXT-0003).
fn first<T>(
    kind: RawKind,
    mut call: impl FnMut(&dyn RawDecoder) -> Result<T, CodecError>,
) -> Result<T, CodecError> {
    let mut worst = CodecError::Unsupported;
    let mut missing = false;
    for decoder in decoders(kind) {
        match call(decoder) {
            Ok(v) => return Ok(v),
            Err(CodecError::RawCodecMissing) => missing = true,
            Err(CodecError::Unsupported) => {}
            Err(e) => worst = e,
        }
    }
    Err(match worst {
        CodecError::Unsupported if missing => CodecError::RawCodecMissing,
        e => e,
    })
}

/// The full develop, for the print preview, zoom and export.
pub fn decode(bytes: &[u8], kind: RawKind) -> Result<Decoded, CodecError> {
    // Image data past the end of the file is refused before any reader makes up the rest.
    if fotollection_rawcore::cut_short(bytes) {
        return Err(CodecError::Decode("the RAW file is cut short".into()));
    }
    one_at_a_time(|| first(kind, |d| d.decode(bytes, kind)))
        .unwrap_or_else(|| Err(CodecError::Decode("the RAW develop failed".into())))
}

/// Runs `develop` holding `DEVELOP`, on a thread of its own: `None` if it panicked.
///
/// Never on the caller's thread. A develop runs rayon work, and a rayon thread waiting for
/// that work takes up other jobs of its pool meanwhile; when one of them asks for a develop
/// too, it waits for a lock its own thread holds, and the pool stops for good (LESSONS
/// 2026-11-03: zooming into a RAW hung every preview after it). A plain thread never takes
/// up other jobs, so whoever holds the lock always finishes; the caller only waits.
fn one_at_a_time<T: Send>(develop: impl FnOnce() -> T + Send) -> Option<T> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                // A poisoned lock only means another develop panicked; it guards nothing.
                let _one = DEVELOP
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                develop()
            })
            .join()
            .ok()
    })
}

/// The developed size, the only one to use for geometry (crops are fractions of it).
pub fn dimensions(bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError> {
    first(kind, |d| d.dimensions(bytes, kind))
}

/// The preview the RAW carries, at most `max_px` on its long edge: 30 to 100 times faster
/// than a develop (RESULT.md §12), for thumbnails and best-shot analysis. Its size can
/// differ from `dimensions`.
pub fn preview(bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
    first(kind, |d| d.preview(bytes, kind, max_px))
}

/// Whether this machine can develop RAW: always, since rawler is built in (ADR-0029).
pub fn available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Develops asked on a small pool while one runs, as zooming asks for regions while the
    /// preview develops: the develop's parallel work must not take up a waiting request that
    /// needs the same lock on the same thread (LESSONS 2026-11-03).
    #[test]
    fn develops_asked_while_one_runs_all_finish() -> Result<(), rayon::ThreadPoolBuildError> {
        use rayon::prelude::*;
        const ASKED: u64 = 8;
        let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build()?;
        let (tx, rx) = std::sync::mpsc::channel();
        for i in 0..ASKED {
            let tx = tx.clone();
            pool.spawn(move || {
                let sum =
                    one_at_a_time(|| (0..400_000u64).into_par_iter().map(|x| x % 7).sum::<u64>());
                let _ = tx.send((i, sum));
            });
        }
        for _ in 0..ASKED {
            let answer = rx.recv_timeout(std::time::Duration::from_secs(20));
            assert!(
                matches!(answer, Ok((_, Some(_)))),
                "a develop never finished: {answer:?}"
            );
        }
        Ok(())
    }

    /// `first` with the decoders answering `answers` in turn.
    fn answers(answers: Vec<CodecError>) -> CodecError {
        let mut answers = answers.into_iter();
        match first::<()>(RawKind::Dng, |_| {
            Err(answers.next().unwrap_or(CodecError::Unsupported))
        }) {
            Err(e) => e,
            Ok(()) => CodecError::Unsupported,
        }
    }

    /// ERR-EXT-0003: a missing codec is reported when no reader said more, and only then.
    #[test]
    fn a_missing_codec_is_reported_when_nothing_said_more() {
        use CodecError as E;
        assert_eq!(
            answers(vec![E::Unsupported, E::Unsupported, E::RawCodecMissing]).code(),
            "import.raw_codec_missing"
        );
        assert_eq!(
            answers(vec![
                E::Unsupported,
                E::Decode("bad".into()),
                E::RawCodecMissing
            ])
            .code(),
            "decode.failed"
        );
        assert_eq!(
            answers(vec![E::Unsupported, E::Unsupported, E::Unsupported]).code(),
            "import.unsupported"
        );
    }

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
            // rawler, on every system (ADR-0029): its size is the develop's, and every file
            // gives a thumbnail.
            let (w, h) = Rawler.dimensions(&bytes, expected)?;
            let developed = Rawler.decode(&bytes, expected)?;
            assert_eq!(
                (developed.image.width, developed.image.height),
                (w, h),
                "{}",
                path.display()
            );
            let thumb = Rawler.preview(&bytes, expected, 400)?;
            assert!(thumb.image.width.max(thumb.image.height) <= 400);
            // Our own develop reads every Bayer sample, at rawler's size (ADR-0040); the
            // X-Trans one is left to the fallback until session 18.
            if expected == RawKind::Raf {
                assert!(matches!(
                    Own.dimensions(&bytes, expected),
                    Err(CodecError::Unsupported)
                ));
                continue;
            }
            let (w, h) = Own.dimensions(&bytes, expected)?;
            assert_eq!(
                (w, h),
                Rawler.dimensions(&bytes, expected)?,
                "{}",
                path.display()
            );
            let developed = Own.decode(&bytes, expected)?;
            assert_eq!(
                (developed.image.width, developed.image.height),
                (w, h),
                "{}",
                path.display()
            );
            let thumb = Own.preview(&bytes, expected, 400)?;
            assert!(thumb.image.width.max(thumb.image.height) <= 400);
            // Cut short, a file is refused by the whole chain: the system's codec would
            // develop it with the missing part as stripes (LESSONS 2026-11-05).
            let cut = &bytes[..bytes.len() * 4 / 10];
            assert!(
                decode(cut, expected).is_err(),
                "{} cut short",
                path.display()
            );
        }
        Ok(())
    }

    #[test]
    fn raw_names_are_camera_formats_not_tiff_or_jpeg() {
        for name in [
            "P1067968.RW2",
            "a.cr3",
            "b.NEF",
            "c.dng",
            "d.arw",
            "e.RAF",
            "f.orf",
            "g.CR2",
        ] {
            assert!(is_raw_name(name), "{name}");
        }
        for name in ["scan.tif", "x.jpg", "y.HEIC", "z.png", "rw2", "a.rw2.jpg"] {
            assert!(!is_raw_name(name), "{name}");
        }
    }
}
