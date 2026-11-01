// SPDX-License-Identifier: GPL-3.0-only
//! What the layout check and the reference decoder cost (ADR-0033), for docs/RESULT.md:
//! reading the header of every JPEG in a folder, and decoding one photo saved as usual (zune)
//! and the same photo with one scan per component (the reference decoder). Photos are only
//! read.
//!
//! `cargo run --release -p fotollection-core --example measure_jpeg_layout -- <folder> <photo.jpg> <same-photo-one-scan-per-component.jpg>`
//!
//! Make the last file with `jpegtran -scans crates/core/tests/fixtures/jpeg/noninterleaved.scans photo.jpg`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::jpeg_layout;

const RUNS: u32 = 5;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [folder, usual, split] = args.as_slice() else {
        eprintln!("usage: measure_jpeg_layout <folder> <photo.jpg> <one-scan-per-component.jpg>");
        std::process::exit(2);
    };

    let files: Vec<Vec<u8>> = std::fs::read_dir(folder)
        .expect("folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jpg")))
        .map(|p| std::fs::read(p).expect("read"))
        .collect();
    let start = Instant::now();
    let mut proven = 0;
    for _ in 0..RUNS {
        proven = files
            .iter()
            .filter(|b| jpeg_layout::read(b).is_none_or(|l| l.zune_proven()))
            .count();
    }
    let per_file = start.elapsed().as_secs_f64() * 1e6 / f64::from(RUNS) / files.len() as f64;
    println!(
        "header check: {} JPEGs, {proven} on zune, {per_file:.2} µs per file",
        files.len()
    );

    let codec = default_codec();
    for (label, path) in [
        ("zune (usual layout)", usual),
        ("reference (one scan per component)", split),
    ] {
        let bytes = std::fs::read(path).expect("read");
        let first = decode_image(codec.as_ref(), &bytes).expect("decode");
        let start = Instant::now();
        for _ in 0..RUNS {
            decode_image(codec.as_ref(), &bytes).expect("decode");
        }
        let ms = start.elapsed().as_secs_f64() * 1e3 / f64::from(RUNS);
        println!(
            "{label}: {}×{} in {ms:.0} ms",
            first.image.width, first.image.height
        );
    }
}
