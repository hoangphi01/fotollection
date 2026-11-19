// SPDX-License-Identifier: GPL-3.0-only
//! FR-145's cost (OPTIMIZATION §5): each photo's instant and local time on a synthetic library
//! of N photos (15,000 by default) shot around the world, a third with GPS time, a third with
//! their own offset, the rest on a camera's clock; median of 5 runs, with the spread.
//!
//! - table: what RollSense pays once after a change, `Library::photo_time` on an empty cache
//!   (each photo's place from its GPS through the bundled places, its zone, then `resolve`);
//! - resolve: `time_base::resolve` alone, the zones already known;
//! - sections: `Library::gallery_sections` (FR-141 C2), the grid's runs for a month picked and
//!   for a day, RollSense's index already built, as after the first tick.
//!
//! `cargo run --release -p fotollection-core --example measure_time_base -- [N]`

#![allow(clippy::unwrap_used, clippy::cast_precision_loss)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use fotollection_core::import::ImportedFile;
use fotollection_core::library::{CullFilter, Library, PhotoId};
use fotollection_core::metadata::PhotoMeta;
use fotollection_core::sorting::facets::SortPick;
use fotollection_core::time_base::{self, TimeFacts, Zone};

/// Cities the photos are taken around: (latitude, longitude, zone offset in minutes).
const CITIES: [(f64, f64, i16); 6] = [
    (47.4979, 19.0402, 120),
    (37.5665, 126.978, 540),
    (10.7769, 106.7009, 420),
    (40.7128, -74.006, -240),
    (35.6762, 139.6503, 540),
    (-33.8688, 151.2093, 600),
];

fn stats(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(15_000);
    let mut lib = Library::new();
    let folder = lib.add_folder(Path::new("/roll"));
    let start = chrono::NaiveDate::from_ymd_opt(2019, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .unwrap();
    for i in 0..n {
        let (lat, lon, off) = CITIES[(i / 200) as usize % CITIES.len()];
        let jitter = f64::from(i % 97) * 0.002;
        let local = start + chrono::Duration::minutes(i64::from(i) * 37);
        let meta = PhotoMeta {
            capture: Some(local),
            offset_minutes: (i % 3 == 1).then_some(off),
            gps: (i % 4 != 3).then_some((lat + jitter, lon - jitter)),
            gps_utc: (i % 3 == 0).then(|| local.and_utc().timestamp() - i64::from(off) * 60),
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            orientation: 1,
            ..PhotoMeta::default()
        };
        let file = ImportedFile {
            path: PathBuf::from(format!("/roll/{i:06}.jpg")),
            size: 1,
            mtime: 0,
        };
        lib.push(folder, &file, &meta);
    }
    // The bundled places and zones are read once, as in the app.
    lib.photo_time(PhotoId(0));
    let mut table = Vec::new();
    for run in 0..5 {
        let home = if run % 2 == 0 {
            "Europe/Budapest"
        } else {
            "Asia/Ho_Chi_Minh"
        };
        lib.set_home_zone(time_base::zone(home));
        let t = Instant::now();
        std::hint::black_box(lib.photo_time(PhotoId(0)));
        table.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let seoul = time_base::zone("Asia/Seoul").unwrap();
    let budapest = time_base::zone("Europe/Budapest").unwrap();
    let facts: Vec<(TimeFacts, Option<&Zone>)> = (0..n)
        .map(|i| {
            let local = i64::from(i) * 37 * 60 + 1_546_300_800;
            let f = TimeFacts {
                local,
                photo_offset: (i % 3 == 1).then_some(540),
                gps_utc: (i % 3 == 0).then_some(local - 32_400),
                ..TimeFacts::default()
            };
            (f, (i % 4 != 3).then_some(&seoul))
        })
        .collect();
    let mut resolve = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        for (f, zone) in &facts {
            std::hint::black_box(time_base::resolve(f, *zone, &budapest));
        }
        resolve.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let (m, lo, hi) = stats(table);
    println!("{n} photos, table: median {m:.1} ms ({lo:.1}–{hi:.1})");
    let (m, lo, hi) = stats(resolve);
    println!("{n} photos, resolve alone: median {m:.2} ms ({lo:.2}–{hi:.2})");
    let day = fotollection_core::calendar::days_from_ymd(2019, 6, 15).unwrap();
    for (what, sort) in [
        (
            "a month",
            vec![SortPick::Month {
                year: 2019,
                month: 6,
            }],
        ),
        ("a day", vec![SortPick::Day { day }]),
    ] {
        let filter = CullFilter {
            sort,
            ..CullFilter::default()
        };
        // The first call builds RollSense's index, as the first tick does in the app.
        let runs = lib.gallery_sections(&filter).len();
        let mut times = Vec::new();
        for _ in 0..5 {
            let t = Instant::now();
            std::hint::black_box(lib.gallery_sections(&filter));
            times.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let (m, lo, hi) = stats(times);
        println!(
            "{n} photos, sections of {what} ({runs} runs): median {m:.2} ms ({lo:.2}–{hi:.2})"
        );
    }
}
