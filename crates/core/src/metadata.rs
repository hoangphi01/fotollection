// SPDX-License-Identifier: GPL-3.0-only
//! EXIF reading with an explicit allowlist and value validation: SECURITY.md §3,
//! Layers 2–3 (FR-002).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use exif::{Exif, Field, In, Reader, Tag, Value};

/// The allowlisted, validated metadata of one photo. Everything else in EXIF is ignored.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhotoMeta {
    /// Camera-local wall-clock capture time.
    pub capture: Option<NaiveDateTime>,
    /// Offset from UTC in minutes, when the camera recorded one.
    pub offset_minutes: Option<i16>,
    /// `(latitude, longitude)` in degrees.
    pub gps: Option<(f64, f64)>,
    /// EXIF orientation 1–8 (1 when absent or invalid).
    pub orientation: u8,
    pub make: Option<String>,
    pub model: Option<String>,
    pub has_embedded_thumbnail: bool,
}

const MAX_STRING_CHARS: usize = 100;

/// Reads metadata from a file. Missing or unreadable EXIF yields defaults rather than an error,
/// so a photo without metadata is still usable (FR-002).
pub fn read_path(path: &Path) -> PhotoMeta {
    let Ok(file) = File::open(path) else {
        return PhotoMeta::with_defaults();
    };
    Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .map_or_else(|_| PhotoMeta::with_defaults(), |exif| from_exif(&exif))
}

/// Reads metadata from an in-memory JPEG.
pub fn read_bytes(bytes: &[u8]) -> PhotoMeta {
    Reader::new()
        .read_from_container(&mut std::io::Cursor::new(bytes))
        .map_or_else(|_| PhotoMeta::with_defaults(), |exif| from_exif(&exif))
}

/// The JPEG thumbnail embedded in EXIF (IFD1), if present and well-formed.
pub fn embedded_thumbnail(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    let exif = Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .ok()?;
    thumbnail_bytes(&exif).map(<[u8]>::to_vec)
}

impl PhotoMeta {
    fn with_defaults() -> Self {
        Self {
            orientation: 1,
            ..Self::default()
        }
    }
}

fn from_exif(exif: &Exif) -> PhotoMeta {
    let field = |tag| exif.get_field(tag, In::PRIMARY);

    let (capture, offset_minutes) = field(Tag::DateTimeOriginal)
        .or_else(|| field(Tag::DateTimeDigitized))
        .and_then(|f| parse_datetime(f, field(Tag::OffsetTimeOriginal)))
        .map_or((None, None), |(dt, off)| (Some(dt), off));

    let orientation = field(Tag::Orientation)
        .and_then(|f| f.value.get_uint(0))
        .and_then(|v| u8::try_from(v).ok())
        .filter(|v| (1..=8).contains(v))
        .unwrap_or(1);

    PhotoMeta {
        capture,
        offset_minutes,
        gps: parse_gps(exif),
        orientation,
        make: field(Tag::Make).and_then(clean_string),
        model: field(Tag::Model).and_then(clean_string),
        has_embedded_thumbnail: thumbnail_bytes(exif).is_some(),
    }
}

fn ascii(field: &Field) -> Option<&[u8]> {
    match &field.value {
        Value::Ascii(values) => values.first().map(Vec::as_slice),
        _ => None,
    }
}

fn parse_datetime(field: &Field, offset: Option<&Field>) -> Option<(NaiveDateTime, Option<i16>)> {
    let mut dt = exif::DateTime::from_ascii(ascii(field)?).ok()?;
    if let Some(raw) = offset.and_then(ascii) {
        // A malformed offset only loses the offset, not the date.
        let _ = dt.parse_offset(raw);
    }
    if !(1900..=2100).contains(&dt.year) {
        return None;
    }
    let date = NaiveDate::from_ymd_opt(i32::from(dt.year), u32::from(dt.month), u32::from(dt.day))?;
    let time = date.and_hms_opt(
        u32::from(dt.hour),
        u32::from(dt.minute),
        u32::from(dt.second),
    )?;
    let offset = dt.offset.filter(|m| m.abs() <= 14 * 60);
    Some((time, offset))
}

fn parse_gps(exif: &Exif) -> Option<(f64, f64)> {
    let coord = |value_tag, ref_tag, negative: u8| -> Option<f64> {
        let field = exif.get_field(value_tag, In::PRIMARY)?;
        let Value::Rational(parts) = &field.value else {
            return None;
        };
        if parts.len() < 3 {
            return None;
        }
        let degrees = parts[0].to_f64() + parts[1].to_f64() / 60.0 + parts[2].to_f64() / 3600.0;
        let sign = exif
            .get_field(ref_tag, In::PRIMARY)
            .and_then(ascii)
            .and_then(<[u8]>::first)
            .is_some_and(|c| c.eq_ignore_ascii_case(&negative));
        Some(if sign { -degrees } else { degrees })
    };
    let lat = coord(Tag::GPSLatitude, Tag::GPSLatitudeRef, b'S')?;
    let lon = coord(Tag::GPSLongitude, Tag::GPSLongitudeRef, b'W')?;
    let valid = lat.is_finite()
        && lon.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lon);
    // (0, 0) "null island" is what broken GPS writes; treat it as missing.
    let null_island = lat.abs() < 1e-6 && lon.abs() < 1e-6;
    (valid && !null_island).then_some((lat, lon))
}

fn clean_string(field: &Field) -> Option<String> {
    let raw = ascii(field)?;
    let text: String = String::from_utf8_lossy(raw)
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_STRING_CHARS)
        .collect();
    let text = text.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

fn thumbnail_bytes(exif: &Exif) -> Option<&[u8]> {
    let offset = exif
        .get_field(Tag::JPEGInterchangeFormat, In::THUMBNAIL)?
        .value
        .get_uint(0)? as usize;
    let length = exif
        .get_field(Tag::JPEGInterchangeFormatLength, In::THUMBNAIL)?
        .value
        .get_uint(0)? as usize;
    let bytes = exif.buf().get(offset..offset.checked_add(length)?)?;
    bytes.starts_with(&[0xFF, 0xD8]).then_some(bytes)
}
