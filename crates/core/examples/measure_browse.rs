// SPDX-License-Identifier: GPL-3.0-only
//! What browsing a folder in the preview costs (FR-091, NFR-012), JPEG and RAW apart, p50 and
//! p95: each photo opened from nothing (a click on a photo far away, before FR-091), without
//! and with the disk cache (which then also writes its standard preview); its
//! standard preview made on disk, as the app does after the thumbnails; each photo opened
//! from that disk copy in a new session; and opened again from memory. Also the disk bytes
//! a photo and the mean difference of a disk-made preview from a fresh one. Files are only
//! read; the standard previews go to a temporary folder.
//!
//! `cargo run --release -p fotollection-core --example measure_browse -- <folder or files>…`
//!
//! `LIMIT=n` takes the first n photos. Peak RSS comes from the caller (`/usr/bin/time -l`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use std::path::PathBuf;
use std::time::Instant;

use fotollection_core::codec::default_codec;
use fotollection_core::import::check_file;
use fotollection_core::library::{Library, PhotoRecord};
use fotollection_core::metadata;
use fotollection_core::preview::{Budget, PreviewCache};
use fotollection_core::raw::is_raw_name;
use fotollection_core::settings::RenderSettings;

fn files(args: impl Iterator<Item = String>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for arg in args {
        let path = PathBuf::from(arg);
        if path.is_dir() {
            let mut inside: Vec<_> = std::fs::read_dir(&path)
                .unwrap()
                .filter_map(|e| Some(e.ok()?.path()))
                .filter(|p| p.is_file())
                .collect();
            inside.sort();
            out.extend(inside);
        } else {
            out.push(path);
        }
    }
    out
}

fn records(paths: &[PathBuf]) -> Vec<PhotoRecord> {
    let mut lib = Library::new();
    let mut out = Vec::new();
    for path in paths {
        let Ok(file) = check_file(path) else { continue };
        let meta = metadata::read_path(&file.path);
        let folder = lib.add_folder(file.path.parent().unwrap());
        if let Some(id) = lib.push(folder, &file, &meta) {
            out.push(lib.record(id).unwrap());
        }
    }
    out
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn report(label: &str, raw: &mut [f64], other: &mut [f64]) {
    println!(
        "| {label} | {} JPEG: p50 {:.0} ms, p95 {:.0} ms | {} RAW: p50 {:.0} ms, p95 {:.0} ms |",
        other.len(),
        percentile(other, 0.5),
        percentile(other, 0.95),
        raw.len(),
        percentile(raw, 0.5),
        percentile(raw, 0.95),
    );
}

fn main() {
    let mut paths = files(std::env::args().skip(1));
    if let Some(n) = std::env::var("LIMIT").ok().and_then(|v| v.parse().ok()) {
        paths.truncate(n);
    }
    let records = records(&paths);
    let codec = default_codec();
    let settings = RenderSettings::default();

    let timed = |cache: &PreviewCache, clear: bool| {
        let (mut raw, mut other) = (Vec::new(), Vec::new());
        for record in &records {
            if clear {
                cache.clear();
            }
            // The preview as the window gets it: rendered and encoded.
            let t = Instant::now();
            let ok = cache
                .render_jpeg(record, &settings, None, None, codec.as_ref())
                .is_ok();
            let ms = t.elapsed().as_secs_f64() * 1e3;
            if ok {
                if is_raw_name(&record.file_name) {
                    raw.push(ms);
                } else {
                    other.push(ms);
                }
            }
        }
        (raw, other)
    };
    println!("| Step | JPEG | RAW |");
    println!("|---|---|---|");

    // Before FR-091: a click on a photo far away, nothing of it kept.
    let (mut raw, mut other) = timed(&PreviewCache::new(), true);
    report("Open a photo, nothing kept", &mut raw, &mut other);

    // The same first view with the disk cache on, as in the app: it also makes the photo's
    // standard preview on disk (audit 2026-11-06, item 8).
    let first = tempfile::tempdir().unwrap();
    let (mut raw, mut other) = timed(
        &PreviewCache::with(Budget::default(), Some(first.path().to_path_buf())),
        true,
    );
    report(
        "Open a photo, nothing kept, disk cache on",
        &mut raw,
        &mut other,
    );
    drop(first);

    // The background builder: each photo's standard preview made on disk.
    let disk = tempfile::tempdir().unwrap();
    let builder = PreviewCache::with(Budget::default(), Some(disk.path().to_path_buf()));
    let (mut raw, mut other) = (Vec::new(), Vec::new());
    for record in &records {
        let t = Instant::now();
        let made = builder
            .ensure_standard(record, &settings, codec.as_ref(), &|| true)
            .is_ok();
        let ms = t.elapsed().as_secs_f64() * 1e3;
        if made {
            if is_raw_name(&record.file_name) {
                raw.push(ms);
            } else {
                other.push(ms);
            }
        }
    }
    report("Make its standard preview on disk", &mut raw, &mut other);

    // A new session: memory empty, the standard previews on disk.
    let session = PreviewCache::with(
        Budget::of(4 * 1024 * 1024 * 1024),
        Some(disk.path().to_path_buf()),
    );
    let (mut raw, mut other) = timed(&session, true);
    report(
        "Open a photo from its standard preview",
        &mut raw,
        &mut other,
    );
    let (mut raw, mut other) = timed(&session, false);
    let _ = timed(&session, false);
    let (mut raw2, mut other2) = timed(&session, false);
    raw.append(&mut raw2);
    other.append(&mut other2);
    report("Open it again, kept in memory", &mut raw, &mut other);

    let bytes: u64 = walk(disk.path());
    // The pixels: a fresh preview against one made from the disk copy, on 20 photos.
    let (mut sum, mut n) = (0f64, 0f64);
    for record in records.iter().take(20) {
        session.clear();
        let a = PreviewCache::new().render_rgb(record, &settings, None, None, codec.as_ref());
        let b = session.render_rgb(record, &settings, None, None, codec.as_ref());
        if let (Ok(a), Ok(b)) = (a, b)
            && a.data.len() == b.data.len()
        {
            sum += a
                .data
                .iter()
                .zip(&b.data)
                .map(|(x, y)| f64::from(x.abs_diff(*y)))
                .sum::<f64>()
                / a.data.len() as f64;
            n += 1.0;
        }
    }
    println!(
        "\nStandard previews: {:.2} MB a photo on disk; mean difference from a fresh preview {:.2} levels of 255 ({} photos).",
        bytes as f64 / records.len().max(1) as f64 / 1e6,
        sum / n.max(1.0),
        n
    );
}

fn walk(dir: &std::path::Path) -> u64 {
    std::fs::read_dir(dir).map_or(0, |entries| {
        entries
            .filter_map(Result::ok)
            .map(|e| {
                let path = e.path();
                if path.is_dir() {
                    walk(&path)
                } else {
                    e.metadata().map_or(0, |m| m.len())
                }
            })
            .sum()
    })
}
