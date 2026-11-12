// SPDX-License-Identifier: GPL-3.0-only
//! Highlight recovery measured against known truth (FR-112, RESULT §41).
//!
//! - **Real frames:** each RAW given is developed whole (the truth), then again with its
//!   sensor clipped at `CUT` of its white level, as if shot brighter; only the pixels the
//!   clip changed are scored, and those the truth itself had clipped are left out.
//! - **Synthetic frames:** a sky with a sun, two lamps of different colours side by side, a
//!   cloud (every channel clipped) and skin, in white-balanced camera RGB, clipped at each
//!   channel's gain, scored on what clipped.
//!
//! The scores, over the scored pixels: the error of `ln(R/G)` and `ln(B/G)` (colour), of
//! `log2` luminance (EV), CIEDE2000 after −2 EV (what lowering the exposure shows), the jump
//! across the clipped area's edge against the truth's (seams), and the develop's time.
//!
//! `--png <folder>` also writes each clipped frame finished (the neutral look), before and
//! after, as it is and two stops down, a quarter of its size, as PPM.
//!
//! `--export <folder>` writes, per frame, the linear develops two stops down as 16-bit sRGB
//! PPM (`truth_off`, `truth_opp`, `cut_off` and `cut_<Method>`, the clipped ones brought back
//! to the truth's scale), the files another raw developer's output is scored beside
//! (RESULT §41, `bench/rawtherapee/compare.sh`).
//!
//! `cargo run --release -p fotollection-core --example highlights_bench -- [--sweep] [--png <folder>] [--export <folder>] <raw>…`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::time::Instant;

use fotollection_core::raw_finish::{de2000, lab};
use fotollection_rawcore::cfa::Mosaic;
use fotollection_rawcore::color::ColorTags;
use fotollection_rawcore::demosaic::CameraRgb;
use fotollection_rawcore::highlights::{self, Method, Params};
use fotollection_rawcore::{Linear, develop_mosaic_with};

/// The share of the white level a real frame is clipped at.
const CUT: f32 = 0.6;

const METHODS: [Method; 4] = [
    Method::White,
    Method::Global,
    Method::Regions,
    Method::Local,
];

#[derive(Debug, Default, Clone, Copy)]
struct Score {
    pixels: usize,
    chroma: f64,
    ev: f64,
    de: f64,
    seam: f64,
    seams: usize,
    /// Where the truth itself clipped (no answer): how coloured the guess is.
    guess: f64,
    guesses: usize,
}

impl Score {
    fn add(&mut self, o: &Self) {
        self.pixels += o.pixels;
        self.chroma += o.chroma;
        self.ev += o.ev;
        self.de += o.de;
        self.seam += o.seam;
        self.seams += o.seams;
        self.guess += o.guess;
        self.guesses += o.guesses;
    }

    fn row(&self) -> String {
        let n = self.pixels.max(1) as f64;
        format!(
            "{:>9} {:>8.3} {:>7.3} {:>7.2} {:>7.3} {:>7.3}",
            self.pixels,
            self.chroma / n,
            self.ev / n,
            self.de / n,
            self.seam / self.seams.max(1) as f64,
            self.guess / self.guesses.max(1) as f64
        )
    }
}

