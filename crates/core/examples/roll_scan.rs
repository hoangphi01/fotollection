// SPDX-License-Identifier: GPL-3.0-only
//! FR-141 S0, measured first: a phone's camera roll read as Fotollection would sort it. Every
//! file of a folder (videos too, `@eaDir` and hidden files skipped) is read once through the
//! app's own `metadata::read_path` (timed), and its first 128 KB are read again for the fields
//! the sorting will need but the app does not read yet: `Software`, `UserComment`,
//! `ImageUniqueID`, the sub-seconds and offsets, the pixel size, whether Apple's `MakerNote` is
//! there, a PNG's size and text chunks, a video's brand and Apple's content identifier. One
//! JSON line per file goes to `--out`, the timings to stdout. Files are only read.
//!
//! `cargo run --release -p fotollection-core --example roll_scan -- <folder> --out roll.jsonl [--threads N]`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use exif::{In, Reader, Tag, Value};
use fotollection_core::metadata;
use rayon::prelude::*;
use serde_json::{Map, Value as Json, json};

const HEAD: u64 = 128 * 1024;
const APPLE_CONTENT_ID: &[u8] = b"com.apple.quicktime.content.identifier";

fn list(folder: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(folder)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            e.depth() == 0 || (name != "@eaDir" && !name.starts_with('.'))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .collect()
}

fn head(path: &Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = File::open(path).and_then(|f| f.take(HEAD).read_to_end(&mut bytes));
    bytes
}

fn text(s: &str) -> Json {
    let t = s
        .trim_matches(|c: char| c == '"' || c.is_whitespace() || c == '\0')
        .to_string();
    if t.is_empty() {
        Json::Null
    } else {
        Json::String(t.chars().take(120).collect())
    }
}

/// The EXIF fields the sorting will read, from the file's head.
fn exif_fields(bytes: &[u8], out: &mut Map<String, Json>) {
    let Ok(exif) = Reader::new().read_from_container(&mut std::io::Cursor::new(bytes)) else {
        out.insert("exif".into(), json!(false));
        return;
    };
    out.insert("exif".into(), json!(true));
    let get = |tag| exif.get_field(tag, In::PRIMARY);
    for (key, tag) in [
        ("make", Tag::Make),
        ("model", Tag::Model),
        ("software", Tag::Software),
        ("lens", Tag::LensModel),
        ("date", Tag::DateTimeOriginal),
        ("date_digitized", Tag::DateTimeDigitized),
        ("date_modified", Tag::DateTime),
        ("offset", Tag::OffsetTimeOriginal),
        ("subsec", Tag::SubSecTimeOriginal),
        ("unique_id", Tag::ImageUniqueID),
    ] {
        if let Some(f) = get(tag) {
            out.insert(key.into(), text(&f.display_value().to_string()));
        }
    }
    if let Some(f) = get(Tag::UserComment) {
        let v = match &f.value {
            Value::Undefined(b, _) if b.len() > 8 => String::from_utf8_lossy(&b[8..]).into_owned(),
            _ => f.display_value().to_string(),
        };
        out.insert("user_comment".into(), text(&v));
    }
    for (key, tag) in [
        ("width", Tag::PixelXDimension),
        ("height", Tag::PixelYDimension),
    ] {
        if let Some(n) = get(tag).and_then(|f| f.value.get_uint(0)) {
            out.insert(key.into(), json!(n));
        }
    }
    if let Some(Value::Undefined(b, _)) = get(Tag::MakerNote).map(|f| &f.value) {
        out.insert("makernote_len".into(), json!(b.len()));
        out.insert("makernote_apple".into(), json!(b.starts_with(b"Apple iOS")));
    }
}

/// A PNG's size and the keywords of its text chunks, from its head.
fn png_fields(bytes: &[u8], out: &mut Map<String, Json>) {
    if bytes.len() < 33 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return;
    }
    let be = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    out.insert("width".into(), json!(be(&bytes[16..20])));
    out.insert("height".into(), json!(be(&bytes[20..24])));
    let mut at = 8;
    let mut chunks = Vec::new();
    while at + 8 <= bytes.len() && chunks.len() < 32 {
        let len = be(&bytes[at..at + 4]) as usize;
        let kind = String::from_utf8_lossy(&bytes[at + 4..at + 8]).into_owned();
        if matches!(kind.as_str(), "tEXt" | "iTXt" | "zTXt") {
            let body = &bytes[at + 8..(at + 8 + len).min(bytes.len())];
            let key = body.split(|&c| c == 0).next().unwrap_or_default();
            chunks.push(format!("{kind}:{}", String::from_utf8_lossy(key)));
        } else if kind != "IDAT" {
            chunks.push(kind.clone());
        }
        if kind == "IDAT" || kind == "IEND" {
            break;
        }
        at += 12 + len;
    }
    out.insert("png_chunks".into(), json!(chunks));
}

