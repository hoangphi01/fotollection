// SPDX-License-Identifier: GPL-3.0-only
//! Mutation testing of every RAW path the app takes (SECURITY.md §3, ADR-0040): real files of
//! any format are cut short, have bytes flipped, extreme values written into their headers
//! or their compressed data damaged, and each result goes through format recognition, our
//! own develop, rawler and the app's whole chain (`raw::decode`, with its fallbacks). A
//! panic in our code is caught and printed with its seed; rawler's are caught by its crate
//! and counted; a decode slower than `SLOW` is printed as a possible hang. The files are only
//! read; nothing is written.
//!
//! `cargo run --release -p fotollection-core --example raw_mutate -- <seconds> <raw>…`
//! (`crates/rawcore/examples/mutate.rs` does the same for our DNG reader alone.)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use fotollection_core::codec::CodecError;
use fotollection_core::raw::{self, Own, RawDecoder, Rawler};

/// A develop this slow on a damaged file is worth a look.
const SLOW: Duration = Duration::from_secs(10);

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
    let head = data.len().min(256 * 1024);
    match rng.below(4) {
        // Cut short anywhere.
        0 => data.truncate(rng.below(data.len())),
        // Flip a few bytes, most of them in the head where the directories are.
        1 => {
            for _ in 0..=rng.below(16) {
                let span = if rng.below(4) == 0 { data.len() } else { head };
                let i = rng.below(span);
                data[i] ^= (rng.next() & 0xFF) as u8 | 1;
            }
        }
        // An extreme 32-bit value somewhere in the head (counts, offsets, sizes).
        2 => {
            let i = rng.below(head.saturating_sub(4));
            let v: u32 = [0, 1, u32::MAX, 0x7FFF_FFFF, 0x0001_0000][rng.below(5)];
            data[i..i + 4].copy_from_slice(&v.to_le_bytes());
        }
        // Damage the compressed data only.
        _ => {
            for _ in 0..=rng.below(64) {
                let i = head + rng.below(data.len() - head);
                data[i] = (rng.next() & 0xFF) as u8;
            }
        }
    }
    data
}

#[derive(Default)]
struct Tally {
    tried: u64,
    not_raw: u64,
    own_ok: u64,
    chain_ok: u64,
    errors: u64,
    rawler_caught: u64,
    panics: u64,
    slow: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seconds: u64 = args[0].parse().expect("seconds");
    let samples: Vec<Vec<u8>> = args[1..]
        .iter()
        .map(|p| std::fs::read(p).expect("read"))
        .collect();
    std::panic::set_hook(Box::new(|_| {}));
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut t = Tally::default();
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    while Instant::now() < end {
        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut rng = Rng(seed | 1);
        let pick = rng.below(samples.len());
        let data = mutate(&mut rng, &samples[pick]);
        t.tried += 1;
        let start = Instant::now();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let kind = raw::kind(&data[..data.len().min(raw::HEAD_BYTES)])?;
            let _ = Own.dimensions(&data, kind);
            let own = Own.decode(&data, kind);
            let rawler = Rawler.decode(&data, kind);
            let _ = raw::dimensions(&data, kind);
            let chain = raw::decode(&data, kind);
            Some((own.is_ok(), rawler, chain))
        }));
        if start.elapsed() > SLOW {
            t.slow += 1;
            println!(
                "SLOW ({:.1} s) with seed {seed:#x}",
                start.elapsed().as_secs_f64()
            );
        }
        match outcome {
            Err(_) => {
                t.panics += 1;
                println!("PANIC with seed {seed:#x}");
            }
            Ok(None) => t.not_raw += 1,
            Ok(Some((own, rawler, chain))) => {
                t.own_ok += u64::from(own);
                if let Err(CodecError::Decode(why)) = &rawler
                    && why.contains("failed on this file")
                {
                    t.rawler_caught += 1;
                }
                match chain {
                    Ok(_) => t.chain_ok += 1,
                    Err(_) => t.errors += 1,
                }
            }
        }
    }
    println!(
        "{} mutated inputs in {seconds} s: {} no longer RAW, {} still developed by our own, {} by the app's chain, {} refused with an error, {} rawler panics caught, {} slow, {} panics",
        t.tried, t.not_raw, t.own_ok, t.chain_ok, t.errors, t.rawler_caught, t.slow, t.panics
    );
    if t.panics > 0 || t.slow > 0 {
        std::process::exit(1);
    }
}
