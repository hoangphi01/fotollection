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

/// True when an ICC profile describes RGB data (its header's colour space is `RGB `). A grey
/// or CMYK profile cannot travel with pixels the decoder has turned into RGB.
pub fn is_rgb_profile(icc: &[u8]) -> bool {
    icc.get(16..20) == Some(b"RGB ")
}

/// An ICC profile's description (its `desc` tag), the name a viewer shows: "sRGB IEC61966-2.1",
/// "Display P3", "Adobe RGB (1998)". Reads version 2 (`desc`, ASCII) and version 4 (`mluc`,
/// UTF-16, the first record) profiles; every offset is checked, so a broken profile gives
/// `None`, never a panic (FR-071, SECURITY.md §3).
pub fn profile_name(icc: &[u8]) -> Option<String> {
    let be = |at: usize| {
        icc.get(at..at.checked_add(4)?)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_be_bytes)
    };
    // The tag table follows the 128-byte header: a count, then (signature, offset, size).
    let count = be(128)?.min(256) as usize;
    let (offset, size) = (0..count).find_map(|i| {
        let at = 132 + 12 * i;
        (icc.get(at..at + 4)? == b"desc")
            .then(|| Some((be(at + 4)? as usize, be(at + 8)? as usize)))?
    })?;
    let tag = icc.get(offset..offset.checked_add(size)?)?;
    let text = match tag.get(..4)? {
        b"desc" => {
            let len = u32::from_be_bytes(tag.get(8..12)?.try_into().ok()?) as usize;
            let ascii = tag.get(12..12usize.checked_add(len)?)?;
            String::from_utf8_lossy(ascii)
                .trim_end_matches('\0')
                .to_owned()
        }
        b"mluc" => {
            let len = u32::from_be_bytes(tag.get(20..24)?.try_into().ok()?) as usize;
            let at = u32::from_be_bytes(tag.get(24..28)?.try_into().ok()?) as usize;
            let units: Vec<u16> = tag
                .get(at..at.checked_add(len)?)?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_be_bytes(*c))
                .collect();
            String::from_utf16_lossy(&units)
                .trim_end_matches('\0')
                .to_owned()
        }
        _ => return None,
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.chars().take(64).collect())
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
pub(crate) mod tests {
    /// A minimal profile: the header with `space` as its colour space, then one `desc` tag of
    /// the given type, as version 2 (`desc`) and version 4 (`mluc`) profiles write it.
    pub(crate) fn named_profile(space: [u8; 4], name: &str, v4: bool) -> Vec<u8> {
        let mut tag = Vec::new();
        if v4 {
            let text: Vec<u8> = name.encode_utf16().flat_map(u16::to_be_bytes).collect();
            tag.extend_from_slice(b"mluc\0\0\0\0");
            for v in [1u32, 12, 0, 0, 0] {
                tag.extend_from_slice(&v.to_be_bytes());
            }
            tag[16..20].copy_from_slice(b"enUS");
            tag[20..24].copy_from_slice(&u32::try_from(text.len()).unwrap_or(0).to_be_bytes());
            tag[24..28].copy_from_slice(&28u32.to_be_bytes());
            tag.extend_from_slice(&text);
        } else {
            tag.extend_from_slice(b"desc\0\0\0\0");
            tag.extend_from_slice(&u32::try_from(name.len() + 1).unwrap_or(0).to_be_bytes());
            tag.extend_from_slice(name.as_bytes());
            tag.push(0);
        }
        let mut icc = vec![0u8; 128];
        icc[16..20].copy_from_slice(&space);
        icc.extend_from_slice(&1u32.to_be_bytes());
        icc.extend_from_slice(b"desc");
        icc.extend_from_slice(&144u32.to_be_bytes());
        icc.extend_from_slice(&u32::try_from(tag.len()).unwrap_or(0).to_be_bytes());
        icc.extend_from_slice(&tag);
        icc
    }

    /// FR-071: the name of a version 2 and a version 4 profile; a profile cut short anywhere
    /// gives `None` or its name, never a panic.
    #[test]
    fn profile_names_from_v2_and_v4_profiles() {
        let v2 = named_profile(*b"RGB ", "Adobe RGB (1998)", false);
        assert_eq!(profile_name(&v2).as_deref(), Some("Adobe RGB (1998)"));
        let v4 = named_profile(*b"RGB ", "Display P3", true);
        assert_eq!(profile_name(&v4).as_deref(), Some("Display P3"));
        for cut in 0..v4.len() {
            let _ = profile_name(&v4[..cut]);
        }
        assert_eq!(profile_name(&[0u8; 200]), None);
    }

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
