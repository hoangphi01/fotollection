// SPDX-License-Identifier: GPL-3.0-only
//! NFR-020: how long the borrowed-place pass takes (FR-031). A synthetic trip of `n` camera
//! photos without GPS and `n` phone photos with GPS, one of each per minute, with visual
//! signatures already computed (as after a best-shot analysis). Times `places::borrow`.
//!
//! `cargo run --release -p fotollection-core --example measure_borrow -- 1000 [worst]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use fotollection_core::culling::similar::{self, Signature};
use fotollection_core::image::RgbImage;
use fotollection_core::import::ImportedFile;
use fotollection_core::library::{Library, PhotoId};
use fotollection_core::metadata::PhotoMeta;
use fotollection_core::places;

/// `flip` mirrors the scene: the same colours with another layout, so every comparison
/// passes the cheap colour check and pays for the layout one (the worst case).
fn scene(seed: u32, flip: bool) -> Signature {
    let (w, h) = (192u32, 128u32);
    let mut data = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            let x = if flip { w - 1 - x } else { x };
            let block = ((x / 24 + y / 24 + seed) % 4) as u8;
            let v = ((x * 255 / w) as u8).wrapping_add((seed % 7) as u8 * 30);
            data.extend_from_slice(&[v, block * 60, 255 - v / (block + 1)]);
        }
    }
    similar::signature(&RgbImage::from_raw(w, h, data).expect("image"))
}

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .map_or(1000, |a| a.parse().expect("count"));
    let worst = std::env::args().nth(2).as_deref() == Some("worst");
    let mut lib = Library::new();
    let folder = lib.add_folder(Path::new("/trip"));
    let mut sigs: HashMap<PhotoId, Signature> = HashMap::new();
    let start = chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
        .and_then(|d| d.and_hms_opt(8, 0, 0))
        .expect("date");
    for i in 0..n {
        let at = start + chrono::Duration::minutes(i64::from(i));
        for (model, gps, name) in [
            ("DC-GX9", None, "P"),
            ("iPhone", Some((47.53, 21.63)), "IMG"),
        ] {
            let meta = PhotoMeta {
                capture: Some(at),
                gps,
                make: Some("Maker".into()),
                model: Some(model.into()),
                ..PhotoMeta::default()
            };
            let file = ImportedFile {
                path: format!("/trip/{name}_{i}.jpg").into(),
                size: 1,
                mtime: 1,
            };
            let id = lib.push(folder, &file, &meta).expect("push");
            sigs.insert(id, scene(i / 3, worst && gps.is_some()));
        }
    }
    let mut times = Vec::new();
    let mut found = 0;
    for _ in 0..5 {
        let t = Instant::now();
        found = places::borrow(&lib, places::BORROW_WINDOW_S, |id| sigs.get(&id)).len();
        times.push(t.elapsed().as_secs_f64());
    }
    times.sort_by(f64::total_cmp);
    let case = if worst { "worst case" } else { "same scenes" };
    println!(
        "{case}, {n} camera + {n} phone photos: borrow pass {:.0} ms (median of 5), {found} places found",
        times[2] * 1e3
    );
}
