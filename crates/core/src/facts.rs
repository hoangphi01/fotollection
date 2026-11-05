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
    /// One short line for the error log's context (FR-071, `docs/ERROR_CONTROL.md`): the names the
    /// interface gets, e.g. `4:2:0 progressive 8-bit displayP3 zune`. No path, no pixels.
    pub fn short(&self) -> String {
        fn name<T: Serialize>(value: &T) -> String {
            serde_json::to_string(value)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned()
        }
        let mut parts: Vec<String> = Vec::new();
        parts.extend(self.chroma.as_ref().map(name));
        parts.extend(self.coding.as_ref().map(name));
        parts.extend(self.sensor.as_ref().map(name));
        parts.extend(self.cfa.as_ref().map(name));
        parts.extend(self.bits.map(|b| format!("{b}-bit")));
        parts.extend(self.space.as_ref().map(name));
        if let Some(d) = self.dng {
            let kinds = [
                (d.lossy, "lossy"),
                (d.jpeg_xl, "jpegXl"),
                (d.float, "float"),
                (d.gain_map, "gainMap"),
                (d.original_raw, "originalRaw"),
            ];
            parts.extend(
                kinds
                    .iter()
                    .filter(|(on, _)| *on)
                    .map(|(_, n)| (*n).to_owned()),
            );
        }
        parts.extend(self.decoder.as_ref().map(name));
        parts.join(" ")
    }

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

/// The D50-adapted primaries (`rXYZ`, `gXYZ`, `bXYZ`) of the common RGB spaces.
const PRIMARIES: &[([[f64; 3]; 3], Space)] = &[
    (
        [
            [0.4361, 0.2225, 0.0139],
            [0.3851, 0.7169, 0.0971],
            [0.1431, 0.0606, 0.7141],
        ],
        Space::Srgb,
    ),
    (
        [
            [0.5151, 0.2412, -0.0011],
            [0.2920, 0.6922, 0.0419],
            [0.1571, 0.0666, 0.7841],
        ],
        Space::DisplayP3,
    ),
    (
        [
            [0.6097, 0.3111, 0.0195],
            [0.2053, 0.6257, 0.0609],
            [0.1492, 0.0632, 0.7446],
        ],
        Space::AdobeRgb,
    ),
    (
        [
            [0.7977, 0.2880, 0.0000],
            [0.1352, 0.7119, 0.0000],
            [0.0313, 0.0001, 0.8249],
        ],
        Space::ProPhoto,
    ),
];

