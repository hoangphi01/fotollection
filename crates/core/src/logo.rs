// SPDX-License-Identifier: GPL-3.0-only
//! User-supplied camera logos (FR-040, ADR-0014). Logos are untrusted input: magic bytes,
//! a size cap and a pixel cap are checked before decoding with a pure-Rust decoder
//! (SECURITY.md §3).

use std::io::Cursor;

use png::{ColorType, Decoder, Limits, Transformations};

pub const MAX_LOGO_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_LOGO_EDGE: u32 = 4096;

/// Straight (non-premultiplied) RGBA, 4 bytes per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LogoError {
    #[error("the logo must be a PNG file")]
    NotPng,
    #[error("the logo file is larger than 2 MB")]
    TooLarge,
    #[error("the logo is larger than {MAX_LOGO_EDGE} px")]
    TooManyPixels,
    #[error("the logo could not be decoded: {0}")]
    Decode(String),
}

impl LogoError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotPng => "logo.not_png",
            Self::TooLarge | Self::TooManyPixels => "logo.too_large",
            Self::Decode(_) => "logo.decode_failed",
        }
    }
}

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Validates and decodes a PNG logo into RGBA.
pub fn decode_png(bytes: &[u8]) -> Result<LogoImage, LogoError> {
    if !bytes.starts_with(PNG_MAGIC) {
        return Err(LogoError::NotPng);
    }
    if bytes.len() > MAX_LOGO_BYTES {
        return Err(LogoError::TooLarge);
    }
    let mut decoder = Decoder::new_with_limits(
        Cursor::new(bytes),
        Limits {
            bytes: 80 * 1024 * 1024,
        },
    );
    decoder.set_transformations(Transformations::normalize_to_color8() | Transformations::ALPHA);
    let mut reader = decoder
        .read_info()
        .map_err(|e| LogoError::Decode(e.to_string()))?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width == 0 || height == 0 || width > MAX_LOGO_EDGE || height > MAX_LOGO_EDGE {
        return Err(LogoError::TooManyPixels);
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| LogoError::Decode("unknown buffer size".into()))?;
    let mut buf = vec![0; size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| LogoError::Decode(e.to_string()))?;
    buf.truncate(info.buffer_size());

    let rgba = match info.color_type {
        ColorType::Rgba => buf,
        ColorType::GrayscaleAlpha => buf
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[g, a]| [g, g, g, a])
            .collect(),
        ColorType::Rgb => buf
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[r, g, b]| [r, g, b, 255])
            .collect(),
        ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        ColorType::Indexed => return Err(LogoError::Decode("unexpanded palette".into())),
    };
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(LogoError::Decode("unexpected pixel layout".into()));
    }
    Ok(LogoImage {
        width,
        height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(width: u32, height: u32, color: ColorType, pixels: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, width, height);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            let Ok(mut writer) = encoder.write_header() else {
                return out;
            };
            let _ = writer.write_image_data(pixels);
        }
        out
    }

    #[test]
    fn fr_040_logo_decodes_rgb_and_gray_alpha_to_rgba() {
        let rgb = png_bytes(2, 1, ColorType::Rgb, &[255, 0, 0, 0, 0, 255]);
        let logo = decode_png(&rgb);
        assert_eq!(
            logo.map(|l| l.rgba),
            Ok(vec![255, 0, 0, 255, 0, 0, 255, 255])
        );

        let ga = png_bytes(1, 1, ColorType::GrayscaleAlpha, &[100, 50]);
        assert_eq!(decode_png(&ga).map(|l| l.rgba), Ok(vec![100, 100, 100, 50]));
    }

    #[test]
    fn fr_040_logo_rejects_non_png_and_garbage() {
        assert_eq!(decode_png(b"GIF89a...."), Err(LogoError::NotPng));
        let mut fake = PNG_MAGIC.to_vec();
        fake.extend_from_slice(b"not really a png");
        assert!(matches!(decode_png(&fake), Err(LogoError::Decode(_))));
    }
}
