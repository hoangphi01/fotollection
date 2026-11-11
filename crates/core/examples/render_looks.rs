// SPDX-License-Identifier: GPL-3.0-only
//! The website's interactive Looks section (site/_includes/home.html): one photo through each
//! of Fotollection's built-in looks, rendered by the app's own engine (`LookSettings::draw`,
//! the path every preview and export takes), so what the visitor clicks is what the app does.
//!
//! `cargo run --release -p fotollection-core --example render_looks -- <upright.jpg> <out-dir> [edge]`
//!
//! Writes `<out-dir>/<slug>.jpg` (quality 92) for each built-in look; `site/screenshots/looks.sh`
//! turns them into the site's WebP.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

use std::path::PathBuf;

use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::look_library::built_in;
use fotollection_core::render::fit_long_edge;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(args.next().expect("an upright JPEG"));
    let out = PathBuf::from(args.next().expect("an output folder"));
    let edge: u32 = args
        .next()
        .map_or(1600, |e| e.parse().expect("an edge in px"));
    std::fs::create_dir_all(&out).unwrap();
    let codec = default_codec();
    let photo = decode_image(codec.as_ref(), &std::fs::read(&input).unwrap())
        .unwrap()
        .image;
    let photo = fit_long_edge(&photo, edge).unwrap();
    for entry in built_in() {
        let slug = entry.id.trim_start_matches("builtin:").to_owned();
        let mut image = photo.clone();
        entry.look.draw(&mut image);
        let jpeg = codec
            .encode(&image, EncodeOptions::screen(92), None)
            .unwrap();
        std::fs::write(out.join(format!("{slug}.jpg")), jpeg).unwrap();
        println!("{slug} {}×{}", image.width, image.height);
    }
}
