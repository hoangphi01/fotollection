// SPDX-License-Identifier: GPL-3.0-only
//! End-to-end: import → metadata → library → export, on synthetic fixtures.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::{Fixture, jpeg, write};
use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{JpegCodec, PureRustCodec};
use fotollection_core::export::{ExportOptions, estimate_export, run_export};
use fotollection_core::import::scan;
use fotollection_core::library::{CullFilter, Flag, FlagFilter, Library, PhotoEdits, PhotoRecord};
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
        ..Fixture::default()
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
fn fr_002_reads_lens_and_exposure() {
    let bytes = jpeg(&Fixture {
        width: 32,
        height: 24,
        lens: Some("LUMIX G 20/F1.7 II"),
        exposure: Some((20, 2.8, (1, 400), 200)),
        ..Fixture::default()
    });
    let meta = metadata::read_bytes(&bytes);
    assert_eq!(meta.lens.as_deref(), Some("LUMIX G 20/F1.7 II"));
    assert_eq!(meta.focal_mm, Some(20.0));
    assert!(meta.f_number.is_some_and(|f| (f - 2.8).abs() < 1e-3));
    assert!(
        meta.exposure_s
            .is_some_and(|s| (s - 1.0 / 400.0).abs() < 1e-6)
    );
    assert_eq!(meta.iso, Some(200));
}

#[test]
fn fr_040_shutter_speed_falls_back_to_the_apex_value() {
    // 1/2500 s is Tv = log2(2500) ≈ 11.29; no ExposureTime in the file.
    let bytes = jpeg(&Fixture {
        width: 32,
        height: 24,
        shutter_apex: Some(2500f64.log2()),
        ..Fixture::default()
    });
    let meta = metadata::read_bytes(&bytes);
    assert!(
        meta.exposure_s
            .is_some_and(|s| (s - 1.0 / 2500.0).abs() < 1e-7),
        "{:?}",
        meta.exposure_s
    );
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
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
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
    // The sizes the UI compares (FR-053): every original read, every file written.
    let originals: u64 = records(&lib).iter().map(|r| r.size).sum();
    #[allow(clippy::cast_precision_loss)]
    let originals = originals as f64;
    assert!((summary.source_bytes - originals).abs() < 1.0);
    assert!(summary.output_bytes > 0.0);

    let day1 = dest.path().join("2026-07-14");
    assert_eq!(std::fs::read(day1.join("IMG_1.jpg")).unwrap(), b"keep me");
    assert!(day1.join("IMG_1_1.jpg").exists() && day1.join("IMG_1_2.jpg").exists());
    assert!(dest.path().join("2026-07-15/IMG_2.jpg").exists());

    // Output is cropped to 2:3 (10x15) and carries no GPS (FR-010).
    let out = std::fs::read(day1.join("IMG_1_1.jpg")).unwrap();
    let decoded = PureRustCodec.decode(&out).unwrap().image;
    assert_eq!((decoded.width, decoded.height), (300, 200));
    assert_eq!(metadata::read_bytes(&out).gps, None);
}

#[test]
fn fr_056_folders_exported_separately_get_their_own_subfolder() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let a = Fixture {
        width: 300,
        height: 200,
        date: Some("2026:07:14 10:00:00"),
        ..Fixture::default()
    };
    write(src.path(), "IMG_1.jpg", &jpeg(&a));
    write(src.path(), "IMG_2.jpg", &jpeg(&a));
    let lib = import(src.path());
    let settings = RenderSettings::default();
    let group = |r: &fotollection_core::library::PhotoRecord| {
        Some(
            if r.file_name == "IMG_1.jpg" {
                "Trip"
            } else {
                "Home"
            }
            .to_owned(),
        )
    };
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &group,
    };
    let summary = run_export(
        &records(&lib),
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!(summary.exported, 2);
    assert!(dest.path().join("Trip/2026-07-14/IMG_1.jpg").exists());
    assert!(dest.path().join("Home/2026-07-14/IMG_2.jpg").exists());
}

