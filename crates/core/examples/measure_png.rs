// SPDX-License-Identifier: GPL-3.0-only
//! Measures PNG decoding (FR-001, FR-072) on synthetic 3840 × 2160 screenshots: 8-bit RGB,
//! 8-bit RGBA (alpha flattened in linear light) and 16-bit RGB (dithered to 8 bits), against
//! the way it decoded before (png's 8-bit normalisation, alpha mixed in sRGB values). Best
//! of five, after one warm-up.
//!
//! `cargo run --release -p fotollection-core --example measure_png`

#![allow(
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

use std::io::Cursor;
use std::time::Instant;

use fotollection_core::codec::{PureRustCodec, decode_image};

const W: u32 = 3840;
const H: u32 = 2160;

fn encode(color: png::ColorType, depth: png::BitDepth) -> Vec<u8> {
    let channels = match color {
        png::ColorType::Rgba => 4,
        _ => 3,
    };
    let bytes = if depth == png::BitDepth::Sixteen {
        2
    } else {
        1
    };
    let mut pixels = Vec::with_capacity((W * H) as usize * channels * bytes);
    for y in 0..H {
        for x in 0..W {
            for c in 0..channels {
                let v = ((x * 65535 / W) ^ (y * 31) ^ (c as u32 * 4099)) as u16;
                let v = if c == 3 { (x * 65535 / W) as u16 } else { v };
                if bytes == 2 {
                    pixels.extend_from_slice(&v.to_be_bytes());
                } else {
                    pixels.push((v >> 8) as u8);
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, W, H);
    encoder.set_color(color);
    encoder.set_depth(depth);
    encoder.set_compression(png::Compression::Fast);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&pixels)
        .unwrap();
    out
}

/// The pre-FR-072 decode: 8-bit normalisation, alpha mixed in sRGB values.
fn before(bytes: &[u8]) -> usize {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buf).unwrap();
    buf.truncate(frame.buffer_size());
    let over = |c: u8, a: u8| {
        ((u16::from(c) * u16::from(a) + 255 * (255 - u16::from(a)) + 127) / 255) as u8
    };
    let out: Vec<u8> = match frame.color_type {
        png::ColorType::Rgba => buf
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[r, g, b, a]| [over(r, a), over(g, a), over(b, a)])
            .collect(),
        _ => buf,
    };
    out.len()
}

fn best(mut run: impl FnMut() -> usize) -> f64 {
    let _ = run();
    (0..5)
        .map(|_| {
            let t = Instant::now();
            let _ = run();
            t.elapsed().as_secs_f64() * 1e3
        })
        .fold(f64::MAX, f64::min)
}

fn main() {
    for (name, color, depth) in [
        ("RGB 8-bit", png::ColorType::Rgb, png::BitDepth::Eight),
        ("RGBA 8-bit", png::ColorType::Rgba, png::BitDepth::Eight),
        ("RGB 16-bit", png::ColorType::Rgb, png::BitDepth::Sixteen),
    ] {
        let bytes = encode(color, depth);
        let old = best(|| before(&bytes));
        let new = best(|| {
            decode_image(&PureRustCodec, &bytes)
                .unwrap()
                .image
                .data
                .len()
        });
        println!("{name}, {W} × {H}: before {old:.0} ms, now {new:.0} ms");
    }
}
