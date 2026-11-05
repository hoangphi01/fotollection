// SPDX-License-Identifier: GPL-3.0-only
//! Writes what the GPU look is checked against (ADR-0039): a photo's preview without its look
//! (`plain.png`), where it shows the photo (`mask.png`), the look's 33³ table as the webview
//! reads it (`table.bin`), and Rust's own preview with the look (`cpu.png`), all lossless.
//! `bench/gpu/parity.mjs` draws `plain.png` through `table.bin` in `WebKit` and Chromium, and
//! `bench/quality/look_parity.py` scores the result against `cpu.png` (ΔE2000).
//!
//! `cargo run --release -p fotollection-core --example look_parity -- <photo.jpg> <out-dir>`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use fotollection_core::codec::default_codec;
use fotollection_core::import::check_file;
use fotollection_core::library::Library;
use fotollection_core::looks::{Adjust, LookSettings, Mono, Tint};
use fotollection_core::metadata;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;

fn png(path: &std::path::Path, image: &fotollection_core::image::RgbImage) {
    let file = std::fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), image.width, image.height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&image.data)
        .unwrap();
}

fn main() {
    let mut args = std::env::args().skip(1);
    let photo = args.next().expect("a photo");
    let out = std::path::PathBuf::from(args.next().expect("an out dir"));
    std::fs::create_dir_all(&out).unwrap();
    let codec = default_codec();
    let file = check_file(photo.as_ref()).expect("a photo");
    let meta = metadata::read_path(&file.path);
    let mut lib = Library::new();
    let folder = lib.add_folder(file.path.parent().expect("parent"));
    let id = lib.push(folder, &file, &meta).expect("photo");
    let record = lib.record(id).expect("record");
    // A strong look on every kind of slider, with the border and the stamp on.
    let mut settings = RenderSettings {
        border: true,
        ..RenderSettings::default()
    };
    settings.look = LookSettings {
        adjust: Adjust {
            brightness: 15,
            contrast: 35,
            highlights: -25,
            shadows: 30,
            warmth: 25,
            tint: -10,
            saturation: 20,
            vibrance: 30,
            fade: 15,
            shadow_tint: Tint {
                hue: 200,
                amount: 40,
            },
            highlight_tint: Tint {
                hue: 60,
                amount: 30,
            },
            mono: Mono::Off,
            ..Adjust::default()
        },
        ..LookSettings::default()
    };
    let cache = PreviewCache::new();
    let mut plain = settings.clone();
    plain.look = LookSettings::default();
    // Both sides before the JPEG encode, so the score is the look's alone.
    let cpu = cache
        .render_rgb(&record, &settings, None, None, codec.as_ref())
        .unwrap();
    let base = cache
        .render_rgb(&record, &plain, None, None, codec.as_ref())
        .unwrap();
    png(&out.join("cpu.png"), &cpu);
    png(&out.join("plain.png"), &base);
    std::fs::write(
        out.join("mask.png"),
        cache
            .render_mask_png(&record, &settings, None, None, codec.as_ref())
            .unwrap(),
    )
    .unwrap();
    std::fs::write(
        out.join("table.bin"),
        PreviewCache::look_table(&record, &settings).unwrap(),
    )
    .unwrap();
    println!("wrote {}: {}×{}", out.display(), cpu.width, cpu.height);
}
