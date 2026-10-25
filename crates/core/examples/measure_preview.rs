// SPDX-License-Identifier: GPL-3.0-only
//! Measures what looking at a photo costs (FR-057, NFR-002), to size the zoom path: the
//! full decode and the RAM it holds, the preview from a cold and a warm cache, the full-
//! resolution render export does, and serving one screen-sized region of that render.
//! Median of 5 per step.
//!
//! `cargo run --release -p fotollection-core --example measure_preview -- <photo.jpg>…`
//!
//! Peak RSS comes from the caller: run it under `/usr/bin/time -l` with one photo.
//! `RAYON_NUM_THREADS=2` approximates a weak two-core machine.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::image::{RgbImage, apply_orientation};
use fotollection_core::import::check_file;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::plan::{PlanInput, plan};
use fotollection_core::preview::PreviewCache;
use fotollection_core::render::{fit_long_edge, render};
use fotollection_core::settings::RenderSettings;

const RUNS: usize = 5;

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

/// The `w`×`h` pixels at (`x`, `y`): what a zoomed screen shows of the full render.
fn region(image: &RgbImage, x: u32, y: u32, w: u32, h: u32) -> RgbImage {
    let mut out = Vec::with_capacity(w as usize * h as usize * 3);
    let row = image.width as usize * 3;
    for line in y..y + h {
        let start = line as usize * row + x as usize * 3;
        out.extend_from_slice(&image.data[start..start + w as usize * 3]);
    }
    RgbImage::from_raw(w, h, out).unwrap()
}

fn main() {
    let codec = default_codec();
    let settings = RenderSettings::default();
    println!(
        "| Photo | Decode | Decoded RAM | Preview cold | Preview warm | Full render | Region 1:1 | Region 2:1 |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for path in std::env::args().skip(1) {
        let file = check_file(path.as_ref()).expect("a JPEG");
        let meta = metadata::read_path(&file.path);
        let mut lib = Library::new();
        let folder = lib.add_folder(file.path.parent().expect("parent"));
        let id = lib.push(folder, &file, &meta).expect("photo");
        let record = lib.record(id).expect("record");
        let bytes = std::fs::read(&path).unwrap();

        let (decode, decoded) = time(|| decode_image(codec.as_ref(), &bytes).unwrap());
        let full = apply_orientation(decoded.image, record.display_orientation());
        let mp = f64::from(full.width) * f64::from(full.height) / 1e6;
        let ram = full.data.len() as f64 / 1e6;

        // A fresh cache each run is a photo seen for the first time.
        let (cold, _) = time(|| {
            PreviewCache::new()
                .render_jpeg(&record, &settings, None, None, codec.as_ref())
                .unwrap()
        });
        let cache = PreviewCache::new();
        cache
            .render_jpeg(&record, &settings, None, None, codec.as_ref())
            .unwrap();
        let (warm, _) = time(|| {
            cache
                .render_jpeg(&record, &settings, None, None, codec.as_ref())
                .unwrap()
        });

        let p = plan(&PlanInput {
            width: full.width,
            height: full.height,
            capture: record.stamp_date(),
            settings: &settings,
            print: settings
                .print_for(full.width, full.height)
                .unwrap_or(fotollection_core::PrintSize::ALL[1]),
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
        });
        let (full_render, rendered) = time(|| render(&full, &p).unwrap());

        // A 2560×1600 screen at 1:1 (100 % of the pixels), then at 2:1 (zoomed out by half).
        let (w, h) = (2560.min(rendered.width), 1600.min(rendered.height));
        let (x, y) = ((rendered.width - w) / 2, (rendered.height - h) / 2);
        let (one, _) = time(|| {
            codec
                .encode(
                    &region(&rendered, x, y, w, h),
                    EncodeOptions::screen(85),
                    None,
                )
                .unwrap()
        });
        let (w2, h2) = ((w * 2).min(rendered.width), (h * 2).min(rendered.height));
        let (x2, y2) = ((rendered.width - w2) / 2, (rendered.height - h2) / 2);
        let (two, _) = time(|| {
            let crop = region(&rendered, x2, y2, w2, h2);
            codec
                .encode(
                    &fit_long_edge(&crop, 2560).unwrap(),
                    EncodeOptions::screen(85),
                    None,
                )
                .unwrap()
        });

        println!(
            "| {} ({}×{}, {mp:.1} MP) | {decode:.0} ms | {ram:.0} MB | {cold:.0} ms | {warm:.0} ms | {full_render:.0} ms | {one:.0} ms | {two:.0} ms |",
            file.path.file_name().unwrap().to_string_lossy(),
            full.width,
            full.height,
        );
    }
}