fn luminance(p: &[f32]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// An sRGB 8-bit pixel of linear `p` lowered by two stops, clipped as a screen shows it.
fn shown(p: &[f32]) -> [u8; 3] {
    let enc = |v: f32| {
        let v = (v * 0.25).clamp(0.0, 1.0);
        let e = if v <= 0.003_130_8 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (e * 255.0).round() as u8
    };
    [enc(p[0]), enc(p[1]), enc(p[2])]
}

/// Scores `got` against `truth` over `mask` (both linear RGB, `w` wide).
fn score(got: &[f32], truth: &[f32], mask: &[bool], w: usize) -> Score {
    let mut s = Score::default();
    let eps = 1e-4f32;
    for (i, &m) in mask.iter().enumerate() {
        if !m {
            continue;
        }
        let (g, t) = (&got[i * 3..i * 3 + 3], &truth[i * 3..i * 3 + 3]);
        let ratio = |p: &[f32], c: usize| (p[c].max(eps) / p[1].max(eps)).ln();
        s.chroma += f64::from(f32::midpoint(
            (ratio(g, 0) - ratio(t, 0)).abs(),
            (ratio(g, 2) - ratio(t, 2)).abs(),
        ));
        s.ev += f64::from((luminance(g).max(eps) / luminance(t).max(eps)).log2().abs());
        s.de += f64::from(de2000(lab(shown(g)), lab(shown(t))));
        s.pixels += 1;
        // The step to the right and below where the mask ends, against the truth's.
        for j in [i + 1, i + w] {
            if j < mask.len() && !mask[j] && (j != i + 1 || j % w != 0) {
                let step = |p: &[f32]| {
                    (luminance(&p[i * 3..i * 3 + 3]).max(eps)
                        / luminance(&p[j * 3..j * 3 + 3]).max(eps))
                    .log2()
                };
                s.seam += f64::from((step(got) - step(truth)).abs());
                s.seams += 1;
            }
        }
    }
    s
}

/// The mosaic and colour tags of a RAW file: our DNG reader, rawler for the rest.
fn read(path: &str) -> Option<(Mosaic, ColorTags)> {
    let bytes = std::fs::read(path).ok()?;
    if fotollection_rawcore::dng::is_dng(&bytes) {
        let d = fotollection_rawcore::dng::parse(&bytes).ok()?;
        return Some((d.mosaic, d.color));
    }
    let u = fotollection_rawlgpl::unpack(&bytes, 200_000_000).ok()?;
    let tags = ColorTags::from_camera(&u.matrices, u.white_balance);
    let m = Mosaic::new(
        u.width, u.height, u.data, u.pattern, u.black, u.white, u.active, u.crop,
    )
    .ok()?;
    Some((m, tags))
}

fn develop(m: &Mosaic, tags: &ColorTags, method: Method, params: &Params) -> (Linear, f64) {
    let t = Instant::now();
    let l = develop_mosaic_with(m, tags, method, params).expect("develop");
    (l, t.elapsed().as_secs_f64() * 1e3)
}

/// Writes linear sRGB `rgb` (`w` × `h`) two stops down as a 16-bit sRGB PPM.
fn export(rgb: &[f32], w: usize, h: usize, path: &std::path::Path) {
    let mut out = format!("P6\n{w} {h}\n65535\n").into_bytes();
    for &v in rgb {
        let v = (v * 0.25).clamp(0.0, 1.0);
        let e = if v <= 0.003_130_8 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        out.extend_from_slice(&((e * 65535.0).round() as u16).to_be_bytes());
    }
    std::fs::write(path, out).expect("write");
}

/// Writes `linear` finished with the neutral look, `stops` brighter, a quarter of its size.
fn picture(linear: &Linear, stops: f32, path: &std::path::Path) {
    let adjust = fotollection_rawcore::RawAdjust {
        exposure: stops,
        ..Default::default()
    };
    let f = fotollection_core::raw_finish::finish_linear(
        linear.clone(),
        None,
        fotollection_core::raw::RawLook::Neutral,
        &adjust,
        false,
    )
    .expect("finish");
    let (w, h) = (f.image.width as usize, f.image.height as usize);
    let (sw, sh) = (w / 4, h / 4);
    let mut out = format!("P6\n{sw} {sh}\n255\n").into_bytes();
    for y in 0..sh {
        for x in 0..sw {
            for c in 0..3 {
                let mut sum = 0u32;
                for dy in 0..4 {
                    for dx in 0..4 {
                        sum += u32::from(f.image.data[((y * 4 + dy) * w + x * 4 + dx) * 3 + c]);
                    }
                }
                out.push((sum / 16) as u8);
            }
        }
    }
    std::fs::write(path, out).expect("write");
}

/// One real frame: the truth, the clipped frame by each method, their scores and times.
fn real(
    path: &str,
    methods: &[(Method, Params)],
    png: Option<&std::path::Path>,
    exported: Option<&std::path::Path>,
) -> Option<Vec<(Score, f64, f64)>> {
    let (mosaic, tags) = read(path)?;
    let base = highlights::DEFAULT.1;
    let (truth, _) = develop(&mosaic, &tags, Method::White, &base);
    let (truth_local, _) = develop(&mosaic, &tags, Method::Local, &base);
    // The develop with nothing new clipped, timed: what recovery costs a frame without it.
    let black = mosaic.black.iter().copied().fold(0f32, f32::max);
    let level = black + CUT * (mosaic.white - black);
    // The clipped frame's white is `level`, so its values are the truth's over `CUT`.
    let scale = (level - black) / (mosaic.white - black);
    let mut cut = mosaic.clone();
    let top = level.round() as u16;
    for v in &mut cut.data {
        *v = (*v).min(top);
    }
    cut.white = level;
    let (old, _) = develop(&cut, &tags, Method::White, &base);
    if let Some(dir) = png {
        let stem = std::path::Path::new(path)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for (method, params) in methods.iter().take(METHODS.len()) {
            let (l, _) = develop(&cut, &tags, *method, params);
            for stops in [0.0f32, -2.0] {
                picture(
                    &l,
                    stops,
                    &dir.join(format!("{stem}-{method:?}{stops:+.0}.ppm")),
                );
            }
        }
    }
    let w = truth.width;
    let to_truth = |l: &Linear| l.rgb.iter().map(|v| v * scale).collect::<Vec<f32>>();
    let old_scaled = to_truth(&old);
    if let Some(dir) = exported {
        let stem = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let dir = dir.join(stem);
        std::fs::create_dir_all(&dir).expect("folder");
        let h = truth.height;
        export(&truth.rgb, w, h, &dir.join("truth_off.ppm"));
        export(&truth_local.rgb, w, h, &dir.join("truth_opp.ppm"));
        export(&old_scaled, w, h, &dir.join("cut_off.ppm"));
        for (method, params) in methods.iter().take(METHODS.len()) {
            let (l, _) = develop(&cut, &tags, *method, params);
            export(
                &to_truth(&l),
                w,
                h,
                &dir.join(format!("cut_{method:?}.ppm")),
            );
        }
    }
    // Scored: what the clip changed, where the truth itself was not clipped.
    let unknown: Vec<bool> = (0..w * truth.height)
        .map(|i| (0..3).any(|c| (truth.rgb[i * 3 + c] - truth_local.rgb[i * 3 + c]).abs() > 1e-6))
        .collect();
    let mask: Vec<bool> = (0..w * truth.height)
        .map(|i| {
            let (o, t, tl) = (
                &old_scaled[i * 3..i * 3 + 3],
                &truth.rgb[i * 3..i * 3 + 3],
                &truth_local.rgb[i * 3..i * 3 + 3],
            );
            let changed = (0..3).any(|c| (o[c] - t[c]).abs() > 0.01 * t[c].max(0.01));
            let truth_clipped = (0..3).any(|c| (t[c] - tl[c]).abs() > 1e-6);
            changed && !truth_clipped
        })
        .collect();
    let mut out = Vec::new();
    for (method, params) in methods {
        let (got, ms) = develop(&cut, &tags, *method, params);
        let (_, whole_ms) = develop(&mosaic, &tags, *method, params);
        let mut s = score(&to_truth(&got), &truth.rgb, &mask, w);
        for (p, _) in got
            .rgb
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&unknown)
            .filter(|(_, u)| **u)
        {
            let ratio = |c: usize| (p[c].max(1e-4) / p[1].max(1e-4)).ln().abs();
            s.guess += f64::from(f32::midpoint(ratio(0), ratio(2)));
            s.guesses += 1;
        }
        out.push((s, ms, whole_ms));
    }
    Some(out)
}

