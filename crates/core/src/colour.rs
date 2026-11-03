// SPDX-License-Identifier: GPL-3.0-only
//! Colour conversion to sRGB (FR-027, FR-053): photos tagged Display P3, Adobe RGB or any
//! other ICC profile are converted, so a site that ignores profiles (or strips them) still
//! shows the right colours. Untagged photos are taken as sRGB. Uses `qcms` (MIT, pure Rust).
//! The screen gets sRGB too ([`screen`], FR-072); print exports keep the photo's profile.

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

/// The RGB primaries a matrix profile states (`rXYZ`, `gXYZ`, `bXYZ`, D50-adapted), the way
/// to tell the common spaces apart whatever their profile is called ("sP3", "A98C" for the
/// compact profiles). `None` for a profile without them, such as a LUT-based one.
pub fn primaries(icc: &[u8]) -> Option<[[f64; 3]; 3]> {
    let be = |at: usize| {
        icc.get(at..at.checked_add(4)?)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_be_bytes)
    };
    let count = be(128)?.min(256) as usize;
    let xyz = |sig: &[u8; 4]| {
        let offset = (0..count).find_map(|i| {
            let at = 132 + 12 * i;
            (icc.get(at..at + 4)? == sig).then(|| be(at + 4))?
        })? as usize;
        // An `XYZ ` tag: the type, 4 reserved bytes, then three s15Fixed16 numbers.
        let tag = icc.get(offset..offset.checked_add(20)?)?;
        (tag.get(..4)? == b"XYZ ").then(|| {
            let n = |i: usize| {
                f64::from(i32::from_be_bytes([
                    tag[i],
                    tag[i + 1],
                    tag[i + 2],
                    tag[i + 3],
                ])) / 65536.0
            };
            [n(8), n(12), n(16)]
        })
    };
    Some([xyz(b"rXYZ")?, xyz(b"gXYZ")?, xyz(b"bXYZ")?])
}

/// What a profile is to the converter, kept per profile so a folder of iPhone photos builds
/// its transform once (qcms precomputes tables in `Transform::new`).
#[derive(Clone)]
enum Cached {
    Srgb,
    Unreadable,
    Convert(std::sync::Arc<Transform>),
}

/// Profiles seen, by their bytes' hash; a handful per library, so a small bound suffices.
const CACHED_PROFILES: usize = 16;

fn cached(icc: &[u8]) -> Cached {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<blake3::Hash, Cached>>> = OnceLock::new();
    let key = blake3::hash(icc);
    let cache = CACHE.get_or_init(Mutex::default);
    // The lock covers the lookup only, never the conversion (a lock held across rayon work
    // can hang the pool, LESSONS 2026-11-03).
    if let Some(hit) = cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
        return hit;
    }
    let made = match Profile::new_from_slice(icc, false) {
        None => Cached::Unreadable,
        Some(source) if source.is_sRGB() => Cached::Srgb,
        Some(source) => Transform::new(
            &source,
            &Profile::new_sRGB(),
            DataType::RGB8,
            Intent::Perceptual,
        )
        .map_or(Cached::Unreadable, |t| {
            Cached::Convert(std::sync::Arc::new(t))
        }),
    };
    if let Ok(mut c) = cache.lock() {
        if c.len() >= CACHED_PROFILES {
            c.clear();
        }
        c.insert(key, made.clone());
    }
    made
}

/// Converts `image` from `icc` to sRGB in place.
pub fn to_srgb(image: &mut RgbImage, icc: Option<&[u8]>) -> Conversion {
    let Some(bytes) = icc else {
        return Conversion::AlreadySrgb;
    };
    let transform = match cached(bytes) {
        Cached::Srgb => return Conversion::AlreadySrgb,
        Cached::Unreadable => return Conversion::Unreadable,
        Cached::Convert(t) => t,
    };
    let row = image.width as usize * 3;
    if row == 0 {
        return Conversion::AlreadySrgb;
    }
    let transform = &*transform;
    image
        .data
        .par_chunks_mut(row * ROWS_PER_TASK)
        .for_each(|rows| transform.apply(rows));
    Conversion::Converted
}

/// The encoding gamma of sRGB as a PNG `gAMA` chunk states it (1/2.2).
const SRGB_PNG_GAMMA: f32 = 0.454_55;

/// Converts a PNG's pixels to sRGB from its `gAMA` (the encoding gamma, 0.45455 for about
/// sRGB) and `cHRM` (white and primaries, xy; sRGB's when absent): a file that states the
/// sRGB values, or nothing, is left as it is (FR-072).
pub fn declared_to_srgb(
    image: &mut RgbImage,
    gamma: Option<f32>,
    chromaticities: Option<png::SourceChromaticities>,
) {
    use qcms::{CIE_xyY, CIE_xyYTRIPLE};
    let xy = |(x, y): (png::ScaledFloat, png::ScaledFloat)| CIE_xyY {
        x: f64::from(x.into_value()),
        y: f64::from(y.into_value()),
        Y: 1.0,
    };
    let srgb_like = |c: &png::SourceChromaticities| {
        let near = |(x, y): (png::ScaledFloat, png::ScaledFloat), (a, b): (f32, f32)| {
            (x.into_value() - a).abs() < 0.005 && (y.into_value() - b).abs() < 0.005
        };
        near(c.white, (0.3127, 0.329))
            && near(c.red, (0.64, 0.33))
            && near(c.green, (0.30, 0.60))
            && near(c.blue, (0.15, 0.06))
    };
    let gamma = gamma.filter(|g| *g > 0.01 && (g - SRGB_PNG_GAMMA).abs() > 0.005);
    let chromaticities = chromaticities.filter(|c| !srgb_like(c));
    if gamma.is_none() && chromaticities.is_none() {
        return;
    }
    let c = chromaticities.unwrap_or_else(|| {
        png::SourceChromaticities::new((0.3127, 0.329), (0.64, 0.33), (0.30, 0.60), (0.15, 0.06))
    });
    let decode = 1.0 / gamma.unwrap_or(SRGB_PNG_GAMMA);
    let Some(source) = Profile::new_rgb_with_gamma_set(
        xy(c.white),
        CIE_xyYTRIPLE {
            red: xy(c.red),
            green: xy(c.green),
            blue: xy(c.blue),
        },
        decode,
        decode,
        decode,
    ) else {
        return;
    };
    let Some(transform) = Transform::new(
        &source,
        &Profile::new_sRGB(),
        DataType::RGB8,
        Intent::Perceptual,
    ) else {
        return;
    };
    let row = image.width as usize * 3;
    if row == 0 {
        return;
    }
    let transform = &transform;
    image
        .data
        .par_chunks_mut(row * ROWS_PER_TASK)
        .for_each(|rows| transform.apply(rows));
}

