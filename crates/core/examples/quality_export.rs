// SPDX-License-Identifier: GPL-3.0-only
//! Exports photos through the real export path at their own size and ratio (print size
//! Original), with no stamp and no frame, so `bench/quality/audit.py` can compare every
//! pixel with the source (NFR-009). Photos are only read.
//!
//! `cargo run --release -p fotollection-core --example quality_export -- <out-dir> <photo>…`
//!
//! With `--share original|facebook|instagram|<long edge>` first, it makes the share export
//! instead (FR-053), for `bench/quality/share_audit.py`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use fotollection_core::cancel::CancellationToken;
use fotollection_core::codec::default_codec;
use fotollection_core::export::{EXPORT_QUALITY, ExportOptions, default_inflight, run_export};
use fotollection_core::import::check_file;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::settings::RenderSettings;
use fotollection_core::share::{ExportTarget, SharePreset};

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let target = if args.first().is_some_and(|a| a == "--share") {
        let preset = match args.get(1).map(String::as_str) {
            Some("original") => SharePreset::Original,
            Some("facebook") => SharePreset::Facebook,
            Some("instagram") => SharePreset::Instagram,
            Some(edge) => SharePreset::Custom(edge.parse().expect("a long edge in pixels")),
            None => panic!("--share needs a preset"),
        };
        args.drain(..2);
        ExportTarget::Share(preset)
    } else {
        ExportTarget::Print
    };
    let (out, photos) = args
        .split_first()
        .expect("usage: quality_export <out-dir> <photo>…");
    let mut lib = Library::new();
    for p in photos {
        let file = check_file(p.as_ref()).expect("readable photo");
        let folder = lib.add_folder(file.path.parent().expect("parent"));
        lib.push(folder, &file, &metadata::read_path(&file.path));
    }
    let records: Vec<_> = lib
        .sorted_ids()
        .into_iter()
        .filter_map(|id| lib.record(id))
        .collect();
    let defaults = RenderSettings::default();
    let settings = RenderSettings {
        print_size: "original".into(),
        stamp: fotollection_core::settings::StampSettings {
            enabled: false,
            ..defaults.stamp.clone()
        },
        export_target: target,
        ..defaults
    };
    let destination = PathBuf::from(out);
    let summary = run_export(
        &records,
        &ExportOptions {
            destination: &destination,
            settings: &settings,
            quality: EXPORT_QUALITY,
            max_inflight: default_inflight(),
            place_for: &|_| None,
            frame_for: &|_| None,
            group_for: &|_| None,
        },
        default_codec().as_ref(),
        &CancellationToken::new(),
        &|_, _| {},
    );
    eprintln!(
        "exported {} (failed {}) at q{EXPORT_QUALITY}",
        summary.exported,
        summary.failed.len()
    );
}
