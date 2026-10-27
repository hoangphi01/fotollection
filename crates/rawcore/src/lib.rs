// SPDX-License-Identifier: GPL-3.0-only
//! Fotollection's own camera RAW decoder (ADR-0030): pure Rust, no `unsafe`, written from
//! public specifications (DNG 1.7, TIFF 6.0, ITU T.81, the Malvar–He–Cutler paper) and our
//! own notes on formats that have none (`docs/raw/`). No code from rawler, rawloader, dcraw,
//! `LibRaw` or `RawTherapee` is read into it.
//!
//! RAW files are untrusted input: every offset, count and size is checked against the bytes
//! there, and a bad file is an error, never a panic.

pub mod cfa;
pub mod color;
pub mod demosaic;
pub mod dng;
pub mod ljpeg;
pub mod tiff;
pub mod tone;

use rayon::prelude::*;
use thiserror::Error;

pub use tone::{
    Linear, Look, Picture, Reference, default_curve, finish, finish_look, match_look,
    match_reference,
};

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum RawError {
    #[error("the file is cut short or points outside itself")]
    Truncated,
    #[error("this file is not a supported RAW: {0}")]
    Unsupported(&'static str),
    #[error("the file is damaged: {0}")]
    Corrupt(&'static str),
    #[error("the photo is too large ({0} pixels)")]
    TooLarge(u64),
}

/// Whether this decoder reads the file (DNG for now; ARW and RW2 follow, ADR-0030).
pub fn supports(data: &[u8]) -> bool {
    dng::is_dng(data)
}

/// The developed size: the camera's default crop.
pub fn dimensions(data: &[u8]) -> Result<(usize, usize), RawError> {
    Ok(dng::parse(data)?.mosaic.size())
}

/// Develops the photo to scene-linear sRGB (demosaic, white balance, the colour model);
/// `finish` gives it its tone.
pub fn develop(data: &[u8]) -> Result<Linear, RawError> {
    let dng = dng::parse(data)?;
    let rendering =
        color::rendering(&dng.color).ok_or(RawError::Unsupported("no colour matrix"))?;
    let camera = demosaic::demosaic(&dng.mosaic, rendering.gains, 1.0);
    let m = rendering.to_srgb;
    #[allow(clippy::cast_possible_truncation)]
    let m32 = m.map(|row| row.map(|v| v as f32));
    let mut rgb = camera.rgb;
    rgb.par_chunks_exact_mut(3).for_each(|p| {
        let (r, g, b) = (p[0], p[1], p[2]);
        for (i, row) in m32.iter().enumerate() {
            p[i] = (row[0] * r + row[1] * g + row[2] * b).max(0.0);
        }
    });
    Ok(Linear {
        width: camera.width,
        height: camera.height,
        rgb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The CC0 samples (`scripts/fetch-raw-samples.py`) when `FOTOLLECTION_RAW_SAMPLES` names
    /// their folder: each file this decoder reads develops at the size `dimensions` gives, and
    /// cut short at 256 places it fails cleanly, never panics. `examples/mutate.rs` goes further.
    #[test]
    fn samples_develop_at_their_size_and_truncation_never_panics()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(dir) = std::env::var_os("FOTOLLECTION_RAW_SAMPLES") else {
            return Ok(());
        };
        for entry in std::fs::read_dir(dir)? {
            let bytes = std::fs::read(entry?.path())?;
            if !supports(&bytes) {
                continue;
            }
            let (w, h) = dimensions(&bytes)?;
            let linear = develop(&bytes)?;
            assert_eq!((linear.width, linear.height), (w, h));
            for i in 0..256 {
                let cut = bytes.len() * i / 256;
                let short = &bytes[..cut];
                let _ = dimensions(short);
                assert!(
                    develop(short).is_err() || cut > bytes.len() / 2,
                    "cut at {cut} still developed"
                );
            }
        }
        Ok(())
    }
}
