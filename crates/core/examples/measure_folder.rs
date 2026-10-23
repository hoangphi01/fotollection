// SPDX-License-Identifier: GPL-3.0-only
//! Measures the pipeline on a real folder, for OPTIMIZATION.md §5. Photos are only read.
//!
//! `cargo run --release -p fotollection-core --example measure_folder -- <photo-folder> [--json]`
//!
//! With `--json`, the human-readable lines go to stderr and one JSON object to stdout
//! (read by `bench/chart.mjs`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::path::PathBuf;
use std::time::Instant;

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::default_codec;
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, default_inflight, run_export};
use fotollection_core::import::scan;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::{self, ThumbCache};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let root = PathBuf::from(
        args.iter()
            .find(|a| !a.starts_with("--"))
            .expect("usage: measure_folder <folder> [--json]"),
    );
    macro_rules! report {
        ($($t:tt)*) => {
            if json { eprintln!($($t)*) } else { println!($($t)*) }
        };
    }
    let codec = default_codec();

    let t = Instant::now();
    let found = scan(&root);
    let metas: Vec<_> = found
        .accepted
        .par_iter()
        .map(|f| metadata::read_path(&f.path))
        .collect();
    let mut lib = Library::new();
    let folder = lib.add_folder(&root);
    for (file, meta) in found.accepted.iter().zip(&metas) {
        lib.push(folder, file, meta);
    }
    let import_ms = t.elapsed().as_secs_f64() * 1e3;
    let records: Vec<_> = lib
        .sorted_ids()
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect();
    report!(
        "import+exif: {} photos in {import_ms:.0} ms ({} rejected)",
        records.len(),
        found.rejected.len()
    );

    let t = Instant::now();
    let t0_count = records
        .par_iter()
        .filter(|r| thumbs::t0(r, codec.as_ref()).is_some())
        .count();
    report!(
        "T0 (embedded) thumbnails: {t0_count} in {:.0} ms",
        t.elapsed().as_secs_f64() * 1e3
    );

    let cache_dir = tempfile::tempdir().unwrap();
    let cache = ThumbCache::new(cache_dir.path().to_path_buf());
    let t = Instant::now();
    records.par_iter().for_each(|r| {
        cache.make_t1(r, codec.as_ref()).unwrap();
    });
    let t1 = t.elapsed().as_secs_f64();
    report!(
        "T1 thumbnails: {} in {:.2} s ({:.1}/s)",
        records.len(),
        t1,
        records.len() as f64 / t1
    );

    let previews = PreviewCache::new();
    let settings = RenderSettings::default();
    let first = &records[0];
    let t = Instant::now();
    previews
        .render_jpeg(first, &settings, None, None, codec.as_ref())
        .unwrap();
    let cold = t.elapsed().as_secs_f64() * 1e3;
    let mut warm = Vec::new();
    for _ in 0..20 {
        let t = Instant::now();
        previews
            .render_jpeg(first, &settings, None, None, codec.as_ref())
            .unwrap();
        warm.push(t.elapsed().as_secs_f64() * 1e3);
    }
    warm.sort_by(f64::total_cmp);
    report!(
        "preview: cold {cold:.0} ms, warm p50 {:.1} ms, p95 {:.1} ms",
        warm[10],
        warm[18]
    );

    let bytes = std::fs::read(&first.path).unwrap();
    let t = Instant::now();
    let decoded = codec.decode(&bytes).unwrap();
    report!(
        "single full decode {}x{}: {:.0} ms",
        decoded.image.width,
        decoded.image.height,
        t.elapsed().as_secs_f64() * 1e3
    );

    let dest = tempfile::tempdir().unwrap();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: EXPORT_QUALITY,
        max_inflight: default_inflight(),
        stamp_font: None,
        frame_for: &|_| None,
    };
    let t = Instant::now();
    let summary = run_export(
        &records,
        &options,
        codec.as_ref(),
        &CancellationToken::new(),
        &|_, _| {},
    );
    let secs = t.elapsed().as_secs_f64();
    report!(
        "export: {} photos in {secs:.2} s ({:.2}/s, inflight {}), {} failed",
        summary.exported,
        f64::from(summary.exported) / secs,
        default_inflight(),
        summary.failed.len()
    );

    if json {
        let input_bytes: u64 = records.iter().map(|r| r.size).sum();
        let out = serde_json::json!({
            "engine": "Fotollection core (Rust, release)",
            "threads": default_inflight(),
            "photos": summary.exported,
            "exportSeconds": secs,
            "exportPerSecond": f64::from(summary.exported) / secs,
            "previewColdMs": cold,
            "previewWarmP50Ms": warm[10],
            "previewWarmP95Ms": warm[18],
            "importMs": import_ms,
            "inputBytes": input_bytes,
        });
        println!("{out:#}");
    }
}
