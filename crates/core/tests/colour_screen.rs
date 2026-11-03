// SPDX-License-Identifier: GPL-3.0-only
//! FR-072: a photo tagged Display P3 or Adobe RGB reaches every screen path in sRGB, the
//! thumbnail and the preview alike, and FR-071 names its space from the profile's primaries.
//! Profiles: the CC0 Compact ICC Profiles in `tests/fixtures/icc`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use fotollection_core::codec::{EncodeOptions, JpegCodec, PureRustCodec, decode_image};
use fotollection_core::facts::Space;
use fotollection_core::image::RgbImage;
use fotollection_core::import::scan;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::preview::{PreviewCache, Region};
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::ThumbCache;

const COLOUR: [u8; 3] = [200, 120, 60];

fn icc(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/icc/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// The middle pixel of a JPEG.
fn middle(codec: &PureRustCodec, jpeg: &[u8]) -> [u8; 3] {
    let image = decode_image(codec, jpeg).unwrap().image;
    let at = ((image.height / 2 * image.width + image.width / 2) * 3) as usize;
    [image.data[at], image.data[at + 1], image.data[at + 2]]
}

#[test]
fn fr_072_tagged_photos_are_srgb_on_screen() {
    let codec = PureRustCodec;
    let dir = tempfile::tempdir().unwrap();
    let flat = RgbImage::from_raw(320, 240, COLOUR.repeat(320 * 240)).unwrap();
    for (name, profile) in [
        ("p3.jpg", "DisplayP3-v4.icc"),
        ("adobe.jpg", "AdobeCompat-v4.icc"),
    ] {
        let jpeg = codec
            .encode(&flat, EncodeOptions::print(98), Some(&icc(profile)))
            .unwrap();
        std::fs::write(dir.path().join(name), jpeg).unwrap();
    }
    let mut lib = Library::new();
    let folder = lib.add_folder(dir.path());
    for file in scan(dir.path()).accepted {
        lib.push(folder, &file, &metadata::read_path(&file.path));
    }
    let thumbs = ThumbCache::new(dir.path().join("cache"));
    let previews = PreviewCache::new();
    for id in lib.sorted_ids() {
        let record = lib.record(id).unwrap();
        let (space, profile) = if record.file_name == "p3.jpg" {
            (Space::DisplayP3, "DisplayP3-v4.icc")
        } else {
            (Space::AdobeRgb, "AdobeCompat-v4.icc")
        };
        // Named by the primaries: these compact profiles are called "sP3" and "A98C".
        assert_eq!(record.facts.space, Some(space), "{}", record.file_name);
        let mut want = RgbImage::from_raw(1, 1, COLOUR.to_vec()).unwrap();
        fotollection_core::colour::to_srgb(&mut want, Some(&icc(profile)));
        let thumb = thumbs.make_t1(&record, &codec).unwrap();
        let preview = previews
            .render_jpeg(&record, &RenderSettings::default(), None, None, &codec)
            .unwrap();
        // The zoom converts the full image the preview kept in its own profile.
        let zoom = previews
            .render_region_jpeg(
                &record,
                &RenderSettings::default(),
                None,
                None,
                Region::new(0.25, 0.25, 0.5, 0.5, 400).unwrap(),
                &codec,
            )
            .unwrap();
        for (path, jpeg) in [("thumbnail", thumb), ("preview", preview), ("zoom", zoom)] {
            let got = middle(&codec, &jpeg);
            for c in 0..3 {
                assert!(
                    got[c].abs_diff(want.data[c]) <= 4,
                    "{} {path}: {got:?}, want {:?}",
                    record.file_name,
                    want.data
                );
            }
            assert_ne!(got, COLOUR, "{} {path} was not converted", record.file_name);
        }
    }
}
