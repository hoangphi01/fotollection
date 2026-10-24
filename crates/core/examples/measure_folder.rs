// SPDX-License-Identifier: GPL-3.0-only
//! Measures the pipeline on a real folder, for OPTIMIZATION.md §5. Photos are only read.
//!
//! `cargo run --release -p fotollection-core --example measure_folder -- <photo-folder> [flags]`
//!
//! - `--json`: the human-readable lines go to stderr and one JSON object to stdout
//!   (read by `bench/chart.mjs`).
//! - `--inflight N`: photos exported in parallel (default: `default_inflight()`).
//! - `--discard-output`: delete exported files as soon as they are written, so batches of
//!   thousands of photos can be measured without the disk space (NFR-004).
//! - `--analyze`: measure best-shot analysis instead (FR-041, NFR-014) and print the groups.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::culling::{self, group, score};
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, default_inflight, run_export};
use fotollection_core::image::apply_orientation;
use fotollection_core::import::scan;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::plan::{PlanInput, plan};
use fotollection_core::preview::PreviewCache;
use fotollection_core::preview::{PREVIEW_LONG_EDGE, T2_LONG_EDGE};
use fotollection_core::render::{fit_long_edge, render};
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::{self, ThumbCache};
use rayon::prelude::*;

/// Resident memory of this process right now, in MB (via `ps`, so it works on macOS and Linux).
fn rss_mb() -> f64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(0.0, |kb| kb / 1024.0)
}

/// Removes finished exports (not `.part` files still being written) under `dir`.
fn discard_finished(dir: &Path) {
    for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if entry.path().extension().is_some_and(|e| e == "jpg") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let discard = args.iter().any(|a| a == "--discard-output");
    let inflight = args
        .iter()
        .position(|a| a == "--inflight")
        .and_then(|i| args.get(i + 1)?.parse().ok())
        .unwrap_or_else(default_inflight);
    let root = PathBuf::from(
        args.iter()
            .enumerate()
            .find(|(i, a)| !a.starts_with("--") && (*i == 0 || args[i - 1] != "--inflight"))
            .map(|(_, a)| a)
            .expect("usage: measure_folder <folder> [--json] [--inflight N] [--discard-output]"),
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
        "import+exif: {} photos in {import_ms:.0} ms ({} rejected), RSS {:.0} MB",
        records.len(),
        found.rejected.len(),
        rss_mb()
    );
    let rss_import = rss_mb();
    if args.iter().any(|a| a == "--analyze") {
        analyze(&records, codec.as_ref(), json);
        return;
    }

    let t = Instant::now();
    let t0_count = records
        .par_iter()
        .filter(|r| thumbs::t0(r, codec.as_ref()).is_some())
        .count();
    let t0_ms = t.elapsed().as_secs_f64() * 1e3;
    report!("T0 (embedded) thumbnails: {t0_count} in {t0_ms:.0} ms");

    let cache_dir = tempfile::tempdir().unwrap();
    let cache = ThumbCache::new(cache_dir.path().to_path_buf());
    let t = Instant::now();
    records.par_iter().for_each(|r| {
        cache.make_t1(r, codec.as_ref()).unwrap();
    });
    let t1 = t.elapsed().as_secs_f64();
    report!(
        "T1 thumbnails: {} in {:.2} s ({:.1}/s), RSS {:.0} MB",
        records.len(),
        t1,
        records.len() as f64 / t1,
        rss_mb()
    );
    let rss_t1 = rss_mb();

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

    // Where the warm preview time goes (OPTIMIZATION.md §2.1): crop + resize + stamp, then
    // JPEG encode at the preview quality (85), on the cached upright T2 image.
    let bytes = std::fs::read(&first.path).unwrap();
    let t2 = fit_long_edge(
        &apply_orientation(
            decode_image(codec.as_ref(), &bytes).unwrap().image,
            first.orientation,
        ),
        T2_LONG_EDGE,
    )
    .unwrap();
    let no_stamp = {
        let mut s = settings.clone();
        s.stamp.enabled = false;
        s
    };
    let stage = |settings: &RenderSettings| {
        let (mut draw, mut encode, mut encode_print) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..20 {
            let t = Instant::now();
            let p = plan(&PlanInput {
                width: t2.width,
                height: t2.height,
                capture: Some(first.capture),
                settings,
                print: settings.validate().unwrap(),
                max_long_edge: Some(PREVIEW_LONG_EDGE),
                frame: None,
                crop: None,
                stamp_font: None,
            });
            let image = render(&t2, &p).unwrap();
            draw.push(t.elapsed().as_secs_f64() * 1e3);
            // The preview's own encoder settings (screen), and print settings for comparison.
            let t = Instant::now();
            codec
                .encode(&image, EncodeOptions::screen(85), None)
                .unwrap();
            encode.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            codec
                .encode(&image, EncodeOptions::print(85), None)
                .unwrap();
            encode_print.push(t.elapsed().as_secs_f64() * 1e3);
        }
        for v in [&mut draw, &mut encode, &mut encode_print] {
            v.sort_by(f64::total_cmp);
        }
        (draw[10], encode[10], encode_print[10])
    };
    let (draw_ms, encode_ms, encode_print_ms) = stage(&settings);
    let (draw_plain_ms, _, _) = stage(&no_stamp);
    report!(
        "preview breakdown (p50): crop+resize {draw_plain_ms:.1} ms, stamp {:.1} ms, encode q85 {encode_ms:.1} ms (4:4:4 + optimized Huffman: {encode_print_ms:.1} ms)",
        draw_ms - draw_plain_ms
    );

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
        max_inflight: inflight,
        stamp_font: None,
        frame_for: &|_| None,
    };
    let t = Instant::now();
    let summary = run_export(
        &records,
        &options,
        codec.as_ref(),
        &CancellationToken::new(),
        &|done, _| {
            if discard && done % 16 == 0 {
                discard_finished(dest.path());
            }
        },
    );
    let secs = t.elapsed().as_secs_f64();
    report!(
        "export: {} photos in {secs:.2} s ({:.2}/s, inflight {inflight}), {} failed, RSS {:.0} MB",
        summary.exported,
        f64::from(summary.exported) / secs,
        summary.failed.len(),
        rss_mb()
    );
    let rss_export = rss_mb();

    if json {
        let input_bytes: u64 = records.iter().map(|r| r.size).sum();
        let out = serde_json::json!({
            "engine": "Fotollection core (Rust, release)",
            "threads": inflight,
            "photos": summary.exported,
            "exportSeconds": secs,
            "exportPerSecond": f64::from(summary.exported) / secs,
            "previewColdMs": cold,
            "previewWarmP50Ms": warm[10],
            "previewWarmP95Ms": warm[18],
            "importMs": import_ms,
            "t0Ms": t0_ms,
            "t1PerSecond": records.len() as f64 / t1,
            "rssAfterImportMb": rss_import,
            "rssAfterT1Mb": rss_t1,
            "rssAfterExportMb": rss_export,
            "previewDrawMs": draw_plain_ms,
            "previewStampMs": draw_ms - draw_plain_ms,
            "previewEncodeMs": encode_ms,
            "previewEncodePrintMs": encode_print_ms,
            "inputBytes": input_bytes,
        });
        println!("{out:#}");
    }
}