/// A video's brand and whether Apple's content identifier is in its head.
fn video_fields(bytes: &[u8], out: &mut Map<String, Json>) {
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        out.insert(
            "brand".into(),
            json!(String::from_utf8_lossy(&bytes[8..12]).trim().to_string()),
        );
    }
    let found = bytes
        .windows(APPLE_CONTENT_ID.len())
        .any(|w| w == APPLE_CONTENT_ID);
    out.insert("apple_content_id_in_head".into(), json!(found));
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let folder = PathBuf::from(
        args.first()
            .expect("usage: roll_scan <folder> --out <file> [--threads N]"),
    );
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };
    let out_path = PathBuf::from(flag("--out").expect("--out <file>"));
    let threads = flag("--threads").map_or_else(num_cpus, |n| n.parse().expect("--threads N"));
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("pool");

    let t = Instant::now();
    let files = list(&folder);
    let list_ms = t.elapsed().as_secs_f64() * 1000.0;

    let t = Instant::now();
    let rows: Vec<(Json, f64, f64)> = files
        .par_iter()
        .map(|path| {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let is_video = matches!(ext.as_str(), "mov" | "mp4" | "m4v");
            let a = Instant::now();
            let meta = (!is_video).then(|| metadata::read_path(path));
            let app_ms = a.elapsed().as_secs_f64() * 1000.0;
            let b = Instant::now();
            let bytes = head(path);
            let mut row = Map::new();
            row.insert(
                "name".into(),
                json!(path.file_name().map(|n| n.to_string_lossy().into_owned())),
            );
            row.insert("ext".into(), json!(ext));
            if let Ok(md) = std::fs::metadata(path) {
                row.insert("bytes".into(), json!(md.len()));
                if let Ok(m) = md.modified().and_then(|m| {
                    m.duration_since(std::time::UNIX_EPOCH)
                        .map_err(std::io::Error::other)
                }) {
                    row.insert("mtime".into(), json!(m.as_secs()));
                }
            }
            if let Some(meta) = meta {
                row.insert(
                    "app_capture".into(),
                    json!(meta.capture.map(|c| c.to_string())),
                );
                row.insert("app_gps".into(), json!(meta.gps.is_some()));
            }
            if is_video {
                video_fields(&bytes, &mut row);
            } else {
                exif_fields(&bytes, &mut row);
                if ext == "png" {
                    png_fields(&bytes, &mut row);
                }
            }
            let extra_ms = b.elapsed().as_secs_f64() * 1000.0;
            (Json::Object(row), app_ms, extra_ms)
        })
        .collect();
    let wall_ms = t.elapsed().as_secs_f64() * 1000.0;

    let mut out = BufWriter::new(File::create(&out_path).expect("create --out"));
    for (row, _, _) in &rows {
        writeln!(out, "{row}").expect("write");
    }
    let mut app: Vec<f64> = rows
        .iter()
        .filter(|r| r.0.get("app_capture").is_some())
        .map(|r| r.1)
        .collect();
    let mut extra: Vec<f64> = rows.iter().map(|r| r.2).collect();
    app.sort_by(f64::total_cmp);
    extra.sort_by(f64::total_cmp);
    println!(
        "{}",
        json!({
            "files": files.len(),
            "threads": threads,
            "list_ms": list_ms.round(),
            "read_wall_ms": wall_ms.round(),
            "s_per_1000": wall_ms / files.len().max(1) as f64,
            "app_read_path_ms": { "p50": percentile(&app, 0.5), "p95": percentile(&app, 0.95), "max": percentile(&app, 1.0) },
            "extra_head_ms": { "p50": percentile(&extra, 0.5), "p95": percentile(&extra, 0.95), "max": percentile(&extra, 1.0) },
        })
    );
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
}
