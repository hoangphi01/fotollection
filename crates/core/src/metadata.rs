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
    /// The body's serial number (`BodySerialNumber`), so two bodies of one model keep
    /// their own clock offset (FR-030).
    pub serial: Option<String>,
    pub has_embedded_thumbnail: bool,
    /// Exposure details for the info frame (FR-040). Absent for manual lenses that
    /// report nothing.
    pub lens: Option<String>,
    pub focal_mm: Option<f32>,
    pub f_number: Option<f32>,
    pub exposure_s: Option<f32>,
    pub iso: Option<u32>,
}

const MAX_STRING_CHARS: usize = 100;

/// Reads metadata from a file. Missing or unreadable EXIF yields defaults rather than an error,
/// so a photo without metadata is still usable (FR-002). A camera RAW goes to [`read_raw`].
pub fn read_path(path: &Path) -> PhotoMeta {
    if let Some(kind) = raw_kind_of(path) {
        return read_raw(path, kind);
    }
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

/// The RAW kind of a file, reading 16 bytes for a JPEG or PNG and the RAW head otherwise.
fn raw_kind_of(path: &Path) -> Option<crate::raw::RawKind> {
    use std::io::Read;
    let mut file = File::open(path).ok()?;
    let mut magic = [0u8; 16];
    let n = file.read(&mut magic).ok()?;
    if magic[..n].starts_with(&[0xFF, 0xD8]) || magic[..n].starts_with(b"\x89PNG") {
        return None;
    }
    let mut head = magic[..n].to_vec();
    file.take(crate::raw::HEAD_BYTES as u64)
        .read_to_end(&mut head)
        .ok()?;
    crate::raw::kind(&head)
}

/// How much of a RAW file is read for its metadata: enough for the IFDs of every tested
/// camera, far less than the 20–60 MB of the file (FR-016, RESULT.md §12).
const RAW_HEAD_BYTES: u64 = 1024 * 1024;

/// Metadata of a camera RAW (FR-016) from the head of the file. RW2 and ORF are TIFF files
/// with their own magic number, RAF carries a JPEG with full EXIF, and CR3 keeps its EXIF in
/// `CMT1` (camera) and `CMT2` (exposure) boxes; the others are plain TIFF. The RAW's own
/// preview is used instead of an EXIF thumbnail, so `has_embedded_thumbnail` is false.
pub fn read_raw(path: &Path, kind: crate::raw::RawKind) -> PhotoMeta {
    use std::io::Read;
    let mut head = Vec::new();
    if File::open(path)
        .and_then(|f| f.take(RAW_HEAD_BYTES).read_to_end(&mut head))
        .is_err()
    {
        return PhotoMeta::with_defaults();
    }
    // A field stored past the head (an ORF maker note) fails the whole read: read it all then.
    let mut meta = raw_meta(path, &head, kind)
        .or_else(|| raw_meta(path, &std::fs::read(path).ok()?, kind))
        .unwrap_or_else(PhotoMeta::with_defaults);
    meta.has_embedded_thumbnail = false;
    meta
}

fn raw_meta(path: &Path, head: &[u8], kind: crate::raw::RawKind) -> Option<PhotoMeta> {
    use crate::raw::RawKind;
    match kind {
        RawKind::Rw2 | RawKind::Orf => {
            // `IIU\0`, `IIRO`, `MMOR`: give the TIFF magic number back, offsets are unchanged.
            let mut tiff = head.to_vec();
            let magic: &[u8] = if tiff.starts_with(b"MM") {
                &[0, 0x2A]
            } else {
                &[0x2A, 0]
            };
            tiff.get_mut(2..4)?.copy_from_slice(magic);
            let exif = Reader::new().read_raw(tiff).ok()?;
            let mut meta = from_exif(&exif);
            // Panasonic keeps the ISO in its own IFD0 tag 0x0017, not in EXIF.
            meta.iso = meta.iso.or_else(|| {
                exif.fields()
                    .find(|f| f.tag.number() == 0x0017 && f.ifd_num == In::PRIMARY)
                    .and_then(|f| f.value.get_uint(0))
                    .filter(|iso| (25..=1_638_400).contains(iso))
            });
            Some(meta)
        }
        RawKind::Cr2 | RawKind::Nef | RawKind::Arw | RawKind::Dng => Reader::new()
            .read_raw(head.to_vec())
            .ok()
            .map(|e| from_exif(&e)),
        RawKind::Raf => {
            // Big-endian offset and length of the embedded JPEG at bytes 84 and 88.
            let be = |at: usize| Some(u32::from_be_bytes(head.get(at..at + 4)?.try_into().ok()?));
            let (offset, length) = (u64::from(be(84)?), u64::from(be(88)?));
            let jpeg = read_range(path, offset, length.min(RAW_HEAD_BYTES))?;
            Some(read_bytes(&jpeg))
        }
        RawKind::Cr3 => {
            let camera = Reader::new()
                .read_raw(cr3_box(head, *b"CMT1")?.to_vec())
                .ok()?;
            let shot =
                cr3_box(head, *b"CMT2").and_then(|b| Reader::new().read_raw(b.to_vec()).ok());
            // Both boxes are a TIFF whose IFD0 holds the tags, so they are matched by number.
            let field = |tag: Tag| {
                camera
                    .fields()
                    .chain(shot.iter().flat_map(Exif::fields))
                    .find(|f| f.tag.number() == tag.number() && f.ifd_num == In::PRIMARY)
            };
            Some(from_fields(field, None, false))
        }
    }
}

/// The payload of the first ISO-BMFF box named `name` found in `head` (CR3's `CMT1`…).
fn cr3_box(head: &[u8], name: [u8; 4]) -> Option<&[u8]> {
    let at = head.windows(4).position(|w| w == name)?.checked_sub(4)?;
    let size = usize::try_from(u32::from_be_bytes(head.get(at..at + 4)?.try_into().ok()?)).ok()?;
    head.get(at + 8..at.checked_add(size)?)
}

fn read_range(path: &Path, offset: u64, length: u64) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut out = Vec::new();
    file.take(length).read_to_end(&mut out).ok()?;
    Some(out)
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
    from_fields(
        |tag| exif.get_field(tag, In::PRIMARY),
        parse_gps(exif),
        thumbnail_bytes(exif).is_some(),
    )
}

