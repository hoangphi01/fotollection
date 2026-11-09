// SPDX-License-Identifier: GPL-3.0-only
//! Measures zooming into a photo the way the preview window does (FR-057): the preview,
//! the first sharp region (the full decode), the next regions of a pan, and the next
//! photo's preview after them. One run each, in that order, as a user meets them.
//!
//! `cargo run --release -p fotollection-core --example measure_zoom -- <photo>…`
// A benchmark, not the app: it starts its own threads and pools (ADR-0048 binds the app).
#![allow(clippy::disallowed_methods)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::default_codec;
use fotollection_core::import::check_file;
use fotollection_core::library::{Library, PhotoRecord};
use fotollection_core::metadata;
use fotollection_core::preview::{PreviewCache, Region};
use fotollection_core::settings::RenderSettings;

fn ms<T>(f: impl FnOnce() -> T) -> (f64, T) {
    let t = Instant::now();
    let out = f();
    (t.elapsed().as_secs_f64() * 1e3, out)
}

fn main() {
    let codec = default_codec();
    let settings = RenderSettings::default();
    let mut lib = Library::new();
    let records: Vec<PhotoRecord> = std::env::args()
        .skip(1)
        .map(|path| {
            let file = check_file(path.as_ref()).expect("a photo");
            let meta = metadata::read_path(&file.path);
            let folder = lib.add_folder(file.path.parent().expect("parent"));
            let id = lib.push(folder, &file, &meta).expect("photo");
            lib.record(id).expect("record")
        })
        .collect();
    if std::env::var_os("CONCURRENT").is_some() {
        concurrent(&records, &settings);
        return;
    }
    let cache = PreviewCache::new();
    println!("| Photo | Preview | First region | Pan regions (each) | Next preview after |");
    println!("|---|---|---|---|---|");
    for (i, record) in records.iter().enumerate() {
        let (preview, _) = ms(|| {
            cache
                .render_jpeg(record, &settings, None, None, codec.as_ref())
                .unwrap()
        });
        // A 2560-px-wide view at about 2:1, then three pans across the photo.
        let at = |x: f64| Region::new(x, 0.3, 0.4, 0.4, 2560).unwrap();
        let (first, _) = ms(|| {
            cache
                .render_region_jpeg(record, &settings, None, None, at(0.1), codec.as_ref())
                .unwrap()
        });
        let pans: Vec<f64> = [0.2, 0.3, 0.4]
            .iter()
            .map(|&x| {
                ms(|| {
                    cache
                        .render_region_jpeg(record, &settings, None, None, at(x), codec.as_ref())
                        .unwrap()
                })
                .0
            })
            .collect();
        let pan = pans.iter().sum::<f64>() / pans.len() as f64;
        let next = records.get(i + 1).map(|next| {
            ms(|| {
                cache
                    .render_jpeg(next, &settings, None, None, codec.as_ref())
                    .unwrap()
            })
            .0
        });
        println!(
            "| {} | {preview:.0} ms | {first:.0} ms | {pan:.0} ms | {} |",
            record.path.file_name().unwrap().to_string_lossy(),
            next.map_or("–".to_owned(), |n| format!("{n:.0} ms")),
        );
    }
}

/// The preview window on the app's two-thread pool: each photo's preview, a sharp region
/// asked every 150 ms while the user zooms and pans, then the next photo. Prints when each
/// answer comes, from the start.
fn concurrent(records: &[PhotoRecord], settings: &RenderSettings) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let cache = std::sync::Arc::new(PreviewCache::new());
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let mut asked = 0;
    for record in records {
        let ask = |label: String, region: Option<Region>| {
            let (cache, record, settings, tx) =
                (cache.clone(), record.clone(), settings.clone(), tx.clone());
            let at = start.elapsed().as_secs_f64();
            pool.spawn(move || {
                let codec = default_codec();
                let ok = match region {
                    Some(r) => cache
                        .render_region_jpeg(&record, &settings, None, None, r, codec.as_ref())
                        .is_ok(),
                    None => cache
                        .render_jpeg(&record, &settings, None, None, codec.as_ref())
                        .is_ok(),
                };
                let _ = tx.send(format!(
                    "{label}: asked {at:.2} s, done {:.2} s{}",
                    start.elapsed().as_secs_f64(),
                    if ok { "" } else { " (error)" }
                ));
            });
        };
        ask(format!("{} preview", record.id.0), None);
        asked += 1;
        std::thread::sleep(std::time::Duration::from_millis(1000));
        for i in 0..6 {
            ask(
                format!("{} region {i}", record.id.0),
                Some(Region::new(0.1 + 0.05 * f64::from(i), 0.3, 0.4, 0.4, 2560).unwrap()),
            );
            asked += 1;
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }
    drop(tx);
    for _ in 0..asked {
        let Ok(line) = rx.recv_timeout(std::time::Duration::from_secs(120)) else {
            println!("no answer within 120 s: the pool is stuck");
            return;
        };
        println!("{line}");
    }
}
