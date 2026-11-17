// SPDX-License-Identifier: GPL-3.0-only
//! FR-141, measured on a real roll: every file of a folder (`@eaDir` and hidden files skipped)
//! read through the app's own `metadata::read_path` and `sorting::classify`, videos through
//! `quicktime::read_path`, each file timed. One JSON line per file goes to `--out`; the summary
//! (timings, classes, Live Photo pairs found) to stdout. With `--labels <csv>` (columns
//! `name,guess,label`, kept out of the repository), each class is scored against the labels.
//! Files are only read.
//!
//! `cargo run --release -p fotollection-core --example roll_scan -- <folder> --out roll.jsonl [--threads N] [--labels labels.csv]`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::sorting::stacks::{self, StackKey};
use fotollection_core::sorting::{self, Source, SourceClass};
use fotollection_core::{metadata, quicktime};
use rayon::prelude::*;
use serde_json::json;

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

struct Row {
    name: String,
    ext: String,
    source: Option<Source>,
    model: Option<String>,
    content_id: Option<String>,
    video_content_id: Option<String>,
    stack_key: Option<StackKey>,
    ms: f64,
}

/// The label a person gives each class, as on the labelling page.
fn label_of(source: &Source) -> &'static str {
    match source.class {
        SourceClass::Camera | SourceClass::AnotherPhone => "camera-iphone",
        SourceClass::Panorama => "camera-panorama",
        SourceClass::Screenshot | SourceClass::ScreenshotCropped => "screenshot",
        SourceClass::App if source.app == Some("Facebook") => "app-facebook-camera",
        SourceClass::App => "app-instagram",
        SourceClass::Edited => "edited",
        SourceClass::FilmScan => "film-scan",
        SourceClass::OtherCamera => "other-camera",
        SourceClass::Unknown => "unknown",
    }
}

