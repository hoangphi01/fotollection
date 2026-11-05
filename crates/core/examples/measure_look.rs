// SPDX-License-Identifier: GPL-3.0-only
//! Measures what a colour look costs (FR-079, FR-070, ADR-0038): building the 33³ table,
//! and applying it to a 1600 px preview and to a 24 MP export. Median of 5 per step.
//!
//! `cargo run --release -p fotollection-core --example measure_look [photo.jpg]`
//!
//! Without a photo, the images are made up (a gradient with noise); the cost per pixel
//! does not depend on the content. `RAYON_NUM_THREADS=2` approximates a weak two-core
//! machine. Budgets: ≤ 5 ms on the preview, ≤ 60 ms on 24 MP (OPTIMIZATION §3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::time::Instant;

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::image::RgbImage;
use fotollection_core::looks::{Adjust, LookSettings, Mono, Table, Tint, apply, bake};
use fotollection_core::render::fit_long_edge;

const RUNS: usize = 5;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn time(mut f: impl FnMut()) -> f64 {
    median(
        (0..RUNS)
            .map(|_| {
                let t = Instant::now();
                f();
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect(),
    )
}

/// A made-up photo: gradients and noise, so every pixel takes a different path.
fn synthetic(width: u32, height: u32) -> RgbImage {
    let mut seed = 0x9e37_79b9_u32;
    let mut data = Vec::with_capacity(width as usize * height as usize * 3);
    for y in 0..height {
        for x in 0..width {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let n = (seed & 0x1f) as u8;
            data.push(((x * 255 / width.max(1)) as u8).saturating_add(n));
            data.push(((y * 255 / height.max(1)) as u8).saturating_add(n));
            data.push((((x + y) * 127 / (width + height).max(1)) as u8).saturating_add(n));
        }
    }
    RgbImage::from_raw(width, height, data).unwrap()
}

/// A strong look: every channel changes, so no shortcut could hide the cost.
fn look() -> Table {
    Table::from_fn(|[r, g, b]| {
        [
            (0.8 * r + 0.2 * g).powf(0.9),
            (0.9 * g + 0.1 * b).powf(1.1),
            (0.05 + 0.9 * b).min(1.0),
        ]
    })
}

fn main() {
    let threads = rayon::current_num_threads();
    let build = time(|| {
        std::hint::black_box(look());
    });
    let table = look();
    // Every slider set: the table a drag builds on each step (FR-079).
    let sliders = LookSettings {
        adjust: Adjust {
            brightness: 20,
            contrast: 30,
            highlights: -20,
            shadows: 25,
            warmth: 15,
            tint: -5,
            saturation: 10,
            vibrance: 30,
            fade: 20,
            shadow_tint: Tint {
                hue: 200,
                amount: 40,
            },
            highlight_tint: Tint {
                hue: 60,
                amount: 30,
            },
            balance: 10,
            mono: Mono::Off,
            ..Adjust::default()
        },
        ..LookSettings::default()
    };
    let bake_sliders = time(|| {
        std::hint::black_box(bake(&sliders));
    });

    let (preview, export) = match std::env::args().nth(1) {
        Some(path) => {
            let bytes = std::fs::read(&path).expect("read the photo");
            let photo = decode_image(default_codec().as_ref(), &bytes)
                .expect("decode")
                .image;
            (fit_long_edge(&photo, 1600).expect("resize"), photo)
        }
        None => (synthetic(1600, 1067), synthetic(6000, 4000)),
    };

    let mut work = preview.clone();
    let on_preview = time(|| {
        work.data.copy_from_slice(&preview.data);
        apply(&mut work, &table);
    });
    // The copy back is part of each run above; measured alone, to subtract it.
    let copy_preview = time(|| work.data.copy_from_slice(&preview.data));

    let mut work = export.clone();
    let on_export = time(|| {
        work.data.copy_from_slice(&export.data);
        apply(&mut work, &table);
    });
    let copy_export = time(|| work.data.copy_from_slice(&export.data));

    let mp = |i: &RgbImage| f64::from(i.width) * f64::from(i.height) / 1e6;
    println!("threads {threads}, median of {RUNS}");
    println!("build the 33³ table: {build:.2} ms; with every slider set: {bake_sliders:.2} ms");
    println!(
        "apply, preview {}×{} ({:.1} MP): {:.2} ms (budget 5)",
        preview.width,
        preview.height,
        mp(&preview),
        on_preview - copy_preview
    );
    println!(
        "apply, export {}×{} ({:.1} MP): {:.1} ms (budget 60)",
        export.width,
        export.height,
        mp(&export),
        on_export - copy_export
    );
}
