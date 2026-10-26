// SPDX-License-Identifier: GPL-3.0-only
//! Camera RAW through rawler (LGPL-2.1), so RAW opens on every system with nothing to
//! install (ADR-0029). This crate is the only one allowed an LGPL dependency; Fotollection
//! reaches it through `fotollection_core::raw`, so our own decoder can replace it format by
//! format, and rawler must be gone before any relicensing (ADR-0008).
//!
//! RAW files are untrusted input (SECURITY.md §3): the size is checked before pixels are
//! developed, and a panic inside rawler is caught and reported as an error.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

use image::DynamicImage;
use rawler::decoders::{RawDecodeParams, RawLoader};
use rawler::imgop::develop::RawDevelop;
use rawler::rawsource::RawSource;
use thiserror::Error;

/// An embedded image: 8-bit sRGB, as stored (orientation is the caller's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb8 {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

/// A developed photo: 16-bit sRGB-encoded, as stored, before any tone finishing (rawler
/// applies no exposure curve or lens shading, so the caller finishes it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb16 {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u16>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RawError {
    #[error("this RAW file is not supported: {0}")]
    Unsupported(String),
    #[error("this RAW file could not be decoded: {0}")]
    Decode(String),
    #[error("this RAW file is too large ({0} pixels)")]
    TooLarge(u64),
    #[error("the RAW decoder failed on this file")]
    Panicked,
}

/// Loading the camera list costs a few milliseconds: once per run.
fn loader() -> &'static RawLoader {
    static LOADER: OnceLock<RawLoader> = OnceLock::new();
    LOADER.get_or_init(RawLoader::new)
}

/// Runs `f`, turning a panic inside rawler into an error (needs `panic = "unwind"`).
fn guarded<T>(f: impl FnOnce() -> Result<T, RawError>) -> Result<T, RawError> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(Err(RawError::Panicked))
}

fn to_rgb8(image: &DynamicImage) -> Rgb8 {
    let rgb = image.to_rgb8();
    Rgb8 {
        width: rgb.width(),
        height: rgb.height(),
        rgb: rgb.into_raw(),
    }
}

/// The developed size in pixels (after the camera's default crop), without developing.
pub fn dimensions(bytes: &[u8]) -> Result<(u32, u32), RawError> {
    guarded(|| {
        let source = RawSource::new_from_slice(bytes);
        let decoder = loader()
            .get_decoder(&source)
            .map_err(|e| RawError::Unsupported(e.to_string()))?;
        let raw = decoder
            .raw_image(&source, &RawDecodeParams::default(), true)
            .map_err(|e| RawError::Decode(e.to_string()))?;
        // The same area the develop's last crop keeps.
        let (w, h) = raw
            .crop_area
            .or(raw.active_area)
            .map_or((raw.width, raw.height), |r| (r.d.w, r.d.h));
        let w = u32::try_from(w).map_err(|_| RawError::TooLarge(u64::MAX))?;
        let h = u32::try_from(h).map_err(|_| RawError::TooLarge(u64::MAX))?;
        Ok((w, h))
    })
}

/// Develops the whole photo (demosaic, white balance, camera colour, sRGB), refusing files
/// with more than `max_pixels` before any pixel is developed.
pub fn develop(bytes: &[u8], max_pixels: u64) -> Result<Rgb16, RawError> {
    guarded(|| {
        let source = RawSource::new_from_slice(bytes);
        let decoder = loader()
            .get_decoder(&source)
            .map_err(|e| RawError::Unsupported(e.to_string()))?;
        let params = RawDecodeParams::default();
        let probe = decoder
            .raw_image(&source, &params, true)
            .map_err(|e| RawError::Decode(e.to_string()))?;
        let pixels = (probe.width as u64).saturating_mul(probe.height as u64);
        if pixels > max_pixels {
            return Err(RawError::TooLarge(pixels));
        }
        let raw = decoder
            .raw_image(&source, &params, false)
            .map_err(|e| RawError::Decode(e.to_string()))?;
        let developed = RawDevelop::default()
            .develop_intermediate(&raw)
            .map_err(|e| RawError::Decode(e.to_string()))?
            .to_dynamic_image()
            .ok_or_else(|| RawError::Decode("no image".to_owned()))?;
        let rgb = developed.to_rgb16();
        Ok(Rgb16 {
            width: rgb.width(),
            height: rgb.height(),
            rgb: rgb.into_raw(),
        })
    })
}

/// The largest image the camera embedded in the file (for thumbnails and analysis), when
/// there is one.
pub fn preview(bytes: &[u8]) -> Result<Option<Rgb8>, RawError> {
    guarded(|| {
        let source = RawSource::new_from_slice(bytes);
        let decoder = loader()
            .get_decoder(&source)
            .map_err(|e| RawError::Unsupported(e.to_string()))?;
        let params = RawDecodeParams::default();
        let image = decoder
            .preview_image(&source, &params)
            .ok()
            .flatten()
            .or_else(|| decoder.full_image(&source, &params).ok().flatten())
            .or_else(|| decoder.thumbnail_image(&source, &params).ok().flatten());
        Ok(image.as_ref().map(to_rgb8))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_is_an_error_not_a_crash() {
        for bytes in [
            &b""[..],
            b"not a raw file",
            &[0u8; 4096],
            b"II*\0\x08\0\0\0",
        ] {
            assert!(develop(bytes, 100_000_000).is_err());
            assert!(dimensions(bytes).is_err());
        }
    }
}
