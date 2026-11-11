// SPDX-License-Identifier: GPL-3.0-only
//! Measures a collage sheet against its two photos alone (FR-035, NFR-021): the warm
//! preview, and the export's render and encode at full resolution. Median of 5 per step.
//!
//! `cargo run --release -p fotollection-core --example measure_sheet -- <a.jpg> <b.jpg>`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::image::{RgbImage, apply_orientation};
use fotollection_core::import::check_file;
use fotollection_core::library::{Library, PhotoRecord};
use fotollection_core::metadata;
use fotollection_core::plan::{PlanInput, plan};
use fotollection_core::preview::{PREVIEW_LONG_EDGE, PreviewCache};
use fotollection_core::render::render;
use fotollection_core::settings::RenderSettings;
use fotollection_core::sheet::{CellSource, render_sheet, sheet_print};

const RUNS: usize = 5;

fn time<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut runs = Vec::new();
    let mut last = None;
    for _ in 0..RUNS {
        let t = Instant::now();
        last = Some(f());
        runs.push(t.elapsed().as_secs_f64() * 1e3);
    }
    runs.sort_by(f64::total_cmp);
    (runs[RUNS / 2], last.unwrap())
}

fn load(lib: &mut Library, path: &str) -> (PhotoRecord, RgbImage) {
    let file = check_file(path.as_ref()).expect("a photo");
    let meta = metadata::read_path(&file.path);
    let folder = lib.add_folder(file.path.parent().expect("parent"));
    let id = lib.push(folder, &file, &meta).expect("photo");
    let record = lib.record(id).expect("record");
    let decoded = decode_image(default_codec().as_ref(), &std::fs::read(path).unwrap()).unwrap();
    let upright = apply_orientation(decoded.image, record.display_orientation());
    (record, upright)
}

fn source(image: &RgbImage) -> CellSource<'_> {
    CellSource {
        image,
        capture: None,
        crop: None,
        straighten: 0.0,
        place: None,
        stamp: None,
        caption: None,
        look: None,
        postcard: None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let codec = default_codec();
    let settings = RenderSettings::default();
    let mut lib = Library::new();
    let (a, image_a) = load(&mut lib, &args[0]);
    let (b, image_b) = load(&mut lib, &args[1]);

    let cache = PreviewCache::new();
    let single = |r: &PhotoRecord| {
        cache
            .render_jpeg(r, &settings, None, None, codec.as_ref())
            .unwrap()
    };
    single(&a);
    single(&b);
    let (preview_a, _) = time(|| single(&a));
    let (preview_b, _) = time(|| single(&b));
    let sheet = || {
        cache
            .render_sheet_jpeg(
                [Some((&a, None)), Some((&b, None))],
                &settings,
                PREVIEW_LONG_EDGE,
                None,
                None,
                fotollection_core::preview::SheetView::Normal,
                codec.as_ref(),
            )
            .unwrap()
    };
    let (preview_sheet, _) = time(sheet);

    let export_one = |image: &RgbImage| {
        let p = plan(&PlanInput {
            width: image.width,
            height: image.height,
            capture: None,
            settings: &settings,
            print: settings.print_for(image.width, image.height).unwrap(),
            max_long_edge: None,
            frame: None,
            crop: None,
            place: None,
            straighten: 0.0,
            ratio: None,
            credit: None,
        });
        let out = render(image, &p).unwrap();
        codec.encode(&out, EncodeOptions::print(95), None).unwrap()
    };
    let (export_a, _) = time(|| export_one(&image_a));
    let (export_b, _) = time(|| export_one(&image_b));
    let (export_sheet, jpeg) = time(|| {
        let out = render_sheet(
            &settings,
            sheet_print(&settings),
            [Some(source(&image_a)), Some(source(&image_b))],
            false,
            None,
            None,
        )
        .unwrap();
        (
            out.width,
            out.height,
            codec
                .encode(&out, EncodeOptions::print(95), None)
                .unwrap()
                .len(),
        )
    });

    println!("| Step | Photo A | Photo B | A + B | Sheet | Budget | Verdict |");
    println!("|---|---|---|---|---|---|---|");
    let row = |step: &str, x: f64, y: f64, s: f64, budget: f64| {
        let verdict = if s <= budget { "met" } else { "over" };
        println!(
            "| {step} | {x:.0} ms | {y:.0} ms | {:.0} ms | {s:.0} ms | ≤ {budget:.0} ms | {verdict} |",
            x + y
        );
    };
    row(
        "Preview, warm",
        preview_a,
        preview_b,
        preview_sheet,
        2.0 * preview_a.max(preview_b),
    );
    row(
        "Export render + encode",
        export_a,
        export_b,
        export_sheet,
        export_a + export_b,
    );
    println!(
        "\nSheet {}×{} px, {:.1} MB",
        jpeg.0,
        jpeg.1,
        jpeg.2 as f64 / 1e6
    );
}
