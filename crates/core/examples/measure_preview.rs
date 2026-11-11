// SPDX-License-Identifier: GPL-3.0-only
//! Measures what looking at a photo costs (FR-057, NFR-002), to size the zoom path: the
//! full decode and the RAM it holds, the preview from a cold and a warm cache, the full-
//! resolution render export does, and serving one screen-sized region of that render.
//! Median of 5 per step.
//!
//! `cargo run --release -p fotollection-core --example measure_preview -- <photo.jpg>…`
//!
//! Peak RSS comes from the caller: run it under `/usr/bin/time -l` with one photo.
//! `RAYON_NUM_THREADS=2` approximates a weak two-core machine. `SHARE_FILL=1` measures a share
//! export to Instagram with a square fill round each photo (FR-075). `LOOK_DRAG=1` also times
//! one step of a colour slider's drag: the preview rendered with a new look each time (FR-079).

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

/// One step of a slider's drag (FR-079): a new look each time, so its table is built too,
/// as when the preview follows the pointer.
fn drag_step(
    cache: &PreviewCache,
    record: &fotollection_core::library::PhotoRecord,
    settings: &RenderSettings,
    codec: &dyn fotollection_core::codec::JpegCodec,
) {
    let mut step = settings.clone();
    let mut runs = Vec::new();
    for i in 0..20i8 {
        step.look.adjust.brightness = i;
        step.look.adjust.saturation = 10;
        let t = Instant::now();
        std::hint::black_box(cache.render_jpeg(record, &step, None, None, codec).unwrap());
        runs.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "drag step (new look each time, preview JPEG): median {:.1} ms",
        median(runs)
    );
    drag_parts(record, settings, codec);
}

/// One drag step split into its parts (ADR-0039 stage 1): building the look's table, the
/// look and stamp on the resized photo, and the screen JPEG encode. The resize is kept
/// between steps, so it is timed once, apart.
fn drag_parts(
    record: &fotollection_core::library::PhotoRecord,
    settings: &RenderSettings,
    codec: &dyn fotollection_core::codec::JpegCodec,
) {
    use fotollection_core::render::{crop_and_resize, finish};
    let bytes = std::fs::read(&record.path).unwrap();
    let full = apply_orientation(
        decode_image(codec, &bytes).unwrap().image,
        record.display_orientation(),
    );
    let source = fit_long_edge(&full, 2048).unwrap();
    let mut step = settings.clone();
    step.look.adjust.saturation = 10;
    let p = plan(&PlanInput {
        width: source.width,
        height: source.height,
        capture: record.stamp_date(),
        settings: &step,
        print: step
            .print_for(source.width, source.height)
            .unwrap_or(fotollection_core::PrintSize::ALL[1]),
        max_long_edge: Some(1600),
        frame: None,
        crop: None,
        place: None,
        straighten: 0.0,
        ratio: None,
        credit: None,
    });
    let (resize, resized) = time(|| crop_and_resize(&source, &p).unwrap());
    let mut i = 0i8;
    let (bake, _) = time(|| {
        i += 1;
        step.look.adjust.brightness = i;
        step.look.table()
    });
    let (look_stamp, image) = time(|| finish(resized.clone(), &p));
    let (encode, jpeg) = time(|| {
        codec
            .encode(&image, EncodeOptions::screen(85), None)
            .unwrap()
    });
    println!(
        "drag parts: resize {resize:.1} ms (kept between steps) · table {bake:.1} · look + stamp + copy {look_stamp:.1} · screen JPEG encode {encode:.1} ms ({} KB, {}×{})",
        jpeg.len() / 1024,
        image.width,
        image.height
    );
}

fn main() {
    let codec = default_codec();
    let settings = if std::env::var_os("SHARE_FILL").is_some() {
        RenderSettings {
            export_target: fotollection_core::share::ExportTarget::Share(
                fotollection_core::share::SharePreset::Instagram,
            ),
            share_fill: fotollection_core::settings::ShareFill {
                enabled: true,
                ..Default::default()
            },
            ..RenderSettings::default()
        }
    } else {
        RenderSettings::default()
    };
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

        if std::env::var_os("LOOK_DRAG").is_some() {
            drag_step(&cache, &record, &settings, codec.as_ref());
        }

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
            ratio: None,
            credit: None,
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
