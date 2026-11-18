// SPDX-License-Identifier: GPL-3.0-only
//! FR-141 S6a, a CLIP candidate measured before it is chosen: loaded in tract, its input made
//! in Rust (`sorting::clip::pixels`), each labelled photo of the roll embedded, then read
//! zero-shot against the labels' text embeddings (`scripts/models/clip-export.sh`). Prints the
//! load time, ms per photo (decode, input, model), peak memory, how close the Rust input
//! comes to CLIP's own (`--check`), and the labels' scores. The labels stay outside the
//! repository; only counts are printed.
//!
//! `cargo run --release -p fotollection-core --example clip_bench -- --model m.onnx
//!  --labels m.labels.json --images <roll> [--threads N] [--check m.check.json <dir>]
//!  [--subjects subjects.csv] [--sources labels.csv] [--dump probs.jsonl] [--one-cut]`
//! Each photo is read as the app reads it, three cuts averaged; `--one-cut` the centre alone.
//! `… --synthetic N [--batch B] [--threads T]` times the model alone on random inputs.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::codec::{PureRustCodec, decode_for_screen};
use fotollection_core::image::{RgbImage, apply_orientation};
use fotollection_core::metadata;
use fotollection_core::sorting::clip::{pixels, pixels_at, zero_shot};
use rayon::prelude::*;
use serde_json::{Value, json};
use tract_onnx::prelude::*;

type Plan = std::sync::Arc<TypedRunnableModel>;

struct Labels {
    keys: Vec<String>,
    vectors: Vec<Vec<f32>>,
    mean: [f32; 3],
    std: [f32; 3],
    size: u32,
    scale: f32,
}

fn labels(path: &Path) -> Labels {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let three = |k: &str| -> [f32; 3] {
        let a: Vec<f32> = v[k]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect();
        [a[0], a[1], a[2]]
    };
    // One row per prompt, its label repeated (a label of one mean vector reads as one prompt).
    let floats = |e: &Value| -> Vec<f32> {
        e.as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect()
    };
    let (mut keys, mut vectors) = (Vec::new(), Vec::new());
    for (k, e) in v["labels"].as_object().unwrap() {
        let rows = e.as_array().unwrap();
        if rows.first().is_some_and(Value::is_array) {
            for r in rows {
                keys.push(k.clone());
                vectors.push(floats(r));
            }
        } else {
            keys.push(k.clone());
            vectors.push(floats(e));
        }
    }
    Labels {
        keys,
        vectors,
        mean: three("mean"),
        std: three("std"),
        size: v["size"].as_u64().unwrap() as u32,
        scale: v["logit_scale"].as_f64().unwrap() as f32,
    }
}

/// A photo's embedding as the app reads it, three cuts averaged (`clip::Embedder::embed`), or
/// the centre alone; and when its input was ready, for the timing.
fn read(
    plan: &Plan,
    labels: &Labels,
    img: &RgbImage,
    one_cut: bool,
) -> Option<(Instant, Vec<f32>)> {
    let cuts: &[f32] = if one_cut || img.width == img.height {
        &[0.5]
    } else {
        &[0.5, 0.0, 1.0]
    };
    let px: Vec<Vec<f32>> = cuts
        .iter()
        .map(|&at| pixels_at(img, labels.size, labels.mean, labels.std, at))
        .collect::<Option<_>>()?;
    let ready = Instant::now();
    let mut e = vec![0f32; 512];
    for p in px {
        let one = embed(plan, p, labels.size);
        let n = one.iter().map(|v| v * v).sum::<f32>().sqrt();
        for (s, v) in e.iter_mut().zip(one) {
            *s += v / n;
        }
    }
    let n = e.iter().map(|v| v * v).sum::<f32>().sqrt();
    for v in &mut e {
        *v /= n;
    }
    Some((ready, e))
}

