// SPDX-License-Identifier: GPL-3.0-only
//! Measures saving the session project as the library grows (NFR-013, ADR-0010): a synthetic
//! library of N photos, every one edited (the worst case: a whole trip rated), then the
//! snapshot, the JSON encoding and the atomic write, median of 5. No photo is read.
//!
//! `cargo run --release -p fotollection-core --example measure_project -- [N…]`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::import::ImportedFile;
use fotollection_core::library::{Flag, Library, PhotoEdits};
use fotollection_core::metadata::PhotoMeta;
use fotollection_core::plan::CropFrame;
use fotollection_core::project;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let sizes: Vec<usize> = std::env::args()
        .skip(1)
        .map(|a| a.parse().expect("a photo count"))
        .collect();
    let sizes = if sizes.is_empty() {
        vec![5_035, 20_000, 70_000]
    } else {
        sizes
    };
    let dir = tempfile::tempdir().expect("temp dir");
    for n in sizes {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/Users/demo/Pictures/Trip"));
        for i in 0..n {
            let file = ImportedFile {
                path: PathBuf::from(format!(
                    "/Users/demo/Pictures/Trip/{:03}/IMG_{i:05}.JPG",
                    i / 1000
                )),
                size: 3_000_000,
                mtime: 1_700_000_000 + i64::try_from(i).unwrap(),
            };
            let id = lib
                .push(folder, &file, &PhotoMeta::default())
                .expect("new photo");
            lib.set_edits(
                id,
                PhotoEdits {
                    rating: u8::try_from(i % 6).unwrap(),
                    flag: (i % 3 == 0).then_some(Flag::Pick),
                    crop: (i % 2 == 0).then_some(CropFrame {
                        cx: 0.5,
                        cy: 0.4,
                        zoom: 1.2,
                    }),
                    ..PhotoEdits::default()
                },
            );
        }
        let path = dir.path().join(format!("session-{n}.fotollection"));
        let (mut snap, mut enc, mut write, mut bytes) = (Vec::new(), Vec::new(), Vec::new(), 0);
        for _ in 0..5 {
            let t = Instant::now();
            let file = project::snapshot(&lib);
            snap.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            let data = project::to_bytes(&file).unwrap();
            enc.push(t.elapsed().as_secs_f64() * 1e3);
            bytes = data.len();
            let t = Instant::now();
            project::save(&path, &file).unwrap();
            write.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let (s, e, w) = (median(snap), median(enc), median(write));
        println!(
            "{n:>6} photos, all edited: snapshot {s:.1} ms (library locked) · JSON {e:.1} ms · write {w:.1} ms · total {:.1} ms · {:.1} MB",
            s + e + w,
            bytes as f64 / 1e6
        );
    }
}
