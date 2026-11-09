// SPDX-License-Identifier: GPL-3.0-only
//! How close a Lightroom preset read by Fotollection comes to Lightroom's own render (FR-089,
//! RESULT §39), measured before any constant is tuned. In `<folder>` (the owner's private
//! reference material, never committed): `PRESET/*.xmp`, `RAW/*.CR2`, `RAW_JPG/*.jpg`
//! (Lightroom's export of each RAW with no preset, Adobe Color) and `EDITED/*.jpg` (the same
//! with one preset only).
//!
//! Track A, the preset's translation alone: the compiled look applied to Lightroom's own
//! plain export, against its export with the preset; the baseline is the plain export as it
//! is. Track B, the app's whole path: our develop of the RAW with the look, against the same.
//! ΔE2000 per pixel at 1000 px: mean, p95, shadows / mid-tones / highlights. Only numbers
//! and generic labels are printed.
//!
//! `cargo run --release -p fotollection-core --example preset_bench -- <folder> [--sweep]`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::print_stdout
)]

use std::path::{Path, PathBuf};

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::image::RgbImage;
use fotollection_core::looks::LookSettings;
use fotollection_core::preset;
use fotollection_core::preset_look;
use fotollection_core::raw_finish::{DeStats, de2000_stats};
use fotollection_core::render::fit_long_edge;

const EDGE: u32 = 1000;

fn files(dir: &Path, ext: &[&str]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    out.retain(|p| {
        p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| ext.iter().any(|x| x.eq_ignore_ascii_case(e)))
    });
    out.sort();
    out
}

fn stem(p: &Path) -> String {
    p.file_stem().unwrap().to_string_lossy().into_owned()
}

fn jpeg(path: &Path) -> RgbImage {
    let codec = default_codec();
    let image = decode_image(codec.as_ref(), &std::fs::read(path).unwrap())
        .unwrap()
        .image;
    fit_long_edge(&image, EDGE).unwrap()
}

/// The look a preset compiles to, its LUT kept in memory as the app keeps it.
fn look_of(compiled: &preset_look::Compiled) -> LookSettings {
    let mut look = compiled.look.clone();
    if let Some(cube) = &compiled.lut {
        let (hash, _) = fotollection_core::lut::import(cube.as_bytes()).unwrap();
        look.lut = Some(hash);
    }
    look
}

fn through(image: &RgbImage, look: &LookSettings) -> RgbImage {
    // As the app draws a look: its dehaze, then its table.
    let mut out = image.clone();
    look.draw(&mut out);
    out
}

fn row(label: &str, s: DeStats) {
    println!(
        "{label:<34} {:>6.2} {:>6.2}   {:>6.2} {:>6.2} {:>6.2}",
        s.mean, s.p95, s.zones[0], s.zones[1], s.zones[2]
    );
}

fn preset_number(p: &preset::Preset, key: &str) -> f32 {
    match p.values.get(key) {
        Some(preset::Value::Number(v)) => *v,
        _ => 0.0,
    }
}

fn arg(args: &[String], flag: &str) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

/// The look with one group of the preset left out, to see what each group costs or gains.
fn without(look: &LookSettings, group: &str) -> LookSettings {
    let mut l = look.clone();
    let a = &mut l.adjust;
    match group {
        "exposure" => a.brightness = 0,
        "contrast" => a.contrast = 0,
        "highlights" => a.highlights = 0,
        "shadows" => a.shadows = 0,
        "white balance" => (a.warmth, a.tint) = (0, 0),
        "saturation" => (a.saturation, a.vibrance) = (0, 0),
        "mixer" => a.mixer = fotollection_core::colour_ops::Mixer::default(),
        "table (curve, grading…)" => l.lut = None,
        "dehaze" => a.dehaze = 0,
        _ => {}
    }
    l
}

const GROUPS: [&str; 9] = [
    "exposure",
    "contrast",
    "highlights",
    "shadows",
    "white balance",
    "saturation",
    "mixer",
    "table (curve, grading…)",
    "dehaze",
];

