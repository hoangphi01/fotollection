// SPDX-License-Identifier: GPL-3.0-only
//! End-to-end: import → metadata → library → export, on synthetic fixtures.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::{Fixture, jpeg, write};
use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{JpegCodec, PureRustCodec};
use fotollection_core::export::{ExportOptions, run_export};
use fotollection_core::import::scan;
use fotollection_core::library::{Library, PhotoRecord};
use fotollection_core::metadata;
use fotollection_core::settings::RenderSettings;

fn import(root: &Path) -> Library {
    let mut lib = Library::new();
    let folder = lib.add_folder(root);
    for file in scan(root).accepted {
        let meta = metadata::read_path(&file.path);
        lib.push(folder, &file, &meta);
    }
    lib
}

fn records(lib: &Library) -> Vec<PhotoRecord> {
    lib.sorted_ids()
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect()
}

#[test]
fn fr_002_reads_date_gps_orientation() {
    let bytes = jpeg(&Fixture {
        width: 64,
        height: 48,
        date: Some("2026:07:14 10:22:31"),
        gps: Some((38.8029, -9.3817)),
        orientation: Some(6),
    });
    let meta = metadata::read_bytes(&bytes);
    assert_eq!(
        meta.capture.map(|d| d.to_string()).as_deref(),
        Some("2026-07-14 10:22:31")
    );
    let (lat, lon) = meta.gps.expect("gps");
    assert!((lat - 38.8029).abs() < 1e-3 && (lon + 9.3817).abs() < 1e-3);
    assert_eq!(meta.orientation, 6);
}

#[test]
fn fr_002_null_island_is_no_gps_and_bad_year_is_no_date() {
    let bytes = jpeg(&Fixture {
        width: 16,
        height: 16,
        date: Some("1850:01:01 00:00:00"),
        gps: Some((0.0, 0.0)),
        ..Fixture::default()
    });
    let meta = metadata::read_bytes(&bytes);
    assert_eq!(meta.gps, None);
    assert_eq!(meta.capture, None);
    assert_eq!(meta.orientation, 1);
}

#[test]
fn fr_009_export_by_date_never_overwrites() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let a = Fixture {
        width: 300,
        height: 200,
        date: Some("2026:07:14 10:00:00"),
        ..Fixture::default()
    };
    write(src.path(), "a/IMG_1.jpg", &jpeg(&a));
    // Same file name from another folder on the same day.
    write(src.path(), "b/IMG_1.jpg", &jpeg(&a));
    write(
        src.path(),
        "IMG_2.jpg",
        &jpeg(&Fixture {
            date: Some("2026:07:15 09:00:00"),
            ..a.clone()
        }),
    );
    // A pre-existing file at the destination must survive.
    write(dest.path(), "2026-07-14/IMG_1.jpg", b"keep me");

    let lib = import(src.path());
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
    };
    let summary = run_export(
        &records(&lib),
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!(summary.exported, 3);
    assert!(summary.failed.is_empty());

    let day1 = dest.path().join("2026-07-14");
    assert_eq!(std::fs::read(day1.join("IMG_1.jpg")).unwrap(), b"keep me");
    assert!(day1.join("IMG_1_1.jpg").exists() && day1.join("IMG_1_2.jpg").exists());
    assert!(dest.path().join("2026-07-15/IMG_2.jpg").exists());

    // Output is cropped to 2:3 (10x15) and carries no EXIF/GPS (FR-010).
    let out = std::fs::read(day1.join("IMG_1_1.jpg")).unwrap();
    let decoded = PureRustCodec.decode(&out).unwrap().image;
    assert_eq!((decoded.width, decoded.height), (300, 200));
    assert_eq!(metadata::read_bytes(&out).gps, None);
}

#[test]
fn fr_011_bad_file_does_not_fail_batch() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let good = Fixture {
        width: 120,
        height: 80,
        date: Some("2026:07:14 10:00:00"),
        ..Fixture::default()
    };
    write(src.path(), "good.jpg", &jpeg(&good));
    // Valid magic bytes, broken body: accepted by import, fails at decode.
    write(
        src.path(),
        "broken.jpg",
        b"\xFF\xD8\xFF\xE0garbage garbage garbage",
    );

    let lib = import(src.path());
    assert_eq!(lib.len(), 2);
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
    };
    let summary = run_export(
        &records(&lib),
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!(summary.exported, 1);
    assert_eq!(summary.failed.len(), 1);
    assert_eq!(summary.failed[0].file_name, "broken.jpg");
    assert_eq!(summary.failed[0].code, "decode.failed");
}

#[test]
fn fr_011_cancel_stops_and_removes_partials() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    for i in 0..6 {
        let f = Fixture {
            width: 200,
            height: 150,
            date: Some("2026:07:14 10:00:00"),
            ..Fixture::default()
        };
        write(src.path(), &format!("IMG_{i}.jpg"), &jpeg(&f));
    }
    let lib = import(src.path());
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 1,
    };
    let cancel = CancellationToken::new();
    let summary = run_export(
        &records(&lib),
        &options,
        &PureRustCodec,
        &cancel,
        &|done, _| {
            if done == 2 {
                cancel.cancel();
            }
        },
    );
    assert!(summary.cancelled);
    assert!(summary.exported < 6);
    let leftovers: Vec<_> = walk(dest.path())
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "part"))
        .collect();
    assert!(leftovers.is_empty(), "partial files left: {leftovers:?}");
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}
