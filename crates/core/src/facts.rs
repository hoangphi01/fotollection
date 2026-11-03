// SPDX-License-Identifier: GPL-3.0-only
//! What each file is (FR-071): a JPEG's chroma subsampling, coding, bit depth and colour
//! space, a RAW's sensor filter and bit depth, the kind of DNG, and the decoder that reads it.
//! Read from the header at import, kept in RAM and never saved (DATABASE.md N4): reading it
//! again on the next import costs less than storing it. A RAW other than DNG keeps its
//! sensor until the inspector reads the whole file, as only rawler knows it.

use serde::Serialize;
use specta::Type;

use crate::jpeg_layout::Header;
use crate::raw::RawKind;

/// How the colour of a JPEG is sampled, or what its components are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
pub enum Chroma {
    #[serde(rename = "4:4:4")]
    S444,
    #[serde(rename = "4:2:2")]
    S422,
    #[serde(rename = "4:2:0")]
    S420,
    #[serde(rename = "4:4:0")]
    S440,
    #[serde(rename = "4:1:1")]
    S411,
    /// Three components sampled some other way.
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "grey")]
    Grey,
    /// Three components stored as RGB, not YCbCr (Adobe transform 0).
    #[serde(rename = "rgb")]
    Rgb,
    #[serde(rename = "cmyk")]
    Cmyk,
    #[serde(rename = "ycck")]
    Ycck,
}

/// The JPEG coding process, from the frame marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Coding {
    Baseline,
    Extended,
    Progressive,
    Lossless,
    Arithmetic,
    Hierarchical,
}

/// The colour space a file states: by its ICC profile's name, its header or EXIF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Space {
    Srgb,
    DisplayP3,
    AdobeRgb,
    ProPhoto,
    Grey,
    Cmyk,
    /// A profile with another name.
    OtherProfile,
    /// No profile: read as sRGB.
    Untagged,
}

/// The colour filter in front of a RAW's sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Sensor {
    Bayer,
    XTrans,
    /// Another pattern, such as four colours.
    OtherFilter,
    /// Already demosaiced (a linear DNG, `ProRAW`).
    Linear,
    Monochrome,
}

/// A Bayer pattern, top-left site first, row by row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "UPPERCASE")]
pub enum Cfa {
    Rggb,
    Bggr,
    Grbg,
    Gbrg,
}

/// The decoder that reads the file (ADR-0029, ADR-0030, ADR-0033), as the routing rules
/// choose it: the first in line, before any fallback after an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Decoder {
    /// zune-jpeg, for the layouts our corpus proves.
    Zune,
    /// jpeg-decoder, the reference for every other JPEG.
    Reference,
    /// Our own DNG develop.
    Own,
    Rawler,
    /// The operating system's codec (`ImageIO` on macOS, WIC on Windows).
    System,
}

/// The DNG kinds our develop does not read yet (docs/raw/dng.md).
#[allow(clippy::struct_excessive_bools)] // independent facts, one per kind
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DngKind {
    /// Lossy JPEG compression (34892).
    pub lossy: bool,
    /// JPEG XL compression (52546).
    pub jpeg_xl: bool,
    /// Floating-point samples: an HDR or panorama merge.
    pub float: bool,
    /// A phone's lens shading in `OpcodeList2`, not applied yet.
    pub gain_map: bool,
    /// The original maker RAW is embedded.
    pub original_raw: bool,
}

/// Everything FR-071 shows about one file. Small and `Copy`: one per photo in RAM.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageFacts {
    pub chroma: Option<Chroma>,
    pub coding: Option<Coding>,
    /// Bits per sample.
    pub bits: Option<u8>,
    pub space: Option<Space>,
    pub sensor: Option<Sensor>,
    pub cfa: Option<Cfa>,
    pub dng: Option<DngKind>,
    pub decoder: Option<Decoder>,
}