#[test]
fn fr_053_the_estimate_is_exact_when_every_photo_is_sampled() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    for (i, w) in [(1, 640), (2, 480), (3, 800)] {
        let f = Fixture {
            width: w,
            height: 400,
            date: Some("2026:07:14 10:00:00"),
            ..Fixture::default()
        };
        write(src.path(), &format!("IMG_{i}.jpg"), &jpeg(&f));
    }
    let lib = import(src.path());
    let settings = RenderSettings {
        export_target: fotollection_core::share::ExportTarget::Share(
            fotollection_core::share::SharePreset::Original,
        ),
        ..RenderSettings::default()
    };
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
    };
    let all = records(&lib);
    let estimate = estimate_export(&all, &options, &PureRustCodec, 10);
    assert_eq!((estimate.photos, estimate.sampled), (3, 3));
    // Nothing was written by the estimate.
    assert_eq!(walk(dest.path()).len(), 0);
    let summary = run_export(
        &all,
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert!((estimate.estimated_bytes - summary.output_bytes).abs() < 1.0);
    assert!((estimate.source_bytes - summary.source_bytes).abs() < 1.0);
    // One photo sampled still gives a figure of the right size.
    let rough = estimate_export(&all, &options, &PureRustCodec, 1);
    assert_eq!(rough.sampled, 1);
    assert!(rough.estimated_bytes > 0.0);
}

#[test]
fn fr_010_prints_keep_the_camera_details_but_never_gps() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let f = Fixture {
        width: 300,
        height: 200,
        date: Some("2026:07:14 10:22:31"),
        gps: Some((47.4979, 19.0402)),
        lens: Some("LUMIX G 20/F1.7 II"),
        exposure: Some((20, 1.7, (1, 250), 200)),
        ..Fixture::default()
    };
    write(src.path(), "IMG_1.jpg", &jpeg(&f));
    let lib = import(src.path());
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 1,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
    };
    let summary = run_export(
        &records(&lib),
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!(summary.exported, 1);
    let out = std::fs::read(dest.path().join("2026-07-14/IMG_1.jpg")).unwrap();
    let meta = metadata::read_bytes(&out);
    assert_eq!(meta.gps, None, "GPS never leaves the machine");
    assert_eq!(
        meta.capture.map(|d| d.to_string()).as_deref(),
        Some("2026-07-14 10:22:31")
    );
    assert_eq!(meta.lens.as_deref(), Some("LUMIX G 20/F1.7 II"));
    assert_eq!(meta.iso, Some(200));
    // Pixels are upright, and the image still decodes.
    assert_eq!(meta.orientation, 1);
    assert!(PureRustCodec.decode(&out).is_ok());
}

#[test]
fn fr_001_png_and_jpeg_export_together() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    write(
        src.path(),
        "IMG_1.jpg",
        &jpeg(&Fixture {
            width: 300,
            height: 200,
            date: Some("2026:07:14 10:00:00"),
            ..Fixture::default()
        }),
    );
    // A 400 × 200 semi-transparent PNG screenshot, with no EXIF date.
    let mut png_file = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_file, 400, 200);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&[0, 0, 0, 0].repeat(400 * 200))
            .unwrap();
    }
    write(src.path(), "screen.png", &png_file);

    let lib = import(src.path());
    let all = records(&lib);
    assert_eq!(all.len(), 2);
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
    };
    let summary = run_export(
        &all,
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!((summary.exported, summary.failed.len()), (2, 0));

    let png_out = walk(dest.path())
        .into_iter()
        .find(|p| p.file_name().is_some_and(|n| n == "screen.jpg"))
        .expect("screen.png is exported as screen.jpg");
    let decoded = PureRustCodec
        .decode(&std::fs::read(png_out).unwrap())
        .unwrap()
        .image;
    // Cropped from 2:1 to the 10x15 ratio, transparency flattened onto white.
    assert_eq!((decoded.width, decoded.height), (300, 200));
    assert!(decoded.data[..3].iter().all(|&c| c > 245));
}

