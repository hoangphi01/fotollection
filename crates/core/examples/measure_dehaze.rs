// SPDX-License-Identifier: GPL-3.0-only
//! Dehaze (FR-115, RESULT §39): its time on the 1600 px preview and a 24 MP export (median
//! of 10, OPTIMIZATION §3), and how much of a known veil it takes out of a synthetic scene
//! whose truth is known (ΔE2000 below the sky, before and after, per amount).
//!
//! `cargo run --release -p fotollection-core --example measure_dehaze`

#![allow(clippy::unwrap_used, clippy::print_stdout, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::dehaze::{apply, below_sky, clear_scene, hazed};
use fotollection_core::raw_finish::de2000_stats;

const AIR: [f32; 3] = [0.85, 0.87, 0.9];

fn median_ms(width: u32, height: u32) -> f64 {
    let (scene, depth) = clear_scene(width, height);
    let veiled = hazed(&scene, &depth, AIR, 1.2);
    let mut times: Vec<f64> = (0..10)
        .map(|_| {
            let mut image = veiled.clone();
            let start = Instant::now();
            apply(&mut image, 60);
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    times.sort_by(f64::total_cmp);
    times[5]
}

fn main() {
    println!("time (median of 10, amount 60)");
    println!("  1600 × 1067 preview  {:>7.2} ms", median_ms(1600, 1067));
    println!("  6000 × 4000 export   {:>7.2} ms", median_ms(6000, 4000));
    println!("\na known veil (t = e^(−β·depth)), ΔE2000 below the sky against the clear scene");
    println!(
        "  {:<6} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "β", "veiled", "+25", "+50", "+75", "+100"
    );
    let (scene, depth) = clear_scene(1600, 1067);
    let truth = below_sky(&scene);
    for beta in [0.5f32, 1.2, 2.0] {
        let veiled = hazed(&scene, &depth, AIR, beta);
        let mut cells = vec![de2000_stats(&below_sky(&veiled), &truth).unwrap().mean];
        for amount in [25i8, 50, 75, 100] {
            let mut out = veiled.clone();
            apply(&mut out, amount);
            cells.push(de2000_stats(&below_sky(&out), &truth).unwrap().mean);
        }
        print!("  {beta:<6} ");
        for c in cells {
            print!("{c:>8.2}");
        }
        println!();
    }
}