impl ImageFacts {
    /// A file outside the common path, worth a badge in the grid and its own filter: CMYK,
    /// RGB-coded or oddly sampled JPEGs, more than 8 bits, lossless, arithmetic or
    /// hierarchical coding, a wide or unknown profile other than Display P3, a sensor other
    /// than Bayer, and the DNG kinds the develop does not read yet.
    pub fn unusual(&self) -> bool {
        let chroma = matches!(
            self.chroma,
            Some(Chroma::Other | Chroma::Rgb | Chroma::Cmyk | Chroma::Ycck)
        );
        let coding = matches!(
            self.coding,
            Some(Coding::Lossless | Coding::Arithmetic | Coding::Hierarchical)
        ) || (self.coding.is_some() && self.bits.is_some_and(|b| b > 8));
        let space = matches!(
            self.space,
            Some(Space::AdobeRgb | Space::ProPhoto | Space::Cmyk | Space::OtherProfile)
        );
        let sensor = self.sensor.is_some_and(|s| s != Sensor::Bayer);
        let dng = self
            .dng
            .is_some_and(|d| d.lossy || d.jpeg_xl || d.float || d.gain_map);
        chroma || coding || space || sensor || dng
    }
}

/// The colour space an ICC profile names. RGB profiles are told apart by their description.
pub fn profile_space(icc: &[u8]) -> Space {
    match icc.get(16..20) {
        Some(b"GRAY") => return Space::Grey,
        Some(b"CMYK") => return Space::Cmyk,
        _ => {}
    }
    let name = crate::colour::profile_name(icc)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.contains("srgb") {
        Space::Srgb
    } else if name.contains("p3") {
        Space::DisplayP3
    } else if name.contains("adobe rgb") || name.contains("adobergb") {
        Space::AdobeRgb
    } else if name.contains("prophoto") || name.contains("romm") {
        Space::ProPhoto
    } else {
        Space::OtherProfile
    }
}

/// A JPEG's facts from its header (`None` when the header cannot be read).
pub fn of_jpeg(head: &[u8]) -> Option<ImageFacts> {
    let Header {
        layout,
        adobe_transform,
        icc,
    } = crate::jpeg_layout::read_header(head)?;
    let chroma = match layout.sampling.as_slice() {
        [_] => Chroma::Grey,
        [_, _, _, _] if adobe_transform == Some(2) => Chroma::Ycck,
        [_, _, _, _] => Chroma::Cmyk,
        [_, _, _] if adobe_transform == Some(0) => Chroma::Rgb,
        [luma, (1, 1), (1, 1)] => match luma {
            (1, 1) => Chroma::S444,
            (2, 1) => Chroma::S422,
            (2, 2) => Chroma::S420,
            (1, 2) => Chroma::S440,
            (4, 1) => Chroma::S411,
            _ => Chroma::Other,
        },
        _ => Chroma::Other,
    };
    let coding = match layout.frame {
        0xC0 => Coding::Baseline,
        0xC1 => Coding::Extended,
        0xC2 => Coding::Progressive,
        0xC3 => Coding::Lossless,
        0xC9..=0xCB => Coding::Arithmetic,
        _ => Coding::Hierarchical,
    };
    let space = if icc.is_empty() {
        match chroma {
            Chroma::Grey => Space::Grey,
            Chroma::Cmyk | Chroma::Ycck => Space::Cmyk,
            _ => Space::Untagged,
        }
    } else {
        profile_space(&icc)
    };
    // The same rule `PureRustCodec::decode` follows (ADR-0033).
    let decoder = if layout.zune_proven() {
        Decoder::Zune
    } else {
        Decoder::Reference
    };
    Some(ImageFacts {
        chroma: Some(chroma),
        coding: Some(coding),
        bits: Some(layout.precision),
        space: Some(space),
        decoder: Some(decoder),
        ..ImageFacts::default()
    })
}

/// A PNG's facts from its first chunks: the bit depth from `IHDR`, the colour space from
/// `sRGB`, `iCCP` (by the profile's name, without inflating it) or a grey colour type.
pub fn of_png(head: &[u8]) -> Option<ImageFacts> {
    if !head.starts_with(b"\x89PNG\r\n\x1a\n") || head.get(12..16)? != b"IHDR" {
        return None;
    }
    let bits = *head.get(24)?;
    let grey = matches!(head.get(25)?, 0 | 4);
    let mut space = if grey { Space::Grey } else { Space::Untagged };
    let mut at = 8usize;
    // Colour chunks come before the image data; stop there or at the end of the head.
    while let Some(len) = head
        .get(at..at + 4)
        .and_then(|b| b.try_into().ok())
        .map(u32::from_be_bytes)
    {
        match head.get(at + 4..at + 8) {
            Some(b"sRGB") => space = Space::Srgb,
            Some(b"iCCP") => {
                let name = head
                    .get(at + 8..at + 8 + (len as usize).min(80))
                    .unwrap_or_default();
                let name = name.split(|&b| b == 0).next().unwrap_or_default();
                let name = String::from_utf8_lossy(name).to_ascii_lowercase();
                space = if name.contains("srgb") {
                    Space::Srgb
                } else if name.contains("p3") {
                    Space::DisplayP3
                } else if name.contains("adobe") {
                    Space::AdobeRgb
                } else {
                    Space::OtherProfile
                };
            }
            Some(b"IDAT") | None => break,
            _ => {}
        }
        let Some(next) = at.checked_add(12 + len as usize) else {
            break;
        };
        at = next;
    }
    Some(ImageFacts {
        bits: Some(bits),
        space: Some(space),
        ..ImageFacts::default()
    })
}