/// One photo: `--preset <xmp> --plain <jpg> --edited <jpg> [--raw <file>]`.
fn one(args: &[String]) {
    let p = preset::parse(&std::fs::read(arg(args, "--preset").unwrap()).unwrap()).unwrap();
    let compiled = preset_look::compile(&p);
    println!(
        "P1: {} settings, not applied: {:?}",
        p.values.len(),
        compiled.unapplied
    );
    let a = compiled.look.adjust;
    println!(
        "compiled: brightness {} contrast {} highlights {} shadows {} warmth {} tint {} saturation {} vibrance {} table {}",
        a.brightness,
        a.contrast,
        a.highlights,
        a.shadows,
        a.warmth,
        a.tint,
        a.saturation,
        a.vibrance,
        compiled.lut.is_some()
    );
    let look = look_of(&compiled);
    let plain = jpeg(&arg(args, "--plain").unwrap());
    let target = jpeg(&arg(args, "--edited").unwrap());
    assert_eq!(
        (plain.width, plain.height),
        (target.width, target.height),
        "the two exports are the same size"
    );
    assert!(
        de2000_stats(&plain, &plain).unwrap().mean < 1e-3,
        "a picture against itself is 0"
    );

    println!("\nTrack A: the look on Lightroom's plain export, against its preset render");
    println!(
        "{:<34} {:>6} {:>6}   {:>6} {:>6} {:>6}",
        "", "mean", "p95", "dark", "mid", "light"
    );
    row("no look (baseline)", de2000_stats(&plain, &target).unwrap());
    let ours = de2000_stats(&through(&plain, &look), &target).unwrap();
    row("our look", ours);
    println!("\neach group left out (lower than `our look` = that group hurts)");
    for g in GROUPS {
        row(
            &format!("without {g}"),
            de2000_stats(&through(&plain, &without(&look, g)), &target).unwrap(),
        );
    }
    sweeps(args, &p, &look, &plain, &target);
    if let Some(raw_path) = arg(args, "--raw") {
        track_b(&raw_path, &look, &plain, &target);
    }
}

/// Each slider of the white balance and the highlights swept, and the balance against the
/// photo's own as shot when `--as-shot <K>,<tint>` gives it.
fn sweeps(
    args: &[String],
    p: &preset::Preset,
    look: &LookSettings,
    plain: &RgbImage,
    target: &RgbImage,
) {
    println!("\nwarmth swept (tint as compiled)");
    for w in (-30..=30).step_by(5) {
        let mut l = look.clone();
        l.adjust.warmth = w;
        row(
            &format!("warmth {w:+}"),
            de2000_stats(&through(plain, &l), target).unwrap(),
        );
    }
    println!("\ntint swept (warmth as compiled)");
    for t in (-30..=30).step_by(5) {
        let mut l = look.clone();
        l.adjust.tint = t;
        row(
            &format!("tint {t:+}"),
            de2000_stats(&through(plain, &l), target).unwrap(),
        );
    }

    println!(
        "\nhighlights swept (the preset says {})",
        look.adjust.highlights
    );
    for h in (-60..=0).step_by(10) {
        let mut l = look.clone();
        l.adjust.highlights = h;
        row(
            &format!("highlights {h:+}"),
            de2000_stats(&through(plain, &l), target).unwrap(),
        );
    }
    // The balance against the photo's own as shot (Lightroom writes it in the plain export's
    // XMP): warmth 1 per 10 K above it, tint as Lightroom's 150 to our 100.
    if let Some(shot) = args
        .iter()
        .position(|a| a == "--as-shot")
        .and_then(|i| args.get(i + 1))
    {
        let (k, t) = shot.split_once(',').unwrap();
        let (k, t): (f32, f32) = (k.parse().unwrap(), t.parse().unwrap());
        let dk = preset_number(p, "Temperature") - k;
        let dt = (preset_number(p, "Tint") - t) * 100.0 / 150.0;
        println!("\nwhite balance against as shot: {dk:+} K, tint {dt:+.1}");
        for per in [5.0f32, 10.0, 15.0, 20.0, 30.0] {
            let mut l = look.clone();
            l.adjust.warmth = (dk / per).round().clamp(-100.0, 100.0) as i8;
            l.adjust.tint = dt.round() as i8;
            let mut both = l.clone();
            both.adjust.highlights = 0;
            row(
                &format!("1 per {per} K (warmth {:+})", l.adjust.warmth),
                de2000_stats(&through(plain, &l), target).unwrap(),
            );
            row(
                "  and no highlights",
                de2000_stats(&through(plain, &both), target).unwrap(),
            );
        }
    }
}

