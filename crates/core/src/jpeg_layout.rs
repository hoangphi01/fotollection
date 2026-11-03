// SPDX-License-Identifier: GPL-3.0-only
//! How a JPEG is laid out, read from its header without decoding (ADR-0033): the frame type,
//! the precision, each component's sampling, and how many components the first scan holds.
//! `zune-jpeg` decodes only the layouts our corpus proves (`tests/jpeg_layouts.rs`); the rest
//! go to the reference decoder, as zune-jpeg 0.5.15 turned one-scan-per-component baseline
//! files into green stripes without an error (LESSONS).

/// What the header says, up to the first scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The SOF marker: 0xC0 baseline, 0xC1 extended, 0xC2 progressive, others rarer.
    pub frame: u8,
    pub precision: u8,
    /// Each component's horizontal and vertical sampling factor.
    pub sampling: Vec<(u8, u8)>,
    /// Components in the first scan; fewer than the frame's means one scan per component.
    pub first_scan: u8,
}

/// The layout and what else the header says about colour (FR-071).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub layout: Layout,
    /// The Adobe APP14 colour transform: 0 none (RGB or CMYK as stored), 1 YCbCr, 2 YCCK.
    pub adobe_transform: Option<u8>,
    /// The ICC profile from the APP2 `ICC_PROFILE` chunks, joined in file order; empty when
    /// there is none.
    pub icc: Vec<u8>,
}

/// Reads the layout, or `None` when the header is not a JPEG's or ends early. Bounds-checked:
/// any input, however broken, returns without a panic (SECURITY.md §3).
pub fn read(bytes: &[u8]) -> Option<Layout> {
    read_header(bytes).map(|h| h.layout)
}

