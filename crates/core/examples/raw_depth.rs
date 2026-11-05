// SPDX-License-Identifier: GPL-3.0-only
//! How far a RAW can be pushed (FR-088, RESULT §28): the same highlights −100 and shadows
//! +100 made the old way, by the look's table on the finished 8-bit picture, and the new
//! way, in linear light before the tone. In the photo's near-white pixels (luma ≥ 250 at
//! rest) it counts the distinct levels and their spread, the detail recovered; in its dark
//! pixels (luma ≤ 30) the distinct levels and the empty histogram bins between the darkest
//! and the brightest, the banding. Also the time of one slider step: the kept 2048 px
//! develop re-finished. Files are only read.
//!
//! `cargo run --release -p fotollection-core --example raw_depth -- <raw>…`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::time::Instant;

use fotollection_core::image::RgbImage;
use fotollection_core::looks::{LookSettings, RawSliders, apply};
use fotollection_core::raw::{self, RawAdjust, RawLook};

fn luma(img: &RgbImage) -> Vec<u8> {
    img.data
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| {
            (0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])).round()
                as u8
        })
        .collect()
}

/// Distinct levels, standard deviation and empty bins between the extremes, over `mask`.
fn stats(values: &[u8], mask: &[bool]) -> (usize, f32, usize) {
    let mut hist = [0usize; 256];
    let (mut sum, mut n) = (0f64, 0f64);
    for (&v, _) in values.iter().zip(mask).filter(|(_, m)| **m) {
        hist[usize::from(v)] += 1;
        sum += f64::from(v);
        n += 1.0;
    }
    let mean = sum / n.max(1.0);
    let var = values
        .iter()
        .zip(mask)
        .filter(|(_, m)| **m)
        .map(|(&v, _)| (f64::from(v) - mean).powi(2))
        .sum::<f64>()
        / n.max(1.0);
    let used: Vec<usize> = (0..256).filter(|&i| hist[i] > 0).collect();
    let gaps = match (used.first(), used.last()) {
        (Some(&lo), Some(&hi)) => (lo..=hi).filter(|&i| hist[i] == 0).count(),
        _ => 0,
    };
    (used.len(), var.sqrt() as f32, gaps)
}

fn table_look(edit: impl Fn(&mut LookSettings)) -> LookSettings {
    let mut look = LookSettings::default();
    edit(&mut look);
    look
}

fn main() {
    println!(
        "{:<28} {:>8} | {:>22} | {:>22} | {:>26} | {:>26}",
        "file",
        "step ms",
        "highlights old lv/sd",
        "highlights new lv/sd",
        "shadows old lv/sd/gaps",
        "shadows new lv/sd/gaps"
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
        let Ok((prepared, _)) = raw::prepare(&bytes, kind) else {
            println!("{name:<28} not prepared");
            continue;
        };
        let base = prepared
            .finish(RawLook::Neutral, &RawAdjust::default())
            .expect("finish");
        let base_luma = luma(&base);
        let bright: Vec<bool> = base_luma.iter().map(|&v| v >= 250).collect();
        let dark: Vec<bool> = base_luma.iter().map(|&v| v <= 30).collect();
        // Old: the table on the finished picture.
        let old = |look: LookSettings| {
            let mut img = base.clone();
            if let Some(t) = look.table() {
                apply(&mut img, &t);
            }
            luma(&img)
        };
        // New: in linear light before the tone.
        let new = |sliders: RawSliders| {
            luma(
                &prepared
                    .finish(RawLook::Neutral, &sliders.adjust())
                    .expect("finish"),
            )
        };
        let hi_old = stats(&old(table_look(|l| l.adjust.highlights = -100)), &bright);
        let hi_new = stats(
            &new(RawSliders {
                highlights: -100,
                amount: 100,
                ..RawSliders::default()
            }),
            &bright,
        );
        let sh_old = stats(&old(table_look(|l| l.adjust.shadows = 100)), &dark);
        let sh_new = stats(
            &new(RawSliders {
                shadows: 100,
                amount: 100,
                ..RawSliders::default()
            }),
            &dark,
        );
        // One step of a RAW slider: the kept develop re-finished, median of 5.
        let mut times: Vec<f64> = (0..5)
            .map(|i| {
                let t = Instant::now();
                let _ = prepared.finish(
                    RawLook::Neutral,
                    &RawSliders {
                        highlights: -20 - i,
                        amount: 100,
                        ..RawSliders::default()
                    }
                    .adjust(),
                );
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        times.sort_by(f64::total_cmp);
        let share = |m: &[bool]| 100.0 * m.iter().filter(|b| **b).count() as f32 / m.len() as f32;
        println!(
            "{name:<28} {:>8.1} | {:>4}/{:>5.1} ({:>4.1} %)     | {:>4}/{:>5.1}             | {:>4}/{:>5.1}/{:>3} ({:>4.1} %) | {:>4}/{:>5.1}/{:>3}",
            times[2],
            hi_old.0,
            hi_old.1,
            share(&bright),
            hi_new.0,
            hi_new.1,
            sh_old.0,
            sh_old.1,
            sh_old.2,
            share(&dark),
            sh_new.0,
            sh_new.1,
            sh_new.2,
        );
    }
}