/// Best-shot analysis on every photo (FR-041): time per photo on the analysis itself (the
/// decode is the T2 preview's job in the app), throughput on the whole pool, then the groups.
fn analyze(
    records: &[fotollection_core::library::PhotoRecord],
    codec: &dyn fotollection_core::codec::JpegCodec,
    json: bool,
) {
    let upright = |r: &fotollection_core::library::PhotoRecord| {
        let bytes = std::fs::read(&r.path).unwrap();
        let image = apply_orientation(decode_image(codec, &bytes).unwrap().image, r.orientation);
        fit_long_edge(&image, T2_LONG_EDGE).unwrap()
    };
    let t = Instant::now();
    culling::models::yunet(512, 384).unwrap();
    culling::models::facemesh().unwrap();
    let load_ms = t.elapsed().as_secs_f64() * 1e3;

    // One thread, so the per-photo time is not inflated by sharing cores.
    let mut timed = Vec::new();
    for r in records {
        let t2 = upright(r);
        let t = Instant::now();
        let analysis = culling::analyze(&t2).unwrap();
        timed.push((t.elapsed().as_secs_f64() * 1e3, analysis));
    }
    let pct = |mut v: Vec<f64>, p: f64| {
        if v.is_empty() {
            return 0.0;
        }
        v.sort_by(f64::total_cmp);
        v[((v.len() - 1) as f64 * p).round() as usize]
    };
    let without: Vec<f64> = timed
        .iter()
        .filter(|(_, a)| a.faces.is_empty())
        .map(|(ms, _)| *ms)
        .collect();
    let with: Vec<f64> = timed
        .iter()
        .filter(|(_, a)| !a.faces.is_empty())
        .map(|(ms, _)| *ms)
        .collect();
    let (n_without, n_with) = (without.len(), with.len());
    let (p50_without, p95_without) = (pct(without.clone(), 0.5), pct(without, 0.95));
    let (p50_with, p95_with) = (pct(with.clone(), 0.5), pct(with, 0.95));

    let t2s: Vec<_> = records.par_iter().map(upright).collect();
    let t = Instant::now();
    t2s.par_iter().for_each(|i| {
        culling::analyze(i).unwrap();
    });
    let pool_secs = t.elapsed().as_secs_f64();

    let shots = records
        .iter()
        .zip(&timed)
        .map(|(r, (_, a))| group::Shot::from_record(r, a.hash))
        .collect();
    let groups = group::group(shots);
    let report = |line: String| {
        if json {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    };
    report(format!(
        "analysis: models load {load_ms:.0} ms; no faces {n_without} photos p50 {p50_without:.1} ms p95 {p95_without:.1} ms; with faces {n_with} photos p50 {p50_with:.1} ms p95 {p95_with:.1} ms (one thread); pool {:.1} photos/s",
        records.len() as f64 / pool_secs
    ));
    report(format!("groups: {}", groups.len()));
    for g in &groups {
        let members: Vec<_> = g
            .iter()
            .map(|id| {
                (
                    *id,
                    &timed[records.iter().position(|r| r.id == *id).unwrap()].1,
                )
            })
            .collect();
        let result = score::rank(&members);
        let name = |id| &records.iter().find(|r| r.id == id).unwrap().file_name;
        let photos: Vec<String> = result
            .photos
            .iter()
            .map(|p| {
                format!(
                    "{} {:+.2} eyes {}/{}",
                    name(p.id),
                    p.score,
                    p.faces - p.closed_eyes,
                    p.faces
                )
            })
            .collect();
        report(format!(
            "  best of {}: {}{} {:?}",
            g.len(),
            photos.join(", "),
            if result.too_close { " (too close)" } else { "" },
            result.reasons
        ));
    }
    if json {
        let out = serde_json::json!({
            "photos": records.len(),
            "modelLoadMs": load_ms,
            "noFacesPhotos": n_without,
            "noFacesP50Ms": p50_without,
            "noFacesP95Ms": p95_without,
            "facesPhotos": n_with,
            "facesP50Ms": p50_with,
            "facesP95Ms": p95_with,
            "poolPerSecond": records.len() as f64 / pool_secs,
            "groups": groups.len(),
        });
        println!("{out:#}");
    }
}
