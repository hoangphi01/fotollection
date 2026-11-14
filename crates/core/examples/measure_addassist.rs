// SPDX-License-Identifier: GPL-3.0-only
//! Measures `AddAssist` (FR-133) for OPTIMIZATION.md §5: the first load (checksum and table),
//! the first search (which folds every name once), code lookups and searches, each the
//! median of five runs where it repeats.
//!
//! `cargo run --release -p fotollection-core --example measure_addassist`

#![allow(clippy::unwrap_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::addassist::{self, Country};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let t = Instant::now();
    let info = addassist::countries().unwrap();
    let load_ms = t.elapsed().as_secs_f64() * 1e3;

    let t = Instant::now();
    addassist::lookup(Country::Jp, "2480024").unwrap();
    let first_lookup_ms = t.elapsed().as_secs_f64() * 1e3;

    let codes: Vec<String> = (0..10_000)
        .map(|i| format!("{:07}", (i * 997) % 10_000_000))
        .collect();
    let lookup_us = median(
        (0..5)
            .map(|_| {
                let t = Instant::now();
                for c in &codes {
                    addassist::lookup(Country::Jp, c).unwrap();
                }
                t.elapsed().as_secs_f64() * 1e6 / codes.len() as f64
            })
            .collect(),
    );

    let queries = [
        "kamakura",
        "かまくら",
        "鎌倉",
        "inamuragasaki",
        "tokyo",
        "248",
        "shinjuku",
        "x",
    ];
    let search_ms = median(
        (0..5)
            .map(|_| {
                let t = Instant::now();
                for q in queries {
                    addassist::search(Country::Jp, q, 20).unwrap();
                }
                t.elapsed().as_secs_f64() * 1e3 / queries.len() as f64
            })
            .collect(),
    );

    println!(
        "AddAssist {} (Japan data {}): first load {load_ms:.0} ms; first lookup (folds every name) {first_lookup_ms:.0} ms; lookup {lookup_us:.1} µs; search {search_ms:.1} ms per query",
        addassist::VERSION,
        info[0].data_date
    );
}