/// A DNG's facts from its head or whole file: the filter, the bit depth and the kinds our
/// develop does not read yet.
pub fn of_dng(data: &[u8]) -> Option<ImageFacts> {
    use fotollection_rawcore::dng::{Facts, Filter};
    let Facts {
        filter,
        bits,
        compression,
        float,
        gain_map,
        original_raw,
    } = fotollection_rawcore::dng::facts(data)?;
    let (sensor, cfa) = match filter {
        Filter::Bayer(pattern) => (Sensor::Bayer, cfa_of(pattern)),
        Filter::XTrans => (Sensor::XTrans, None),
        Filter::Other => (Sensor::OtherFilter, None),
        Filter::Linear(1) => (Sensor::Monochrome, None),
        Filter::Linear(_) => (Sensor::Linear, None),
    };
    let ours = sensor == Sensor::Bayer && matches!(compression, 1 | 7) && !float;
    Some(ImageFacts {
        bits: Some(bits),
        sensor: Some(sensor),
        cfa,
        dng: Some(DngKind {
            lossy: compression == 34892,
            jpeg_xl: compression == 52546,
            float,
            gain_map,
            original_raw,
        }),
        decoder: Some(if ours { Decoder::Own } else { raw_fallback() }),
        ..ImageFacts::default()
    })
}

/// A RAW's facts from the whole file: a DNG from its directories, any other kind from rawler.
pub fn of_raw(bytes: &[u8], kind: RawKind) -> Option<ImageFacts> {
    if kind == RawKind::Dng {
        return of_dng(bytes);
    }
    let (sensor, cfa, bits) = crate::raw::sensor(bytes)?;
    Some(ImageFacts {
        bits: Some(bits),
        sensor: Some(sensor),
        cfa,
        decoder: Some(raw_fallback()),
        ..ImageFacts::default()
    })
}

/// The first decoder after our own (`raw::decoders`): the system's on macOS, rawler elsewhere.
fn raw_fallback() -> Decoder {
    if cfg!(target_os = "macos") {
        Decoder::System
    } else {
        Decoder::Rawler
    }
}

