// SPDX-License-Identifier: GPL-3.0-only
//! The demosaic's score card (ADR-0037, v0.11.0 session 15): pictures whose truth is known
//! are turned into a Bayer mosaic, demosaiced, and compared with the truth. Each algorithm
//! gets the same charts, so a new one (RCD, then Markesteijn for X-Trans) is measured
//! against the ones before it, not against our eyes.
//!
//! - `zone`: a grey zone plate up to past the red and blue Nyquist: false colour, moiré.
//! - `star`: a grey Siemens star, 72 spokes: false colour and zipper at every angle.
//! - `edges`: saturated colour bars on grey, vertical, horizontal and diagonal: edge error.
//! - `ramps`: smooth colour gradients: what every method should get right.
//! - `flat`: flat grey with the two greens 2 % apart, as on many sensors: the maze (worm)
//!   pattern, the green channel's spread where it should be flat.
//!
//! `cargo run --release -p fotollection-rawcore --example charts [out.json]`
//!
//! Scores on the sRGB-encoded result, inside an 8-pixel margin: PSNR (dB, higher is better),
//! false colour (mean chroma on grey truth, 0–255, lower), edge error (mean absolute error
//! within 2 px of an edge, lower), maze (spread of green on the flat chart, lower).

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::cast_possible_wrap,
    clippy::float_cmp,
    clippy::manual_midpoint,
    clippy::type_complexity,
    clippy::format_push_string
)]

use fotollection_rawcore::cfa::Mosaic;
use fotollection_rawcore::demosaic::{CameraRgb, demosaic};

const N: usize = 768;
const MARGIN: usize = 8;
const WHITE: f32 = 4095.0;
/// RGGB: red at even rows and columns, blue at odd ones.
const PATTERN: [u8; 4] = [0, 1, 1, 2];

/// A picture in linear light, 0–1, row by row.
struct Truth {
    name: &'static str,
    rgb: Vec<[f32; 3]>,
    /// The truth has no colour (false colour is scored).
    grey: bool,
    /// Green sites at blue rows read this much more (the maze test).
    gb_gain: f32,
}

fn to_linear(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn to_encoded(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn charts() -> Vec<Truth> {
    let c = (N / 2) as f32;
    let grey = |v: f32| [to_linear(v); 3];
    let at = |f: &dyn Fn(f32, f32) -> [f32; 3]| -> Vec<[f32; 3]> {
        (0..N * N)
            .map(|i| f((i % N) as f32 - c, (i / N) as f32 - c))
            .collect()
    };
    // The local frequency is r / N cycles a pixel: green's Nyquist (0.5) at r = c, red's and
    // blue's (0.25) at r = c / 2. Past green's Nyquist no method can be right, so it stops.
    let zone = at(&|x, y| {
        let r2 = x * x + y * y;
        if r2 > c * c {
            return grey(0.5);
        }
        grey(0.5 + 0.4 * (std::f32::consts::PI * r2 / N as f32).cos())
    });
    let star = at(&|x, y| {
        let spoke = (y.atan2(x) * 36.0 / std::f32::consts::PI).floor() as i32;
        grey(if spoke.rem_euclid(2) == 0 { 0.85 } else { 0.12 })
    });
    let colours = [
        [0.8, 0.1, 0.1],
        [0.1, 0.7, 0.15],
        [0.12, 0.2, 0.85],
        [0.85, 0.8, 0.1],
    ];
    let edges = at(&|x, y| {
        let band = |v: f32| ((v + c) / 48.0).floor() as i32;
        let pick = if x < -c / 3.0 {
            band(x)
        } else if x < c / 3.0 {
            band(y)
        } else {
            band(x + y)
        };
        if pick.rem_euclid(2) == 0 {
            grey(0.5)
        } else {
            colours[pick.rem_euclid(8) as usize / 2].map(to_linear)
        }
    });
    let ramps = at(&|x, y| {
        let (u, v) = ((x + c) / N as f32, (y + c) / N as f32);
        [u, v, 1.0 - 0.5 * (u + v)].map(|e| to_linear(0.1 + 0.8 * e))
    });
    let flat = vec![grey(0.45); N * N];
    vec![
        Truth {
            name: "zone",
            rgb: zone,
            grey: true,
            gb_gain: 1.0,
        },
        Truth {
            name: "star",
            rgb: star,
            grey: true,
            gb_gain: 1.0,
        },
        Truth {
            name: "edges",
            rgb: edges,
            grey: false,
            gb_gain: 1.0,
        },
        Truth {
            name: "ramps",
            rgb: ramps,
            grey: false,
            gb_gain: 1.0,
        },
        Truth {
            name: "flat",
            rgb: flat,
            grey: true,
            gb_gain: 1.02,
        },
    ]
}

fn mosaic(truth: &Truth) -> Mosaic {
    let data = (0..N * N)
        .map(|i| {
            let (x, y) = (i % N, i / N);
            let site = PATTERN[(y & 1) * 2 + (x & 1)];
            let gain = if site == 1 && y & 1 == 1 {
                truth.gb_gain
            } else {
                1.0
            };
            (truth.rgb[i][usize::from(site)] * gain * WHITE)
                .round()
                .clamp(0.0, WHITE) as u16
        })
        .collect();
    Mosaic::new(
        N,
        N,
        data,
        PATTERN,
        [0.0; 4],
        WHITE,
        [0, 0, N, N],
        [0, 0, N, N],
    )
    .unwrap()
}

/// The plain bilinear demosaic, the floor every method must beat.
fn bilinear(m: &Mosaic) -> CameraRgb {
    let v = |x: isize, y: isize| {
        let (x, y) = (
            x.clamp(0, N as isize - 1) as usize,
            y.clamp(0, N as isize - 1) as usize,
        );
        (m.level(x, y), m.site(x, y))
    };
    let mut rgb = vec![0f32; N * N * 3];
    for y in 0..N as isize {
        for x in 0..N as isize {
            let mut sum = [0f32; 3];
            let mut count = [0f32; 3];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (value, site) = v(x + dx, y + dy);
                    let weight = if dx == 0 && dy == 0 {
                        4.0
                    } else if dx == 0 || dy == 0 {
                        2.0
                    } else {
                        1.0
                    };
                    sum[usize::from(site)] += value * weight;
                    count[usize::from(site)] += weight;
                }
            }
            let (own, site) = v(x, y);
            let o = (y as usize * N + x as usize) * 3;
            for ch in 0..3 {
                rgb[o + ch] = if ch == usize::from(site) {
                    own
                } else {
                    sum[ch] / count[ch].max(1.0)
                };
            }
        }
    }
    CameraRgb {
        width: N,
        height: N,
        rgb,
    }
}

