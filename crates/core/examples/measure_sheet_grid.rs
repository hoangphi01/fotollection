// SPDX-License-Identifier: GPL-3.0-only
//! Measures what the Collage grid costs after one setting changes (FR-035, NFR-021): every
//! sheet shown is rendered again at the grid's size, from the preview sources (as before
//! 2026-11-01) and from the photos' thumbnails (the app since). Median of 3 passes after a
//! first one; the thumbnails are made beforehand, as browsing the folder does.
//!
//! `cargo run --release -p fotollection-core --example measure_sheet_grid -- <photo.jpg>…`
//!
//! An even number of photos makes the sheets: 20 photos are the 10 sheets a grid shows.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::codec::default_codec;
use fotollection_core::import::check_file;
use fotollection_core::library::{Library, PhotoRecord};
use fotollection_core::metadata;
use fotollection_core::preview::PreviewCache;
use fotollection_core::settings::RenderSettings;
use fotollection_core::thumbs::ThumbCache;

/// The grid's sheets, as the app asks for them (`fotothumb://…/sheet/…&small=1`).
const SMALL_SHEET: u32 = 640;

fn main() {
    let codec = default_codec();
    let mut lib = Library::new();
    let records: Vec<PhotoRecord> = std::env::args()
        .skip(1)
        .filter_map(|path| {
            // Files the app would skip are skipped here too.
            let file = check_file(path.as_ref()).ok()?;
            let meta = metadata::read_path(&file.path);
            let folder = lib.add_folder(file.path.parent()?);
            let id = lib.push(folder, &file, &meta)?;
            lib.record(id)
        })
        .collect();
    let dir = tempfile::tempdir().unwrap();
    let thumbs = ThumbCache::new(dir.path().to_path_buf());
    for record in &records {
        thumbs.make_t1(record, codec.as_ref()).unwrap();
    }
    let thumbnail = |r: &PhotoRecord| thumbs.cached_t1(r);
    for (name, from_thumbnails) in [("preview sources", false), ("thumbnails", true)] {
        let cache = PreviewCache::new();
        let mut settings = RenderSettings::default();
        let pass = |settings: &RenderSettings| {
            let t = Instant::now();
            for pair in records.chunks(2) {
                let photos = [Some((&pair[0], None)), pair.get(1).map(|r| (r, None))];
                let small = from_thumbnails.then_some(&thumbnail as &dyn Fn(&_) -> _);
                cache
                    .render_sheet_jpeg(photos, settings, SMALL_SHEET, small, codec.as_ref())
                    .unwrap();
            }
            t.elapsed().as_secs_f64() * 1e3
        };
        let first = pass(&settings);
        let mut runs: Vec<f64> = (0..3)
            .map(|i| {
                // A setting changes, as a switch in the panel does.
                settings.collage.cut_line = i % 2 == 0;
                pass(&settings)
            })
            .collect();
        runs.sort_by(f64::total_cmp);
        println!(
            "{name}: {} sheets from {} photos at {SMALL_SHEET} px: first {first:.0} ms, after a change {:.0} ms ({:.1} ms a sheet)",
            records.len().div_ceil(2),
            records.len(),
            runs[1],
            runs[1] / records.len().div_ceil(2) as f64
        );
    }
}