/// Track B: our develop of the RAW, against Lightroom's plain export, then with the look.
fn track_b(raw_path: &Path, look: &LookSettings, plain: &RgbImage, target: &RgbImage) {
    let bytes = std::fs::read(raw_path).unwrap();
    let kind = fotollection_core::raw::kind(&bytes).unwrap();
    let developed = fotollection_core::raw::decode(
        &bytes,
        kind,
        fotollection_core::raw::RawLook::Camera,
        &fotollection_core::raw::RawAdjust::default(),
    )
    .unwrap();
    let ours_dev = fit_long_edge(&developed.image, EDGE).unwrap();
    println!(
        "\nTrack B: our develop of the RAW ({}×{}, profile {})",
        ours_dev.width,
        ours_dev.height,
        developed.icc.is_some()
    );
    // The RAW's develop is not turned yet; the export is: try both quarter turns.
    let ours_dev = [6u8, 8]
        .into_iter()
        .map(|o| fotollection_core::image::apply_orientation(ours_dev.clone(), o))
        .chain(std::iter::once(ours_dev.clone()))
        .filter(|i| (i.width, i.height) == (plain.width, plain.height))
        .min_by(|a, b| {
            let da = de2000_stats(a, plain).unwrap().mean;
            let db = de2000_stats(b, plain).unwrap().mean;
            da.total_cmp(&db)
        })
        .unwrap_or(ours_dev);
    if (ours_dev.width, ours_dev.height) == (plain.width, plain.height) {
        row(
            "our develop vs plain export",
            de2000_stats(&ours_dev, plain).unwrap(),
        );
        row(
            "our develop + look vs preset",
            de2000_stats(&through(&ours_dev, look), target).unwrap(),
        );
    } else {
        println!(
            "sizes differ from the export: {:?} vs {:?}",
            (ours_dev.width, ours_dev.height),
            (plain.width, plain.height)
        );
    }
}

fn main() {
    let all: Vec<String> = std::env::args().skip(1).collect();
    if all.iter().any(|a| a == "--preset") {
        one(&all);
        return;
    }
    let mut args = all.into_iter();
    let folder = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "Material Fotollection".into()),
    );
    let presets: Vec<(String, preset::Preset)> =
        files(&folder.join("PRESET"), &["xmp", "lrtemplate"])
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    format!("P{}", i + 1),
                    preset::parse(&std::fs::read(p).unwrap()).unwrap(),
                )
            })
            .collect();
    for (label, p) in &presets {
        let compiled = preset_look::compile(p);
        println!(
            "{label}: {} settings, not applied: {:?}",
            p.values.len(),
            compiled.unapplied
        );
    }
    let looks: Vec<(String, LookSettings)> = presets
        .iter()
        .map(|(l, p)| (l.clone(), look_of(&preset_look::compile(p))))
        .collect();

    println!(
        "\nTrack A: the preset on Lightroom's plain export, against Lightroom's preset render"
    );
    println!(
        "{:<34} {:>6} {:>6}   {:>6} {:>6} {:>6}",
        "", "mean", "p95", "dark", "mid", "light"
    );
    let mut pairs = Vec::new();
    for edited in files(&folder.join("EDITED"), &["jpg", "jpeg"]) {
        let name = stem(&edited);
        let plain = jpeg(&folder.join("RAW_JPG").join(format!("{name}.jpg")));
        let target = jpeg(&edited);
        let same = de2000_stats(&plain, &plain).unwrap();
        assert!(same.mean < 1e-3, "a picture against itself is 0");
        row(
            &format!("{name} no look (baseline)"),
            de2000_stats(&plain, &target).unwrap(),
        );
        let mut best: Option<(String, f32)> = None;
        for (label, look) in &looks {
            let s = de2000_stats(&through(&plain, look), &target).unwrap();
            row(&format!("{name} + {label}"), s);
            if best.as_ref().is_none_or(|b| s.mean < b.1) {
                best = Some((label.clone(), s.mean));
            }
        }
        let (label, _) = best.unwrap();
        println!("{name} pairs with {label}");
        let look = &looks.iter().find(|(l, _)| *l == label).unwrap().1;
        for scale in [1.0f32, 0.75, 0.5, 0.25, 0.0] {
            let mut l = look.clone();
            l.adjust.highlights = (f32::from(look.adjust.highlights) * scale).round() as i8;
            row(
                &format!("  highlights × {scale}"),
                de2000_stats(&through(&plain, &l), &target).unwrap(),
            );
        }
        pairs.push((name, label));
    }
}
