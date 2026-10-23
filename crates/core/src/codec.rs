// SPDX-License-Identifier: GPL-3.0-only
//! JPEG decode/encode behind a trait (ADR-0012), so the implementation can be swapped
//! for libjpeg-turbo if benchmarks justify it (ADR-0004).

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
    #[error("not a decodable JPEG: {0}")]
    Decode(String),
    #[error("image is too large ({0}×{1})")]
    TooLarge(u32, u32),
    #[error("JPEG encoding failed: {0}")]
    Encode(String),
}

impl CodecError {
    /// Stable machine-readable code (SECURITY.md §8).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Decode(_) => "decode.failed",
            Self::TooLarge(..) => "import.too_large",
            Self::Encode(_) => "encode.failed",
        }
    }
}

/// A decoded image plus its embedded ICC profile, if any.
pub struct Decoded {
    pub image: RgbImage,
    pub icc: Option<Vec<u8>>,
}

pub trait JpegCodec: Send + Sync {
    fn decode(&self, bytes: &[u8]) -> Result<Decoded, CodecError>;
    fn encode(
        &self,
        image: &RgbImage,
        quality: u8,
        icc: Option<&[u8]>,
    ) -> Result<Vec<u8>, CodecError>;
}

/// The codec used by the app today.
pub fn default_codec() -> Arc<dyn JpegCodec> {
    Arc::new(PureRustCodec)
}

/// `zune-jpeg` for decoding, `jpeg-encoder` for encoding. No C dependencies.
pub struct PureRustCodec;

impl JpegCodec for PureRustCodec {
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
        quality: u8,
        icc: Option<&[u8]>,
    ) -> Result<Vec<u8>, CodecError> {
        let (Ok(width), Ok(height)) = (u16::try_from(image.width), u16::try_from(image.height))
        else {
            return Err(CodecError::TooLarge(image.width, image.height));
        };
        let mut out = Vec::with_capacity(image.data.len() / 8);
        let mut encoder = Encoder::new(&mut out, quality.clamp(1, 100));
        // 4:4:4 keeps coloured stamp text crisp (GLOSSARY: chroma subsampling).
        encoder.set_sampling_factor(SamplingFactor::R_4_4_4);
        encoder.set_optimized_huffman_tables(true);
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
        let jpeg = codec.encode(&image, 95, None).unwrap_or_default();
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

    #[test]
    fn fr_011_garbage_is_a_decode_error_not_a_panic() {
        let result = PureRustCodec.decode(b"\xFF\xD8\xFFnot really a jpeg");
        assert!(matches!(result, Err(CodecError::Decode(_))));
    }
}
