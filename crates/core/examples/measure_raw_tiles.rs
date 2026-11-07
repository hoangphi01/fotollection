// SPDX-License-Identifier: GPL-3.0-only
//! How far a camera RAW's look tile is from its preview (FR-079, RESULT §34). A tile is cut
//! from the photo's thumbnail, which for a RAW is the camera's embedded JPEG (ADR-0023); its
//! preview is our neutral develop (FR-072). For each file: both made the tile's way (the
//! centre square, 192 px, `preview::look_tile`), through *Original* and through *Warm old
//! film*, and the mean ΔE2000 between them; and what a tile from the develop costs when the
//! develop is not kept yet. Files are only read; only numbers are printed.
//!
//! `cargo run --release -p fotollection-core --example measure_raw_tiles -- <raw files>`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use fotollection_core::codec::{decode_for_screen, default_codec};
use fotollection_core::look_library::built_in;
use fotollection_core::preview::look_tile;
use fotollection_core::raw::{self, RawAdjust, RawLook};
use fotollection_core::raw_finish::mean_de2000;
use fotollection_core::render::fit_long_edge;
use fotollection_core::thumbs::T1_LONG_EDGE;

fn main() {
    let codec = default_codec();
    let looks = built_in();
    let warm = looks
        .iter()
        .find(|l| l.id == "builtin:warm-film")
        .expect("warm film");
    let original = &looks[0];
    println!(
        "{:<28} {:>14} {:>14} {:>12}",
        "file", "ΔE original", "ΔE warm film", "develop ms"
    );
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read");
        let name = std::path::Path::new(&path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let Some(kind) = raw::kind(&bytes) else {
            continue;
        };
        let thumb = decode_for_screen(codec.as_ref(), &bytes, T1_LONG_EDGE).expect("preview");
        let mut embedded = fit_long_edge(&thumb.image, T1_LONG_EDGE).expect("fit");
        fotollection_core::colour::to_srgb(&mut embedded, thumb.icc.as_deref());
        let t = Instant::now();
        let Ok((prepared, _)) = raw::prepare(&bytes, kind) else {
            println!("{name:<28} not developed");
            continue;
        };
        let developed = prepared
            .finish(RawLook::Neutral, &RawAdjust::default())
            .expect("finish");
        let develop_ms = t.elapsed().as_secs_f64() * 1e3;
        let developed = fit_long_edge(&developed, T1_LONG_EDGE).expect("fit");
        let de = |look: &fotollection_core::looks::LookSettings| {
            let a = look_tile(&embedded, look).expect("tile");
            let b = look_tile(&developed, look).expect("tile");
            mean_de2000(&a, &b).unwrap_or(f32::NAN)
        };
        println!(
            "{name:<28} {:>14.2} {:>14.2} {:>12.0}",
            de(&original.look),
            de(&warm.look),
            develop_ms
        );
    }
}