/// The prompts' probabilities summed per label, as the app does (`clip::subjects`).
fn by_label(labels: &Labels, which: &[usize], p: &[f32]) -> BTreeMap<String, f32> {
    let mut out = BTreeMap::new();
    for (&i, p) in which.iter().zip(p) {
        *out.entry(labels.keys[i].clone()).or_insert(0.0) += p;
    }
    out
}

fn embed(plan: &Plan, px: Vec<f32>, size: u32) -> Vec<f32> {
    let s = size as usize;
    let input = tract_ndarray::Array4::from_shape_vec((1, 3, s, s), px).unwrap();
    let out = plan.run(tvec!(Tensor::from(input).into())).unwrap();
    out[0]
        .to_plain_array_view::<f32>()
        .unwrap()
        .iter()
        .copied()
        .collect()
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let n = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (n(a) * n(b)).max(f32::EPSILON)
}

fn photo(path: &Path) -> Option<RgbImage> {
    let bytes = std::fs::read(path).ok()?;
    let image = decode_for_screen(&PureRustCodec, &bytes, 1024).ok()?.image;
    Some(apply_orientation(
        image,
        metadata::read_path(path).orientation,
    ))
}

fn csv_rows(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| {
            l.trim_start_matches('\u{feff}')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect()
        })
        .collect()
}

/// The source the owner's label means, among the four the pixels decide.
fn source_of(label: &str) -> Option<&'static str> {
    Some(match label {
        "meme" => "src-meme",
        "web-download" => "src-web",
        "screenshot" => "src-screenshot",
        "camera-iphone"
        | "other-camera"
        | "received-chat"
        | "edited"
        | "app-instagram"
        | "app-instagram-camera" => "src-camera",
        _ => return None,
    })
}

/// The model in tract, its input fixed to `batch` squares of `size`.
fn load(path: &Path, size: u32, batch: usize) -> Plan {
    let s = size as usize;
    // The files record shapes for a batch of any size; the input below fixes it to one.
    tract_onnx::onnx()
        .with_ignore_value_info(true)
        .with_ignore_output_shapes(true)
        .model_for_path(path)
        .and_then(|m| m.with_input_fact(0, f32::fact([batch, 3, s, s]).into()))
        .and_then(tract_onnx::prelude::InferenceModelExt::into_optimized)
        .and_then(TypedModel::into_runnable)
        .expect("model")
}

/// How close the Rust input comes to CLIP's own, on the export's check images.
fn check(plan: &Plan, labels: &Labels, refs: &Path, dir: &Path) -> Value {
    let refs: BTreeMap<String, Vec<f32>> =
        serde_json::from_str(&std::fs::read_to_string(refs).unwrap()).unwrap();
    let cos: Vec<f32> = refs
        .iter()
        .filter_map(|(name, want)| {
            let img = photo(&dir.join(name))?;
            let got = embed(
                plan,
                pixels(&img, labels.size, labels.mean, labels.std)?,
                labels.size,
            );
            Some(cosine(&got, want))
        })
        .collect();
    json!({"images": cos.len(), "min_cosine": cos.iter().copied().fold(1.0, f32::min)})
}

