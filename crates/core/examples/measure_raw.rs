// SPDX-License-Identifier: GPL-3.0-only
//! Measures what a camera RAW costs (FR-016, NFR-017) through the operating system's codecs:
//! the size from the header, the full develop, the embedded preview at the sizes the app
//! asks for, and whether today's EXIF reader understands the container. Median of 5.
//!
//! `python3 -I scripts/fetch-raw-samples.py <dir>` then
//! `cargo run --release -p fotollection-core --example measure_raw -- <dir>/*`
//!
//! Peak RSS comes from the caller: run it under `/usr/bin/time -l` with one file.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_oscodec::{Format, Limits, RawKind, decode, dimensions, preview};

const RUNS: usize = 5;
const LIMITS: Limits = Limits {
    max_edge: 30_000,
    max_pixels: 200_000_000,
};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn time<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut runs = Vec::new();
    let mut last = None;
    for _ in 0..RUNS {
        let t = Instant::now();
        last = Some(f());
        runs.push(t.elapsed().as_secs_f64() * 1e3);
    }
    (median(runs), last.unwrap())
}

fn main() {
    println!(
        "| File | Developed | Header size | Full develop | Preview ≤ 400 px | Preview ≤ 2048 px | EXIF (kamadak) |"
    );
    println!("|---|---|---|---|---|---|---|");
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).unwrap();
        let name = std::path::Path::new(&path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let ext = name
            .rsplit('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let format = Format::Raw(RawKind::from_extension(&ext).expect("a RAW extension"));
        let (t_dim, dim) = time(|| dimensions(&bytes, format));
        let (t_full, full) = time(|| decode(&bytes, format, LIMITS));
        let (t_small, small) = time(|| preview(&bytes, format, 400, LIMITS));
        let (t_big, big) = time(|| preview(&bytes, format, 2048, LIMITS));
        let exif = exif::Reader::new()
            .read_from_container(&mut std::io::Cursor::new(&bytes))
            .map_or("✗", |e| {
                e.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                    .map_or("read, no date", |_| "date ✓")
            });
        let size = |r: &Result<fotollection_oscodec::OsImage, _>| {
            r.as_ref()
                .map_or_else(|e| format!("{e}"), |i| format!("{}×{}", i.width, i.height))
        };
        println!(
            "| {name} | {} | {t_dim:.0} ms | {t_full:.0} ms{} | {t_small:.0} ms ({}) | {t_big:.0} ms ({}) | {exif} |",
            dim.map_or_else(|e| format!("{e}"), |(w, h)| format!("{w}×{h}")),
            if full.is_ok() { "" } else { " (failed)" },
            size(&small),
            size(&big),
        );
        let _ = full;
    }
}
