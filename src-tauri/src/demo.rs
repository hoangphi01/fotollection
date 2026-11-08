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
use fotollection_core::looks::{Adjust, LookSettings};
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
        "collage": record_collage(app, out)?,
        "postcard": record_postcard(app, out, &BORDERED)?,
        "postcard_full": record_postcard(app, out, &FULL)?,
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
                None,
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
        // The Looks card's tiles (FR-079): the photo through each built-in look.
        for look in fotollection_core::look_library::built_in() {
            write(
                &out.join(format!(
                    "img/looktile/{key}-{}.jpg",
                    look.id.replace(':', "-")
                )),
                &crate::looks::look_tile_bytes(&state, id, &look.id).map_err(|e| e.message)?,
            )?;
        }
        write(
            &out.join(format!("img/source/{key}.jpg")),
            &source_bytes(&state, id, false).map_err(|e| e.message)?,
        )?;
    }
    common.insert("photo_details", Value::Object(details));
    common.insert(
        "list_looks",
        serde_json::to_value(fotollection_core::look_library::built_in())
            .map_err(|e| e.to_string())?,
    );
    common.insert("get_crop_geometry", Value::Object(geometry));
    common.insert("print_advice", Value::Object(advice));
    to_json(common)
}

/// Three looks: the retro date stamp, the "Shot on" info frame, and a colour look with its
/// plain preview beside it (the site's before-and-after, FR-096), each with its previews.
fn record_scenarios(app: &AppHandle, ids: &[PhotoId], out: &Path) -> Result<Value, String> {
    let state = app.state::<AppState>();
    let mut scenarios = BTreeMap::new();
    for scenario in ["stamp", "frame", "look"] {
        let mut settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
        // A colour look of Faves' sliders for the before and after; none in the others.
        settings.look = if scenario == "look" {
            demo_look()
        } else {
            LookSettings::default()
        };
        settings.stamp = StampSettings {
            enabled: scenario == "stamp",
            place: true,
            ..settings.stamp
        };
        ok(commands::set_settings(app.clone(), app.state(), settings))?;
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
                &preview_bytes(&state, id, crate::commands::PreviewPart::All)
                    .map_err(|e| e.message)?,
            )?;
            if scenario == "look" {
                write(
                    &out.join(format!("img/preview/look-plain/{key}.jpg")),
                    &preview_bytes(&state, id, crate::commands::PreviewPart::Plain)
                        .map_err(|e| e.message)?,
                )?;
            }
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
    // What follows is recorded without the look.
    let mut settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
    settings.look = LookSettings::default();
    ok(commands::set_settings(app.clone(), app.state(), settings))?;
    to_json(scenarios)
}

/// A warm, clear look for the before and after: Faves' sliders as the owner set them on a
/// Prague RAW (2026-11-06).
fn demo_look() -> LookSettings {
    LookSettings {
        adjust: Adjust {
            brightness: 9,
            contrast: 44,
            highlights: -36,
            shadows: -48,
            warmth: 22,
            tint: 10,
            saturation: 19,
            vibrance: 3,
            ..Adjust::default()
        },
        ..LookSettings::default()
    }
}

/// Collage mode (FR-035): the gallery's sheets with the date stamp and the 3 mm border, each
/// sheet's facts and its images, small for the grid and large for Space.
fn record_collage(app: &AppHandle, out: &Path) -> Result<Value, String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
    settings.stamp.enabled = true;
    settings.border = true;
    ok(commands::set_settings(app.clone(), app.state(), settings))?;
    let sheets = crate::collage::list_sheets(app.state(), CullFilter::default())
        .map_err(|e| format!("{e:?}"))?;
    let mut info = serde_json::Map::new();
    let mut geometry = serde_json::Map::new();
    for &sheet in &sheets {
        let key = match sheet.second {
            Some(second) => format!("{}-{}", sheet.first.0, second.0),
            None => sheet.first.0.to_string(),
        };
        info.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(crate::collage::sheet_info(
                app.clone(),
                sheet,
            )))?,
        );
        for id in std::iter::once(sheet.first).chain(sheet.second) {
            geometry.insert(
                id.0.to_string(),
                ok(tauri::async_runtime::block_on(commands::get_crop_geometry(
                    app.clone(),
                    id,
                    Some(sheet),
                )))?,
            );
        }
        for (small, suffix) in [(true, "-small"), (false, "")] {
            write(
                &out.join(format!("img/sheet/{key}{suffix}.jpg")),
                &crate::collage::sheet_bytes(
                    &state,
                    sheet,
                    small,
                    fotollection_core::preview::SheetView::Normal,
                )
                .map_err(|e| e.message)?,
            )?;
        }
    }
    Ok(json!({
        "get_settings": ok(commands::get_settings(app.state()))?,
        "list_sheets": to_json(&sheets)?,
        "sheet_info": info,
        "get_crop_geometry": geometry,
    }))
}

