// SPDX-License-Identifier: GPL-3.0-only
//! Dev-only: records what the real app answers for a demo folder, so the website's
//! screenshots show genuine output without anyone's own photos (site/screenshots/README.md).
//!
//! `FOTOLLECTION_DEMO_FOLDER=<photos> FOTOLLECTION_DEMO_DUMP=<out> target/debug/fotollection`
//! runs with its own settings, cache and project under `<out>/.state`, imports the folder,
//! runs the best-shot analysis, then writes `<out>/fixtures.json` (IPC answers by command)
//! and the image protocol's bytes under `<out>/img/`, and exits.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fotollection_core::frame::FrameSettings;
use fotollection_core::library::{CullFilter, Flag, PhotoId};
use fotollection_core::settings::StampSettings;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::commands::{self, preview_bytes, source_bytes, thumbnail_bytes};
use crate::state::AppState;
use crate::{culling, document};

/// The demo's directories, when the dump was requested.
pub fn requested() -> Option<(PathBuf, PathBuf)> {
    let folder = std::env::var_os("FOTOLLECTION_DEMO_FOLDER")?;
    let out = std::env::var_os("FOTOLLECTION_DEMO_DUMP")?;
    Some((PathBuf::from(folder), PathBuf::from(out)))
}

/// Settings, cache and project of the dump, under `<out>/.state`.
pub fn state_dirs(out: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let root = out.join(".state");
    (
        root.join("thumbs"),
        root.join("config"),
        root.join("session.fotollection"),
    )
}

fn to_json<T: serde::Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

