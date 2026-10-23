// SPDX-License-Identifier: GPL-3.0-only
//! Measures the pipeline on a real folder, for OPTIMIZATION.md §5. Photos are only read.
//!
//! `cargo run --release -p fotollection-core --example measure_folder -- <photo-folder>`

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
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: measure_folder <folder>"),
    );
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
    println!(
        "import+exif: {} photos in {import_ms:.0} ms ({} rejected)",
        records.len(),
        found.rejected.len()
    );

    let t = Instant::now();
    let t0_count = records
        .par_iter()
        .filter(|r| thumbs::t0(r, codec.as_ref()).is_some())
        .count();
    println!(
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
    println!(
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
        .render_jpeg(first, &settings, codec.as_ref())
        .unwrap();
    let cold = t.elapsed().as_secs_f64() * 1e3;
    let mut warm = Vec::new();
    for _ in 0..20 {
        let t = Instant::now();
        previews
            .render_jpeg(first, &settings, codec.as_ref())
            .unwrap();
        warm.push(t.elapsed().as_secs_f64() * 1e3);
    }
    warm.sort_by(f64::total_cmp);
    println!(
        "preview: cold {cold:.0} ms, warm p50 {:.1} ms, p95 {:.1} ms",
        warm[10], warm[18]
    );

    let bytes = std::fs::read(&first.path).unwrap();
    let t = Instant::now();
    let decoded = codec.decode(&bytes).unwrap();
    println!(
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
    println!(
        "export: {} photos in {secs:.2} s ({:.2}/s, inflight {}), {} failed",
        summary.exported,
        f64::from(summary.exported) / secs,
        default_inflight(),
        summary.failed.len()
    );
}
