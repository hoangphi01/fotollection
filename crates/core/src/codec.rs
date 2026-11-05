// SPDX-License-Identifier: GPL-3.0-only
//! JPEG decode/encode behind a trait (ADR-0012), so the implementation can be swapped
//! for libjpeg-turbo if benchmarks justify it (ADR-0004). Other input formats (PNG) are
//! decoded here too, outside the trait: callers use [`decode_image`] and
//! [`image_dimensions`], which pick the decoder by magic bytes.

use std::io::Cursor;
use std::sync::Arc;

use jpeg_encoder::{ColorType, Encoder, SamplingFactor};
use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use crate::image::RgbImage;

/// Largest accepted edge, and pixel count (decompression-bomb guard, SECURITY.md §3).
pub const MAX_EDGE: u32 = 30_000;
pub const MAX_PIXELS: u64 = 200_000_000;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("not a decodable image: {0}")]
    Decode(String),
    #[error("image is too large ({0}×{1})")]
    TooLarge(u32, u32),
    #[error("JPEG encoding failed: {0}")]
    Encode(String),
    /// HEIC on Windows without the HEIF/HEVC extensions (ADR-0017).
    #[error("the HEIC codec is not installed")]
    CodecMissing,
    /// Camera RAW on Windows without the Raw Image Extension (ADR-0023).
    #[error("the RAW codec is not installed")]
    RawCodecMissing,
    /// A format this platform cannot decode.
    #[error("this format is not supported here")]
    Unsupported,
    /// A JPEG that ends before its image does: decoders fill the rest with grey and say
    /// nothing, so it is refused before decoding (ERR-IMG-0006).
    #[error("the file ends before the image does")]
    Truncated,
}

impl CodecError {
    /// Stable machine-readable code (SECURITY.md §8).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Decode(_) => "decode.failed",
            Self::TooLarge(..) => "import.too_large",
            Self::Encode(_) => "encode.failed",
            Self::CodecMissing => "import.heic_codec_missing",
            Self::RawCodecMissing => "import.raw_codec_missing",
            Self::Unsupported => "import.unsupported",
            Self::Truncated => "decode.truncated",
        }
    }
}

/// A decoded image plus its embedded ICC profile, if any.
pub struct Decoded {
    pub image: RgbImage,
    pub icc: Option<Vec<u8>>,
}

/// Chroma resolution of an encoded JPEG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chroma {
    /// 4:4:4: full colour resolution, keeps coloured stamp text crisp (GLOSSARY).
    Full,
    /// 4:2:0: half colour resolution in both directions, for images that are only viewed.
    Half,
}

/// How to encode one JPEG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeOptions {
    pub quality: u8,
    pub chroma: Chroma,
    /// A second pass that builds per-image Huffman tables: a few % smaller, noticeably slower.
    pub optimize_huffman: bool,
}

impl EncodeOptions {
    /// Print exports: full chroma (4:4:4) (FR-008), with the standard Huffman tables: the
    /// optimized ones made 20 MP files only 0.9 % smaller but cost a third more time and kept
    /// the whole image's coefficients, about 160 MB a photo (OPTIMIZATION §5, 2026-11-04).
    /// Share exports use 4:2:0 (`share.rs`), previews and thumbnails `screen`.
    pub const fn print(quality: u8) -> Self {
        Self {
            quality,
            chroma: Chroma::Full,
            optimize_huffman: false,
        }
    }

    /// Previews and thumbnails, which are only shown on screen and never printed (NFR-002).
    pub const fn screen(quality: u8) -> Self {
        Self {
            quality,
            chroma: Chroma::Half,
            optimize_huffman: false,
        }
    }
}

