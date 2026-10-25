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
    /// A format this platform cannot decode.
    #[error("this format is not supported here")]
    Unsupported,
}

impl CodecError {
    /// Stable machine-readable code (SECURITY.md §8).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Decode(_) => "decode.failed",
            Self::TooLarge(..) => "import.too_large",
            Self::Encode(_) => "encode.failed",
            Self::CodecMissing => "import.heic_codec_missing",
            Self::Unsupported => "import.unsupported",
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
    /// Files that leave the app: full chroma, smallest file (FR-008).
    pub const fn print(quality: u8) -> Self {
        Self {
            quality,
            chroma: Chroma::Full,
            optimize_huffman: true,
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

/// HEIF/HEIC: an ISO-BMFF `ftyp` box at offset 4 (EXTENSION.md §4).
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

/// Decodes any supported input photo: PNG and HEIC here, everything else through `codec`
/// (JPEG).
pub fn decode_image(codec: &dyn JpegCodec, bytes: &[u8]) -> Result<Decoded, CodecError> {
    if bytes.starts_with(PNG_MAGIC) {
        decode_png(bytes)
    } else if is_heif(bytes) {
        decode_heif(bytes)
    } else {
        codec.decode(bytes)
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

/// PNG → RGB8 (FR-001): 16-bit, grey and palette images are normalized to 8-bit colour,
/// and alpha is flattened onto white, as a print would show it. The iCCP profile is kept.
fn decode_png(bytes: &[u8]) -> Result<Decoded, CodecError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    let (width, height) = (reader.info().width, reader.info().height);
    // Checked before the pixel buffer is allocated (decompression-bomb guard).
    check_size(width, height)?;
    let icc = reader.info().icc_profile.as_ref().map(|p| p.to_vec());
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| CodecError::Decode("unknown buffer size".into()))?;
    let mut buf = vec![0; size];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| CodecError::Decode(e.to_string()))?;
    buf.truncate(frame.buffer_size());

    let over_white = |c: u8, a: u8| {
        let (c, a) = (u16::from(c), u16::from(a));
        // Rounded c·a/255 + 255·(1 − a/255); always ≤ 255.
        u8::try_from((c * a + 255 * (255 - a) + 127) / 255).unwrap_or(u8::MAX)
    };
    let data: Vec<u8> = match frame.color_type {
        png::ColorType::Rgb => buf,
        png::ColorType::Rgba => buf
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[r, g, b, a]| [over_white(r, a), over_white(g, a), over_white(b, a)])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|&v| [v, v, v]).collect(),
        png::ColorType::GrayscaleAlpha => buf
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[v, a]| {
                let v = over_white(v, a);
                [v, v, v]
            })
            .collect(),
        png::ColorType::Indexed => return Err(CodecError::Decode("unexpanded palette".into())),
    };
    let Some(image) = RgbImage::from_raw(width, height, data) else {
        return Err(CodecError::Decode("unexpected pixel layout".into()));
    };
    Ok(Decoded { image, icc })
}

/// The codec used by the app today.
pub fn default_codec() -> Arc<dyn JpegCodec> {
    Arc::new(PureRustCodec)
}

/// `zune-jpeg` for decoding, `jpeg-encoder` for encoding. No C dependencies.
pub struct PureRustCodec;

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
        // Opaque red, fully transparent, and half-transparent black.
        let bytes = png_bytes(
            (3, 1),
            png::ColorType::Rgba,
            png::BitDepth::Eight,
            &[255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 128],
            None,
        );
        assert_eq!(
            decode_png_pixels(&bytes),
            [255, 0, 0, 255, 255, 255, 127, 127, 127]
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
}
