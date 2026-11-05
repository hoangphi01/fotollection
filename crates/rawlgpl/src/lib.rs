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

/// The colour filter in front of the sensor, as rawler reads it (FR-071).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// A 2×2 red, green and blue pattern, named top-left first (`RGGB`, `BGGR`…).
    Bayer(String),
    /// Fujifilm's 6×6 pattern.
    XTrans,
    /// Another pattern (four colours such as `CYGM`, an unusual size), by its name.
    Other(String),
    /// Already demosaiced samples.
    Linear,
    /// One sample per pixel and no filter.
    Mono,
}

/// The sensor's filter and bit depth, read without developing (FR-071).
pub fn sensor(bytes: &[u8]) -> Result<(Filter, u8), RawError> {
    use rawler::rawimage::RawPhotometricInterpretation as P;
    guarded(|| {
        let source = RawSource::new_from_slice(bytes);
        let decoder = loader()
            .get_decoder(&source)
            .map_err(|e| RawError::Unsupported(e.to_string()))?;
        let raw = decoder
            .raw_image(&source, &RawDecodeParams::default(), true)
            .map_err(|e| RawError::Decode(e.to_string()))?;
        let filter = match &raw.photometric {
            P::Cfa(config) => {
                let cfa = &config.cfa;
                let rgb = cfa.name.chars().all(|c| matches!(c, 'R' | 'G' | 'B'));
                match (cfa.width, cfa.height) {
                    (2, 2) if rgb => Filter::Bayer(cfa.name.clone()),
                    (6, 6) if rgb => Filter::XTrans,
                    _ => Filter::Other(cfa.name.clone()),
                }
            }
            P::LinearRaw => Filter::Linear,
            P::BlackIsZero => Filter::Mono,
        };
        // rawler stores samples in 16 bits; the white level tells the sensor's own depth
        // (16383 is 14 bits).
        let white = raw.whitelevel.0.iter().copied().max().unwrap_or(0);
        let bits = if white > 0 {
            32 - white.leading_zeros()
        } else {
            u32::try_from(raw.bps).unwrap_or(16)
        };
        Ok((filter, u8::try_from(bits).unwrap_or(u8::MAX)))
    })
}

/// The sensor's samples and what is needed to develop them, in our own terms (ADR-0040,
/// FR-072): rawler only unpacks, and Fotollection's core develops. Positions follow DNG:
/// `pattern` and `black` start at the active area's top-left corner.
#[derive(Debug, Clone, PartialEq)]
pub struct Unpacked {
    pub width: usize,
    pub height: usize,
    /// One sample per pixel, row by row.
    pub data: Vec<u16>,
    /// The 2×2 filter colours (0 red, 1 green, 2 blue), row by row.
    pub pattern: [u8; 4],
    /// Black level per pattern position.
    pub black: [f32; 4],
    pub white: f32,
    /// `[top, left, bottom, right]` of the pixels that saw light.
    pub active: [usize; 4],
    /// `[x, y, width, height]` of the camera's default crop, inside the active area.
    pub crop: [usize; 4],
    /// The camera's white balance at the shot, as gains on camera red, green and blue.
    pub white_balance: Option<[f32; 3]>,
    /// XYZ to camera matrices, row by row, with their illuminant as an EXIF `LightSource`.
    pub matrices: Vec<(u16, [f64; 9])>,
}