/// A synthetic scene: its name, width, height and truth at each pixel.
type Scene = (
    &'static str,
    usize,
    usize,
    Box<dyn Fn(usize, usize) -> [f32; 3]>,
);

/// The synthetic scenes, white-balanced camera RGB with each channel clipping at its gain.
fn scenes() -> Vec<Scene> {
    vec![
        (
            "sky and sun",
            600,
            400,
            Box::new(|x, y| {
                let lift = 1.0 + 1.6 * (1.0 - y as f32 / 400.0) + 0.0008 * x as f32;
                let sun =
                    6.0 * (-((x as f32 - 420.0).powi(2) + (y as f32 - 90.0).powi(2)) / 900.0).exp();
                [
                    0.35 * lift + sun,
                    0.55 * lift + sun * 0.95,
                    1.25 * lift + sun * 0.8,
                ]
            }),
        ),
        (
            "two lamps",
            600,
            300,
            Box::new(|x, y| {
                let r = (
                    (x as f32 - 300.0).abs().max(0.0) / 300.0,
                    (y as f32 - 150.0) / 150.0,
                );
                let glow = 2.6 * (1.0 - (r.0 * r.0 + r.1 * r.1).sqrt()).max(0.0) + 0.2;
                if x < 300 {
                    [0.95 * glow, 0.5 * glow, 0.18 * glow]
                } else {
                    [0.25 * glow, 0.5 * glow, 1.0 * glow]
                }
            }),
        ),
        (
            "cloud",
            400,
            400,
            Box::new(|x, y| {
                let d = (x as f32 - 200.0).hypot(y as f32 - 200.0) / 200.0;
                let v = 3.5 * (1.0 - d).max(0.0) + 0.25;
                [0.92 * v, 0.95 * v, 0.9 * v]
            }),
        ),
        (
            "skin",
            400,
            400,
            Box::new(|x, y| {
                let d = (x as f32 - 200.0).hypot(y as f32 - 150.0) / 220.0;
                let v = 2.4 * (1.0 - d).max(0.0) + 0.15;
                [0.9 * v, 0.62 * v, 0.48 * v]
            }),
        ),
    ]
}