#[derive(Default)]
struct Score {
    psnr: f64,
    false_colour: f64,
    edge_error: f64,
    maze: f64,
}

fn score(truth: &Truth, out: &CameraRgb) -> Score {
    let enc = |v: f32| f64::from(to_encoded(v)) * 255.0;
    let inside = |i: usize| {
        let (x, y) = (i % N, i / N);
        (MARGIN..N - MARGIN).contains(&x) && (MARGIN..N - MARGIN).contains(&y)
    };
    let (mut se, mut n, mut chroma, mut edge, mut edge_n) = (0f64, 0f64, 0f64, 0f64, 0f64);
    let mut greens = Vec::new();
    for i in (0..N * N).filter(|&i| inside(i)) {
        let got = [0, 1, 2].map(|ch| enc(out.rgb[i * 3 + ch]));
        let want = truth.rgb[i].map(enc);
        for ch in 0..3 {
            se += (got[ch] - want[ch]).powi(2);
        }
        n += 3.0;
        if truth.grey {
            let (hi, lo) = got
                .iter()
                .fold((f64::MIN, f64::MAX), |(a, b), &v| (a.max(v), b.min(v)));
            chroma += hi - lo;
        }
        // Near an edge of the truth: a neighbour within 2 px differs.
        let (x, y) = (i % N, i / N);
        let near = (-2isize..=2).any(|d| {
            let j = (y as isize * N as isize + (x as isize + d)) as usize;
            let k = ((y as isize + d) * N as isize + x as isize) as usize;
            truth.rgb[j] != truth.rgb[i] || truth.rgb[k] != truth.rgb[i]
        });
        if near && !truth.grey {
            edge += (0..3).map(|ch| (got[ch] - want[ch]).abs()).sum::<f64>() / 3.0;
            edge_n += 1.0;
        }
        greens.push(got[1]);
    }
    let mse = se / n;
    let mean = greens.iter().sum::<f64>() / greens.len() as f64;
    let spread =
        (greens.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / greens.len() as f64).sqrt();
    Score {
        psnr: if mse > 0.0 {
            10.0 * (255.0 * 255.0 / mse).log10()
        } else {
            99.0
        },
        false_colour: if truth.grey { chroma / (n / 3.0) } else { 0.0 },
        edge_error: if edge_n > 0.0 { edge / edge_n } else { 0.0 },
        maze: if truth.gb_gain == 1.0 { 0.0 } else { spread },
    }
}

fn main() {
    let out = std::env::args().nth(1);
    let methods: Vec<(&str, fn(&Mosaic) -> CameraRgb)> = vec![
        ("bilinear", bilinear),
        ("malvar", |m| demosaic(m, [1.0; 3], f32::MAX)),
    ];
    let mut json = String::from("{\n  \"date\": \"");
    json.push_str(
        &std::process::Command::new("date")
            .arg("+%Y-%m-%d")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default(),
    );
    json.push_str("\",\n  \"command\": \"cargo run --release -p fotollection-rawcore --example charts\",\n  \"size\": 768,\n  \"methods\": {\n");
    println!(
        "{:<10} {:<6} {:>7} {:>13} {:>11} {:>6}",
        "method", "chart", "PSNR", "false colour", "edge error", "maze"
    );
    let mut ms = vec![0f64; methods.len()];
    for (mi, (name, run)) in methods.iter().enumerate() {
        json.push_str(&format!("    \"{name}\": {{"));
        for (ci, truth) in charts().iter().enumerate() {
            let m = mosaic(truth);
            let start = std::time::Instant::now();
            let got = run(&m);
            ms[mi] += start.elapsed().as_secs_f64() * 1000.0;
            let s = score(truth, &got);
            println!(
                "{name:<10} {:<6} {:>7.2} {:>13.2} {:>11.2} {:>6.2}",
                truth.name, s.psnr, s.false_colour, s.edge_error, s.maze
            );
            json.push_str(&format!(
                "{}\"{}\": {{ \"psnr\": {:.2}, \"falseColour\": {:.3}, \"edgeError\": {:.3}, \"maze\": {:.3} }}",
                if ci == 0 { " " } else { ", " },
                truth.name, s.psnr, s.false_colour, s.edge_error, s.maze
            ));
        }
        json.push_str(&format!(", \"ms\": {:.1} }}", ms[mi]));
        json.push_str(if mi + 1 == methods.len() { "\n" } else { ",\n" });
    }
    json.push_str("  }\n}\n");
    for (mi, (name, _)) in methods.iter().enumerate() {
        println!("{name}: {:.1} ms for the five charts", ms[mi]);
    }
    if let Some(path) = out {
        std::fs::write(&path, json).unwrap();
        println!("wrote {path}");
    }
}
