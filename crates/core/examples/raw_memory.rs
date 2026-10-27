// SPDX-License-Identifier: GPL-3.0-only
//! One RAW develop through one decoder and nothing else, so `/usr/bin/time -l` reports that
//! decoder's peak memory (ADR-0030 quality gate). Prints the time.
//!
//! `cargo run --release -p fotollection-core --example raw_memory -- <os|rawler|rawcore> <raw>`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use fotollection_core::raw::{self, RawDecoder};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bytes = std::fs::read(&args[1]).expect("read");
    let kind = raw::kind(&bytes).expect("a RAW file");
    let t = Instant::now();
    let size = match args[0].as_str() {
        "os" => raw::Os
            .decode(&bytes, kind)
            .map(|d| (d.image.width as usize, d.image.height as usize))
            .expect("os"),
        "rawler" => raw::Rawler
            .decode(&bytes, kind)
            .map(|d| (d.image.width as usize, d.image.height as usize))
            .expect("rawler"),
        _ => {
            let linear = fotollection_rawcore::develop(&bytes).expect("rawcore");
            let pic = fotollection_rawcore::finish(
                &linear,
                &fotollection_rawcore::default_curve(&linear),
            );
            (pic.width, pic.height)
        }
    };
    println!(
        "{}: {:.0} ms, {}×{}",
        args[0],
        t.elapsed().as_secs_f64() * 1e3,
        size.0,
        size.1
    );
}
