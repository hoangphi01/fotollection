// SPDX-License-Identifier: GPL-3.0-only
//! Measures the metadata pass of an import (FR-002, FR-071, NFR-001) on a real folder: the
//! way it read before FR-071 (a 16-byte sniff, then EXIF through a buffered reader) against
//! `metadata::read_path` now (one 128 KB head for EXIF and the image facts). Each runs on
//! all threads, as the import does, best of five, after one warm-up so the page cache is warm
//! for both.
//!
//! `cargo run --release -p fotollection-core --example measure_import -- <photo-folder>`
//! `[--facts]`: also print each file's facts at import and once its whole file is read.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

use fotollection_core::import::scan;
use fotollection_core::metadata;
use rayon::prelude::*;

/// The pre-FR-071 pass: sniff 16 bytes in one open, then EXIF in another.
fn before(path: &PathBuf) -> bool {
    let mut magic = [0u8; 16];
    let _ = File::open(path).and_then(|mut f| f.read(&mut magic));
    File::open(path).is_ok_and(|f| {
        exif::Reader::new()
            .read_from_container(&mut BufReader::new(f))
            .is_ok()
    })
}

fn best(mut run: impl FnMut() -> usize) -> (f64, usize) {
    let _ = run();
    let mut best = f64::MAX;
    let mut n = 0;
    for _ in 0..5 {
        let t = Instant::now();
        n = run();
        best = best.min(t.elapsed().as_secs_f64() * 1e3);
    }
    (best, n)
}

fn main() {
    let folder = PathBuf::from(std::env::args().nth(1).expect("a photo folder"));
    let files: Vec<PathBuf> = scan(&folder).accepted.into_iter().map(|f| f.path).collect();
    let (old, with_exif) = best(|| files.par_iter().filter(|p| before(p)).count());
    let (new, with_facts) = best(|| {
        files
            .par_iter()
            .map(|p| metadata::read_path(p))
            .filter(|m| m.facts != fotollection_core::facts::ImageFacts::default())
            .count()
    });
    if std::env::args().any(|a| a == "--facts") {
        for p in &files {
            let head = metadata::read_path(p).facts;
            let bytes = std::fs::read(p).unwrap();
            let full = fotollection_core::raw::kind(&bytes)
                .and_then(|k| fotollection_core::facts::of_raw(&bytes, k));
            println!(
                "{}: {head:?}{}",
                p.file_name().unwrap().to_string_lossy(),
                full.map_or(String::new(), |f| format!("\n    whole file: {f:?}"))
            );
        }
    }
    let unusual = files
        .par_iter()
        .filter(|p| metadata::read_path(p).facts.unusual())
        .count();
    println!(
        "{} photos: before {old:.1} ms ({with_exif} with EXIF), now {new:.1} ms ({with_facts} with facts, {unusual} unusual), {:+.0} %",
        files.len(),
        (new / old - 1.0) * 100.0
    );
}
