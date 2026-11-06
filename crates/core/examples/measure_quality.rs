// SPDX-License-Identifier: GPL-3.0-only
//! What an export's JPEG quality costs on a print (FR-092), for the export dialog's badges and
//! docs/RESULT.md: each photo of a folder is sized for a 20 × 30 cm print at 300 DPI
//! (3543 px on its long edge), encoded as prints are (4:4:4) at qualities 60 to 100, decoded,
//! and compared with the quality-100 encode in OKLab: the mean and the 99th percentile of ΔE
//! (OKLab distance × 100, about 1 for a just noticeable step), and the file size. Photos are
//! only read; nothing is written. Only aggregate numbers are printed.
//!
//! `cargo run --release -p fotollection-core --example measure_quality -- <folder> [photos]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use fotollection_core::codec::{EncodeOptions, decode_image, default_codec};
use fotollection_core::image::RgbImage;
use fotollection_core::render::fit_long_edge;

/// 30 cm at 300 DPI.
const PRINT_LONG_EDGE: u32 = 3543;
const QUALITIES: [u8; 9] = [60, 65, 70, 75, 80, 85, 90, 95, 100];

fn linear(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab(px: &[u8]) -> [f32; 3] {
    let (r, g, b) = (linear(px[0]), linear(px[1]), linear(px[2]));
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_99 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// Mean and 99th percentile of ΔE (OKLab × 100), on every 4th pixel to keep it quick.
fn difference(a: &RgbImage, b: &RgbImage) -> (f64, f64) {
    let mut d: Vec<f32> = a
        .data
        .chunks_exact(3)
        .zip(b.data.chunks_exact(3))
        .step_by(4)
        .map(|(p, q)| {
            let (x, y) = (oklab(p), oklab(q));
            100.0 * ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt()
        })
        .collect();
    d.sort_by(f32::total_cmp);
    let mean = d.iter().map(|v| f64::from(*v)).sum::<f64>() / d.len() as f64;
    (mean, f64::from(d[(d.len() * 99) / 100]))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let folder = args
        .first()
        .expect("usage: measure_quality <folder> [photos]");
    let count: usize = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(12);
    let codec = default_codec();
    let mut paths: Vec<_> = std::fs::read_dir(folder)
        .expect("folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jpg")))
        .collect();
    paths.sort();
    paths.truncate(count);
    // Per quality: summed mean ΔE, the worst p99, summed bytes.
    let mut rows = vec![(0.0_f64, 0.0_f64, 0.0_f64); QUALITIES.len()];
    for path in &paths {
        let bytes = std::fs::read(path).expect("read");
        let photo = decode_image(codec.as_ref(), &bytes).expect("decode").image;
        let print = fit_long_edge(&photo, PRINT_LONG_EDGE).expect("resize");
        let encode = |q| {
            codec
                .encode(&print, EncodeOptions::print(q), None)
                .expect("encode")
        };
        let reference = decode_image(codec.as_ref(), &encode(100))
            .expect("decode")
            .image;
        for (i, &q) in QUALITIES.iter().enumerate() {
            let jpeg = encode(q);
            let shown = decode_image(codec.as_ref(), &jpeg).expect("decode").image;
            let (mean, p99) = difference(&reference, &shown);
            rows[i].0 += mean;
            rows[i].1 = rows[i].1.max(p99);
            rows[i].2 += jpeg.len() as f64;
        }
    }
    let n = paths.len() as f64;
    println!(
        "{} photos at {PRINT_LONG_EDGE} px (20 × 30 cm, 300 DPI), 4:4:4, against quality 100",
        paths.len()
    );
    println!("| Quality | Mean ΔE | Worst p99 ΔE | Mean size |");
    println!("|---|---|---|---|");
    for (i, q) in QUALITIES.iter().enumerate() {
        let (mean, p99, bytes) = rows[i];
        println!(
            "| {q} | {:.2} | {:.2} | {:.1} MB |",
            mean / n,
            p99,
            bytes / n / 1e6
        );
    }
}
