// SPDX-License-Identifier: GPL-3.0-only
//! The time to draw text (FR-134, OPTIMIZATION §3): lines of each bundled face, through the
//! app's own `text::draw_text`, at the sizes a print uses (a date stamp, an info frame, a
//! postcard's message at 600 DPI). One face loaded once, as the app keeps them; the median of
//! five runs per face and in all. Built on any commit with `crate::glyph`, so two font engines
//! are timed on the same machine and the same words.
//!
//! `cargo run --release -p fotollection-core --example measure_text`
#![allow(clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::glyph::{FontArc, FontRef, VariableFont};
use fotollection_core::image::RgbImage;
use fotollection_core::text::{TextRun, draw_text};

const LINES: [&str; 3] = [
    "Greetings from Prague! The night tram home. 2026-11-15",
    "Chúc ngủ ngon từ Praha! Tàu điện đêm về nhà. Ảnh đẹp nhất",
    "東京都港区芝公園 서울특별시 종로구 세종대로 ΑΒΓΔ Жизнь",
];
const SIZES: [f32; 3] = [40.0, 90.0, 160.0];

fn faces() -> Vec<(&'static str, FontArc)> {
    let face = |name, bytes: &'static [u8]| (name, FontArc::try_from_slice(bytes).expect(name));
    let mut inter =
        FontRef::try_from_slice(include_bytes!("../assets/fonts/Inter.ttf")).expect("Inter");
    inter.set_variation(b"wght", 600.0);
    vec![
        face(
            "DSEG14",
            include_bytes!("../assets/fonts/DSEG14Classic-Italic.ttf"),
        ),
        ("Inter-600", FontArc::new(inter)),
        face(
            "XanhMono",
            include_bytes!("../assets/fonts/XanhMono-Regular.ttf"),
        ),
        face(
            "PatrickHand",
            include_bytes!("../assets/fonts/PatrickHand-Regular.ttf"),
        ),
        face(
            "KleeOne",
            include_bytes!("../assets/fonts/KleeOne-SemiBold-subset.ttf"),
        ),
        face(
            "NanumPen",
            include_bytes!("../assets/fonts/NanumPenScript-Regular-subset.ttf"),
        ),
    ]
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let faces = faces();
    let mut image = RgbImage::new(4000, 800);
    let mut totals = Vec::new();
    let mut per_face = vec![Vec::new(); faces.len()];
    for _ in 0..5 {
        let all = Instant::now();
        for (i, (_, font)) in faces.iter().enumerate() {
            let start = Instant::now();
            for _ in 0..4 {
                for (row, text) in LINES.iter().enumerate() {
                    for px in SIZES {
                        let run = TextRun {
                            font,
                            text,
                            px,
                            tracking: 0.0,
                        };
                        draw_text(
                            &mut image,
                            &run,
                            10.0,
                            row as f32 * 200.0,
                            [20, 20, 20],
                            1.0,
                        );
                    }
                }
            }
            per_face[i].push(start.elapsed().as_secs_f64() * 1000.0);
        }
        totals.push(all.elapsed().as_secs_f64() * 1000.0);
    }
    for ((name, _), times) in faces.iter().zip(per_face) {
        println!("{name:12} {:8.2} ms for 36 lines", median(times));
    }
    println!("all faces    {:8.2} ms (median of 5)", median(totals));
}
