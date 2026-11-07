// SPDX-License-Identifier: GPL-3.0-only
//! The built-in looks' measured checks (FR-079, RESULT §35), before the owner's eye: for each
//! look, (1) the colour cast it lays on a grey ramp, mean ΔE2000 against the same ramp made
//! grey again (a look meant to tint shows it, the others should stay under 2); (2) the pixels
//! it pushes to 255 beyond those already there, in % of a folder's photos at 400 px (at most
//! 0.5); and (3) the closest other look, mean ΔE2000 on the photos (at least 3, so no two
//! tiles look the same). Photos are only read; only numbers are printed.
//!
//! `cargo run --release -p fotollection-core --example measure_looks -- <folder> [photos]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::image::RgbImage;
use fotollection_core::look_library::built_in;
use fotollection_core::looks::{LookSettings, apply};
use fotollection_core::raw_finish::mean_de2000;
use fotollection_core::render::fit_long_edge;

fn through(image: &RgbImage, look: &LookSettings) -> RgbImage {
    let mut out = image.clone();
    if let Some(table) = look.table() {
        apply(&mut out, &table);
    }
    out
}

fn greyed(image: &RgbImage) -> RgbImage {
    let mut out = image.clone();
    for p in out.data.as_chunks_mut::<3>().0 {
        let y = (0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]))
            .round() as u8;
        *p = [y, y, y];
    }
    out
}

fn clipped(image: &RgbImage) -> usize {
    image
        .data
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|p| p.contains(&255))
        .count()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let folder = args.next().expect("a folder of JPEGs");
    let limit: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(24);
    let codec = default_codec();
    let mut files: Vec<_> = std::fs::read_dir(&folder)
        .expect("folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jpg")))
        .collect();
    files.sort();
    files.truncate(limit);
    let photos: Vec<RgbImage> = files
        .iter()
        .map(|f| {
            fit_long_edge(
                &decode_image(codec.as_ref(), &std::fs::read(f).unwrap())
                    .unwrap()
                    .image,
                400,
            )
            .unwrap()
        })
        .collect();
    let ramp = {
        let mut img = RgbImage::new(256, 8);
        for (i, p) in img.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            let v = (i % 256) as u8;
            *p = [v, v, v];
        }
        img
    };
    let looks = built_in();
    let looked: Vec<Vec<RgbImage>> = looks
        .iter()
        .map(|l| photos.iter().map(|p| through(p, &l.look)).collect())
        .collect();
    let pixels: usize = photos.iter().map(|p| (p.width * p.height) as usize).sum();
    let before: usize = photos.iter().map(clipped).sum();
    println!("{} photos at 400 px", photos.len());
    println!(
        "{:<16} {:>10} {:>12} {:>24}",
        "look", "grey cast", "new clip %", "closest look (ΔE)"
    );
    for (i, look) in looks.iter().enumerate() {
        let on_ramp = through(&ramp, &look.look);
        let cast = mean_de2000(&on_ramp, &greyed(&on_ramp)).unwrap();
        let after: usize = looked[i].iter().map(clipped).sum();
        let new_clip = 100.0 * (after.saturating_sub(before)) as f64 / pixels as f64;
        let closest = looks
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(j, other)| {
                let de: f32 = looked[i]
                    .iter()
                    .zip(&looked[j])
                    .map(|(a, b)| mean_de2000(a, b).unwrap())
                    .sum::<f32>()
                    / photos.len() as f32;
                (de, other.id.trim_start_matches("builtin:").to_owned())
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();
        println!(
            "{:<16} {:>10.2} {:>12.2} {:>16} {:>6.2}",
            look.id.trim_start_matches("builtin:"),
            cast,
            new_clip,
            closest.1,
            closest.0
        );
    }
}
