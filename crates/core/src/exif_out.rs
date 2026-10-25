// SPDX-License-Identifier: GPL-3.0-only
//! A minimal EXIF writer for shared exports (FR-053, FR-010): the camera, lens, exposure,
//! capture time and `Software = "Fotollection <version>"` (shown as the editing app by
//! iOS Photos and others, as Snapseed does). The fields come from a fixed allowlist;
//! **there is never a GPS IFD**.

use chrono::NaiveDateTime;

use crate::library::ShotInfo;

const ASCII: u16 = 2;
const SHORT: u16 = 3;
const LONG: u16 = 4;
const RATIONAL: u16 = 5;
/// Longest text written in one field; EXIF text is short by nature.
const MAX_TEXT: usize = 64;

struct Entry {
    tag: u16,
    kind: u16,
    count: u32,
    data: Vec<u8>,
}

/// EXIF text: printable ASCII only, NUL-terminated, trimmed to a sane length.
fn ascii(tag: u16, text: &str) -> Entry {
    let mut data: Vec<u8> = text
        .chars()
        .filter(|c| c.is_ascii() && !c.is_ascii_control())
        .take(MAX_TEXT)
        .map(|c| c as u8)
        .collect();
    data.push(0);
    Entry {
        tag,
        kind: ASCII,
        count: u32::try_from(data.len()).unwrap_or(1),
        data,
    }
}

fn rational(tag: u16, num: u32, den: u32) -> Entry {
    let mut data = num.to_le_bytes().to_vec();
    data.extend_from_slice(&den.max(1).to_le_bytes());
    Entry {
        tag,
        kind: RATIONAL,
        count: 1,
        data,
    }
}

/// A value in tenths, as a rational (f/2.8 → 28/10, 23 mm → 230/10).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn tenths(tag: u16, value: f32) -> Option<Entry> {
    (value.is_finite() && value > 0.0).then(|| rational(tag, (value * 10.0).round() as u32, 10))
}

/// Exposure time: 1/250 s as 1/250, long exposures in tenths of a second.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn exposure(seconds: f32) -> Option<Entry> {
    if !(seconds.is_finite() && seconds > 0.0) {
        return None;
    }
    Some(if seconds < 1.0 {
        rational(0x829A, 1, (1.0 / seconds).round() as u32)
    } else {
        rational(0x829A, (seconds * 10.0).round() as u32, 10)
    })
}

/// Writes one IFD at `start` (an offset from the TIFF header); returns its bytes. `next`
/// entries' data that does not fit in 4 bytes goes right after the IFD.
fn ifd(mut entries: Vec<Entry>, start: u32) -> Vec<u8> {
    entries.sort_by_key(|e| e.tag);
    let count = u16::try_from(entries.len()).unwrap_or(0);
    let mut table = count.to_le_bytes().to_vec();
    let mut extra = Vec::new();
    let data_start = start + 2 + u32::from(count) * 12 + 4;
    for e in &entries {
        table.extend_from_slice(&e.tag.to_le_bytes());
        table.extend_from_slice(&e.kind.to_le_bytes());
        table.extend_from_slice(&e.count.to_le_bytes());
        if e.data.len() <= 4 {
            let mut inline = e.data.clone();
            inline.resize(4, 0);
            table.extend_from_slice(&inline);
        } else {
            let offset = data_start + u32::try_from(extra.len()).unwrap_or(0);
            table.extend_from_slice(&offset.to_le_bytes());
            extra.extend_from_slice(&e.data);
            if extra.len() % 2 == 1 {
                extra.push(0); // values start on a word boundary
            }
        }
    }
    table.extend_from_slice(&0u32.to_le_bytes()); // no next IFD
    table.extend_from_slice(&extra);
    table
}

