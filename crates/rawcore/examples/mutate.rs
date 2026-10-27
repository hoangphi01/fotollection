// SPDX-License-Identifier: GPL-3.0-only
//! Mutation testing of our own RAW decoder on stable Rust (ADR-0030's "no panic" gate): for
//! a while, real samples are cut short, have bytes flipped and directory values rewritten,
//! and each result is decoded; a panic is caught and printed with its seed, so it can be
//! replayed and turned into a test. Prints how many inputs were tried.
//!
//! `cargo run --release -p fotollection-rawcore --example mutate -- <seconds> <raw>…`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

/// xorshift64*: small, seedable, good enough to pick positions.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn mutate(rng: &mut Rng, original: &[u8]) -> Vec<u8> {
    let mut data = original.to_vec();
    match rng.below(4) {
        // Cut short anywhere.
        0 => data.truncate(rng.below(data.len())),
        // Flip a few bytes, most of them in the first 256 KB where the directories are.
        1 => {
            for _ in 0..=rng.below(16) {
                let span = if rng.below(4) == 0 {
                    data.len()
                } else {
                    data.len().min(256 * 1024)
                };
                let i = rng.below(span);
                data[i] ^= (rng.next() & 0xFF) as u8 | 1;
            }
        }
        // Write an extreme 32-bit value somewhere in the head (counts, offsets, sizes).
        2 => {
            let i = rng.below(data.len().min(256 * 1024).saturating_sub(4));
            let v: u32 = [0, 1, u32::MAX, 0x7FFF_FFFF, 0x0001_0000][rng.below(5)];
            data[i..i + 4].copy_from_slice(&v.to_le_bytes());
        }
        // Damage the compressed data only.
        _ => {
            let start = data.len().min(256 * 1024);
            for _ in 0..=rng.below(64) {
                let i = start + rng.below(data.len() - start);
                data[i] = (rng.next() & 0xFF) as u8;
            }
        }
    }
    data
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seconds: u64 = args[0].parse().expect("seconds");
    let samples: Vec<Vec<u8>> = args[1..]
        .iter()
        .map(|p| std::fs::read(p).expect("read"))
        .collect();
    // Quiet: panics are reported below, with their seed.
    std::panic::set_hook(Box::new(|_| {}));
    let end = Instant::now() + Duration::from_secs(seconds);
    let (mut tried, mut panics, mut decoded) = (0u64, 0u64, 0u64);
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    while Instant::now() < end {
        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut rng = Rng(seed | 1);
        let sample = &samples[rng.below(samples.len())];
        let data = mutate(&mut rng, sample);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let _ = fotollection_rawcore::dimensions(&data);
            fotollection_rawcore::develop(&data).is_ok()
        }));
        tried += 1;
        match outcome {
            Ok(true) => decoded += 1,
            Ok(false) => {}
            Err(_) => {
                panics += 1;
                println!("PANIC with seed {seed:#x}");
            }
        }
    }
    println!("{tried} mutated inputs in {seconds} s: {decoded} still decoded, {panics} panics");
    if panics > 0 {
        std::process::exit(1);
    }
}