pub trait JpegCodec: Send + Sync {
    /// Pixel dimensions from the header only (no pixel decoding).
    fn dimensions(&self, bytes: &[u8]) -> Result<(u32, u32), CodecError>;
    fn decode(&self, bytes: &[u8]) -> Result<Decoded, CodecError>;
    fn encode(
        &self,
        image: &RgbImage,
        options: EncodeOptions,
        icc: Option<&[u8]>,
    ) -> Result<Vec<u8>, CodecError>;
}

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// HEIF/HEIC: an ISO-BMFF `ftyp` box at offset 4 (EXTENSION.md §4). Canon's CR3 uses the
/// same container and is told apart by `raw::kind` first.
fn is_heif(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[4..8] == b"ftyp"
}

fn os_error(e: fotollection_oscodec::OsCodecError) -> CodecError {
    use fotollection_oscodec::OsCodecError as E;
    match e {
        E::Unsupported => CodecError::Unsupported,
        E::CodecMissing => CodecError::CodecMissing,
        E::TooLarge(w, h) => CodecError::TooLarge(w, h),
        E::Decode(why) => CodecError::Decode(why),
    }
}

/// HEIC through the operating system's codec (ADR-0017), checked against the same limits.
fn decode_heif(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let image = fotollection_oscodec::decode(
        bytes,
        fotollection_oscodec::Format::Heif,
        fotollection_oscodec::Limits {
            max_edge: MAX_EDGE,
            max_pixels: MAX_PIXELS,
        },
    )
    .map_err(os_error)?;
    let Some(rgb) = RgbImage::from_raw(image.width, image.height, image.rgb) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    Ok(Decoded {
        image: rgb,
        icc: image.icc,
    })
}

/// Decodes any supported input photo: PNG, camera RAW (developed) and HEIC here, everything
/// else through `codec` (JPEG).
pub fn decode_image(codec: &dyn JpegCodec, bytes: &[u8]) -> Result<Decoded, CodecError> {
    decode_photo(codec, bytes, crate::raw::RawLook::Neutral)
}

/// [`decode_image`] for a photo with edits: a camera RAW is finished as `look` asks
/// (ADR-0040); every other format ignores it.
pub fn decode_photo(
    codec: &dyn JpegCodec,
    bytes: &[u8],
    look: crate::raw::RawLook,
) -> Result<Decoded, CodecError> {
    if bytes.starts_with(PNG_MAGIC) {
        decode_png(bytes)
    } else if let Some(kind) = crate::raw::kind(bytes) {
        crate::raw::decode(bytes, kind, look)
    } else if is_heif(bytes) {
        decode_heif(bytes)
    } else {
        codec.decode(bytes)
    }
}

/// Decodes for the screen, where at most `max_px` on the long edge is needed: a camera RAW
/// gives its embedded preview instead of a full develop (ADR-0023), every other format
/// decodes as usual. Never use the result for geometry: a RAW's preview can have another
/// size than the photo.
pub fn decode_for_screen(
    codec: &dyn JpegCodec,
    bytes: &[u8],
    max_px: u32,
) -> Result<Decoded, CodecError> {
    match crate::raw::kind(bytes) {
        Some(kind) => crate::raw::preview(bytes, kind, max_px),
        None => decode_image(codec, bytes),
    }
}

/// Pixel dimensions of any supported input photo, from the header only.
pub fn image_dimensions(codec: &dyn JpegCodec, bytes: &[u8]) -> Result<(u32, u32), CodecError> {
    if bytes.starts_with(PNG_MAGIC) {
        let reader = png::Decoder::new(Cursor::new(bytes))
            .read_info()
            .map_err(|e| CodecError::Decode(e.to_string()))?;
        let info = reader.info();
        Ok((info.width, info.height))
    } else if let Some(kind) = crate::raw::kind(bytes) {
        crate::raw::dimensions(bytes, kind)
    } else if is_heif(bytes) {
        fotollection_oscodec::dimensions(bytes, fotollection_oscodec::Format::Heif)
            .map_err(os_error)
    } else {
        codec.dimensions(bytes)
    }
}

