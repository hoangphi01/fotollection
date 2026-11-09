// SPDX-License-Identifier: GPL-3.0-only
//! What a thumbnail through its photo's look costs (FR-114, ADR-0047): each JPEG of a folder
//! made into its 400 px T1 as the app makes it, then `look_thumbs::tint_jpeg` (decode, the
//! look's table, encode) timed per thumbnail, and a batch of 500 on a pool of two threads, as
//! the app's background pool colours a grid after a batch. Photos are only read; only aggregate
//! numbers are printed.
//!
//! `cargo run --release -p fotollection-core --example measure_look_thumbs -- <folder> [photos]`
// A benchmark, not the app: it starts its own threads and pools (ADR-0048 binds the app).
#![allow(clippy::disallowed_methods)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::time::Instant;

use fotollection_core::codec::{EncodeOptions, decode_for_screen, default_codec};
use fotollection_core::look_library::built_in;
use fotollection_core::look_thumbs::tint_jpeg;
use fotollection_core::render::fit_long_edge;
use rayon::prelude::*;

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let folder = args
        .next()
        .expect("usage: measure_look_thumbs <folder> [photos]");
    let limit: usize = args.next().map_or(24, |n| n.parse().expect("a count"));
    let codec = default_codec();
    let look = built_in()
        .into_iter()
        .find(|e| e.id == "builtin:c200-natural")
        .unwrap()
        .look;
    let mut thumbs = Vec::new();
    let mut paths: Vec<_> = std::fs::read_dir(&folder)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("jpg")))
        .collect();
    paths.sort();
    for path in paths.into_iter().take(limit) {
        let bytes = std::fs::read(&path).unwrap();
        let decoded = decode_for_screen(codec.as_ref(), &bytes, 400).unwrap();
        let small = fit_long_edge(&decoded.image, 400).unwrap();
        thumbs.push(
            codec
                .encode(&small, EncodeOptions::screen(82), None)
                .unwrap(),
        );
    }
    tint_jpeg(&thumbs[0], &look, codec.as_ref()).unwrap(); // the look's table, built once
    let mut times = Vec::new();
    for _ in 0..10 {
        for t in &thumbs {
            let start = Instant::now();
            tint_jpeg(t, &look, codec.as_ref()).unwrap();
            times.push(start.elapsed().as_secs_f64() * 1e3);
        }
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let start = Instant::now();
    pool.install(|| {
        (0..500).into_par_iter().for_each(|i| {
            tint_jpeg(&thumbs[i % thumbs.len()], &look, codec.as_ref()).unwrap();
        });
    });
    let batch = start.elapsed().as_secs_f64() * 1e3;
    println!(
        "{} thumbnails, 10 runs: one {:.2} ms p50, {:.2} ms p95 · 500 on two threads {batch:.0} ms",
        thumbs.len(),
        pct(&mut times.clone(), 0.5),
        pct(&mut times, 0.95)
    );
}
