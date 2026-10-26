// SPDX-License-Identifier: GPL-3.0-only
//! Step 0 of ADR-0029, measured before rawler replaces the OS codecs for camera RAW: for
//! each sample, the full develop and the embedded preview through the OS (`ImageIO` here) and
//! through rawler, median of 3, and the three pictures written as 1600 px PNGs so
//! `bench/quality/raw_compare.py` can score their colour (ΔE2000) and structure (SSIM).
//! Samples are only read; the PNGs go to the folder given.
//!
//! `cargo run --release -p fotollection-core --example measure_rawler -- <out-dir> <raw>…`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::image::RgbImage;
use fotollection_core::render::fit_long_edge;
use fotollection_core::{raw, raw_finish};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn timed<T>(mut f: impl FnMut() -> T) -> (T, f64) {
    let mut times = Vec::new();
    let mut last = None;
    for _ in 0..3 {
        let t = Instant::now();
        last = Some(f());
        times.push(t.elapsed().as_secs_f64() * 1e3);
    }
    (last.unwrap(), median(times))
}

fn write_png(path: &Path, image: &RgbImage) {
    let small = fit_long_edge(image, 1600).expect("reduce");
    let file = std::io::BufWriter::new(std::fs::File::create(path).expect("create png"));
    let mut encoder = png::Encoder::new(file, small.width, small.height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(&small.data))
        .expect("write png");
}

fn rgb(image: fotollection_rawlgpl::Rgb8) -> RgbImage {
    RgbImage {
        width: image.width,
        height: image.height,
        data: image.rgb,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&out).expect("out dir");
    println!(
        "{:<28} {:>11} {:>11} {:>9} {:>9}  {:>11} {:>11}",
        "sample", "OS develop", "rawler", "OS prev.", "rawler p.", "OS size", "rawler size"
    );
    for input in &args[1..] {
        let path = Path::new(input);
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let bytes = std::fs::read(path).expect("read sample");
        let kind = raw::kind(&bytes).expect("a RAW file");
        let (os, os_ms) = timed(|| raw::decode(&bytes, kind));
        // rawler's develop finished to the camera's look with its own embedded image.
        let (rl, rl_ms) = timed(|| {
            let developed = fotollection_rawlgpl::develop(&bytes, 200_000_000)?;
            let reference = fotollection_rawlgpl::preview(&bytes)
                .ok()
                .flatten()
                .map(rgb);
            let curves = reference
                .as_ref()
                .and_then(|r| raw_finish::match_reference(&developed.rgb, r))
                .unwrap_or_else(|| raw_finish::default_curves(&developed.rgb));
            Ok::<_, fotollection_rawlgpl::RawError>(raw_finish::apply(
                &developed.rgb,
                developed.width,
                developed.height,
                &curves,
            ))
        });
        let (os_prev, os_prev_ms) = timed(|| raw::preview(&bytes, kind, 2048));
        let (rl_prev, rl_prev_ms) = timed(|| fotollection_rawlgpl::preview(&bytes));
        let size = |w: u32, h: u32| format!("{w}×{h}");
        let os_size = os
            .as_ref()
            .map_or("error".into(), |d| size(d.image.width, d.image.height));
        let rl_size = rl
            .as_ref()
            .map_or_else(|e| format!("{e}"), |d| size(d.width, d.height));
        println!(
            "{name:<28} {os_ms:>9.0}ms {rl_ms:>9.0}ms {os_prev_ms:>7.0}ms {rl_prev_ms:>7.0}ms  {os_size:>11} {rl_size:>11}"
        );
        if let Ok(d) = os {
            write_png(&out.join(format!("{name}.os.png")), &d.image);
        }
        if let Ok(d) = rl {
            write_png(&out.join(format!("{name}.rawler.png")), &d);
        }
        if let Ok(d) = os_prev {
            write_png(&out.join(format!("{name}.embedded.png")), &d.image);
        }
        match rl_prev {
            Ok(Some(_)) => {}
            Ok(None) => println!("  rawler: no embedded preview"),
            Err(e) => println!("  rawler preview: {e}"),
        }
    }
}
