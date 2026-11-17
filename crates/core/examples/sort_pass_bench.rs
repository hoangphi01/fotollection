// SPDX-License-Identifier: GPL-3.0-only
//! FR-141 S6, the app's pixels pass timed end to end (`sorting::pass`): a folder imported as
//! the app imports it, then each photo's hash and CLIP embedding on `--threads` (2 is the
//! app's background pool), cold (an empty cache and no thumbnail, so each file is decoded)
//! then warm (from the cache). Prints ms a photo and the minutes for 15,000 (NFR-022).
//!
//! `cargo run --release -p fotollection-core --example sort_pass_bench -- <folder> [--threads N] [--rounds R]`
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::default_codec;
use fotollection_core::import::scan;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::sorting::clip::{Embedder, MODEL_BLAKE3, subjects};
use fotollection_core::sorting::pass::SortingCache;
use fotollection_core::thumbs::ThumbCache;
use rayon::prelude::*;
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };
    let root = std::path::PathBuf::from(args.first().expect("usage: sort_pass_bench <folder>"));
    let threads: usize = flag("--threads").map_or(2, |n| n.parse().unwrap());
    // The same files again, each round a fresh path set to the cache (a bigger sample).
    let rounds: usize = flag("--rounds").map_or(1, |n| n.parse().unwrap());

    let t = Instant::now();
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/models/tinyclip-39m.w8.onnx"
    ))
    .unwrap();
    let embedder = Embedder::load(&bytes, MODEL_BLAKE3).unwrap();
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;

    let found = scan(&root);
    let mut lib = Library::new();
    let folder = lib.add_folder(&root);
    for file in &found.accepted {
        lib.push(folder, file, &metadata::read_path(&file.path));
    }
    let records: Vec<_> = lib
        .sorted_ids()
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect();
    let work = tempfile::tempdir().unwrap();
    let thumbs = ThumbCache::new(work.path().join("thumbs"));
    let codec = default_codec();
    let pool = fotollection_core::pool::contained(Some(threads), "sort-bench").unwrap();

    let mut passes = Vec::new();
    let mut top = std::collections::BTreeMap::<&str, usize>::new();
    for warm in [false, true] {
        let mut total = 0usize;
        let t = Instant::now();
        for round in 0..rounds {
            let cache = SortingCache::new(work.path().join(format!("sorting-{round}")));
            let results: Vec<_> = pool.install(|| {
                records
                    .par_iter()
                    .filter_map(|r| cache.pass(r, &thumbs, codec.as_ref(), &embedder).ok())
                    .collect()
            });
            total += results.len();
            if warm && round == 0 {
                for p in &results {
                    *top.entry(subjects(&p.embedding)[0].0).or_default() += 1;
                }
            }
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / total.max(1) as f64;
        passes.push(json!({
            "warm": warm, "photos": total, "ms_per_photo": ms,
            "minutes_for_15000": ms * 15_000.0 / 60_000.0,
        }));
    }
    println!(
        "{}",
        json!({
            "threads": threads, "files": records.len(), "load_ms": load_ms,
            "passes": passes, "top_subject": top,
        })
    );
}