/// The payload of an APP1 Exif segment (starting with `Exif\0\0`) for one photo.
pub fn exif_app1(shot: &ShotInfo, capture: Option<NaiveDateTime>, software: &str) -> Vec<u8> {
    let stamp = capture.map(|dt| dt.format("%Y:%m:%d %H:%M:%S").to_string());

    let mut exif = Vec::new();
    if let Some(e) = shot.exposure_s.and_then(exposure) {
        exif.push(e);
    }
    if let Some(e) = shot.f_number.and_then(|f| tenths(0x829D, f)) {
        exif.push(e);
    }
    if let Some(iso) = shot.iso.and_then(|i| u16::try_from(i).ok()) {
        exif.push(Entry {
            tag: 0x8827,
            kind: SHORT,
            count: 1,
            data: iso.to_le_bytes().to_vec(),
        });
    }
    if let Some(s) = &stamp {
        exif.push(ascii(0x9003, s));
    }
    if let Some(e) = shot.focal_mm.and_then(|f| tenths(0x920A, f)) {
        exif.push(e);
    }
    if let Some(lens) = &shot.lens {
        exif.push(ascii(0xA434, lens));
    }

    let mut zeroth = vec![ascii(0x0131, software)];
    if let Some(make) = &shot.make {
        zeroth.push(ascii(0x010F, make));
    }
    if let Some(model) = &shot.model {
        zeroth.push(ascii(0x0110, model));
    }
    if let Some(s) = &stamp {
        zeroth.push(ascii(0x0132, s));
    }
    // The Exif IFD pointer is filled in once IFD0's size is known.
    zeroth.push(Entry {
        tag: 0x8769,
        kind: LONG,
        count: 1,
        data: vec![0; 4],
    });
    let first = ifd(std::mem::take(&mut zeroth), 8);
    let exif_start = 8 + u32::try_from(first.len()).unwrap_or(0);
    // Rebuild IFD0 with the real pointer (same size, so offsets do not move).
    let mut zeroth = vec![ascii(0x0131, software)];
    if let Some(make) = &shot.make {
        zeroth.push(ascii(0x010F, make));
    }
    if let Some(model) = &shot.model {
        zeroth.push(ascii(0x0110, model));
    }
    if let Some(s) = &stamp {
        zeroth.push(ascii(0x0132, s));
    }
    zeroth.push(Entry {
        tag: 0x8769,
        kind: LONG,
        count: 1,
        data: exif_start.to_le_bytes().to_vec(),
    });

    let mut out = b"Exif\0\0".to_vec();
    out.extend_from_slice(b"II");
    out.extend_from_slice(&42u16.to_le_bytes());
    out.extend_from_slice(&8u32.to_le_bytes());
    out.extend_from_slice(&ifd(zeroth, 8));
    out.extend_from_slice(&ifd(exif, exif_start));
    out
}

/// `jpeg` with an APP1 segment holding `payload`, placed after SOI (and after a JFIF APP0
/// when there is one). `None` if `jpeg` is not a JPEG or the payload is too large.
pub fn insert_app1(jpeg: &[u8], payload: &[u8]) -> Option<Vec<u8>> {
    if jpeg.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let length = u16::try_from(payload.len() + 2).ok()?;
    let mut at = 2;
    if jpeg.get(2..4)? == [0xFF, 0xE0] {
        let app0 = u16::from_be_bytes([*jpeg.get(4)?, *jpeg.get(5)?]);
        at = 4 + usize::from(app0);
    }
    let mut out = Vec::with_capacity(jpeg.len() + payload.len() + 4);
    out.extend_from_slice(jpeg.get(..at)?);
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(payload);
    out.extend_from_slice(jpeg.get(at..)?);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{EncodeOptions, JpegCodec, PureRustCodec};
    use crate::image::RgbImage;

    fn shot() -> ShotInfo {
        ShotInfo {
            make: Some("FUJIFILM".into()),
            model: Some("X-T3".into()),
            lens: Some("XF56mmF1.2 R".into()),
            focal_mm: Some(56.0),
            f_number: Some(1.2),
            exposure_s: Some(1.0 / 250.0),
            iso: Some(160),
        }
    }

    #[test]
    fn fr_053_exif_reads_back_with_fotollection_and_no_gps() {
        let jpeg = PureRustCodec
            .encode(&RgbImage::new(32, 24), EncodeOptions::screen(80), None)
            .unwrap_or_default();
        let capture =
            chrono::NaiveDate::from_ymd_opt(2018, 10, 6).and_then(|d| d.and_hms_opt(12, 53, 41));
        let payload = exif_app1(&shot(), capture, "Fotollection 0.4.0");
        let tagged = insert_app1(&jpeg, &payload).unwrap_or_default();
        let exif = exif::Reader::new()
            .read_from_container(&mut std::io::Cursor::new(&tagged))
            .unwrap_or_else(|e| panic!("EXIF should parse: {e}"));
        let text = |tag| {
            exif.get_field(tag, exif::In::PRIMARY)
                .map(|f| f.display_value().to_string())
                .unwrap_or_default()
        };
        assert!(text(exif::Tag::Software).contains("Fotollection 0.4.0"));
        assert!(text(exif::Tag::Make).contains("FUJIFILM"));
        assert!(text(exif::Tag::LensModel).contains("XF56mm"));
        assert!(text(exif::Tag::DateTimeOriginal).contains("2018-10-06 12:53:41"));
        assert!(text(exif::Tag::ExposureTime).contains("1/250"));
        assert!(text(exif::Tag::PhotographicSensitivity).contains("160"));
        // Never a location.
        assert!(
            exif.fields()
                .all(|f| f.ifd_num != exif::In::PRIMARY || f.tag != exif::Tag::GPSInfoIFDPointer)
        );
        assert!(
            exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
                .is_none()
        );
        // The image itself still decodes.
        assert!(crate::codec::decode_image(&PureRustCodec, &tagged).is_ok());
    }

    #[test]
    fn fr_053_text_is_sanitised_and_bad_input_is_refused() {
        let mut s = shot();
        s.model = Some("X-T3\u{0}\u{7}ľ".into());
        let payload = exif_app1(&s, None, "Fotollection");
        assert!(!payload.windows(2).any(|w| w == [0, 7]));
        assert!(insert_app1(b"not a jpeg", &payload).is_none());
    }
}
