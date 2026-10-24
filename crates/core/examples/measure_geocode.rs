// SPDX-License-Identifier: GPL-3.0-only
//! Measures the offline places (FR-005, FR-032) for OPTIMIZATION.md §5: first load (checksum
//! and index), reverse lookups and name search.
//!
//! `cargo run --release -p fotollection-core --example measure_geocode`

#![allow(clippy::unwrap_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::geocode;

fn main() {
    let t = Instant::now();
    let snapshot = geocode::snapshot().unwrap();
    let load_ms = t.elapsed().as_secs_f64() * 1e3;

    // A grid of positions across Europe and South-East Asia.
    let points: Vec<(f64, f64)> = (0..100)
        .flat_map(|i| (0..100).map(move |j| (8.0 + f64::from(i) * 0.5, -5.0 + f64::from(j) * 1.2)))
        .collect();
    let t = Instant::now();
    let found = points
        .iter()
        .filter(|(lat, lon)| geocode::reverse(*lat, *lon).unwrap().is_some())
        .count();
    let reverse_us = t.elapsed().as_secs_f64() * 1e6 / points.len() as f64;

    let queries = [
        "debre",
        "Hà Nội",
        "chi minh",
        "paris",
        "lisb",
        "sintra",
        "prag",
        "x",
    ];
    let t = Instant::now();
    for q in queries {
        geocode::search(q, 10).unwrap();
    }
    let search_ms = t.elapsed().as_secs_f64() * 1e3 / queries.len() as f64;

    println!(
        "places snapshot {snapshot}: first load {load_ms:.0} ms; reverse {reverse_us:.1} µs per lookup ({found}/{} on land); search {search_ms:.1} ms per query",
        points.len()
    );
}