fn check_size(width: u32, height: u32) -> Result<(), CodecError> {
    if width == 0 || height == 0 {
        return Err(CodecError::Decode("empty image".into()));
    }
    if width > MAX_EDGE || height > MAX_EDGE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(CodecError::TooLarge(width, height));
    }
    Ok(())
}

/// The 4 × 4 Bayer matrix, for ordered dithering: thresholds 0–15, spread so neighbours differ.
const BAYER_4: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

/// A sample as 0–1: 8-bit as stored, 16-bit big-endian.
fn sample(buf: &[u8], sixteen: bool, i: usize) -> f32 {
    if sixteen {
        f32::from(u16::from_be_bytes([buf[2 * i], buf[2 * i + 1]])) / 65535.0
    } else {
        f32::from(buf[i]) / 255.0
    }
}

/// Steps of the linear → sRGB table: fine enough that the darkest step is under one 8-bit
/// level.
const ENCODE_STEPS: usize = 16_384;

/// sRGB curve tables, built once: decode by 16-bit value, encode by linear step.
struct Curves {
    decode: Vec<f32>,
    encode: Vec<f32>,
}

fn curves() -> &'static Curves {
    static CURVES: std::sync::OnceLock<Curves> = std::sync::OnceLock::new();
    CURVES.get_or_init(|| {
        #[allow(clippy::cast_precision_loss)]
        let decode = (0..=u16::MAX)
            .map(|v| {
                let v = f32::from(v) / 65535.0;
                if v <= 0.040_45 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            })
            .collect();
        #[allow(clippy::cast_precision_loss)]
        let encode = (0..=ENCODE_STEPS)
            .map(|i| {
                let v = i as f32 / ENCODE_STEPS as f32;
                if v <= 0.003_130_8 {
                    12.92 * v
                } else {
                    1.055 * v.powf(1.0 / 2.4) - 0.055
                }
            })
            .collect();
        Curves { decode, encode }
    })
}

impl Curves {
    fn decode(&self, v: f32) -> f32 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        self.decode[(v * 65535.0).round() as usize & 0xFFFF]
    }

    fn encode(&self, linear: f32) -> f32 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let i = (linear.clamp(0.0, 1.0) * ENCODE_STEPS as f32).round() as usize;
        self.encode[i.min(ENCODE_STEPS)]
    }
}

/// PNG → RGB8 (FR-001, FR-072): palette and low-bit images are expanded; 16-bit samples
/// come down to 8 bits through a 4 × 4 ordered dither, so a smooth gradient keeps no bands;
/// alpha is flattened onto white in linear light, as a print would show it. The iCCP
/// profile is kept; without one, `gAMA` and `cHRM` (unless `sRGB` says it is sRGB) are
/// turned into sRGB here, as no profile can carry them on.
fn decode_png(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder
        .read_info()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let (width, height) = (reader.info().width, reader.info().height);
    // Checked before the pixel buffer is allocated (decompression-bomb guard).
    check_size(width, height)?;
    let info = reader.info();
    let icc = info.icc_profile.as_ref().map(|p| p.to_vec());
    let declared = (info.srgb.is_none() && icc.is_none()).then(|| {
        (
            // The chunks as read: `source_gamma` is only filled when encoding.
            info.gama_chunk.map(png::ScaledFloat::into_value),
            info.chrm_chunk,
        )
    });
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| CodecError::Decode("unknown buffer size".into()))?;
    let mut buf = vec![0; size];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    buf.truncate(frame.buffer_size());
    let sixteen = frame.bit_depth == png::BitDepth::Sixteen;
    let channels = match frame.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return Err(CodecError::Decode("unexpanded palette".into())),
    };
    let (w, h) = (width as usize, height as usize);
    if w == 0 || buf.len() < w * h * channels * if sixteen { 2 } else { 1 } {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    }
    let alpha = channels % 2 == 0;
    let curves = curves();
    // 8-bit without alpha needs nothing done: the common screenshot, as fast as before.
    let data = match (sixteen, channels) {
        (false, 3) => buf,
        (false, 1) => buf.iter().flat_map(|&v| [v, v, v]).collect(),
        _ => flatten(&buf, w, h, channels, sixteen, alpha, curves),
    };
    let Some(mut image) = RgbImage::from_raw(width, height, data) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    if let Some((gamma, chromaticities)) = declared {
        crate::colour::declared_to_srgb(&mut image, gamma, chromaticities);
    }
    Ok(Decoded { image, icc })
}