/// `name,guess,label` rows; the label is the last column, the name the first.
fn labels(path: &Path) -> HashMap<String, String> {
    let text = std::fs::read_to_string(path).expect("--labels file");
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let mut parts = l.rsplitn(3, ',');
            let label = parts.next()?.trim().to_string();
            let _guess = parts.next()?;
            let name = parts.next()?.trim_matches('"').to_string();
            (!label.is_empty()).then_some((name, label))
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let folder = PathBuf::from(
        args.first()
            .expect("usage: roll_scan <folder> --out <file>"),
    );
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };
    let out_path = PathBuf::from(flag("--out").expect("--out <file>"));
    let threads = flag("--threads").map(|n| n.parse::<usize>().expect("--threads N"));
    let pool = fotollection_core::pool::contained(threads, "roll-scan").expect("pool");

    let t = Instant::now();
    let files = list(&folder);
    let list_ms = t.elapsed().as_secs_f64() * 1000.0;

    let t = Instant::now();
    let rows: Vec<Row> = pool.install(|| files.par_iter().map(|path| read(path)).collect());
    let wall_ms = t.elapsed().as_secs_f64() * 1000.0;

    let mine = my_phones(&rows);
    let mut out = BufWriter::new(File::create(&out_path).expect("create --out"));
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    let mut sources = Vec::with_capacity(rows.len());
    for r in &rows {
        let source = r
            .source
            .clone()
            .map(|s| sorting::refine(s, r.model.as_deref(), &mine));
        let class = source.as_ref().map_or_else(
            || format!("video-{}", r.ext),
            |s| {
                let app = s.app.map(|a| format!("({a})")).unwrap_or_default();
                format!("{:?}{app}", s.class)
            },
        );
        *classes.entry(class.clone()).or_default() += 1;
        let line = json!({
            "name": r.name, "class": class, "rule": source.as_ref().map(|s| s.rule),
            "content_id": r.content_id, "video_content_id": r.video_content_id, "ms": r.ms,
            "photo_id": r.stack_key.as_ref().and_then(|k| k.photo_id.as_deref()), "burst_id": r.stack_key.as_ref().and_then(|k| k.burst_id.as_deref()), "capture_s": r.stack_key.as_ref().and_then(|k| k.capture_s),
        });
        writeln!(out, "{line}").expect("write");
        sources.push(source);
    }
    let mut ms: Vec<f64> = rows.iter().map(|r| r.ms).collect();
    ms.sort_by(f64::total_cmp);
    let p = |q: f64| {
        ms.get((ms.len().saturating_sub(1) as f64 * q).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    println!(
        "{}",
        json!({
            "files": rows.len(), "list_ms": list_ms.round(), "read_wall_ms": wall_ms.round(),
            "s_per_1000": wall_ms / rows.len().max(1) as f64,
            "per_file_ms": { "p50": p(0.5), "p95": p(0.95), "max": p(1.0) },
            "my_phones": mine, "classes": classes,
            "live_photo": live_photos(&rows),
        })
    );
    let truth = flag("--labels")
        .map(|p| labels(Path::new(p)))
        .unwrap_or_default();
    println!("{}", stack_report(&rows, &folder, &truth));
    score(&rows, &sources, &truth);
}

/// One file read as the app reads it, timed.
fn read(path: &Path) -> Row {
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let a = Instant::now();
    let mut row = Row {
        name,
        ext: ext.clone(),
        source: None,
        model: None,
        content_id: None,
        video_content_id: None,
        stack_key: None,
        ms: 0.0,
    };
    if matches!(ext.as_str(), "mov" | "mp4" | "m4v") {
        row.video_content_id = quicktime::read_path(path).and_then(|v| v.content_id);
    } else {
        let meta = metadata::read_path(path);
        row.source = Some(sorting::classify(&meta, &ext));
        row.model.clone_from(&meta.model);
        let capture = meta.capture.map(|c| c.and_utc().timestamp());
        let mut key = StackKey::new(0, &row.name, capture, meta.apple.as_ref());
        key.derived = row.source.as_ref().is_some_and(|s| s.class.is_derived());
        row.stack_key = Some(key);
        row.content_id = meta.apple.and_then(|n| n.content_id);
    }
    row.ms = a.elapsed().as_secs_f64() * 1000.0;
    row
}

/// The library's own phones, from its camera shots.
fn my_phones(rows: &[Row]) -> Vec<String> {
    sorting::my_phones(
        rows.iter()
            .filter(|r| {
                r.source
                    .as_ref()
                    .is_some_and(|s| s.class == SourceClass::Camera)
            })
            .filter_map(|r| r.model.as_deref()),
    )
}

/// The stacks of the stills (FR-141 S3), timed; the Live Photo videos found beside their
/// stills; the labelled twins left out of a stack.
fn stack_report(rows: &[Row], folder: &Path, truth: &HashMap<String, String>) -> serde_json::Value {
    let stills: Vec<&Row> = rows.iter().filter(|r| r.stack_key.is_some()).collect();
    let keys: Vec<StackKey> = stills.iter().filter_map(|r| r.stack_key.clone()).collect();
    let t = Instant::now();
    let found = stacks::stacks(&keys);
    let stacks_ms = t.elapsed().as_secs_f64() * 1000.0;
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut stacked = HashSet::new();
    for s in &found {
        *kinds.entry(format!("{:?}", s.kinds)).or_default() += 1;
        stacked.extend(s.members.iter().map(|&m| stills[m as usize].name.as_str()));
    }
    let live = stills
        .iter()
        .filter_map(|r| Some((r, r.content_id.as_deref()?)))
        .filter(|(r, id)| stacks::live_video(&folder.join(&r.name), id).is_some())
        .count();
    let twins_missed: Vec<&str> = truth
        .iter()
        .filter(|(n, l)| l.starts_with("heic-jpg-twin") && !stacked.contains(n.as_str()))
        .map(|(n, _)| n.as_str())
        .collect();
    json!({
        "stacks": found.len(), "photos_in_stacks": stacked.len(), "stacks_ms": stacks_ms,
        "by_kinds": kinds, "live_videos_found": live, "labelled_twins_not_stacked": twins_missed,
    })
}

/// Videos, those with Apple's content identifier, and those paired with a still by it.
fn live_photos(rows: &[Row]) -> serde_json::Value {
    let stills: HashSet<&str> = rows
        .iter()
        .filter_map(|r| r.content_id.as_deref())
        .collect();
    let paired = rows
        .iter()
        .filter_map(|r| r.video_content_id.as_deref())
        .filter(|id| stills.contains(id))
        .count();
    json!({
        "videos": rows.iter().filter(|r| r.source.is_none()).count(),
        "videos_with_content_id": rows.iter().filter(|r| r.video_content_id.is_some()).count(),
        "paired_with_a_still": paired,
    })
}

/// Each class against the labels: right of all, and what the wrong ones were.
fn score(rows: &[Row], sources: &[Option<Source>], truth: &HashMap<String, String>) {
    if truth.is_empty() {
        return;
    }
    let mut score: BTreeMap<&str, (usize, usize, BTreeMap<String, usize>)> = BTreeMap::new();
    for (r, source) in rows.iter().zip(sources) {
        let (Some(source), Some(label)) = (source, truth.get(&r.name)) else {
            continue;
        };
        // The two Instagram labels were merged (the effect field is not the app's camera);
        // what no metadata rule can tell apart is one class until the pixels (S6).
        let label = match label.as_str() {
            "app-instagram-camera" => "app-instagram",
            "meme" | "web-download" | "received-chat" | "apple-no-makernote" | "unsure" => {
                "unknown"
            }
            "video-mov" | "heic-jpg-twin?" => continue,
            l => l,
        };
        let guess = label_of(source);
        let e = score.entry(guess).or_default();
        e.1 += 1;
        if guess == label {
            e.0 += 1;
        } else {
            *e.2.entry(label.to_string()).or_default() += 1;
        }
    }
    for (class, (right, all, wrong)) in score {
        println!("{class:20} {right}/{all} right; wrong: {wrong:?}");
    }
}
