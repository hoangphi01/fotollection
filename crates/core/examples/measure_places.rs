// SPDX-License-Identifier: GPL-3.0-only
//! Step 0 of v0.8.0 (FR-030, FR-031, FR-058), measured before building:
//!
//! - A: sorting by capture time and recomputing an "effective time" column (EXIF time plus a
//!   per-camera offset) on a synthetic library of N photos, median of 5.
//! - B: the visual signature from a T1 thumbnail (decode the 400 px JPEG, reduce to 256 px,
//!   sign), per photo, on real photos.
//! - C: a proxy for matching a camera photo to a phone photo of the same scene: each photo
//!   against a closer, re-framed and darker crop of itself (positives) and against photos
//!   five or more frames away (negatives), with `structure_zoomed` over a few zooms and
//!   `color`. Real phone/camera pairs differ more (lens, perspective, processing), so this
//!   only bounds what to expect.
//! - D: the cost of looking for `.xmp` sidecars next to each photo on import.
//!
//! Photos are only read, never written. Nothing is cached on disk.
//!
//! `cargo run --release -p fotollection-core --example measure_places -- <photo folder> [N…]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::cast_possible_wrap
)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::codec::{EncodeOptions, decode_for_screen, default_codec};
use fotollection_core::culling::similar::{self, Signature};
use fotollection_core::image::RgbImage;
use fotollection_core::import::ImportedFile;
use fotollection_core::library::Library;
use fotollection_core::metadata::PhotoMeta;
use fotollection_core::render::fit_long_edge;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn percentile(v: &[f32], p: f64) -> f32 {
    let mut v = v.to_vec();
    v.sort_by(f32::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn part_a(sizes: &[usize]) {
    println!("A. Sorting by time and recomputing the effective time");
    for &n in sizes {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/Users/demo/Pictures/Trip"));
        let base = chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap();
        for i in 0..n {
            // Three cameras interleaved, a shot every ~7 s, shuffled a little.
            let jitter = ((i * 7919) % 13) as i64;
            let meta = PhotoMeta {
                capture: Some(base + chrono::Duration::seconds(i as i64 * 7 + jitter)),
                make: Some(["Panasonic", "Apple", "Sony"][i % 3].to_owned()),
                model: Some(["DC-GX9", "iPhone 15", "ILCE-6400"][i % 3].to_owned()),
                ..PhotoMeta::default()
            };
            let file = ImportedFile {
                path: PathBuf::from(format!("/Users/demo/Pictures/Trip/IMG_{i:05}.JPG")),
                size: 3_000_000,
                mtime: 1_700_000_000,
            };
            lib.push(folder, &file, &meta).expect("new photo");
        }
        let mut sort = Vec::new();
        for _ in 0..5 {
            let t = Instant::now();
            let ids = lib.sorted_ids();
            sort.push(t.elapsed().as_secs_f64() * 1e3);
            assert_eq!(ids.len(), n);
        }
        // The effective column: capture + offset of the photo's camera (one pass).
        let capture: Vec<i64> = (0..n as i64).map(|i| 1_780_000_000 + i * 7).collect();
        let camera: Vec<u16> = (0..n).map(|i| (i % 3) as u16).collect();
        let offsets = [3_720i64, 0, -45];
        let mut recompute = Vec::new();
        let mut effective = vec![0i64; n];
        for _ in 0..5 {
            let t = Instant::now();
            for i in 0..n {
                effective[i] = capture[i] + offsets[camera[i] as usize];
            }
            recompute.push(t.elapsed().as_secs_f64() * 1e3);
        }
        std::hint::black_box(&effective);
        println!(
            "  {n:>6} photos: sorted_ids {:.1} ms · effective column {:.2} ms",
            median(sort),
            median(recompute)
        );
    }
}

fn crop(image: &RgbImage, zoom: f32, aspect: f32, dx: f32) -> RgbImage {
    let (w, h) = (image.width as f32, image.height as f32);
    let mut cw = w / zoom;
    let mut ch = cw / aspect;
    if ch > h / zoom {
        ch = h / zoom;
        cw = ch * aspect;
    }
    let x0 = ((w - cw) / 2.0 + dx * w).clamp(0.0, w - cw) as u32;
    let y0 = ((h - ch) / 2.0) as u32;
    let (cw, ch) = (cw as u32, ch as u32);
    let mut data = Vec::with_capacity((cw * ch * 3) as usize);
    for y in y0..y0 + ch {
        let row = ((y * image.width + x0) * 3) as usize;
        data.extend_from_slice(&image.data[row..row + (cw * 3) as usize]);
    }
    RgbImage {
        width: cw,
        height: ch,
        data,
    }
}

fn darker(mut image: RgbImage, factor: f32) -> RgbImage {
    for v in &mut image.data {
        *v = (f32::from(*v) * factor) as u8;
    }
    image
}

/// T1 as the app makes it (400 px JPEG), then what reading it back costs: decode, 256 px, sign.
fn sign_from_t1(t1: &[u8]) -> (Signature, f64) {
    let codec = default_codec();
    let t = Instant::now();
    let decoded = codec.decode(t1).expect("decode T1");
    let small = fit_long_edge(&decoded.image, 256).expect("reduce");
    let sig = similar::signature(&small);
    (sig, t.elapsed().as_secs_f64() * 1e3)
}

fn to_t1(image: &RgbImage) -> Vec<u8> {
    let small = fit_long_edge(image, 400).expect("T1 size");
    default_codec()
        .encode(&small, EncodeOptions::screen(80), None)
        .expect("encode T1")
}

/// The best match over the zooms a phone and a camera lens differ by.
const ZOOMS: [f32; 5] = [1.0, 1.25, 1.5, 2.0, 2.5];

fn score(a: &Signature, b: &Signature) -> (f32, f32, f64) {
    let t = Instant::now();
    let structure = ZOOMS
        .iter()
        .map(|&z| similar::structure_zoomed(a, b, z))
        .fold(-1f32, f32::max);
    let color = similar::color(a, b);
    (structure, color, t.elapsed().as_secs_f64() * 1e3)
}

fn parts_b_c_d(folder: &Path) {
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .expect("photo folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        })
        .collect();
    files.sort();
    let codec = default_codec();
    let mut originals = Vec::new();
    for path in &files {
        let bytes = std::fs::read(path).expect("read photo");
        let decoded = decode_for_screen(codec.as_ref(), &bytes, 1600).expect("decode photo");
        originals.push(fit_long_edge(&decoded.image, 1600).expect("reduce"));
    }
    println!(
        "\nB. Visual signature from a T1 thumbnail ({} photos)",
        originals.len()
    );
    let mut sign_ms = Vec::new();
    let mut wide = Vec::new();
    let mut close = Vec::new();
    for image in &originals {
        let (sig, ms) = sign_from_t1(&to_t1(image));
        sign_ms.push(ms);
        wide.push(sig);
        // The "camera" view: 1.3–2× closer, 3:2 instead of 4:3, a little off centre, darker.
        let zoom = 1.3 + (wide.len() % 4) as f32 * 0.25;
        let view = darker(crop(image, zoom, 1.5, 0.03), 0.8);
        close.push(sign_from_t1(&to_t1(&view)).0);
    }
    println!(
        "  decode T1 + 256 px + sign: median {:.2} ms, max {:.2} ms per photo",
        median(sign_ms.clone()),
        sign_ms.iter().copied().fold(0.0, f64::max)
    );

    println!(
        "\nC. Same scene, closer and darker (positives) vs photos ≥ 5 frames away (negatives)"
    );
    let (mut pos_s, mut pos_c, mut neg_s, mut neg_c, mut cmp_ms) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..wide.len() {
        let (s, c, ms) = score(&wide[i], &close[i]);
        pos_s.push(s);
        pos_c.push(c);
        cmp_ms.push(ms);
        for j in (i + 5..wide.len()).step_by(7) {
            let (s, c, _) = score(&wide[i], &close[j]);
            neg_s.push(s);
            neg_c.push(c);
        }
    }
    println!(
        "  positives ({}): structure p5 {:.2} / median {:.2}, colour p5 {:.2} / median {:.2}",
        pos_s.len(),
        percentile(&pos_s, 0.05),
        percentile(&pos_s, 0.5),
        percentile(&pos_c, 0.05),
        percentile(&pos_c, 0.5)
    );
    println!(
        "  negatives ({}): structure median {:.2} / p95 {:.2} / max {:.2}, colour median {:.2} / p95 {:.2}",
        neg_s.len(),
        percentile(&neg_s, 0.5),
        percentile(&neg_s, 0.95),
        percentile(&neg_s, 1.0),
        percentile(&neg_c, 0.5),
        percentile(&neg_c, 0.95)
    );
    for (s_min, c_min) in [(0.5, 0.6), (0.6, 0.65), (0.6, 0.72), (0.7, 0.72)] {
        let tp = pos_s
            .iter()
            .zip(&pos_c)
            .filter(|&(&s, &c)| s >= s_min && c >= c_min)
            .count();
        let fp = neg_s
            .iter()
            .zip(&neg_c)
            .filter(|&(&s, &c)| s >= s_min && c >= c_min)
            .count();
        println!(
            "  threshold structure ≥ {s_min}, colour ≥ {c_min}: found {tp}/{} positives, {fp}/{} negatives pass",
            pos_s.len(),
            neg_s.len()
        );
    }
    println!(
        "  one comparison over {} zooms: median {:.3} ms",
        ZOOMS.len(),
        median(cmp_ms)
    );

    // The real pass: for each camera photo, its 8 nearest photos in time from another device
    // (indices stand in for time here), a cheap colour check first, the layout only for those
    // that pass, all photos in parallel.
    let mut color_ms = Vec::new();
    for i in 0..wide.len() {
        let t = Instant::now();
        std::hint::black_box(similar::color(&wide[i], &close[(i + 3) % close.len()]));
        color_ms.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let pass = |i: usize| -> usize {
        let lo = i.saturating_sub(4);
        let hi = (i + 4).min(wide.len() - 1);
        (lo..=hi)
            .filter(|&j| similar::color(&close[i], &wide[j]) >= 0.65)
            .filter(|&j| {
                ZOOMS
                    .iter()
                    .map(|&z| similar::structure_zoomed(&wide[j], &close[i], z))
                    .fold(-1f32, f32::max)
                    >= 0.6
            })
            .count()
    };
    let t = Instant::now();
    let single: usize = (0..wide.len()).map(pass).sum();
    let one_thread = t.elapsed().as_secs_f64() * 1e3;
    let t = Instant::now();
    let parallel: usize = {
        use rayon::prelude::*;
        (0..wide.len()).into_par_iter().map(pass).sum()
    };
    let all_threads = t.elapsed().as_secs_f64() * 1e3;
    assert_eq!(single, parallel);
    println!(
        "  colour check alone: median {:.4} ms · matching pass for {} camera photos, 8 nearest each, colour first: {:.0} ms on one thread, {:.0} ms on all cores ({:.2} ms per photo)",
        median(color_ms),
        wide.len(),
        one_thread,
        all_threads,
        all_threads / wide.len() as f64
    );

    println!(
        "\nD. Looking for .xmp sidecars on import ({} photos)",
        files.len()
    );
    let mut look = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        let mut found = 0;
        for p in &files {
            let a = p.with_extension("xmp");
            let mut b = p.clone().into_os_string();
            b.push(".xmp");
            found += usize::from(a.exists()) + usize::from(Path::new(&b).exists());
        }
        std::hint::black_box(found);
        look.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let per = median(look.clone()) / files.len() as f64;
    println!(
        "  two existence checks per photo: {:.2} ms for {} photos ({:.1} µs each, {:.0} ms per 1,000)",
        median(look),
        files.len(),
        per * 1e3,
        per * 1e3
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let folder = args.first().map(PathBuf::from);
    let sizes: Vec<usize> = args.iter().skip(1).filter_map(|a| a.parse().ok()).collect();
    let sizes = if sizes.is_empty() {
        vec![5_035, 20_000, 70_000]
    } else {
        sizes
    };
    part_a(&sizes);
    match folder {
        Some(f) => parts_b_c_d(&f),
        None => println!("\n(B–D need a folder of JPEG photos as the first argument)"),
    }
}
