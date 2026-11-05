// SPDX-License-Identifier: GPL-3.0-only
//! Every file of a folder through every step the app takes, one row per file: import (or
//! the reason it is refused), the embedded thumbnail (T0), the 400 px thumbnail (T1), the
//! print preview, and the export, each with its `ERR-` code when it fails (`ERROR_CONTROL.md`).
//! For damaged, renamed or hostile files: each must give an error the user can report,
//! never a crash or a hang. Files are only read; the thumbnails and exports go to temporary
//! folders that are removed.
//!
//! `cargo run --release -p fotollection-core --example check_files -- <folder>`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::default_codec;
use fotollection_core::errors::err_code;
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, run_export};
use fotollection_core::import::scan;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::{self, ThumbCache};

fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    let codec = default_codec();
    let found = scan(&root);
    for r in &found.rejected {
        println!(
            "{:<28} refused at import: {:?}",
            r.path.file_name().unwrap().to_string_lossy(),
            r.reason
        );
    }
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
    let cache_dir = tempfile::tempdir().unwrap();
    let cache = ThumbCache::new(cache_dir.path().to_path_buf());
    let settings = RenderSettings::default();
    for record in &records {
        let name = record
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let t = Instant::now();
        let t0 = if thumbs::t0(record, codec.as_ref()).is_some() {
            "ok"
        } else {
            "none"
        };
        let t1 = cache
            .make_t1(record, codec.as_ref())
            .map_or_else(|e| err_code(e.code()).to_owned(), |_| "ok".to_owned());
        let preview = PreviewCache::new()
            .render_jpeg(record, &settings, None, None, codec.as_ref())
            .map_or_else(|e| err_code(e.code()).to_owned(), |_| "ok".to_owned());
        println!(
            "{name:<28} T0 {t0:<5} T1 {t1:<13} preview {preview:<13} ({:.1} s)",
            t.elapsed().as_secs_f64()
        );
    }
    let dest = tempfile::tempdir().unwrap();
    let options = ExportOptions {
        destination: dest.path(),
        settings: &settings,
        quality: EXPORT_QUALITY,
        max_inflight: 2,
        place_for: &|_| None,
        frame_for: &|_| None,
        group_for: &|_| None,
        by_day: false,
        postcard_font: None,
    };
    let summary = run_export(
        &records,
        &options,
        codec.as_ref(),
        &CancellationToken::new(),
        &|_, _| {},
    );
    println!("export: {} written", summary.exported);
    for f in &summary.failed {
        println!(
            "{:<28} export failed: {} ({})",
            f.file_name,
            err_code(&f.code),
            f.message
        );
    }
}