/// 16-bit or alpha PNG samples to RGB8: dithered, flattened onto white in linear light.
fn flatten(
    buf: &[u8],
    w: usize,
    h: usize,
    channels: usize,
    sixteen: bool,
    alpha: bool,
    curves: &Curves,
) -> Vec<u8> {
    use rayon::prelude::*;
    let mut data = vec![0u8; w * h * 3];
    data.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let p = (y * w + x) * channels;
            let a = if alpha {
                sample(buf, sixteen, p + channels - 1)
            } else {
                1.0
            };
            if !sixteen && a >= 1.0 {
                for c in 0..3 {
                    row[x * 3 + c] = buf[p + if channels < 3 { 0 } else { c }];
                }
                continue;
            }
            // A fraction of one 8-bit step: ordered for 16-bit samples, a plain rounding for
            // 8-bit ones, which are exact already.
            let dither = if sixteen {
                (f32::from(BAYER_4[(y % 4) * 4 + x % 4]) + 0.5) / 16.0
            } else {
                0.5
            };
            for c in 0..3 {
                let v = sample(buf, sixteen, p + if channels < 3 { 0 } else { c });
                let v = if a < 1.0 {
                    curves.encode(curves.decode(v) * a + (1.0 - a))
                } else {
                    v
                };
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    row[x * 3 + c] = (v * 255.0 + dither).floor().clamp(0.0, 255.0) as u8;
                }
            }
        }
    });
    data
}

/// The codec used by the app today.
pub fn default_codec() -> Arc<dyn JpegCodec> {
    Arc::new(PureRustCodec)
}

/// `zune-jpeg` for decoding the layouts it has proven, `jpeg-decoder` for the rest and when
/// zune fails (ADR-0033), `jpeg-encoder` for encoding. No C dependencies.
pub struct PureRustCodec;

/// The fast decoder, for the layouts `jpeg_layout` says it handles.
fn decode_zune(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let options = DecoderOptions::default()
        .jpeg_set_out_colorspace(ColorSpace::RGB)
        .set_max_width(MAX_EDGE as usize)
        .set_max_height(MAX_EDGE as usize);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);
    decoder
        .decode_headers()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| CodecError::Decode("missing image header".into()))?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(CodecError::TooLarge(width, height));
    }

    let pixels = decoder
        .decode()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let Some(image) = RgbImage::from_raw(width, height, pixels) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    Ok(Decoded {
        image,
        icc: decoder.icc_profile(),
    })
}

/// The reference decoder (ADR-0033): slower, and right on every layout the corpus holds.
fn decode_reference(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder
        .read_info()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| CodecError::Decode("missing image header".into()))?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    check_size(width, height)?;
    let pixels = decoder
        .decode()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let rgb = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels,
        jpeg_decoder::PixelFormat::L8 => pixels.iter().flat_map(|&l| [l, l, l]).collect(),
        // 16-bit grey, in the machine's byte order (jpeg-decoder's `to_ne_bytes`): its high byte.
        jpeg_decoder::PixelFormat::L16 => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&pair| {
                let [high, _] = u16::from_ne_bytes(pair).to_be_bytes();
                [high, high, high]
            })
            .collect(),
        // jpeg-decoder has already undone Adobe's inversion: each value is the amount of ink,
        // so a channel is what its ink and the key leave of white.
        jpeg_decoder::PixelFormat::CMYK32 => pixels
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let white_k = 255 - u16::from(p[3]);
                p[..3].iter().map(move |&c| {
                    #[allow(clippy::cast_possible_truncation)]
                    let v = ((255 - u16::from(c)) * white_k / 255) as u8;
                    v
                })
            })
            .collect(),
    };
    let Some(image) = RgbImage::from_raw(width, height, rgb) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    Ok(Decoded {
        image,
        icc: decoder.icc_profile(),
    })
}

