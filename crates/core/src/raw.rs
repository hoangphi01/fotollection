// SPDX-License-Identifier: GPL-3.0-only
//! Camera RAW input (FR-016, ADR-0023, ADR-0029, ADR-0030): which RAW a file is, from its
//! bytes alone, and its decoders, tried in the order of [`decoders`]: our own DNG reader,
//! rawler, and the operating system's codecs (first after our own on macOS, the last fallback
//! elsewhere). The embedded preview feeds thumbnails and best-shot analysis; the full develop
//! feeds the print preview and export.

pub use fotollection_oscodec::RawKind;

pub use crate::raw_finish::RawLook;
pub use fotollection_rawcore::RawAdjust;

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
/// (RESULT.md §12), so a batch export of RAW stays within the 2 GB ceiling (NFR-004). The
/// photo on screen goes first (FR-091): a background develop waits while any interactive one
/// for the photo on screen waits.
static DEVELOP: Gate = Gate::new();

/// Who is waiting for a develop (FR-091): the photo on screen, or work the user does not wait
/// for (standard previews, prefetch, the export's estimate, an export, an analysis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Something the user waits for now. `Some(photo)` goes first only while that photo is
    /// the one [`show`] last named: a render left behind by fast browsing then waits like
    /// background work. `None` for a view of no single photo (a collage sheet).
    Interactive(Option<u64>),
    Background,
}

thread_local! {
    /// Background unless the caller says otherwise: only what the user waits for claims the
    /// front of the queue (audit 2026-11-06).
    static PRIORITY: std::cell::Cell<Priority> = const { std::cell::Cell::new(Priority::Background) };
}

/// Runs `work` with `priority` for every develop it asks for on this thread. Work for many
/// photos on a pool sets it inside each item, on the thread that runs it.
pub fn with_priority<T>(priority: Priority, work: impl FnOnce() -> T) -> T {
    struct Restore(Priority);
    impl Drop for Restore {
        fn drop(&mut self) {
            PRIORITY.with(|p| p.set(self.0));
        }
    }
    let _restore = Restore(PRIORITY.with(|p| p.replace(priority)));
    work()
}

/// `photo` is now the one on screen (FR-091): develops asked for another photo stop going
/// first. Called when the preview asks for a photo, before its work is queued.
pub fn show(photo: u64) {
    DEVELOP.show(photo);
}

/// No photo shown yet.
const NONE_SHOWN: u64 = u64::MAX;

struct GateState {
    busy: bool,
    /// Waiters that go first now: interactive, for the photo on screen.
    interactive_waiting: usize,
    /// The photo on screen ([`show`]).
    shown: u64,
    /// Every develop waiting for a turn.
    waiting: usize,
}

struct Gate {
    state: std::sync::Mutex<GateState>,
    turn: std::sync::Condvar,
}

/// Holds the develop's turn; giving it back wakes the waiters, even after a panic.
struct Turn<'a>(&'a Gate);