#[test]
fn fr_022_export_uses_only_the_filtered_set() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    for (name, date) in [
        ("a.jpg", "2026:07:14 10:00:00"),
        ("b.jpg", "2026:07:14 10:00:01"),
        ("c.jpg", "2026:07:14 10:00:02"),
    ] {
        let fixture = Fixture {
            width: 60,
            height: 40,
            date: Some(date),
            ..Fixture::default()
        };
        write(src.path(), name, &jpeg(&fixture));
    }
    let mut lib = import(src.path());
    let ids = lib.sorted_ids();
    let flag = |flag, rating| PhotoEdits {
        flag: Some(flag),
        rating,
        ..PhotoEdits::default()
    };
    lib.set_edits(ids[0], flag(Flag::Pick, 5));
    lib.set_edits(ids[2], flag(Flag::Reject, 0));

    let picks = CullFilter {
        flag: FlagFilter::Picks,
        ..CullFilter::default()
    };
    let kept: Vec<_> = lib
        .filtered_ids(&picks)
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect();
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 2,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
    };
    let summary = run_export(
        &kept,
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!(summary.exported, 1);
    let names: Vec<_> = walk(dest.path())
        .iter()
        .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
        .collect();
    assert_eq!(names, ["a.jpg"]);

    // "All" keeps rejects too: the grid and the export always show the same set.
    assert_eq!(lib.filtered_ids(&CullFilter::default()).len(), 3);
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
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
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
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
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

/// Copies a directory tree (test helper).
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn trip_with_edits(root: &Path) -> (Library, std::path::PathBuf) {
    let photos = root.join("Lisbon");
    for (name, date) in [
        ("a.jpg", "2026:07:14 10:00:00"),
        ("b.jpg", "2026:07:14 11:00:00"),
    ] {
        let fixture = Fixture {
            width: 60,
            height: 40,
            date: Some(date),
            ..Fixture::default()
        };
        write(&photos, name, &jpeg(&fixture));
    }
    let mut lib = import(&photos);
    let a = lib.find_path(&photos.join("a.jpg")).unwrap();
    lib.set_edits(
        a,
        PhotoEdits {
            rating: 5,
            flag: Some(Flag::Pick),
            ..PhotoEdits::default()
        },
    );
    (lib, photos)
}

#[test]
fn fr_023_save_open_roundtrip_on_another_machine() {
    use fotollection_core::bundle::{self, Bundle, Opened};
    use fotollection_core::project;

    // Machine A: photos in Trip/Lisbon, the project saved next to them as Trip/Trip.fotollection.
    let a = tempfile::tempdir().unwrap();
    let trip = a.path().join("Trip");
    let (lib, _) = trip_with_edits(&trip);
    let mut file = project::snapshot(&lib);
    project::relocate(&mut file, &trip);
    let saved = Bundle {
        project: file,
        settings: fotollection_core::settings_store::StoredSettings::default(),
        fonts: std::collections::BTreeMap::default(),
        logos: std::collections::BTreeMap::default(),
    };
    bundle::save(&trip.join("Trip.fotollection"), &saved).unwrap();

    // Machine B: the whole Trip folder copied somewhere else (USB stick, other user...).
    let b = tempfile::tempdir().unwrap();
    let copied = b.path().join("Copied Trip");
    copy_dir(&trip, &copied);
    std::fs::remove_dir_all(&trip).unwrap(); // and machine A's copy is gone
    let Ok(Opened::Bundle(opened)) = bundle::open(&copied.join("Trip.fotollection")) else {
        panic!("the bundle must open on machine B");
    };
    let found = project::resolve_folders(&opened.project, Some(&copied));
    let folder = found[0].path.clone().expect("found relative to the bundle");
    assert_eq!(folder, copied.join("Lisbon"));

    let mut fresh = import(&folder);
    let summary = project::restore_into(
        &mut fresh,
        &opened.project,
        &[(found[0].id, folder.clone())],
    );
    assert_eq!((summary.restored, summary.missing), (1, 0));
    let a_id = fresh.find_path(&folder.join("a.jpg")).unwrap();
    assert_eq!(fresh.edits(a_id).rating, 5);
    assert_eq!(fresh.edits(a_id).flag, Some(Flag::Pick));
}

#[test]
fn fr_023_relink_missing_folder() {
    use fotollection_core::project;

    let root = tempfile::tempdir().unwrap();
    let (lib, photos) = trip_with_edits(root.path());
    let file = project::snapshot(&lib); // absolute paths, like an old project
    // The photos were moved; the saved path no longer exists.
    let moved = root.path().join("Archive/Lisbon 2026");
    std::fs::create_dir_all(moved.parent().unwrap()).unwrap();
    std::fs::rename(&photos, &moved).unwrap();
    let found = project::resolve_folders(&file, None);
    assert_eq!(found[0].path, None, "reported as missing");

    // The user points the folder at its new place.
    let mut fresh = import(&moved);
    let summary = project::restore_into(&mut fresh, &file, &[(found[0].id, moved.clone())]);
    assert_eq!(summary.restored, 1);
    let a_id = fresh.find_path(&moved.join("a.jpg")).unwrap();
    assert_eq!(fresh.edits(a_id).rating, 5);
}

/// HEIC through the OS codec (ADR-0017), on macOS where `sips` can write it: import with
/// EXIF date, then export to JPEG. The HEIC is made from a synthetic JPEG; no photo is
/// committed.
#[cfg(target_os = "macos")]
#[test]
fn fr_001_heic_imports_and_exports() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let jpg = write(
        src.path(),
        "tmp/source.jpg",
        &jpeg(&Fixture {
            width: 300,
            height: 200,
            date: Some("2026:05:01 09:30:00"),
            ..Fixture::default()
        }),
    );
    let heic = src.path().join("IMG_0001.HEIC");
    let out = std::process::Command::new("sips")
        .args(["-s", "format", "heic"])
        .arg(&jpg)
        .arg("--out")
        .arg(&heic)
        .output()
        .unwrap();
    assert!(out.status.success(), "sips failed: {out:?}");
    std::fs::remove_dir_all(src.path().join("tmp")).unwrap();

    let lib = import(src.path());
    let all = records(&lib);
    assert_eq!(all.len(), 1, "the HEIC is accepted");
    assert_eq!(
        all[0].capture.to_string(),
        "2026-05-01 09:30:00",
        "EXIF is read from the HEIC"
    );
    let settings = RenderSettings::default();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: 90,
        max_inflight: 1,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
    };
    let summary = run_export(
        &all,
        &options,
        &PureRustCodec,
        &CancellationToken::new(),
        &|_, _| {},
    );
    assert_eq!((summary.exported, summary.failed.len()), (1, 0));
    let exported = dest.path().join("2026-05-01/IMG_0001.jpg");
    let decoded = PureRustCodec
        .decode(&std::fs::read(exported).unwrap())
        .unwrap()
        .image;
    assert_eq!((decoded.width, decoded.height), (300, 200));
}