/// Unpacks a 2×2 RGB mosaic without developing it, refusing files with more than
/// `max_pixels` first. Anything else (X-Trans, four colours, float or demosaiced data, a
/// sensor turned 45°) is `Unsupported`, for another reader to take.
pub fn unpack(bytes: &[u8], max_pixels: u64) -> Result<Unpacked, RawError> {
    use rawler::rawimage::{RawImageData, RawPhotometricInterpretation as P};
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
        let P::Cfa(config) = &probe.photometric else {
            return Err(RawError::Unsupported("not a colour filter mosaic".into()));
        };
        if !(config.cfa.is_rgb() && config.cfa.width == 2 && config.cfa.height == 2) {
            return Err(RawError::Unsupported(format!("filter {}", config.cfa.name)));
        }
        if probe.cpp != 1 || probe.fuji_rotation_width.is_some() {
            return Err(RawError::Unsupported("sensor layout".into()));
        }
        let raw = decoder
            .raw_image(&source, &params, false)
            .map_err(|e| RawError::Decode(e.to_string()))?;
        let P::Cfa(config) = &raw.photometric else {
            return Err(RawError::Unsupported("not a colour filter mosaic".into()));
        };
        let (width, height) = (raw.width, raw.height);
        let full = (0, 0, width, height);
        let (left, top, w, h) = raw
            .active_area
            .map_or(full, |r| (r.p.x, r.p.y, r.d.w, r.d.h));
        // rawler's crop counts from the whole image; ours from the active area.
        let crop = raw.crop_area.map_or([0, 0, w, h], |r| {
            [
                r.p.x.saturating_sub(left),
                r.p.y.saturating_sub(top),
                r.d.w,
                r.d.h,
            ]
        });
        let (pattern, black) = from_active_corner(
            |y, x| config.cfa.color_at(y, x),
            raw.blacklevel.as_bayer_array(),
            top,
            left,
        );
        // One white for every site: the lowest, so no site clips unnoticed.
        #[allow(clippy::cast_precision_loss)]
        let white = raw.whitelevel.0.iter().copied().min().unwrap_or(0) as f32;
        let wb = raw.wb_coeffs;
        let white_balance =
            (wb[..3].iter().all(|v| v.is_finite() && *v > 0.0)).then(|| [wb[0], wb[1], wb[2]]);
        let matrices = raw
            .color_matrix
            .iter()
            .filter_map(|(illuminant, m)| {
                let m: [f32; 9] = m.get(..9)?.try_into().ok()?;
                Some((*illuminant as u16, m.map(f64::from)))
            })
            .collect();
        let RawImageData::Integer(data) = raw.data else {
            return Err(RawError::Unsupported("floating-point samples".into()));
        };
        if data.len() != width * height {
            return Err(RawError::Decode("unexpected sample count".into()));
        }
        Ok(Unpacked {
            width,
            height,
            data,
            pattern,
            black,
            white,
            active: [top, left, top + h, left + w],
            crop,
            white_balance,
            matrices,
        })
    })
}

/// rawler counts the filter pattern and the black levels from the whole image's corner;
/// DNG, and our core, from the active area's (LESSONS: a repeating pattern has an origin).
fn from_active_corner(
    color_at: impl Fn(usize, usize) -> usize,
    levels: [f32; 4],
    top: usize,
    left: usize,
) -> ([u8; 4], [f32; 4]) {
    let at = |i: usize| (top + i / 2, left + i % 2);
    let pattern = [0, 1, 2, 3].map(|i| {
        let (y, x) = at(i);
        u8::try_from(color_at(y, x)).unwrap_or(u8::MAX)
    });
    let black = [0, 1, 2, 3].map(|i| {
        let (y, x) = at(i);
        levels[(y % 2) * 2 + x % 2]
    });
    (pattern, black)
}

/// Develops the whole photo (demosaic, white balance, camera colour, sRGB), refusing files
/// with more than `max_pixels` before any pixel is developed. Only for the formats our own
/// core does not unpack yet (`unpack` refuses them): X-Trans until v0.11.0's session 18.
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

    /// An active area from row 1, column 1 of an RGGB image starts on blue: the pattern and
    /// the black levels both move with it.
    #[test]
    fn fr_072_pattern_counts_from_the_active_corner() {
        let rggb = |y: usize, x: usize| [[0, 1], [1, 2]][y % 2][x % 2];
        let levels = [10.0, 11.0, 12.0, 13.0];
        assert_eq!(
            from_active_corner(rggb, levels, 0, 0),
            ([0, 1, 1, 2], levels)
        );
        assert_eq!(
            from_active_corner(rggb, levels, 1, 1),
            ([2, 1, 1, 0], [13.0, 12.0, 11.0, 10.0])
        );
        assert_eq!(
            from_active_corner(rggb, levels, 0, 1),
            ([1, 0, 2, 1], [11.0, 10.0, 13.0, 12.0])
        );
    }

    #[test]
    fn garbage_is_an_error_not_a_crash() {
        for bytes in [
            &b""[..],
            b"not a raw file",
            &[0u8; 4096],
            b"II*\0\x08\0\0\0",
        ] {
            assert!(develop(bytes, 100_000_000).is_err());
            assert!(unpack(bytes, 100_000_000).is_err());
            assert!(dimensions(bytes).is_err());
        }
    }
}