fn ok<T: serde::Serialize, E: std::fmt::Debug>(result: Result<T, E>) -> Result<Value, String> {
    to_json(result.map_err(|e| format!("{e:?}"))?)
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Imports, analyses, records, then quits the app.
pub fn run(app: &AppHandle, folder: PathBuf, out: &Path) {
    let result = record(app, folder, out);
    if let Err(e) = &result {
        eprintln!("demo dump failed: {e}");
    }
    app.exit(i32::from(result.is_err()));
}

fn record(app: &AppHandle, folder: PathBuf, out: &Path) -> Result<(), String> {
    let ids = prepare(app, folder)?;
    let fixtures = json!({
        "common": record_common(app, &ids, out)?,
        "scenarios": record_scenarios(app, &ids, out)?,
    });
    let text = serde_json::to_string_pretty(&fixtures).map_err(|e| e.to_string())?;
    write(&out.join("fixtures.json"), text.as_bytes())?;
    eprintln!("demo: wrote {}", out.display());
    Ok(())
}

/// Imports the folder, waits for the best-shot analysis, and culls a few photos.
fn prepare(app: &AppHandle, folder: PathBuf) -> Result<Vec<PhotoId>, String> {
    let state = app.state::<AppState>();
    let summary = commands::import_paths(app, vec![folder]);
    eprintln!("demo: imported {} photos", summary.added);

    // Best-shot analysis runs as a background job; wait for it.
    culling::analyze_library(app.clone(), app.state()).map_err(|e| format!("{e:?}"))?;
    loop {
        std::thread::sleep(Duration::from_millis(200));
        let running = state.analyses.lock().map_err(|e| e.to_string())?.running;
        if running.is_none() {
            break;
        }
    }

    let ids = state
        .library
        .read()
        .map_err(|e| e.to_string())?
        .sorted_ids();
    // A culled look: a few picks and stars, one reject.
    let pick: Vec<PhotoId> = ids.iter().copied().step_by(3).collect();
    ok(commands::set_photo_flag(
        app.clone(),
        app.state(),
        pick,
        Some(Flag::Pick),
    ))?;
    ok(commands::set_photo_rating(
        app.clone(),
        app.state(),
        ids.iter().copied().step_by(4).collect(),
        5,
    ))?;
    ok(commands::set_photo_rating(
        app.clone(),
        app.state(),
        ids.iter().copied().skip(1).step_by(5).collect(),
        3,
    ))?;
    if let Some(&last) = ids.last() {
        ok(commands::set_photo_flag(
            app.clone(),
            app.state(),
            vec![last],
            Some(Flag::Reject),
        ))?;
    }
    Ok(ids)
}

/// Answers that do not depend on the look, and the thumbnails and source images.
fn record_common(app: &AppHandle, ids: &[PhotoId], out: &Path) -> Result<Value, String> {
    let state = app.state::<AppState>();
    let mut common = BTreeMap::new();
    common.insert("app_info", to_json(commands::app_info())?);
    common.insert("list_print_sizes", to_json(commands::list_print_sizes())?);
    common.insert("list_fonts", ok(commands::list_fonts(app.state()))?);
    common.insert(
        "get_ui_settings",
        ok(commands::get_ui_settings(app.state()))?,
    );
    common.insert(
        "get_document",
        to_json(document::get_document(app.state()))?,
    );
    common.insert("list_groups", ok(culling::list_groups(app.state()))?);
    // The Places card and the place picker (FR-005, FR-032), with one search typed.
    common.insert(
        "list_places",
        ok(tauri::async_runtime::block_on(crate::places::list_places(
            app.clone(),
        )))?,
    );
    common.insert(
        "search_places",
        ok(crate::places::search_places("pra".into(), 20))?,
    );
    common.insert(
        "list_photos",
        ok(commands::list_photos(app.state(), CullFilter::default()))?,
    );
    let mut details = serde_json::Map::new();
    let mut geometry = serde_json::Map::new();
    let mut advice = serde_json::Map::new();
    for &id in ids {
        let key = id.0.to_string();
        details.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(commands::photo_details(
                app.clone(),
                id,
            )))?,
        );
        geometry.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(commands::get_crop_geometry(
                app.clone(),
                id,
            )))?,
        );
        advice.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(commands::print_advice(
                app.clone(),
                id,
            )))?,
        );
        let thumb = thumbnail_bytes(&state, id).ok_or("no thumbnail")?;
        write(&out.join(format!("img/thumb/{key}.jpg")), &thumb)?;
        write(
            &out.join(format!("img/source/{key}.jpg")),
            &source_bytes(&state, id)?,
        )?;
    }
    common.insert("photo_details", Value::Object(details));
    common.insert("get_crop_geometry", Value::Object(geometry));
    common.insert("print_advice", Value::Object(advice));
    to_json(common)
}

/// Two looks: the retro date stamp, and the "Shot on" info frame, with their previews.
fn record_scenarios(app: &AppHandle, ids: &[PhotoId], out: &Path) -> Result<Value, String> {
    let state = app.state::<AppState>();
    let mut scenarios = BTreeMap::new();
    for scenario in ["stamp", "frame"] {
        let mut settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
        settings.stamp = StampSettings {
            enabled: scenario == "stamp",
            place: true,
            ..settings.stamp
        };
        ok(commands::set_settings(app.state(), settings))?;
        let frame = FrameSettings {
            enabled: scenario == "frame",
            ..state
                .frame
                .read()
                .map_err(|e| e.to_string())?
                .settings
                .clone()
        };
        ok(commands::set_frame_settings(app.state(), frame))?;

        let mut stamps = serde_json::Map::new();
        for &id in ids {
            let key = id.0.to_string();
            stamps.insert(key.clone(), ok(commands::get_photo_stamp(app.state(), id))?);
            write(
                &out.join(format!("img/preview/{scenario}/{key}.jpg")),
                &preview_bytes(&state, id)?,
            )?;
        }
        scenarios.insert(
            scenario,
            json!({
                "get_settings": ok(commands::get_settings(app.state()))?,
                "get_frame_config": ok(commands::get_frame_config(app.state()))?,
                "get_photo_stamp": stamps,
            }),
        );
    }
    to_json(scenarios)
}