/// A decoded photo as the screen shows it (FR-072): sRGB, whatever profile it came with. Every
/// image the interface gets passes here (thumbnails, previews, zoom, the crop source, the
/// best-shot analysis); a profile `qcms` cannot read leaves the pixels as they are.
pub fn screen(decoded: crate::codec::Decoded) -> RgbImage {
    let mut image = decoded.image;
    to_srgb(&mut image, decoded.icc.as_deref());
    image
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

    /// A profile file, its matrix to XYZ and its curve.
    type Case = (&'static str, [[f64; 3]; 3], fn(f64) -> f64);

    fn icc(name: &str) -> Vec<u8> {
        let path = format!("{}/tests/fixtures/icc/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).unwrap_or_default()
    }

    /// The sRGB value of an encoded colour in a space with `to_xyz` (D65) and `decode` as its
    /// curve, computed here from the published matrices, not by qcms.
    fn expected(rgb: [u8; 3], to_xyz: [[f64; 3]; 3], decode: impl Fn(f64) -> f64) -> [f64; 3] {
        const XYZ_TO_SRGB: [[f64; 3]; 3] = [
            [3.240_454_2, -1.537_138_5, -0.498_531_4],
            [-0.969_266_0, 1.876_010_8, 0.041_556_0],
            [0.055_643_4, -0.204_025_9, 1.057_225_2],
        ];
        let lin = rgb.map(|v| decode(f64::from(v) / 255.0));
        let xyz = to_xyz.map(|r| r[0] * lin[0] + r[1] * lin[1] + r[2] * lin[2]);
        XYZ_TO_SRGB.map(|r| {
            let v = (r[0] * xyz[0] + r[1] * xyz[1] + r[2] * xyz[2]).clamp(0.0, 1.0);
            let e = if v <= 0.003_130_8 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            e * 255.0
        })
    }

    fn srgb_decode(v: f64) -> f64 {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// FR-072: a Display P3 and an Adobe RGB photo reach the screen as the sRGB values the
    /// published matrices give (within 2 levels of 255); grey stays grey; an untagged or sRGB
    /// photo is untouched. Profiles: the CC0 Compact ICC Profiles (tests/fixtures/icc).
    #[test]
    fn screen_converts_p3_and_adobe_rgb_to_srgb() {
        const P3: [[f64; 3]; 3] = [
            [0.486_570_9, 0.265_667_7, 0.198_217_3],
            [0.228_974_6, 0.691_738_5, 0.079_286_9],
            [0.0, 0.045_113_4, 1.043_944_4],
        ];
        const ADOBE: [[f64; 3]; 3] = [
            [0.576_730_9, 0.185_554_0, 0.188_185_2],
            [0.297_376_9, 0.627_349_1, 0.075_274_1],
            [0.027_034_3, 0.070_687_2, 0.991_108_5],
        ];
        let colours: [[u8; 3]; 5] = [
            [200, 120, 60],
            [60, 160, 90],
            [90, 110, 210],
            [128, 128, 128],
            [240, 230, 40],
        ];
        let image =
            crate::image::RgbImage::from_raw(5, 1, colours.iter().flatten().copied().collect())
                .unwrap_or_else(|| crate::image::RgbImage::new(5, 1));
        let cases: [Case; 2] = [
            ("DisplayP3-v4.icc", P3, srgb_decode),
            ("AdobeCompat-v4.icc", ADOBE, |v| v.powf(563.0 / 256.0)),
        ];
        for (name, to_xyz, decode) in cases {
            let profile = icc(name);
            assert!(!profile.is_empty(), "{name} is missing");
            let out = screen(crate::codec::Decoded {
                image: image.clone(),
                icc: Some(profile),
            });
            for (i, colour) in colours.iter().enumerate() {
                let want = expected(*colour, to_xyz, decode);
                let got = &out.data[i * 3..i * 3 + 3];
                for c in 0..3 {
                    assert!(
                        (f64::from(got[c]) - want[c]).abs() <= 2.0,
                        "{name} {colour:?}: got {got:?}, want {want:?}"
                    );
                }
            }
            let grey = &out.data[9..12];
            assert!(
                grey[0].abs_diff(grey[1]) <= 1 && grey[1].abs_diff(grey[2]) <= 1,
                "{name} grey {grey:?}"
            );
        }
        let untagged = screen(crate::codec::Decoded {
            image: image.clone(),
            icc: None,
        });
        assert_eq!(untagged.data, image.data);
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
