// SPDX-License-Identifier: GPL-3.0-only
//! Mutation testing of a user's font (FR-042, FR-134, SECURITY.md §3): real fonts are cut
//! short, have bytes flipped, extreme values written into their tables or their glyph data
//! damaged, and each result goes through what the app does with a font chosen in Settings:
//! `fonts::load` (the magic bytes, the size, the whole read of every glyph), then, if it was
//! accepted, its metrics and a line drawn at three sizes. A panic is caught and printed with
//! its seed; a load or a line slower than `SLOW` is printed as a possible hang. The fonts are
//! only read; nothing is written.
//!
//! `cargo run --release -p fotollection-core --example font_mutate -- <seconds> <ttf|otf>…`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use fotollection_core::fonts;
use fotollection_core::glyph::{Font, PxScale, ScaleFont, point};

/// A load or a line this slow on a damaged font is worth a look.
const SLOW: Duration = Duration::from_secs(2);

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
    // The table directory and the first tables (head, hhea, maxp, cmap, loca) sit near the start.
    let head = data.len().min(16 * 1024);
    match rng.below(4) {
        0 => data.truncate(rng.below(data.len())),
        1 => {
            for _ in 0..=rng.below(16) {
                let span = if rng.below(4) == 0 { data.len() } else { head };
                let i = rng.below(span);
                data[i] ^= (rng.next() & 0xFF) as u8 | 1;
            }
        }
        2 => {
            let i = rng.below(head.saturating_sub(4));
            let v: u32 = [0, 1, u32::MAX, 0x7FFF_FFFF, 0x0001_0000, 0xFFFF][rng.below(6)];
            data[i..i + 4].copy_from_slice(&v.to_be_bytes());
        }
        _ => {
            for _ in 0..=rng.below(64) {
                let i = head + rng.below(data.len().saturating_sub(head));
                if let Some(b) = data.get_mut(i) {
                    *b = (rng.next() & 0xFF) as u8;
                }
            }
        }
    }
    data
}

/// What the app does with a font: load it, then draw a line with it.
fn exercise(data: Vec<u8>) -> bool {
    let Ok((_, font)) = fonts::load(data) else {
        return false;
    };
    let _ = (font.units_per_em(), font.height_unscaled());
    for px in [12.0, 48.0, 300.0] {
        let scaled = font.as_scaled(PxScale::from(px));
        let mut x = 0.0;
        for c in "Ag0ưẫЖλ東서".chars() {
            let id = scaled.glyph_id(c);
            if let Some(outlined) =
                font.outline_glyph(id.with_scale_and_position(PxScale::from(px), point(x, px)))
            {
                let mut sum = 0.0;
                outlined.draw(|_, _, v| sum += v);
                assert!(sum.is_finite(), "coverage not finite");
            }
            x += scaled.h_advance(id);
        }
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seconds: u64 = args
        .first()
        .and_then(|s| s.parse().ok())
        .expect("usage: font_mutate <seconds> <font>…");
    let originals: Vec<(String, Vec<u8>)> = args[1..]
        .iter()
        .map(|p| (p.clone(), std::fs::read(p).expect("read font")))
        .collect();
    assert!(!originals.is_empty(), "give at least one font");
    std::panic::set_hook(Box::new(|_| {}));
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let (mut runs, mut accepted, mut panics, mut slow) = (0u64, 0u64, 0u64, 0u64);
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    while Instant::now() < deadline {
        for (name, original) in &originals {
            seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut rng = Rng(seed | 1);
            let data = mutate(&mut rng, original);
            let start = Instant::now();
            match catch_unwind(AssertUnwindSafe(|| exercise(data))) {
                Ok(true) => accepted += 1,
                Ok(false) => {}
                Err(_) => {
                    panics += 1;
                    println!("PANIC {name} seed {seed:#x}");
                }
            }
            if start.elapsed() > SLOW {
                slow += 1;
                println!("SLOW {name} seed {seed:#x}: {:?}", start.elapsed());
            }
            runs += 1;
        }
    }
    println!("{runs} mutated fonts, {accepted} accepted and drawn, {panics} panics, {slow} slow");
    if panics > 0 {
        std::process::exit(1);
    }
}
