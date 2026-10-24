// SPDX-License-Identifier: GPL-3.0-only
//! Scores best-shot suggestions (FR-041) against a hand-labelled ground truth, for the
//! README and docs/RESULT.md. Photos are only read.
//!
//! `cargo run --release -p fotollection-core --example eval_bestshot -- <folder> <truth.json> [--json] [--pairs]`
//!
//! - Grouping: pairwise precision and recall over photo pairs (pairs marked `maybe` are
//!   ignored), and how many labelled groups were found (two or more members together).
//! - Picks: in labelled groups with a `best` list, whether the suggested photo is acceptable.
//! - `--pairs`: prints time gap and similarity of neighbouring photos, for tuning.
//! - `--explain`: prints the score parts of every photo in the groups found.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use fotollection_core::codec::{decode_image, default_codec};
use fotollection_core::culling::{self, PhotoAnalysis};
use fotollection_core::image::apply_orientation;
use fotollection_core::import::scan;
use fotollection_core::library::{Library, PhotoId};
use fotollection_core::metadata;
use rayon::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Truth {
    groups: Vec<TruthGroup>,
    maybe: Vec<[String; 2]>,
}

#[derive(Deserialize)]
struct TruthGroup {
    photos: Vec<String>,
    best: Option<Vec<String>>,
}

/// `IMG_0425.JPG` → `0425`
fn short(name: &str) -> String {
    name.trim_start_matches("IMG_")
        .split('.')
        .next()
        .unwrap_or(name)
        .to_owned()
}