#[test]
fn fr_053_share_export_resizes_keeps_ratio_and_writes_safe_exif() {
    use fotollection_core::share::{ExportTarget, SharePreset};
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    write(
        src.path(),
        "IMG_1.jpg",
        &jpeg(&Fixture {
            width: 3000,
            height: 2000,
            date: Some("2026:07:14 10:00:00"),
            gps: Some((47.5, 19.04)),
            lens: Some("XF56mmF1.2 R"),
            exposure: Some((56, 1.2, (1, 250), 160)),
            ..Fixture::default()
        }),
    );
    let lib = import(src.path());
    let export = |preset| {
        let settings = RenderSettings {
            export_target: ExportTarget::Share(preset),
            ..RenderSettings::default()
        };
        let options = ExportOptions {
            destination: dest.path(),
            settings: &settings,
            quality: 90,
            max_inflight: 1,
            place_for: &|_| None,
            frame_for: &|_| None,
            group_for: &|_| None,
        };
        let summary = run_export(
            &records(&lib),
            &options,
            &PureRustCodec,
            &CancellationToken::new(),
            &|_, _| {},
        );
        assert_eq!(summary.exported, 1);
    };

    export(SharePreset::Custom(1000));
    let out = std::fs::read(dest.path().join("Share/2026-07-14/IMG_1.jpg")).unwrap();
    let image = PureRustCodec.decode(&out).unwrap().image;
    // The photo's own 3:2 ratio, not the 10 × 15 print crop.
    assert_eq!((image.width, image.height), (1000, 667));
    let meta = metadata::read_bytes(&out);
    assert_eq!(meta.gps, None, "never a location");
    assert_eq!(meta.lens.as_deref(), Some("XF56mmF1.2 R"));
    let exif = exif::Reader::new()
        .read_from_container(&mut std::io::Cursor::new(&out))
        .unwrap();
    let software = exif
        .get_field(exif::Tag::Software, exif::In::PRIMARY)
        .unwrap()
        .display_value()
        .to_string();
    assert!(software.contains("Fotollection"), "{software}");

    // Original keeps every pixel, only the file is lighter.
    export(SharePreset::Original);
    let out = std::fs::read(dest.path().join("Share/2026-07-14/IMG_1_1.jpg")).unwrap();
    let image = PureRustCodec.decode(&out).unwrap().image;
    assert_eq!((image.width, image.height), (3000, 2000));
}