/// Builds the allowlisted metadata from a tag lookup, so containers that split EXIF in
/// pieces (CR3) share the same validation.
fn from_fields<'a>(
    field: impl Fn(Tag) -> Option<&'a Field>,
    gps: Option<(f64, f64)>,
    has_embedded_thumbnail: bool,
) -> PhotoMeta {
    let (capture, offset_minutes) = field(Tag::DateTimeOriginal)
        .or_else(|| field(Tag::DateTimeDigitized))
        .and_then(|f| parse_datetime(f, field(Tag::OffsetTimeOriginal)))
        .map_or((None, None), |(dt, off)| (Some(dt), off));

    let orientation = field(Tag::Orientation)
        .and_then(|f| f.value.get_uint(0))
        .and_then(|v| u8::try_from(v).ok())
        .filter(|v| (1..=8).contains(v))
        .unwrap_or(1);

    let lens = field(Tag::LensModel).and_then(clean_string).map(|model| {
        match field(Tag::LensMake).and_then(clean_string) {
            Some(make) if !model.starts_with(&make) => format!("{make} {model}"),
            _ => model,
        }
    });

    PhotoMeta {
        capture,
        offset_minutes,
        gps,
        orientation,
        make: field(Tag::Make).and_then(clean_string),
        model: field(Tag::Model).and_then(clean_string),
        serial: field(Tag::BodySerialNumber).and_then(clean_string),
        has_embedded_thumbnail,
        lens,
        focal_mm: field(Tag::FocalLength).and_then(|f| rational(f, 0.5..=5000.0)),
        f_number: field(Tag::FNumber).and_then(|f| rational(f, 0.5..=128.0)),
        // Some files keep only the APEX shutter value (after editing, or from some lenses).
        exposure_s: field(Tag::ExposureTime)
            .and_then(|f| rational(f, EXPOSURE_RANGE))
            .or_else(|| field(Tag::ShutterSpeedValue).and_then(apex_shutter)),
        iso: field(Tag::PhotographicSensitivity)
            .and_then(|f| f.value.get_uint(0))
            .filter(|iso| (25..=1_638_400).contains(iso)),
    }
}

