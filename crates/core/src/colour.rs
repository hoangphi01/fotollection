// SPDX-License-Identifier: GPL-3.0-only
//! Colour conversion to sRGB (FR-027, FR-053): photos tagged Display P3, Adobe RGB or any
//! other ICC profile are converted, so a site that ignores profiles (or strips them) still
//! shows the right colours. Untagged photos are taken as sRGB. Uses `qcms` (MIT, pure Rust).

use qcms::{DataType, Intent, Profile, Transform};
use rayon::prelude::*;

use crate::image::RgbImage;

/// Rows converted per parallel task.
const ROWS_PER_TASK: usize = 64;

/// What [`to_srgb`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conversion {
    /// The pixels are now sRGB; the old profile must not be embedded any more.
    Converted,
    /// No profile, or already sRGB: nothing to do, nothing to embed.
    AlreadySrgb,
    /// A profile `qcms` cannot read: the pixels are untouched and the profile must be kept,
    /// so viewers still read them right.
    Unreadable,
}

/// Converts `image` from `icc` to sRGB in place.
pub fn to_srgb(image: &mut RgbImage, icc: Option<&[u8]>) -> Conversion {
    let Some(bytes) = icc else {
        return Conversion::AlreadySrgb;
    };
    let Some(source) = Profile::new_from_slice(bytes, false) else {
        return Conversion::Unreadable;
    };
    if source.is_sRGB() {
        return Conversion::AlreadySrgb;
    }
    let Some(transform) = Transform::new(
        &source,
        &Profile::new_sRGB(),
        DataType::RGB8,
        Intent::Perceptual,
    ) else {
        return Conversion::Unreadable;
    };
    let row = image.width as usize * 3;
    if row == 0 {
        return Conversion::AlreadySrgb;
    }
    let transform = &transform;
    image
        .data
        .par_chunks_mut(row * ROWS_PER_TASK)
        .for_each(|rows| transform.apply(rows));
    Conversion::Converted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_027_untagged_and_srgb_photos_are_left_alone() {
        let mut image = RgbImage::new(4, 4);
        image.data.fill(120);
        assert_eq!(to_srgb(&mut image, None), Conversion::AlreadySrgb);
        assert_eq!(
            to_srgb(&mut image, Some(b"not a profile")),
            Conversion::Unreadable
        );
        assert!(image.data.iter().all(|&v| v == 120));
    }
}