/// Each labelled photo read zero-shot: the subjects' top-1 (right of all, and per label right,
/// predicted, actual) and the sources' confusion (truth → guess).
#[allow(clippy::type_complexity)]
fn score(
    results: &[(String, Vec<f32>, [f64; 3])],
    labels: &Labels,
    subjects: &HashMap<String, BTreeSet<String>>,
    sources: &HashMap<String, &'static str>,
) -> (
    String,
    BTreeMap<String, [usize; 3]>,
    BTreeMap<String, BTreeMap<String, usize>>,
) {
    let index = |prefix: bool| -> Vec<usize> {
        (0..labels.keys.len())
            .filter(|&i| labels.keys[i].starts_with("src-") == prefix)
            .collect()
    };
    let (src_i, subj_i) = (index(true), index(false));
    let pick = |e: &[f32], which: &[usize]| -> (String, f32) {
        let vecs: Vec<Vec<f32>> = which.iter().map(|&i| labels.vectors[i].clone()).collect();
        let p = zero_shot(e, &vecs, labels.scale);
        by_label(labels, which, &p)
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap()
    };
    let mut subject_score: BTreeMap<String, [usize; 3]> = BTreeMap::new(); // right, predicted, actual
    let (mut s_right, mut s_all) = (0, 0);
    let mut source_confusion: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for (name, e, _) in results {
        if let Some(tags) = subjects.get(name) {
            let (top, _) = pick(e, &subj_i);
            s_all += 1;
            s_right += usize::from(tags.contains(&top));
            subject_score.entry(top.clone()).or_default()[1] += 1;
            if tags.contains(&top) {
                subject_score.entry(top.clone()).or_default()[0] += 1;
            }
            for t in tags {
                subject_score.entry(t.clone()).or_default()[2] += 1;
            }
        }
        if let Some(truth) = sources.get(name) {
            let (top, _) = pick(e, &src_i);
            *source_confusion
                .entry((*truth).to_string())
                .or_default()
                .entry(top)
                .or_default() += 1;
        }
    }
    (
        format!("{s_right}/{s_all}"),
        subject_score,
        source_confusion,
    )
}

