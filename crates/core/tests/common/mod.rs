// SPDX-License-Identifier: GPL-3.0-only
//! Synthetic fixtures: JPEGs with chosen EXIF, generated in tests so no personal photos
//! are ever committed (TESTING.md §3).

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use fotollection_core::codec::{EncodeOptions, JpegCodec, PureRustCodec};
use fotollection_core::image::RgbImage;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;

#[derive(Default, Clone)]
pub struct Fixture {
    pub width: u32,
    pub height: u32,
    /// EXIF `YYYY:MM:DD HH:MM:SS`
    pub date: Option<&'static str>,
    /// `(lat, lon)`, degrees
    pub gps: Option<(f64, f64)>,
    pub orientation: Option<u16>,
    pub lens: Option<&'static str>,
    /// (focal mm, f-number, exposure seconds as (num, den), ISO)
    pub exposure: Option<(u32, f64, (u32, u32), u16)>,
}

fn dms(value: f64) -> Vec<uR64> {
    let v = value.abs();
    let deg = v.floor();
    let min = ((v - deg) * 60.0).floor();
    let sec = ((v - deg) * 60.0 - min) * 60.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = |x: f64, d: u32| uR64 {
        nominator: (x * f64::from(d)).round() as u32,
        denominator: d,
    };
    vec![r(deg, 1), r(min, 1), r(sec, 1000)]
}

/// A gradient JPEG with the requested EXIF fields.
pub fn jpeg(f: &Fixture) -> Vec<u8> {
    let (w, h) = (f.width.max(8), f.height.max(8));
    let mut image = RgbImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 3) as usize;
            image.data[i] = u8::try_from(x * 255 / w).unwrap();
            image.data[i + 1] = u8::try_from(y * 255 / h).unwrap();
            image.data[i + 2] = 128;
        }
    }
    let mut bytes = PureRustCodec
        .encode(&image, EncodeOptions::print(90), None)
        .unwrap();

    let mut meta = Metadata::new();
    if let Some(date) = f.date {
        meta.set_tag(ExifTag::DateTimeOriginal(date.to_owned()));
    }
    if let Some((lat, lon)) = f.gps {
        meta.set_tag(ExifTag::GPSLatitudeRef(
            if lat < 0.0 { "S" } else { "N" }.to_owned(),
        ));
        meta.set_tag(ExifTag::GPSLatitude(dms(lat)));
        meta.set_tag(ExifTag::GPSLongitudeRef(
            if lon < 0.0 { "W" } else { "E" }.to_owned(),
        ));
        meta.set_tag(ExifTag::GPSLongitude(dms(lon)));
    }
    if let Some(o) = f.orientation {
        meta.set_tag(ExifTag::Orientation(vec![o]));
    }
    if let Some(lens) = f.lens {
        meta.set_tag(ExifTag::LensModel(lens.to_owned()));
    }
    if let Some((focal, fnum, (num, den), iso)) = f.exposure {
        meta.set_tag(ExifTag::FocalLength(vec![uR64 {
            nominator: focal,
            denominator: 1,
        }]));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let fnum = uR64 {
            nominator: (fnum * 10.0).round() as u32,
            denominator: 10,
        };
        meta.set_tag(ExifTag::FNumber(vec![fnum]));
        meta.set_tag(ExifTag::ExposureTime(vec![uR64 {
            nominator: num,
            denominator: den,
        }]));
        meta.set_tag(ExifTag::ISO(vec![iso]));
    }
    if f.date.is_some()
        || f.gps.is_some()
        || f.orientation.is_some()
        || f.lens.is_some()
        || f.exposure.is_some()
    {
        meta.write_to_vec(&mut bytes, FileExtension::JPEG).unwrap();
    }
    bytes
}

pub fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, bytes).unwrap();
    path
}