impl JpegCodec for PureRustCodec {
    fn dimensions(&self, bytes: &[u8]) -> Result<(u32, u32), CodecError> {
        let mut decoder = JpegDecoder::new(ZCursor::new(bytes));
        decoder
            .decode_headers()
            .map_err(|e| CodecError::Decode(e.to_string()))?;
        let info = decoder
            .info()
            .ok_or_else(|| CodecError::Decode("missing image header".into()))?;
        Ok((u32::from(info.width), u32::from(info.height)))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Decoded, CodecError> {
        // A file cut short decodes "fine" with grey where the data stopped: refuse it.
        if crate::jpeg_layout::ends_early(bytes) {
            return Err(CodecError::Truncated);
        }
        // A layout zune-jpeg has not proven, or a header we cannot read, goes to the reference
        // decoder: a wrong decode comes back without an error, so it cannot be caught
        // afterwards (ADR-0033).
        let proven = crate::jpeg_layout::read(bytes).is_some_and(|l| l.zune_proven());
        let decoded = if proven {
            match decode_zune(bytes) {
                Ok(decoded) => Ok(decoded),
                Err(e @ CodecError::TooLarge(..)) => Err(e),
                // A file zune refuses may still be a valid JPEG.
                Err(e) => decode_reference(bytes).map_err(|_| e),
            }
        } else {
            decode_reference(bytes)
        };
        // The pixels are RGB now: a grey or CMYK profile no longer describes them, and
        // embedding it in an RGB print would make the lab read them wrong.
        decoded.map(|d| Decoded {
            icc: d.icc.filter(|p| crate::colour::is_rgb_profile(p)),
            ..d
        })
    }

    fn encode(
        &self,
        image: &RgbImage,
        options: EncodeOptions,
        icc: Option<&[u8]>,
    ) -> Result<Vec<u8>, CodecError> {
        let (Ok(width), Ok(height)) = (u16::try_from(image.width), u16::try_from(image.height))
        else {
            return Err(CodecError::TooLarge(image.width, image.height));
        };
        let mut out = Vec::with_capacity(image.data.len() / 8);
        let mut encoder = Encoder::new(&mut out, options.quality.clamp(1, 100));
        encoder.set_sampling_factor(match options.chroma {
            Chroma::Full => SamplingFactor::R_4_4_4,
            Chroma::Half => SamplingFactor::R_4_2_0,
        });
        encoder.set_optimized_huffman_tables(options.optimize_huffman);
        if let Some(icc) = icc {
            encoder
                .add_icc_profile(icc)
                .map_err(|e| CodecError::Encode(e.to_string()))?;
        }
        encoder
            .encode(&image.data, width, height, ColorType::Rgb)
            .map_err(|e| CodecError::Encode(e.to_string()))?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_001_cmyk_reads_the_same_on_both_decoders() {
        // zune is the route (tests/jpeg_layouts.rs); the reference decoder is the fallback
        // when zune refuses a file, so its CMYK conversion must agree, not come out negative.
        let bytes = layout_fixture("cmyk.jpg");
        let (Ok(zune), Ok(reference)) = (decode_zune(&bytes), decode_reference(&bytes)) else {
            panic!("cmyk.jpg did not decode");
        };
        let worst = zune
            .image
            .data
            .iter()
            .zip(&reference.image.data)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(u8::MAX);
        assert!(worst <= 3, "the decoders differ by up to {worst}");
    }

    #[test]
    fn fr_001_a_file_cut_short_is_refused() {
        let codec = PureRustCodec;
        assert!(matches!(
            codec.decode(&layout_fixture("truncated-420.jpg")),
            Err(CodecError::Truncated)
        ));
        assert!(codec.decode(&layout_fixture("baseline-420.jpg")).is_ok());
    }

    #[test]
    fn only_an_rgb_profile_travels_with_rgb_pixels() {
        let mut rgb = vec![0u8; 128];
        rgb[16..20].copy_from_slice(b"RGB ");
        let mut grey = vec![0u8; 128];
        grey[16..20].copy_from_slice(b"GRAY");
        assert!(crate::colour::is_rgb_profile(&rgb));
        assert!(!crate::colour::is_rgb_profile(&grey));
        assert!(!crate::colour::is_rgb_profile(&[0u8; 8]));
    }

    #[test]
    fn roundtrip_keeps_size_and_roughly_the_colour() {
        let codec = PureRustCodec;
        let mut image = RgbImage::new(64, 48);
        for px in image.data.as_chunks_mut::<3>().0 {
            *px = [200, 100, 50];
        }
        let jpeg = codec
            .encode(&image, EncodeOptions::print(95), None)
            .unwrap_or_default();
        let decoded = codec.decode(&jpeg).map(|d| d.image);
        let decoded = decoded.unwrap_or_else(|_| RgbImage::new(0, 0));
        assert_eq!((decoded.width, decoded.height), (64, 48));
        let px = &decoded.data[..3];
        assert!(
            px.iter()
                .zip([200u8, 100, 50])
                .all(|(a, b)| a.abs_diff(b) <= 3)
        );
    }

    fn png_bytes(
        (width, height): (u32, u32),
        color: png::ColorType,
        depth: png::BitDepth,
        pixels: &[u8],
        icc: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut info = png::Info::with_size(width, height);
        info.color_type = color;
        info.bit_depth = depth;
        info.icc_profile = icc.map(|p| std::borrow::Cow::Owned(p.to_vec()));
        let mut out = Vec::new();
        if let Ok(encoder) = png::Encoder::with_info(&mut out, info)
            && let Ok(mut writer) = encoder.write_header()
        {
            let _ = writer.write_image_data(pixels);
        }
        out
    }

    fn decode_png_pixels(bytes: &[u8]) -> Vec<u8> {
        decode_image(&PureRustCodec, bytes).map_or_else(|_| Vec::new(), |d| d.image.data)
    }

    /// FR-072: a 16-bit value between two 8-bit levels dithers to both, its mean kept (a
    /// smooth 16-bit gradient keeps no bands); truncation gave the lower level everywhere.
    #[test]
    fn fr_072_png_16_bit_dithers_to_8() {
        // 100.25 in 8-bit units, as 16-bit grey, on a 4 × 4 block (one dither tile).
        let v: u16 = 25_764; // 100.25 × 65535 / 255, rounded
        let pixels: Vec<u8> = std::iter::repeat_n(v.to_be_bytes(), 16).flatten().collect();
        let bytes = png_bytes(
            (4, 4),
            png::ColorType::Grayscale,
            png::BitDepth::Sixteen,
            &pixels,
            None,
        );
        let out = decode_png_pixels(&bytes);
        assert_eq!(out.len(), 48);
        let mean = out.iter().map(|&b| f64::from(b)).sum::<f64>() / 48.0;
        assert!((mean - 100.25).abs() < 0.07, "mean {mean}");
        assert!(out.iter().all(|&b| b == 100 || b == 101));
    }

    /// FR-072: a PNG that declares linear light (`gAMA` 1.0) shows as sRGB: linear 0.5 is
    /// sRGB 188; one that declares sRGB's own gamma is left as it is.
    #[test]
    fn fr_072_png_gamma_is_converted() {
        let with_gamma = |gamma: f32| {
            let mut info = png::Info::with_size(1, 1);
            info.color_type = png::ColorType::Rgb;
            info.bit_depth = png::BitDepth::Eight;
            info.source_gamma = Some(png::ScaledFloat::new(gamma));
            let mut out = Vec::new();
            if let Ok(encoder) = png::Encoder::with_info(&mut out, info)
                && let Ok(mut writer) = encoder.write_header()
            {
                let _ = writer.write_image_data(&[128, 128, 128]);
            }
            decode_png_pixels(&out)
        };
        let linear = with_gamma(1.0);
        assert!(linear.iter().all(|&v| v.abs_diff(188) <= 2), "{linear:?}");
        assert_eq!(with_gamma(0.454_55), [128, 128, 128]);
    }

    #[test]
    fn fr_001_png_rgb_decodes() {
        let bytes = png_bytes(
            (2, 1),
            png::ColorType::Rgb,
            png::BitDepth::Eight,
            &[255, 0, 0, 0, 0, 255],
            None,
        );
        assert_eq!(decode_png_pixels(&bytes), [255, 0, 0, 0, 0, 255]);
        let dims = image_dimensions(&PureRustCodec, &bytes).ok();
        assert_eq!(dims, Some((2, 1)));
    }

    #[test]
    fn fr_001_png_alpha_flattens_on_white() {
        // Opaque red, fully transparent, and half-transparent black. Mixed in linear light,
        // half black on white is half the light: 187, not the 127 of mixing sRGB values.
        let bytes = png_bytes(
            (3, 1),
            png::ColorType::Rgba,
            png::BitDepth::Eight,
            &[255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 128],
            None,
        );
        assert_eq!(
            decode_png_pixels(&bytes),
            [255, 0, 0, 255, 255, 255, 187, 187, 187]
        );
    }

    #[test]
    fn fr_001_png_16bit_and_grey() {
        // 16-bit RGB keeps the high byte.
        let rgb16 = png_bytes(
            (1, 1),
            png::ColorType::Rgb,
            png::BitDepth::Sixteen,
            &[0x12, 0x34, 0xAB, 0xCD, 0xFF, 0xFF],
            None,
        );
        assert_eq!(decode_png_pixels(&rgb16), [0x12, 0xAB, 0xFF]);
        let grey = png_bytes(
            (2, 1),
            png::ColorType::Grayscale,
            png::BitDepth::Eight,
            &[10, 200],
            None,
        );
        assert_eq!(decode_png_pixels(&grey), [10, 10, 10, 200, 200, 200]);
    }

    #[test]
    fn fr_001_png_keeps_icc() {
        let profile = b"fake-but-opaque ICC profile bytes";
        let bytes = png_bytes(
            (1, 1),
            png::ColorType::Rgb,
            png::BitDepth::Eight,
            &[1, 2, 3],
            Some(profile),
        );
        let icc = decode_image(&PureRustCodec, &bytes)
            .ok()
            .and_then(|d| d.icc);
        assert_eq!(icc.as_deref(), Some(&profile[..]));
    }

    #[test]
    fn fr_001_png_rejects_oversize_and_garbage() {
        // The header claims 40,000 × 1 px: refused before any pixel buffer is allocated.
        let mut huge = png_bytes(
            (1, 1),
            png::ColorType::Rgb,
            png::BitDepth::Eight,
            &[0; 3],
            None,
        );
        huge[16..20].copy_from_slice(&40_000u32.to_be_bytes());
        // Fix the IHDR CRC so only the size check can fail.
        let crc = crc32(&huge[12..29]);
        huge[29..33].copy_from_slice(&crc.to_be_bytes());
        assert!(matches!(
            decode_image(&PureRustCodec, &huge),
            Err(CodecError::TooLarge(40_000, 1))
        ));

        let mut garbage = PNG_MAGIC.to_vec();
        garbage.extend_from_slice(b"not really a png");
        assert!(matches!(
            decode_image(&PureRustCodec, &garbage),
            Err(CodecError::Decode(_))
        ));
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in bytes {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    /// Luma sampling factors from the first SOF marker: (2, 2) means 4:2:0, (1, 1) 4:4:4.
    fn luma_sampling(jpeg: &[u8]) -> Option<(u8, u8)> {
        let mut i = 2;
        while i + 4 <= jpeg.len() {
            let (marker, len) = (
                jpeg[i + 1],
                usize::from(u16::from_be_bytes([jpeg[i + 2], jpeg[i + 3]])),
            );
            if matches!(marker, 0xC0..=0xC2) {
                let f = *jpeg.get(i + 11)?;
                return Some((f >> 4, f & 0x0F));
            }
            i += 2 + len;
        }
        None
    }

    #[test]
    fn fr_008_prints_keep_full_chroma_and_previews_halve_it() {
        let image = RgbImage::new(64, 48);
        let print = PureRustCodec.encode(&image, EncodeOptions::print(95), None);
        let screen = PureRustCodec.encode(&image, EncodeOptions::screen(85), None);
        assert_eq!(print.ok().as_deref().and_then(luma_sampling), Some((1, 1)));
        assert_eq!(screen.ok().as_deref().and_then(luma_sampling), Some((2, 2)));
    }

    #[test]
    fn fr_011_garbage_is_a_decode_error_not_a_panic() {
        let result = PureRustCodec.decode(b"\xFF\xD8\xFFnot really a jpeg");
        assert!(matches!(result, Err(CodecError::Decode(_))));
    }

    fn layout_fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/jpeg")
            .join(name);
        std::fs::read(path).unwrap_or_default()
    }

    #[test]
    fn fr_001_the_reference_decoder_reads_every_layout_like_zune() -> Result<(), CodecError> {
        // The fallback must be as right as the fast path wherever both can decode.
        for name in [
            "baseline-444.jpg",
            "baseline-420.jpg",
            "baseline-440.jpg",
            "baseline-411.jpg",
            "progressive-420.jpg",
            "restart-420.jpg",
            "grey.jpg",
        ] {
            let bytes = layout_fixture(name);
            let (zune, reference) = (decode_zune(&bytes)?, decode_reference(&bytes)?);
            assert_eq!(zune.image.width, reference.image.width, "{name}");
            let diff: u64 = zune
                .image
                .data
                .iter()
                .zip(&reference.image.data)
                .map(|(a, b)| u64::from(a.abs_diff(*b)))
                .sum();
            let mean = diff / zune.image.data.len() as u64;
            assert!(mean <= 2, "{name}: mean difference {mean}");
        }
        Ok(())
    }

    #[test]
    fn fr_001_our_own_small_flat_jpegs_keep_their_colour() -> Result<(), CodecError> {
        // zune-jpeg 0.5.15 darkens small flat images with optimised Huffman tables
        // (zune-image #496); our thumbnails are such files.
        for colour in [[0, 255, 1], [200, 30, 30], [250, 250, 249], [13, 13, 13]] {
            let mut image = RgbImage::new(16, 16);
            for px in image.data.as_chunks_mut::<3>().0 {
                px.copy_from_slice(&colour);
            }
            for options in [EncodeOptions::print(90), EncodeOptions::screen(82)] {
                let jpeg = PureRustCodec.encode(&image, options, None)?;
                let back = PureRustCodec.decode(&jpeg)?;
                let centre = &back.image.data[(8 * 16 + 8) * 3..(8 * 16 + 8) * 3 + 3];
                for (got, want) in centre.iter().zip(colour) {
                    assert!(got.abs_diff(want) <= 6, "{colour:?} came back {centre:?}");
                }
            }
        }
        Ok(())
    }
}