const GAINS: [f32; 3] = [2.0, 1.0, 1.6];

fn synthetic(methods: &[(Method, Params)]) -> Vec<(String, Vec<(Score, f64)>)> {
    scenes()
        .into_iter()
        .map(|(name, w, h, f)| {
            let mut truth = Vec::with_capacity(w * h * 3);
            for y in 0..h {
                for x in 0..w {
                    truth.extend(f(x, y));
                }
            }
            let seen: Vec<f32> = truth
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[0].min(GAINS[0]), p[1].min(GAINS[1]), p[2].min(GAINS[2])])
                .collect();
            let mask: Vec<bool> = truth
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| (0..3).any(|c| p[c] >= GAINS[c] * 0.85))
                .collect();
            let scores = methods
                .iter()
                .map(|(m, params)| {
                    let mut cam = CameraRgb {
                        width: w,
                        height: h,
                        rgb: seen.clone(),
                    };
                    let t = Instant::now();
                    highlights::recover(&mut cam, GAINS, *m, params);
                    let ms = t.elapsed().as_secs_f64() * 1e3;
                    (score(&cam.rgb, &truth, &mask, w), ms)
                })
                .collect();
            (name.to_owned(), scores)
        })
        .collect()
}

/// The cost at 24 MP: a frame with nothing clipped, and one with half its pixels clipped.
fn timing(methods: &[(Method, Params)]) {
    let (w, h) = (6000, 4000);
    for (label, lift) in [
        ("24 MP, nothing clipped", 0.5f32),
        ("24 MP, half clipped", 2.0),
    ] {
        let rgb: Vec<f32> = (0..h)
            .flat_map(|y| {
                (0..w).flat_map(move |x| {
                    let v = lift * (1.0 - y as f32 / h as f32) + 0.0001 * x as f32 + 0.05;
                    [0.35 * v, 0.55 * v, 1.0 * v]
                })
            })
            .collect::<Vec<f32>>()
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0].min(GAINS[0]), p[1].min(GAINS[1]), p[2].min(GAINS[2])])
            .collect();
        print!("{label:<26}");
        for (m, params) in methods {
            let mut best = f64::MAX;
            for _ in 0..5 {
                let mut cam = CameraRgb {
                    width: w,
                    height: h,
                    rgb: rgb.clone(),
                };
                let t = Instant::now();
                highlights::recover(&mut cam, GAINS, *m, params);
                best = best.min(t.elapsed().as_secs_f64() * 1e3);
            }
            print!(" {m:?} {best:>6.1} ms");
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sweep = args.iter().any(|a| a == "--sweep");
    let png = args
        .iter()
        .position(|a| a == "--png")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from);
    let exported = args
        .iter()
        .position(|a| a == "--export")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from);
    let files: Vec<&String> = args
        .iter()
        .filter(|a| {
            let p = Some(std::path::PathBuf::from(a.as_str()));
            !a.starts_with("--") && p != png && p != exported
        })
        .collect();
    let base = highlights::DEFAULT.1;
    let mut methods: Vec<(Method, Params)> = METHODS.iter().map(|&m| (m, base)).collect();
    // The sweep of RESULT §41: the threshold, the edge's width and the tile; `edge`, `dark`
    // and `enough` were swept as well and changed nothing for the better.
    if sweep {
        for threshold in [0.85, 0.9, 0.94, 0.97] {
            for band in [3, 6, 10] {
                methods.push((
                    Method::Global,
                    Params {
                        threshold,
                        band,
                        ..base
                    },
                ));
                for tile in [32, 64] {
                    methods.push((
                        Method::Local,
                        Params {
                            threshold,
                            band,
                            tile,
                            ..base
                        },
                    ));
                }
            }
        }
    }
    let name =
        |(m, p): &(Method, Params)| format!("{m:?} t{:.2} b{} k{}", p.threshold, p.band, p.tile);
    let header = format!(
        "{:<30} {:>9} {:>8} {:>7} {:>7} {:>7} {:>7}",
        "method", "pixels", "chroma", "EV", "ΔE −2EV", "seam", "guess"
    );

    println!("== synthetic (camera RGB, gains {GAINS:?})");
    for (scene, scores) in synthetic(&methods) {
        println!("-- {scene}\n{header}");
        for (m, (s, ms)) in methods.iter().zip(scores) {
            println!("{:<30} {} {ms:>6.1} ms", name(m), s.row());
        }
    }
    timing(&methods[..METHODS.len()]);

    if files.is_empty() {
        return;
    }
    println!("== real frames, clipped at {:.0} % of white", CUT * 100.0);
    let mut totals = vec![Score::default(); methods.len()];
    for path in files {
        let file = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let Some(rows) = real(path, &methods, png.as_deref(), exported.as_deref()) else {
            println!("-- {file}: not read");
            continue;
        };
        println!("-- {file}\n{header} {:>9} {:>9}", "cut ms", "whole ms");
        for (i, (m, (s, ms, whole))) in methods.iter().zip(rows).enumerate() {
            totals[i].add(&s);
            println!("{:<30} {} {ms:>7.0} ms {whole:>7.0} ms", name(m), s.row());
        }
    }
    println!("== all real frames\n{header}");
    let mut order: Vec<usize> = (0..methods.len()).collect();
    order.sort_by(|&a, &b| {
        let de = |s: &Score| s.de / s.pixels.max(1) as f64;
        de(&totals[a]).total_cmp(&de(&totals[b]))
    });
    for i in order {
        println!("{:<30} {}", name(&methods[i]), totals[i].row());
    }
}