/// One recording of the demo postcards (FR-073): how the photo half is set, and the
/// messages typed on three cards from the named photo on (the owner's, `demo_set.txt`), or
/// from the first card when the set does not have it. Each is sent to the National Museum
/// in Prague, a public address, on the address's three lines.
struct PostcardDemo {
    key: &'static str,
    border: bool,
    fill: bool,
    from: &'static str,
    messages: [&'static str; 3],
}
const NATIONAL_MUSEUM: &str = "Národní muzeum\nVáclavské náměstí 68\n110 00 Praha 1, Czechia";
/// The red tram by day, with the 3 mm border round the photo.
const BORDERED: PostcardDemo = PostcardDemo {
    key: "sheet-postcard",
    border: true,
    fill: false,
    from: "P1067865",
    messages: [
        "Ahoj Praha!\nPozdravy z Prahy.",
        "Ahoj Praha!\nLove from the funicular.",
        "Ahoj Praha!\nThương gửi cả nhà.",
    ],
};
/// The trams by night, the photo filling its half to the edge.
const FULL: PostcardDemo = PostcardDemo {
    key: "sheet-postcard-full",
    border: false,
    fill: true,
    from: "P1068073",
    messages: [
        "Ahoj Praha!\nDobrou noc z Prahy.",
        "Ahoj Praha!\nPozdrav z tramvaje.",
        "Ahoj Praha!\nThe last 22 home.",
    ],
};

/// Postcards (FR-073), after the collage: the same photos, one per card, held landscape,
/// three with a typed message; each card's facts, its text and its images.
fn record_postcard(app: &AppHandle, out: &Path, demo: &PostcardDemo) -> Result<Value, String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.read().map_err(|e| e.to_string())?.0.clone();
    settings.collage.postcard.enabled = true;
    settings.border = demo.border;
    settings.collage.fill = demo.fill;
    ok(commands::set_settings(app.clone(), app.state(), settings))?;
    let sheets = crate::collage::list_sheets(app.state(), CullFilter::default())
        .map_err(|e| format!("{e:?}"))?;
    let start = {
        let lib = state.library.read().map_err(|e| e.to_string())?;
        sheets
            .iter()
            .position(|sheet| {
                lib.summary(sheet.first)
                    .is_some_and(|s| s.file_name.starts_with(demo.from))
            })
            .unwrap_or(0)
    };
    let texted = sheets.iter().cycle().skip(start).take(sheets.len());
    for (sheet, message) in texted.zip(demo.messages) {
        for (part, text) in [
            (fotollection_core::settings::PostcardPart::Message, message),
            (
                fotollection_core::settings::PostcardPart::Address,
                NATIONAL_MUSEUM,
            ),
        ] {
            ok(commands::set_postcard_text(
                app.clone(),
                app.state(),
                sheet.first,
                part,
                Some(text.to_owned()),
            ))?;
        }
    }
    let mut info = serde_json::Map::new();
    let mut geometry = serde_json::Map::new();
    let mut texts = serde_json::Map::new();
    for &sheet in &sheets {
        let key = sheet.first.0.to_string();
        info.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(crate::collage::sheet_info(
                app.clone(),
                sheet,
            )))?,
        );
        geometry.insert(
            key.clone(),
            ok(tauri::async_runtime::block_on(commands::get_crop_geometry(
                app.clone(),
                sheet.first,
                Some(sheet),
            )))?,
        );
        texts.insert(
            key.clone(),
            ok(commands::get_postcard_text(app.state(), sheet.first))?,
        );
        for (small, suffix) in [(true, "-small"), (false, "")] {
            write(
                &out.join(format!("img/{}/{key}{suffix}.jpg", demo.key)),
                &crate::collage::sheet_bytes(
                    &state,
                    sheet,
                    small,
                    fotollection_core::preview::SheetView::Normal,
                )
                .map_err(|e| e.message)?,
            )?;
        }
    }
    Ok(json!({
        "get_settings": ok(commands::get_settings(app.state()))?,
        "list_sheets": to_json(&sheets)?,
        "sheet_info": info,
        "get_crop_geometry": geometry,
        "get_postcard_text": texts,
    }))
}
