// SPDX-License-Identifier: GPL-3.0-only
//! How each RAW is finished (ADR-0040, RESULT §27): for every file, the two camera fits'
//! errors against its embedded JPEG (mean CIE76 ΔE of block means), the one "Like the
//! camera's JPEG" takes, and the time of the neutral and the camera finishes. Files are only
//! read.
//!
//! `cargo run --release -p fotollection-core --example raw_fits -- <raw>…`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use fotollection_core::raw::{self, RawLook};
use fotollection_core::raw_finish::finish_linear;

fn main() {
    println!(
        "{:<28} {:>10} {:>10} {:>14} {:>9} {:>9}",
        "file", "curve+mat", "3 curves", "taken", "neutral", "camera"
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
        let Ok((linear, reference)) = raw::develop_linear(&bytes, kind) else {
            println!("{name:<28} not developed by our own");
            continue;
        };
        let t = Instant::now();
        let _ = finish_linear(&linear, reference.as_ref(), RawLook::Neutral);
        let neutral = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let f = finish_linear(&linear, reference.as_ref(), RawLook::Camera).expect("finish");
        let camera = t.elapsed().as_secs_f64() * 1e3;
        let score = |s: Option<f32>| s.map_or("–".to_owned(), |v| format!("{v:.2}"));
        println!(
            "{name:<28} {:>10} {:>10} {:>14} {neutral:>7.0}ms {camera:>7.0}ms",
            score(f.scores[0]),
            score(f.scores[1]),
            f.fit.map_or("none".to_owned(), |fit| format!("{fit:?}"))
        );
    }
}