impl Gate {
    const fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(GateState {
                busy: false,
                interactive_waiting: 0,
                shown: NONE_SHOWN,
                waiting: 0,
            }),
            turn: std::sync::Condvar::new(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GateState> {
        // A poisoned lock only means a develop panicked; the state is plain fields.
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn show(&self, photo: u64) {
        let mut s = self.lock();
        if s.shown != photo {
            s.shown = photo;
            drop(s);
            // Waiters for the photo left behind count themselves out.
            self.turn.notify_all();
        }
    }

    fn enter(&self, priority: Priority) -> Turn<'_> {
        let mut s = self.lock();
        s.waiting += 1;
        // Whether this waiter is counted in `interactive_waiting`: only while its photo is
        // still the one on screen, so a render left behind stops holding back the others.
        let mut counted = false;
        loop {
            let urgent = match priority {
                Priority::Interactive(None) => true,
                Priority::Interactive(Some(photo)) => s.shown == photo,
                Priority::Background => false,
            };
            if urgent != counted {
                if urgent {
                    s.interactive_waiting += 1;
                } else {
                    s.interactive_waiting -= 1;
                }
                counted = urgent;
            }
            if !s.busy && (urgent || s.interactive_waiting == 0) {
                break;
            }
            s = self
                .turn
                .wait(s)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        if counted {
            s.interactive_waiting -= 1;
        }
        s.waiting -= 1;
        s.busy = true;
        Turn(self)
    }

    /// How many develops wait for a turn right now, for the tests.
    #[cfg(test)]
    fn waiting(&self) -> usize {
        self.lock().waiting
    }
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        let mut s = self.0.lock();
        s.busy = false;
        drop(s);
        self.0.turn.notify_all();
    }
}

/// One way to read camera RAW (ADR-0029). Fotollection tries them in [`decoders`] order,
/// so our own decoder can take over format by format without changing any caller.
pub trait RawDecoder: Sync {
    /// The full develop in scene-linear light, with what finishing it needs (FR-088).
    fn develop(&self, bytes: &[u8], kind: RawKind) -> Result<Developed, CodecError>;
    /// The full develop, adjusted in linear light and finished as `look` asks (ADR-0040).
    fn decode(
        &self,
        bytes: &[u8],
        kind: RawKind,
        look: RawLook,
        adjust: &RawAdjust,
    ) -> Result<Decoded, CodecError> {
        let d = self.develop(bytes, kind)?;
        let finished =
            crate::raw_finish::finish_linear(d.linear, d.reference.as_ref(), look, adjust, d.toned)
                .ok_or(CodecError::TooLarge(LIMITS.max_edge, LIMITS.max_edge))?;
        Ok(Decoded {
            image: finished.image,
            icc: d.icc,
        })
    }
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
    /// The system's develop, already toned: its neutral is its own rendering, the camera's
    /// look fits one curve per channel of it to the embedded JPEG. Back to linear light from
    /// 8 bits, so the RAW sliders still act before its tone, with no headroom above white.
    fn develop(&self, bytes: &[u8], kind: RawKind) -> Result<Developed, CodecError> {
        intact(bytes, kind)?;
        let format = fotollection_oscodec::Format::Raw(kind);
        let developed =
            decoded(fotollection_oscodec::decode(bytes, format, LIMITS).map_err(error)?)?;
        let table: Vec<f32> = (0..=255u8)
            .map(|v| srgb_to_linear(f32::from(v) / 255.0))
            .collect();
        Ok(Developed {
            linear: fotollection_rawcore::Linear {
                width: developed.image.width as usize,
                height: developed.image.height as usize,
                rgb: developed
                    .image
                    .data
                    .iter()
                    .map(|&v| table[usize::from(v)])
                    .collect(),
            },
            reference: embedded_reference(bytes, kind),
            toned: true,
            icc: developed.icc,
        })
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
    /// rawler's develop (sRGB-encoded, untoned) back to linear light, then finished as our own.
    fn develop(&self, bytes: &[u8], _kind: RawKind) -> Result<Developed, CodecError> {
        let developed =
            fotollection_rawlgpl::develop(bytes, LIMITS.max_pixels).map_err(rawler_error)?;
        let reference = fotollection_rawlgpl::preview(bytes)
            .ok()
            .flatten()
            .and_then(rgb8);
        let table: Vec<f32> = (0..=u16::MAX)
            .map(|v| srgb_to_linear(f32::from(v) / 65535.0))
            .collect();
        Ok(Developed {
            linear: fotollection_rawcore::Linear {
                width: developed.width as usize,
                height: developed.height as usize,
                rgb: developed
                    .rgb
                    .iter()
                    .map(|&v| table[usize::from(v)])
                    .collect(),
            },
            reference,
            toned: false,
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
            None => {
                self.decode(bytes, kind, RawLook::Neutral, &RawAdjust::default())?
                    .image
            }
        };
        let small = crate::render::fit_long_edge(&image, max_px)
            .map_err(|e| CodecError::Decode(e.to_string()))?;
        Ok(Decoded {
            image: small,
            icc: None,
        })
    }
}

/// A RAW developed to scene-linear light, before its adjustments and its finish (FR-088).
pub struct Developed {
    pub linear: fotollection_rawcore::Linear,
    /// The camera's embedded JPEG, which the finish is fitted to.
    pub reference: Option<RgbImage>,
    /// Already toned by the system's codec.
    pub toned: bool,
    pub icc: Option<Vec<u8>>,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Our own scene-linear develop of a Bayer RAW and the camera's embedded JPEG, unfinished:
/// for measuring the finishes (`examples/raw_fits.rs`).
pub fn develop_linear(
    bytes: &[u8],
    kind: RawKind,
) -> Result<(fotollection_rawcore::Linear, Option<RgbImage>), CodecError> {
    Ok((Own::linear(bytes, kind)?, Own::embedded(bytes, kind)))
}

/// The camera's embedded JPEG, whatever reads the file, for fitting the camera's look.
fn embedded_reference(bytes: &[u8], kind: RawKind) -> Option<RgbImage> {
    Own::embedded(bytes, kind)
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
    fn develop(&self, bytes: &[u8], kind: RawKind) -> Result<Developed, CodecError> {
        Ok(Developed {
            linear: Self::linear(bytes, kind)?,
            reference: Self::embedded(bytes, kind),
            toned: false,
            icc: None,
        })
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
            None => {
                self.decode(bytes, kind, RawLook::Neutral, &RawAdjust::default())?
                    .image
            }
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
pub fn decode(
    bytes: &[u8],
    kind: RawKind,
    look: RawLook,
    adjust: &RawAdjust,
) -> Result<Decoded, CodecError> {
    // Image data past the end of the file is refused before any reader makes up the rest.
    if fotollection_rawcore::cut_short(bytes) {
        return Err(CodecError::Decode("the RAW file is cut short".into()));
    }
    one_at_a_time(|| first(kind, |d| d.decode(bytes, kind, look, adjust)))
        .unwrap_or_else(|| Err(CodecError::Decode("the RAW develop failed".into())))
}

/// The develop the preview keeps (FR-088): at most `raw_finish::FIT_EDGE` across, with its
/// fits, so each step of a RAW slider re-finishes only this copy.
pub fn prepare(bytes: &[u8], kind: RawKind) -> Result<Kept, CodecError> {
    prepare_while(bytes, kind, &|| true)?
        .ok_or_else(|| CodecError::Decode("the RAW develop failed".into()))
}

/// A develop the preview keeps, and the profile it finishes to.
pub type Kept = (crate::raw_finish::Prepared, Option<Vec<u8>>);

/// [`prepare`], unless `wanted` says no once the develop's turn comes: `Ok(None)` then, and
/// nothing developed (FR-091). A prefetch the user has browsed past, or a photo whose standard
/// preview another thread wrote while this one waited, costs no develop.
pub fn prepare_while(
    bytes: &[u8],
    kind: RawKind,
    wanted: &(dyn Fn() -> bool + Sync),
) -> Result<Option<Kept>, CodecError> {
    if fotollection_rawcore::cut_short(bytes) {
        return Err(CodecError::Decode("the RAW file is cut short".into()));
    }
    let developed = one_at_a_time_on(&DEVELOP, wanted, || {
        first(kind, |d| {
            let developed = d.develop(bytes, kind)?;
            Ok((
                crate::raw_finish::Prepared::new(
                    &developed.linear,
                    developed.reference,
                    developed.toned,
                ),
                developed.icc,
            ))
        })
    });
    match developed {
        Turned::Done(result) => result.map(Some),
        Turned::NotWanted => Ok(None),
        Turned::Failed => Err(CodecError::Decode("the RAW develop failed".into())),
    }
}

/// The pool a develop's parallel work runs on, used by nobody else: whoever holds `DEVELOP`
/// always has its threads (`None` if the system refused to start them).
fn develop_pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: std::sync::OnceLock<Option<rayon::ThreadPool>> = std::sync::OnceLock::new();
    POOL.get_or_init(|| crate::pool::contained(None, "develop").ok())
        .as_ref()
}

/// Runs `develop` holding `DEVELOP`, on a thread of its own, its parallel work on the
/// develop's own pool: `None` if it panicked.
///
/// Never on the caller's thread. A develop runs rayon work, and a rayon thread waiting for
/// that work takes up other jobs of its pool meanwhile; when one of them asks for a develop
/// too, it waits for a lock its own thread holds, and the pool stops for good (LESSONS
/// 2026-11-03: zooming into a RAW hung every preview after it). A plain thread never takes
/// up other jobs, so whoever holds the lock always finishes; the caller only waits.
///
/// Nor on the global pool: callers on it wait here without lending their thread, and when
/// every one of its threads waits for `DEVELOP`, the develop holding it finds none left for
/// its parallel work and never ends (LESSONS 2026-11-05: the export estimate's samples).
fn one_at_a_time<T: Send>(develop: impl FnOnce() -> T + Send) -> Option<T> {
    match one_at_a_time_on(&DEVELOP, &|| true, develop) {
        Turned::Done(value) => Some(value),
        Turned::NotWanted | Turned::Failed => None,
    }
}

/// What a develop asked of a gate came to.
enum Turned<T> {
    Done(T),
    /// `wanted` said no once the turn came: nothing ran.
    NotWanted,
    /// The develop panicked.
    Failed,
}

/// [`one_at_a_time`] on `gate`, asking `wanted` once the turn is taken.
fn one_at_a_time_on<T: Send>(
    gate: &Gate,
    wanted: &(dyn Fn() -> bool + Sync),
    develop: impl FnOnce() -> T + Send,
) -> Turned<T> {
    // Read on the caller's thread, where the work set it.
    let priority = PRIORITY.with(std::cell::Cell::get);
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _turn = gate.enter(priority);
                if !wanted() {
                    return Turned::NotWanted;
                }
                Turned::Done(match develop_pool() {
                    Some(pool) => pool.install(develop),
                    None => develop(),
                })
            })
            .join()
            .unwrap_or(Turned::Failed)
    })
}

/// The developed size, the only one to use for geometry (crops are fractions of it).
pub fn dimensions(bytes: &[u8], kind: RawKind) -> Result<(u32, u32), CodecError> {
    // Contained (ADR-0048): a reader that panics on this file is its error, not the app's.
    crate::supervise::contained(|| first(kind, |d| d.dimensions(bytes, kind))).unwrap_or_else(
        || {
            Err(CodecError::Decode(
                "the RAW reader failed on this file".into(),
            ))
        },
    )
}

/// The preview the RAW carries, at most `max_px` on its long edge: 30 to 100 times faster
/// than a develop (RESULT.md §12), for thumbnails and best-shot analysis. Its size can
/// differ from `dimensions`.
pub fn preview(bytes: &[u8], kind: RawKind, max_px: u32) -> Result<Decoded, CodecError> {
    crate::supervise::contained(|| first(kind, |d| d.preview(bytes, kind, max_px))).unwrap_or_else(
        || {
            Err(CodecError::Decode(
                "the RAW reader failed on this file".into(),
            ))
        },
    )
}

/// Whether this machine can develop RAW: always, since rawler is built in (ADR-0029).
pub fn available() -> bool {
    true
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests start bare threads to race the code.
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

    /// Develops asked from every thread of the global pool at once, as the export estimate
    /// asks for its samples: each waits for `DEVELOP` without lending its thread, so the
    /// develop holding it must not need that pool for its parallel work (LESSONS 2026-11-05).
    /// Hangs on the code before the develop's own pool.
    #[test]
    fn develops_asked_from_the_whole_global_pool_all_finish() {
        use rayon::prelude::*;
        let asked = 2 * rayon::current_num_threads();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let done = (0..asked)
                .into_par_iter()
                .filter_map(|_| {
                    one_at_a_time(|| (0..400_000u64).into_par_iter().map(|x| x % 7).sum::<u64>())
                })
                .count();
            let _ = tx.send(done);
        });
        let answer = rx.recv_timeout(std::time::Duration::from_secs(30));
        assert_eq!(answer, Ok(asked), "the develops never finished");
    }

    /// Polls `ready` until it holds (10 s at most): the tests wait for threads to be queued
    /// at the gate by counting them, never by guessing a time.
    fn until(ready: impl Fn() -> bool) {
        let start = std::time::Instant::now();
        while !ready() {
            assert!(
                start.elapsed() < std::time::Duration::from_secs(10),
                "timed out"
            );
            std::thread::yield_now();
        }
    }

    /// A develop on `gate` that records `name` in `order` once it runs, then holds the turn
    /// until `release` is sent (or dropped).
    fn queued(
        gate: &'static Gate,
        order: &std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
        name: &'static str,
        priority: Priority,
        release: std::sync::mpsc::Receiver<()>,
    ) -> std::thread::JoinHandle<()> {
        let order = std::sync::Arc::clone(order);
        std::thread::spawn(move || {
            with_priority(priority, || {
                one_at_a_time_on(gate, &|| true, move || {
                    order.lock().map(|mut o| o.push(name)).ok();
                    let _ = release.recv();
                })
            });
        })
    }

    fn order_of(order: &std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>) -> Vec<&'static str> {
        order.lock().map(|o| o.clone()).unwrap_or_default()
    }

    /// FR-091: the photo on screen goes before work the user does not wait for. While one
    /// develop runs, a background develop asks, then an interactive one: the interactive one
    /// runs next. Queued by counting the waiters, not by sleeping (audit 2026-11-06).
    #[test]
    fn fr_091_an_interactive_develop_goes_before_a_background_one() {
        static GATE: Gate = Gate::new();
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let (hold, release) = std::sync::mpsc::channel();
        let first = queued(&GATE, &order, "first", Priority::Background, release);
        until(|| order_of(&order).len() == 1);
        let (_, free) = std::sync::mpsc::channel();
        let background = queued(&GATE, &order, "background", Priority::Background, free);
        until(|| GATE.waiting() == 1);
        let (_, free) = std::sync::mpsc::channel();
        let interactive = queued(
            &GATE,
            &order,
            "interactive",
            Priority::Interactive(None),
            free,
        );
        until(|| GATE.waiting() == 2);
        drop(hold);
        for t in [first, background, interactive] {
            assert!(t.join().is_ok());
        }
        assert_eq!(order_of(&order), ["first", "interactive", "background"]);
    }

    /// FR-091, audit 2026-11-06: a render left behind by fast browsing no longer goes first.
    /// Photo 1's develop waits; the user moves on to photo 2: photo 1's stops counting as
    /// interactive, and photo 2's, asked after it, runs before it.
    #[test]
    fn fr_091_a_render_for_a_photo_left_behind_waits_like_background_work() {
        static GATE: Gate = Gate::new();
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let (hold, release) = std::sync::mpsc::channel();
        let first = queued(&GATE, &order, "first", Priority::Background, release);
        until(|| order_of(&order).len() == 1);
        GATE.show(1);
        let (_, free) = std::sync::mpsc::channel();
        let left = queued(
            &GATE,
            &order,
            "left behind",
            Priority::Interactive(Some(1)),
            free,
        );
        until(|| GATE.lock().interactive_waiting == 1);
        GATE.show(2);
        until(|| GATE.lock().interactive_waiting == 0);
        let (_, free) = std::sync::mpsc::channel();
        let shown = queued(&GATE, &order, "shown", Priority::Interactive(Some(2)), free);
        until(|| GATE.waiting() == 2 && GATE.lock().interactive_waiting == 1);
        drop(hold);
        for t in [first, left, shown] {
            assert!(t.join().is_ok());
        }
        assert_eq!(order_of(&order), ["first", "shown", "left behind"]);
    }

    /// FR-091, audit 2026-11-06: a develop no longer wanted when its turn comes (a prefetch
    /// browsed past) does not run.
    #[test]
    fn fr_091_a_develop_no_longer_wanted_at_its_turn_does_not_run() {
        static GATE: Gate = Gate::new();
        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let (hold, release) = std::sync::mpsc::channel();
        let first = queued(&GATE, &order, "first", Priority::Background, release);
        until(|| order_of(&order).len() == 1);
        let wanted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let asked = {
            let wanted = std::sync::Arc::clone(&wanted);
            std::thread::spawn(move || {
                let check = move || wanted.load(std::sync::atomic::Ordering::SeqCst);
                matches!(one_at_a_time_on(&GATE, &check, || ()), Turned::NotWanted)
            })
        };
        until(|| GATE.waiting() == 1);
        wanted.store(false, std::sync::atomic::Ordering::SeqCst);
        drop(hold);
        assert!(first.join().is_ok());
        assert!(asked.join().unwrap_or(false), "the develop ran");
    }

    /// Audit 2026-11-06: a develop asked by code that set no priority waits behind the photo
    /// on screen; only the preview claims the front.
    #[test]
    fn fr_091_a_develop_is_background_unless_marked() {
        let priority = std::thread::spawn(|| PRIORITY.with(std::cell::Cell::get)).join();
        assert_eq!(priority.ok(), Some(Priority::Background));
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
            let developed =
                Rawler.decode(&bytes, expected, RawLook::Neutral, &RawAdjust::default())?;
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
            let developed = Own.decode(&bytes, expected, RawLook::Camera, &RawAdjust::default())?;
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
                decode(cut, expected, RawLook::Neutral, &RawAdjust::default()).is_err(),
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