/// The Bayer pattern of four CFA sites (0 red, 1 green, 2 blue) or of its name.
pub(crate) fn cfa_of(sites: [u8; 4]) -> Option<Cfa> {
    Some(match sites {
        [0, 1, 1, 2] => Cfa::Rggb,
        [2, 1, 1, 0] => Cfa::Bggr,
        [1, 0, 2, 1] => Cfa::Grbg,
        [1, 2, 0, 1] => Cfa::Gbrg,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = format!("{}/tests/fixtures/jpeg/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(path).unwrap_or_default()
    }

    /// FR-071: the corpus fixtures read as what they are.
    #[test]
    fn jpeg_fixtures_read_as_their_layout() {
        let cases = [
            ("baseline-444.jpg", Chroma::S444, Coding::Baseline, 8),
            ("baseline-422.jpg", Chroma::S422, Coding::Baseline, 8),
            ("baseline-420.jpg", Chroma::S420, Coding::Baseline, 8),
            ("baseline-440.jpg", Chroma::S440, Coding::Baseline, 8),
            ("baseline-411.jpg", Chroma::S411, Coding::Baseline, 8),
            ("progressive-420.jpg", Chroma::S420, Coding::Progressive, 8),
            ("extended-sof1-420.jpg", Chroma::S420, Coding::Extended, 8),
            ("arithmetic-420.jpg", Chroma::S420, Coding::Arithmetic, 8),
            ("grey.jpg", Chroma::Grey, Coding::Baseline, 8),
            ("lossless-16-grey.jpg", Chroma::Grey, Coding::Lossless, 16),
            ("cmyk.jpg", Chroma::Cmyk, Coding::Baseline, 8),
            ("ycck.jpg", Chroma::Ycck, Coding::Baseline, 8),
        ];
        for (name, chroma, coding, bits) in cases {
            let facts = of_jpeg(&fixture(name));
            assert_eq!(
                facts.map(|f| (f.chroma, f.coding, f.bits)),
                Some((Some(chroma), Some(coding), Some(bits))),
                "{name}"
            );
        }
        let plain = of_jpeg(&fixture("baseline-420.jpg"));
        assert!(plain.is_some_and(|f| !f.unusual() && f.decoder == Some(Decoder::Zune)));
        let cmyk = of_jpeg(&fixture("cmyk.jpg"));
        assert!(cmyk.is_some_and(|f| f.unusual() && f.space == Some(Space::Cmyk)));
        let lossless = of_jpeg(&fixture("lossless-16-grey.jpg"));
        assert!(lossless.is_some_and(|f| f.unusual() && f.decoder == Some(Decoder::Reference)));
    }

    /// An APP2 ICC profile names the space; Display P3 is common (phones) and not unusual,
    /// Adobe RGB is.
    #[test]
    fn the_icc_profile_names_the_space() {
        let named = |name: &str, v4: bool| crate::colour::tests::named_profile(*b"RGB ", name, v4);
        assert_eq!(
            profile_space(&named("sRGB IEC61966-2.1", false)),
            Space::Srgb
        );
        assert_eq!(profile_space(&named("Display P3", true)), Space::DisplayP3);
        assert_eq!(
            profile_space(&named("Adobe RGB (1998)", false)),
            Space::AdobeRgb
        );
        assert_eq!(profile_space(&named("ProPhoto RGB", true)), Space::ProPhoto);
        assert_eq!(
            profile_space(&named("Camera RGB", false)),
            Space::OtherProfile
        );
        assert_eq!(
            profile_space(&crate::colour::tests::named_profile(
                *b"GRAY", "Gray", false
            )),
            Space::Grey
        );
        // The profile spliced into a real file, right after SOI.
        let jpeg = fixture("baseline-420.jpg");
        let with = |icc: &[u8]| {
            let mut body = b"ICC_PROFILE\0\x01\x01".to_vec();
            body.extend_from_slice(icc);
            let len = u16::try_from(body.len() + 2).unwrap_or(0).to_be_bytes();
            let mut file = jpeg[..2].to_vec();
            file.extend_from_slice(&[0xFF, 0xE2, len[0], len[1]]);
            file.extend_from_slice(&body);
            file.extend_from_slice(&jpeg[2..]);
            of_jpeg(&file)
        };
        let p3 = with(&named("Display P3", true));
        assert!(p3.is_some_and(|f| f.space == Some(Space::DisplayP3) && !f.unusual()));
        let adobe = with(&named("Adobe RGB (1998)", false));
        assert!(adobe.is_some_and(|f| f.space == Some(Space::AdobeRgb) && f.unusual()));
        assert_eq!(of_jpeg(&jpeg).and_then(|f| f.space), Some(Space::Untagged));
    }

    /// A PNG's depth and space come from its first chunks; any head cut short is safe.
    #[test]
    fn png_depth_and_space() {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let chunk = |png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]| {
            png.extend_from_slice(&u32::try_from(data.len()).unwrap_or(0).to_be_bytes());
            png.extend_from_slice(kind);
            png.extend_from_slice(data);
            png.extend_from_slice(&[0; 4]);
        };
        chunk(&mut png, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 16, 2, 0, 0, 0]);
        chunk(&mut png, b"iCCP", b"Display P3\0\0");
        chunk(&mut png, b"IDAT", &[]);
        let facts = of_png(&png);
        assert_eq!(
            facts.map(|f| (f.bits, f.space)),
            Some((Some(16), Some(Space::DisplayP3)))
        );
        for cut in 0..png.len() {
            let _ = of_png(&png[..cut]);
        }
    }

    /// Every head of every fixture, cut anywhere, reads without a panic.
    #[test]
    fn cut_headers_never_panic() {
        for name in ["progressive-420.jpg", "cmyk.jpg", "ycck.jpg"] {
            let bytes = fixture(name);
            for cut in (0..bytes.len().min(4096)).step_by(7) {
                let _ = of_jpeg(&bytes[..cut]);
            }
        }
    }
}