/// Exposure times a camera can record, in seconds.
const EXPOSURE_RANGE: std::ops::RangeInclusive<f64> = 1.0 / 64_000.0..=3600.0;

/// Exposure time from `ShutterSpeedValue`, an APEX value Tv where t = 2^−Tv seconds.
/// Rounded to the camera's usual steps, as `ExposureTime` would read (1/2500, not 1/2521).
#[allow(clippy::cast_possible_truncation)]
fn apex_shutter(field: &Field) -> Option<f32> {
    let tv = match &field.value {
        Value::SRational(v) => v.first()?.to_f64(),
        Value::Rational(v) => v.first()?.to_f64(),
        _ => return None,
    };
    let t = (-tv).exp2();
    if !(t.is_finite() && EXPOSURE_RANGE.contains(&t)) {
        return None;
    }
    let t = if t < 1.0 {
        1.0 / nice_denominator(1.0 / t)
    } else {
        (t * 10.0).round() / 10.0
    };
    Some(t as f32)
}

/// The nearest shutter denominator a camera displays (…, 2000, 2500, 3200, 4000, …).
fn nice_denominator(d: f64) -> f64 {
    const STEPS: [f64; 31] = [
        1.3, 1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 13.0, 15.0, 20.0, 25.0, 30.0, 40.0,
        50.0, 60.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 320.0, 400.0, 500.0, 640.0, 800.0,
        1000.0, 1250.0,
    ];
    // Above 1250 the steps repeat ×10 (1600, 2000, 2500, 3200…).
    let (scale, d) = if d > 1250.0 {
        (10.0, d / 10.0)
    } else {
        (1.0, d)
    };
    let nearest = STEPS
        .iter()
        .copied()
        .min_by(|a, b| ((a / d).ln().abs()).total_cmp(&(b / d).ln().abs()))
        .unwrap_or(d);
    nearest * scale
}

/// The first rational of a field, validated against `range` (SECURITY.md §3, Layer 3).
#[allow(clippy::cast_possible_truncation)] // camera values fit f32 comfortably
fn rational(field: &Field, range: std::ops::RangeInclusive<f64>) -> Option<f32> {
    let Value::Rational(values) = &field.value else {
        return None;
    };
    let v = values.first()?.to_f64();
    (v.is_finite() && range.contains(&v)).then_some(v as f32)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The CC0 samples from `scripts/fetch-raw-samples.py`, when `FOTOLLECTION_RAW_SAMPLES`
    /// names their folder: every one gives its camera and capture time (FR-016).
    #[test]
    fn fr_016_raw_samples_have_their_exif() -> Result<(), Box<dyn std::error::Error>> {
        let Some(dir) = std::env::var_os("FOTOLLECTION_RAW_SAMPLES") else {
            return Ok(());
        };
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let Some(kind) = crate::raw::RawKind::from_extension(&ext) else {
                continue;
            };
            let meta = read_raw(&path, kind);
            eprintln!(
                "{}: {:?} {:?} {:?} f/{:?} {:?}s ISO {:?} {}mm orient {}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default(),
                meta.make,
                meta.model,
                meta.capture,
                meta.f_number,
                meta.exposure_s,
                meta.iso,
                meta.focal_mm.unwrap_or_default(),
                meta.orientation,
            );
            assert!(meta.make.is_some(), "{}: no make", path.display());
            assert!(
                meta.capture.is_some(),
                "{}: no capture time",
                path.display()
            );
            assert!(meta.iso.is_some(), "{}: no ISO", path.display());
            assert!(!meta.has_embedded_thumbnail);
        }
        Ok(())
    }
}
