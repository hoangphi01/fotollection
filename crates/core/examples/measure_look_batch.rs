// SPDX-License-Identifier: GPL-3.0-only
//! What giving one look to many photos costs (FR-114): a synthetic library of N photos (half
//! with a crop, a third with a look of their own), then the batch as `apply_look_to_photos`
//! runs it under the library's write lock (each photo's edits read, its look set, one History
//! step), and its undo; median of 10, a fresh history each run. No photo is read.
//!
//! `cargo run --release -p fotollection-core --example measure_look_batch -- [N…]`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::history::History;
use fotollection_core::import::ImportedFile;
use fotollection_core::library::{Library, PhotoEdits, PhotoId};
use fotollection_core::look_library::built_in;
use fotollection_core::metadata::PhotoMeta;
use fotollection_core::plan::CropFrame;

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
        vec![1_000, 10_000]
    } else {
        sizes
    };
    let looks = built_in();
    let given = looks
        .iter()
        .find(|e| e.id == "builtin:c200-natural")
        .unwrap()
        .look
        .clone();
    let own = looks
        .iter()
        .find(|e| e.id == "builtin:warm-film")
        .unwrap()
        .look
        .clone();
    for n in sizes {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/Users/demo/Pictures/Trip"));
        let mut ids: Vec<PhotoId> = Vec::with_capacity(n);
        for i in 0..n {
            let file = ImportedFile {
                path: PathBuf::from(format!("/Users/demo/Pictures/Trip/IMG_{i:05}.JPG")),
                size: 3_000_000,
                mtime: 1_700_000_000 + i64::try_from(i).unwrap(),
            };
            let id = lib
                .push(folder, &file, &PhotoMeta::default())
                .expect("new photo");
            lib.set_edits(
                id,
                PhotoEdits {
                    crop: (i % 2 == 0).then_some(CropFrame {
                        cx: 0.5,
                        cy: 0.4,
                        zoom: 1.2,
                    }),
                    look: (i % 3 == 0).then(|| own.clone()),
                    ..PhotoEdits::default()
                },
            );
            ids.push(id);
        }
        let (mut apply, mut undo) = (Vec::new(), Vec::new());
        for _ in 0..10 {
            let mut history = History::default();
            let t = Instant::now();
            let changes: Vec<(PhotoId, PhotoEdits)> = ids
                .iter()
                .map(|&id| {
                    let mut edits = lib.edits(id);
                    edits.look = Some(given.clone());
                    (id, edits)
                })
                .collect();
            history.apply_as(&mut lib, &changes, None);
            apply.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            history.undo(&mut lib).expect("a step to undo");
            undo.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "{n:>6} photos: apply {:.2} ms · undo {:.2} ms (median of 10, one History step)",
            median(apply),
            median(undo)
        );
    }
}
