// SPDX-License-Identifier: GPL-3.0-only
//! What the develop view's histogram costs (FR-098, OPTIMIZATION §3): each JPEG of a folder
//! is decoded and fitted to the preview's 1600 px, then `histogram::measure` runs through a
//! look of eight sliders, counting every pixel (step 1), every second pixel of every second
//! row (step 2) or every third (step 3), 20 times each. Prints the p50 and p95 time per step,
//! and how far the sampled curves stray from the full count (the largest difference of a
//! level's share, in per mille). Photos are only read; only aggregate numbers are printed.
//!
//! `cargo run --release -p fotollection-core --example measure_histogram -- <folder> [photos]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::doc_markdown
)]

use std::time::Instant;

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::histogram::{Histogram, measure};
use fotollection_core::looks::LookSettings;
use fotollection_core::render::fit_long_edge;

const PREVIEW_LONG_EDGE: u32 = 1600;
const RUNS: usize = 20;

fn share_gap(full: &Histogram, part: &Histogram) -> f64 {
    let mut worst = 0.0f64;
    for (a, b) in [
        (&full.r, &part.r),
        (&full.g, &part.g),
        (&full.b, &part.b),
        (&full.l, &part.l),
    ] {
        for (x, y) in a.iter().zip(b) {
            let gap = (f64::from(*x) / f64::from(full.samples)
                - f64::from(*y) / f64::from(part.samples))
            .abs();
            worst = worst.max(gap * 1000.0);
        }
    }
    worst
}

fn percentile(times: &mut [f64], p: f64) -> f64 {
    times.sort_by(f64::total_cmp);
    let i = ((times.len() - 1) as f64 * p).round() as usize;
    times[i]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let folder = args
        .next()
        .expect("usage: measure_histogram <folder> [photos]");
    let limit: usize = args
        .next()
        .map_or(usize::MAX, |n| n.parse().expect("a number of photos"));
    let codec = default_codec();
    let mut paths: Vec<_> = std::fs::read_dir(&folder)
        .expect("folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        })
        .collect();
    paths.sort();
    paths.truncate(limit);
    let previews: Vec<_> = paths
        .iter()
        .map(|p| {
            let bytes = std::fs::read(p).expect("read");
            fit_long_edge(
                &decode_image(codec.as_ref(), &bytes).expect("decode").image,
                PREVIEW_LONG_EDGE,
            )
            .expect("resize")
        })
        .collect();
    let mut look = LookSettings::default();
    let a = &mut look.adjust;
    (
        a.brightness,
        a.contrast,
        a.highlights,
        a.shadows,
        a.warmth,
        a.tint,
        a.saturation,
        a.vibrance,
    ) = (9, 44, -36, -48, 22, 10, 19, 3);
    let table = look.table().expect("a look that changes something");
    println!(
        "{} photos at {PREVIEW_LONG_EDGE} px, {RUNS} runs each, a look of eight sliders",
        previews.len()
    );
    for step in [1u32, 2, 3] {
        let mut times = Vec::new();
        let mut gap = 0.0f64;
        for preview in &previews {
            let full = measure(preview, Some(&table), 1);
            for _ in 0..RUNS {
                let start = Instant::now();
                let h = measure(preview, Some(&table), step);
                times.push(start.elapsed().as_secs_f64() * 1000.0);
                gap = gap.max(share_gap(&full, &h));
            }
        }
        let p50 = percentile(&mut times, 0.5);
        let p95 = percentile(&mut times, 0.95);
        println!(
            "step {step}: p50 {p50:.2} ms, p95 {p95:.2} ms, largest gap of a level's share {gap:.2} ‰"
        );
    }
}