/// The colour space an ICC profile names: by its primaries, else by its description.
pub fn profile_space(icc: &[u8]) -> Space {
    match icc.get(16..20) {
        Some(b"GRAY") => return Space::Grey,
        Some(b"CMYK") => return Space::Cmyk,
        _ => {}
    }
    // The primaries first: names vary ("Adobe RGB (1998)", "A98C"), the colorants do not.
    if let Some(p) = crate::colour::primaries(icc) {
        let near = |known: &[[f64; 3]; 3]| {
            p.iter()
                .flatten()
                .zip(known.iter().flatten())
                .all(|(a, b)| (a - b).abs() < 0.005)
        };
        for (known, space) in PRIMARIES {
            if near(known) {
                return *space;
            }
        }
    }
    let name = crate::colour::profile_name(icc)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.contains("srgb") {
        Space::Srgb
    } else if name.contains("p3") {
        Space::DisplayP3
    } else if name.contains("adobe rgb") || name.contains("adobergb") || name.contains("a98") {
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

/// An HEIF's (HEIC, AVIF) facts from its head: the colour space from the first `colr`
/// property (an ICC profile, or `nclx` primaries: 1 is BT.709/sRGB, 12 Display P3) and the
/// bit depth from the first `pixi`, found in `meta` → `iprp` → `ipco` (ISO/IEC 23008-12).
/// Every box size is checked against the bytes, at most 4,096 boxes are walked.
pub fn of_heif(head: &[u8]) -> Option<ImageFacts> {
    if head.get(4..8)? != b"ftyp" {
        return None;
    }
    let mut facts = ImageFacts::default();
    let mut walked = 0usize;
    // (start, end) of the box lists still to read, with how many header bytes each list's
    // parent skips before its children (4 for the FullBox `meta`).
    let mut lists = vec![(0usize, head.len())];
    while let Some((mut at, end)) = lists.pop() {
        while at + 8 <= end && walked < 4096 {
            walked += 1;
            let size = u32::from_be_bytes(head.get(at..at + 4)?.try_into().ok()?) as usize;
            let kind = head.get(at + 4..at + 8)?;
            // 0 runs to the end; 1 is a 64-bit size, never needed for these small boxes.
            let size = match size {
                0 => end - at,
                1 => break,
                s if s < 8 => break,
                s => s,
            };
            let body = (at + 8, (at + size).min(end));
            match kind {
                b"meta" => lists.push((body.0 + 4, body.1)),
                b"iprp" | b"ipco" => lists.push(body),
                b"colr" if facts.space.is_none() => {
                    let data = head.get(body.0..body.1)?;
                    facts.space = match data.get(..4)? {
                        b"prof" | b"rICC" => Some(profile_space(data.get(4..)?)),
                        b"nclx" => match u16::from_be_bytes([*data.get(4)?, *data.get(5)?]) {
                            1 => Some(Space::Srgb),
                            12 => Some(Space::DisplayP3),
                            _ => Some(Space::OtherProfile),
                        },
                        _ => None,
                    };
                }
                // FullBox: version and flags, the channel count, then bits per channel.
                b"pixi" if facts.bits.is_none() => facts.bits = head.get(body.0 + 5).copied(),
                _ => {}
            }
            at += size;
        }
    }
    (facts != ImageFacts::default()).then_some(ImageFacts {
        decoder: Some(Decoder::System),
        ..facts
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
    // Our own develop reads every Bayer sensor (ADR-0040), apart from the formats macOS's
    // codec reads first for its lens correction (`raw::decoders`).
    let decoder = if crate::raw::os_first(kind) {
        Decoder::System
    } else if sensor == Sensor::Bayer {
        Decoder::Own
    } else {
        raw_fallback()
    };
    Some(ImageFacts {
        bits: Some(bits),
        sensor: Some(sensor),
        cfa,
        decoder: Some(decoder),
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
        assert_eq!(
            of_jpeg(&fixture("progressive-420.jpg"))
                .map(|f| f.short())
                .as_deref(),
            Some("4:2:0 progressive 8-bit untagged zune")
        );
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

    /// An HEIC's `colr` (nclx P3 or a profile) and `pixi` are found under `meta`; any head
    /// cut short is safe.
    #[test]
    fn heif_space_and_depth() {
        let boxed = |kind: &[u8; 4], body: &[u8]| {
            let mut b = u32::try_from(body.len() + 8)
                .unwrap_or(0)
                .to_be_bytes()
                .to_vec();
            b.extend_from_slice(kind);
            b.extend_from_slice(body);
            b
        };
        let nclx = boxed(
            b"colr",
            &[b"nclx".as_slice(), &[0, 12, 0, 13, 0, 6, 0x80]].concat(),
        );
        let pixi = boxed(b"pixi", &[0, 0, 0, 0, 3, 10, 10, 10]);
        let ipco = boxed(b"ipco", &[nclx, pixi].concat());
        let iprp = boxed(b"iprp", &ipco);
        let meta = boxed(b"meta", &[vec![0, 0, 0, 0], iprp].concat());
        let file = [boxed(b"ftyp", b"heic\0\0\0\0mif1heic"), meta].concat();
        assert_eq!(
            of_heif(&file).map(|f| (f.space, f.bits)),
            Some((Some(Space::DisplayP3), Some(10)))
        );
        for cut in 0..file.len() {
            let _ = of_heif(&file[..cut]);
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