fn pair(a: &str, b: &str) -> (String, String) {
    if a < b {
        (a.to_owned(), b.to_owned())
    } else {
        (b.to_owned(), a.to_owned())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let show_pairs = args.iter().any(|a| a == "--pairs");
    let explain = args.iter().any(|a| a == "--explain");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let root = PathBuf::from(
        positional
            .first()
            .expect("usage: eval_bestshot <folder> <truth.json>"),
    );
    let truth: Truth = serde_json::from_slice(
        &std::fs::read(
            positional
                .get(1)
                .expect("usage: eval_bestshot <folder> <truth.json>"),
        )
        .unwrap(),
    )
    .unwrap();

    let found = scan(&root);
    let mut lib = Library::new();
    let folder = lib.add_folder(&root);
    for file in &found.accepted {
        lib.push(folder, file, &metadata::read_path(&file.path));
    }
    let records: Vec<_> = lib
        .sorted_ids()
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect();
    let codec = default_codec();
    let analyses: HashMap<PhotoId, PhotoAnalysis> = records
        .par_iter()
        .map(|r| {
            let bytes = std::fs::read(&r.path).unwrap();
            let image = apply_orientation(
                decode_image(codec.as_ref(), &bytes).unwrap().image,
                r.orientation,
            );
            (r.id, culling::analyze(&image).unwrap())
        })
        .collect();
    let groups = culling::suggest(&records, &analyses);
    let name_of: HashMap<PhotoId, String> = records
        .iter()
        .map(|r| (r.id, short(&r.file_name)))
        .collect();

    if show_pairs {
        for w in records.windows(2) {
            let (a, b) = (&analyses[&w[0].id], &analyses[&w[1].id]);
            let gap = w[1].capture.and_utc().timestamp() - w[0].capture.and_utc().timestamp();
            let (structure, color) = culling::group::similarity(a, b);
            println!(
                "{} {} gap {gap:>5}s structure {structure:.3} color {color:.3}",
                name_of[&w[0].id], name_of[&w[1].id],
            );
        }
    }

    // Pairs the app put together, pairs the truth puts together, and pairs to ignore.
    let predicted: HashSet<(String, String)> = groups
        .iter()
        .flat_map(|g| {
            let names: Vec<_> = g
                .result
                .photos
                .iter()
                .map(|p| name_of[&p.id].clone())
                .collect();
            let mut pairs = Vec::new();
            for (i, a) in names.iter().enumerate() {
                for b in &names[i + 1..] {
                    pairs.push(pair(a, b));
                }
            }
            pairs
        })
        .collect();
    let mut expected = HashSet::new();
    for g in &truth.groups {
        for (i, a) in g.photos.iter().enumerate() {
            for b in &g.photos[i + 1..] {
                expected.insert(pair(a, b));
            }
        }
    }
    let ignored: HashSet<_> = truth.maybe.iter().map(|[a, b]| pair(a, b)).collect();
    let scored_predicted: HashSet<_> = predicted.difference(&ignored).cloned().collect();
    let true_positive = scored_predicted.intersection(&expected).count();
    let precision = if scored_predicted.is_empty() {
        1.0
    } else {
        true_positive as f64 / scored_predicted.len() as f64
    };
    let recall = true_positive as f64 / expected.len().max(1) as f64;
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    // A labelled group is found when the app puts two or more of its photos together.
    let group_of: HashMap<String, usize> = groups
        .iter()
        .enumerate()
        .flat_map(|(i, g)| g.result.photos.iter().map(move |p| (p.id, i)))
        .map(|(id, i)| (name_of[&id].clone(), i))
        .collect();
    let mut found_groups = 0;
    let (mut picks_scored, mut picks_right) = (0, 0);
    let mut lines = Vec::new();
    for g in &truth.groups {
        let mut counts: HashMap<usize, usize> = HashMap::new();
        for p in &g.photos {
            if let Some(&i) = group_of.get(p) {
                *counts.entry(i).or_default() += 1;
            }
        }
        let hit = counts
            .iter()
            .filter(|(_, n)| **n >= 2)
            .max_by_key(|(_, n)| **n)
            .map(|(i, _)| *i);
        let Some(i) = hit else {
            lines.push(format!("  missed  {}", g.photos.join(" ")));
            continue;
        };
        found_groups += 1;
        let suggested = &name_of[&groups[i].result.photos[0].id];
        if explain {
            for p in &groups[i].result.photos {
                let a = &analyses[&p.id];
                lines.push(format!(
                    "      {} score {:+.2} sharp {:+.2} steady {:+.2} exp {:+.2} hi {:+.2} noise {:+.2} eyes {:+.2} faces {} (subject {:.0}, face {:.0})",
                    name_of[&p.id],
                    p.score,
                    p.sharpness,
                    p.steadiness,
                    p.exposure,
                    p.highlights,
                    p.noise,
                    p.eyes,
                    p.faces,
                    a.metrics.subject_sharpness,
                    a.faces.iter().map(|f| f.sharpness).fold(0.0, f32::max),
                ));
            }
        }
        let verdict = match &g.best {
            Some(best) => {
                picks_scored += 1;
                if best.contains(suggested) {
                    picks_right += 1;
                    "right"
                } else {
                    "WRONG"
                }
            }
            None => "any",
        };
        lines.push(format!(
            "  found   {} → {suggested} ({verdict}{})",
            g.photos.join(" "),
            if groups[i].result.too_close {
                ", too close"
            } else {
                ""
            }
        ));
    }
    let extra: Vec<String> = groups
        .iter()
        .filter(|g| {
            let names: Vec<_> = g
                .result
                .photos
                .iter()
                .map(|p| name_of[&p.id].clone())
                .collect();
            !truth
                .groups
                .iter()
                .any(|t| names.iter().filter(|n| t.photos.contains(n)).count() >= 2)
        })
        .map(|g| {
            g.result
                .photos
                .iter()
                .map(|p| name_of[&p.id].clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let report = |line: String| {
        if json {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    };
    report(format!(
        "grouping: precision {:.0} %, recall {:.0} %, F1 {:.2}; labelled groups found {found_groups}/{}; app groups {} ({} not in the labels)",
        precision * 100.0,
        recall * 100.0,
        f1,
        truth.groups.len(),
        groups.len(),
        extra.len()
    ));
    report(format!("picks: {picks_right}/{picks_scored} acceptable"));
    for line in &lines {
        report(line.clone());
    }
    for e in &extra {
        report(format!("  extra   {e}"));
    }
    if json {
        let out = serde_json::json!({
            "photos": records.len(),
            "labelledGroups": truth.groups.len(),
            "foundGroups": found_groups,
            "appGroups": groups.len(),
            "extraGroups": extra.len(),
            "pairPrecision": precision,
            "pairRecall": recall,
            "pairF1": f1,
            "picksScored": picks_scored,
            "picksRight": picks_right,
        });
        println!("{out:#}");
    }
}