/// The model alone on random inputs: `n` photos in batches of `batch` over `threads`, the
/// median of five runs, for a machine without the photos (REF-MAC).
fn synthetic(path: &Path, labels: &Labels, n: usize, batch: usize, threads: usize) -> Value {
    let s = labels.size as usize;
    let t = Instant::now();
    let plan = load(path, labels.size, batch);
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;
    let pool = fotollection_core::pool::contained(Some(threads), "clip-bench").unwrap();
    let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
    let input: Vec<f32> = (0..batch * 3 * s * s)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % 2000) as f32 / 1000.0 - 1.0
        })
        .collect();
    let tensor =
        Tensor::from(tract_ndarray::Array4::from_shape_vec((batch, 3, s, s), input).unwrap());
    let runs = n.div_ceil(batch);
    let mut walls: Vec<f64> = (0..5)
        .map(|_| {
            let t = Instant::now();
            pool.install(|| {
                (0..runs).into_par_iter().for_each(|_| {
                    plan.run(tvec!(tensor.clone().into())).unwrap();
                });
            });
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    walls.sort_by(f64::total_cmp);
    let per = walls[2] / (runs * batch) as f64;
    json!({
        "model": path.file_name().map(|n| n.to_string_lossy()), "bytes": std::fs::metadata(path).map(|m| m.len()).ok(),
        "load_ms": load_ms, "batch": batch, "threads": threads, "photos": runs * batch,
        "ms_per_photo": per, "minutes_for_15000": per * 15_000.0 / 60_000.0,
        "runs_ms": walls,
    })
}

/// Each photo's probabilities, the subjects' and the sources' apart, one JSON line each.
fn dump(path: &str, results: &[(String, Vec<f32>, [f64; 3])], labels: &Labels) {
    use std::io::Write;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    for (name, e, _) in results {
        let mut line = serde_json::Map::new();
        line.insert("name".into(), json!(name));
        for src in [true, false] {
            let which: Vec<usize> = (0..labels.keys.len())
                .filter(|&i| labels.keys[i].starts_with("src-") == src)
                .collect();
            let vecs: Vec<Vec<f32>> = which.iter().map(|&i| labels.vectors[i].clone()).collect();
            let p = zero_shot(e, &vecs, labels.scale);
            let m: serde_json::Map<String, Value> = by_label(labels, &which, &p)
                .into_iter()
                .map(|(k, p)| (k, json!(p)))
                .collect();
            line.insert(
                if src { "sources" } else { "subjects" }.into(),
                Value::Object(m),
            );
        }
        writeln!(out, "{}", Value::Object(line)).unwrap();
    }
}

/// The owner's labels: each photo's subjects, and each JPEG or PNG's source among the four.
#[allow(clippy::type_complexity)]
fn truths(
    subjects: Option<&String>,
    sources: Option<&String>,
) -> (
    HashMap<String, BTreeSet<String>>,
    HashMap<String, &'static str>,
) {
    let subjects: HashMap<String, BTreeSet<String>> = subjects
        .map(|p| {
            csv_rows(p)
                .into_iter()
                .filter(|r| r.len() > 1 && !r[1].is_empty())
                .map(|r| (r[0].clone(), r[1].split(' ').map(str::to_string).collect()))
                .collect()
        })
        .unwrap_or_default();
    let sources: HashMap<String, &'static str> = sources
        .map(|p| {
            csv_rows(p)
                .into_iter()
                .filter(|r| {
                    r.len() > 2 && r[0].to_lowercase().ends_with(".jpg")
                        || r.len() > 2 && r[0].to_lowercase().ends_with(".jpeg")
                        || r.len() > 2 && r[0].to_lowercase().ends_with(".png")
                })
                .filter_map(|r| Some((r[0].clone(), source_of(&r[2])?)))
                .collect()
        })
        .unwrap_or_default();
    (subjects, sources)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };
    let model_path = PathBuf::from(flag("--model").expect("--model"));
    let labels = labels(Path::new(flag("--labels").expect("--labels")));
    if let Some(n) = flag("--synthetic") {
        let batch = flag("--batch").map_or(1, |b| b.parse().unwrap());
        let threads = flag("--threads").map_or(1, |t| t.parse().unwrap());
        println!(
            "{}",
            synthetic(&model_path, &labels, n.parse().unwrap(), batch, threads)
        );
        return;
    }
    let images = PathBuf::from(flag("--images").expect("--images"));
    let threads = flag("--threads").map_or(1, |n| n.parse().unwrap());
    let pool = fotollection_core::pool::contained(Some(threads), "clip-bench").unwrap();

    let t = Instant::now();
    let plan = load(&model_path, labels.size, 1);
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;

    let check = flag("--check").map(|c| {
        let dir = &args[args.iter().position(|a| a == "--check").unwrap() + 2];
        check(&plan, &labels, Path::new(c), Path::new(dir))
    });
    let (subjects, sources) = truths(flag("--subjects"), flag("--sources"));
    let one_cut = args.iter().any(|a| a == "--one-cut");
    let names: BTreeSet<&String> = subjects.keys().chain(sources.keys()).collect();

    let t = Instant::now();
    let results: Vec<(String, Vec<f32>, [f64; 3])> = pool.install(|| {
        names
            .par_iter()
            .filter_map(|name| {
                let a = Instant::now();
                let img = photo(&images.join(name))?;
                let b = Instant::now();
                let (c, e) = read(&plan, &labels, &img, one_cut)?;
                let ms = |x: Instant, y: Instant| (y - x).as_secs_f64() * 1000.0;
                Some((
                    (*name).clone(),
                    e,
                    [ms(a, b), ms(b, c), ms(c, Instant::now())],
                ))
            })
            .collect()
    });
    let wall_ms = t.elapsed().as_secs_f64() * 1000.0;

    if let Some(path) = flag("--dump") {
        dump(path, &results, &labels);
    }
    let (subjects_top1, subject_score, source_confusion) =
        score(&results, &labels, &subjects, &sources);
    let mut stage: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for (_, _, ms) in &results {
        for k in 0..3 {
            stage[k].push(ms[k]);
        }
    }
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    };
    let peak_kb = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM"))
                .map(str::to_string)
        });
    println!(
        "{}",
        json!({
            "model": model_path.file_name().map(|n| n.to_string_lossy()), "bytes": std::fs::metadata(&model_path).map(|m| m.len()).ok(),
            "threads": threads, "load_ms": load_ms, "images": results.len(), "wall_ms": wall_ms,
            "ms_per_image_wall": wall_ms / results.len().max(1) as f64,
            "p50_ms": {"decode": p50(&mut stage[0]), "input": p50(&mut stage[1]), "model": p50(&mut stage[2])},
            "peak": peak_kb, "check": check,
            "subjects_top1": subjects_top1, "subjects": subject_score,
            "sources_confusion": source_confusion,
        })
    );
}
