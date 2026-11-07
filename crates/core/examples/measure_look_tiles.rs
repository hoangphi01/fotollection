// SPDX-License-Identifier: GPL-3.0-only
//! What the Looks card's tiles cost (FR-079, OPTIMIZATION §3): each JPEG of a folder is
//! decoded and fitted to the thumbnail's long edge (the small source the tiles are cut
//! from, kept per photo), then every built-in look's tile is made (`preview::look_tile`) and
//! encoded as the app serves it, 20 times each. Prints p50 and p95 per tile, the first tile
//! of a look (its table built) apart, and the whole card's eight tiles.
//!
//! `cargo run --release -p fotollection-core --example measure_look_tiles -- <folder> [photos]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::time::Instant;

use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::look_library::built_in;
use fotollection_core::preview::look_tile;
use fotollection_core::render::fit_long_edge;

const RUNS: usize = 20;

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let folder = args.next().expect("a folder of JPEGs");
    let limit: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(24);
    let codec = default_codec();
    let looks = built_in();
    let mut files: Vec<_> = std::fs::read_dir(&folder)
        .expect("folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        })
        .collect();
    files.sort();
    files.truncate(limit);
    let (mut tile, mut first, mut card) = (Vec::new(), Vec::new(), Vec::new());
    for file in &files {
        let bytes = std::fs::read(file).expect("read");
        let decoded = decode_image(codec.as_ref(), &bytes).expect("decode").image;
        let small = fit_long_edge(&decoded, fotollection_core::thumbs::T1_LONG_EDGE).expect("fit");
        let one = |look| {
            let t = Instant::now();
            let image = look_tile(&small, look).expect("tile");
            let _ = codec
                .encode(&image, EncodeOptions::screen(85), None)
                .expect("encode");
            t.elapsed().as_secs_f64() * 1e3
        };
        for look in &looks {
            first.push(one(&look.look));
        }
        for _ in 0..RUNS {
            let start = Instant::now();
            for look in &looks {
                tile.push(one(&look.look));
            }
            card.push(start.elapsed().as_secs_f64() * 1e3);
        }
    }
    println!(
        "{} photos, {} looks, {} runs",
        files.len(),
        looks.len(),
        RUNS
    );
    println!(
        "tile        p50 {:.2} ms  p95 {:.2} ms",
        pct(&mut tile, 0.5),
        pct(&mut tile, 0.95)
    );
    println!(
        "first tile  p50 {:.2} ms  p95 {:.2} ms",
        pct(&mut first, 0.5),
        pct(&mut first, 0.95)
    );
    println!(
        "card of {}   p50 {:.2} ms  p95 {:.2} ms",
        looks.len(),
        pct(&mut card, 0.5),
        pct(&mut card, 0.95)
    );
}