/// Reads the header up to the first scan: the layout, the Adobe transform and the ICC
/// profile. Bounds-checked like [`read`].
pub fn read_header(bytes: &[u8]) -> Option<Header> {
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let mut at = 2;
    // The frame header seen so far: marker, precision, sampling.
    let mut frame: Option<Layout> = None;
    let mut adobe_transform = None;
    let mut icc = Vec::new();
    loop {
        // Markers may be padded with any number of 0xFF.
        while *bytes.get(at)? != 0xFF {
            at += 1;
        }
        while *bytes.get(at)? == 0xFF {
            at += 1;
        }
        let marker = *bytes.get(at)?;
        at += 1;
        // Standalone markers carry no length.
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if marker == 0xD9 {
            return None;
        }
        let length = usize::from(u16::from_be_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
        let body = bytes.get(at + 2..at + length.max(2))?;
        match marker {
            // SOF0–SOF15, except DHT (C4), JPG (C8) and DAC (CC).
            0xC0..=0xCF if !matches!(marker, 0xC4 | 0xC8 | 0xCC) => {
                let precision = *body.first()?;
                let count = usize::from(*body.get(5)?);
                let mut sampling = Vec::with_capacity(count);
                for i in 0..count {
                    let factors = *body.get(6 + 3 * i + 1)?;
                    sampling.push((factors >> 4, factors & 0x0F));
                }
                frame = Some(Layout {
                    frame: marker,
                    precision,
                    sampling,
                    first_scan: 0,
                });
            }
            // APP2: "ICC_PROFILE\0", the chunk's number and count, then the profile's bytes.
            0xE2 if body.starts_with(b"ICC_PROFILE\0") => {
                icc.extend_from_slice(body.get(14..).unwrap_or_default());
            }
            // APP14: "Adobe", version, two flag words, then the transform.
            0xEE if body.starts_with(b"Adobe") => adobe_transform = body.get(11).copied(),
            0xDA => {
                return Some(Header {
                    layout: Layout {
                        first_scan: *body.first()?,
                        ..frame?
                    },
                    adobe_transform,
                    icc,
                });
            }
            _ => {}
        }
        at += length.max(2);
    }
}

/// True when the image data starts but never ends: no end-of-image marker after the first
/// scan, as in a file cut short by a card pulled out or a download that stopped. Inside
/// entropy-coded data a 0xFF is always followed by 0x00 or a restart marker, so `FF D9` there
/// can only be the end of the image; an embedded thumbnail sits in APP1, before the first
/// scan. A file with no scan at all is not "cut short": the decoder reports it.
pub fn ends_early(bytes: &[u8]) -> bool {
    first_scan(bytes).is_some_and(|scan| {
        !bytes
            .get(scan..)
            .is_some_and(|rest| rest.windows(2).any(|w| w == [0xFF, 0xD9]))
    })
}

/// Where the first scan header (SOS) starts, walking the marker segments as `read` does.
fn first_scan(bytes: &[u8]) -> Option<usize> {
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let mut at = 2;
    loop {
        while *bytes.get(at)? != 0xFF {
            at += 1;
        }
        while *bytes.get(at)? == 0xFF {
            at += 1;
        }
        let marker = *bytes.get(at)?;
        at += 1;
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if marker == 0xDA {
            return Some(at);
        }
        if marker == 0xD9 {
            return None;
        }
        let length = usize::from(u16::from_be_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
        at += length.max(2);
    }
}

impl Layout {
    /// True for the layouts `zune-jpeg` decodes correctly, as `tests/jpeg_layouts.rs` proves:
    /// 8-bit baseline or progressive (extended SOF1 has no fixture yet), grey, Adobe CMYK or
    /// colour halved in any of the usual ways (4:4:4, 4:2:2, 4:2:0, 4:4:0, 4:1:1), and in a
    /// baseline file every component in the first scan. A new layout joins only with a
    /// fixture that proves it.
    pub fn zune_proven(&self) -> bool {
        let frame = matches!(self.frame, 0xC0 | 0xC2);
        let usual = match self.sampling.as_slice() {
            // Grey, and Adobe CMYK or YCCK (`cmyk.jpg`, `ycck.jpg`).
            [_] | [_, _, _, _] => true,
            [luma, cb, cr] => {
                *cb == (1, 1) && *cr == (1, 1) && matches!(luma, (1 | 2, 1 | 2) | (4, 1))
            }
            _ => false,
        };
        let count = u8::try_from(self.sampling.len()).unwrap_or(u8::MAX);
        let interleaved = self.frame == 0xC2 || self.first_scan == count;
        frame && self.precision == 8 && usual && interleaved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/jpeg")
            .join(name);
        std::fs::read(path).unwrap_or_default()
    }

    #[test]
    fn fr_001_layouts_are_read_from_the_header() {
        let baseline = read(&fixture("baseline-420.jpg"));
        assert_eq!(
            baseline,
            Some(Layout {
                frame: 0xC0,
                precision: 8,
                sampling: vec![(2, 2), (1, 1), (1, 1)],
                first_scan: 3,
            })
        );
        assert!(baseline.is_some_and(|l| l.zune_proven()));
        let split = read(&fixture("noninterleaved-420.jpg"));
        assert_eq!(split.as_ref().map(|l| l.first_scan), Some(1));
        assert!(
            split.is_some_and(|l| !l.zune_proven()),
            "one scan per component"
        );
        assert!(read(&fixture("progressive-420.jpg")).is_some_and(|l| l.zune_proven()));
        assert!(read(&fixture("grey.jpg")).is_some_and(|l| l.zune_proven()));
    }

    #[test]
    fn fr_001_a_broken_header_is_none_not_a_panic() {
        let good = fixture("baseline-420.jpg");
        for cut in 0..good.len().min(700) {
            let _ = read(&good[..cut]);
        }
        assert_eq!(read(b""), None);
        assert_eq!(read(&[0xFF, 0xD8, 0xFF]), None);
        assert_eq!(read(&[0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x00]), None);
        // A length shorter than itself must not loop or slice backwards.
        assert_eq!(
            read(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x01, 0xFF, 0xD9]),
            None
        );
    }
}
