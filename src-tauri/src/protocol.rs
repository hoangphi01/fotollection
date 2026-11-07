// SPDX-License-Identifier: GPL-3.0-only
//! `fotothumb://` serves thumbnails and previews as JPEG bytes, so pixels never cross
//! IPC as JSON or base64 (ARCHITECTURE.md §6).
//!
//! - `fotothumb://localhost/thumb/{id}`
//! - `fotothumb://localhost/preview/{id}?rev={settings revision}`
//! - `fotothumb://localhost/preview/{id}?rev=…&region={x},{y},{w},{h}&px={width}`: one part
//!   of the full-resolution render, for zoom (FR-057); fractions of the photo, checked here
//! - `fotothumb://localhost/source/{id}?rev=…`
//! - `fotothumb://localhost/sheet/{first}/{second}?rev=…[&small=1]`: a collage sheet
//!   (FR-035); `{second}` is left out for an odd last photo
//!
//! On Windows the webview addresses it as `http://fotothumb.localhost/…`.

// The handler signature is fixed by Tauri.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::library::PhotoId;
use fotollection_core::preview::Region;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::commands::{preview_bytes, region_bytes, source_bytes, thumbnail_bytes};
use crate::state::AppState;
use fotollection_core::raw::Priority;

pub const SCHEME: &str = "fotothumb";

enum Route {
    Thumb(PhotoId),
    /// The preview; `true` without the colour look, for *Before / After* (FR-079).
    Preview(PhotoId, crate::commands::PreviewPart),
    /// Where the preview shows the photo, a grey PNG, for the look on the GPU (ADR-0039).
    Mask(PhotoId),
    /// The photo's colour look as a 33³ RGBA8 table, for the look on the GPU (ADR-0039).
    Look(PhotoId),
    /// The photo through one look of the library, small and square, for the Looks card
    /// (FR-079): `looktile/<id>?look=<library id>`.
    LookTile(PhotoId, String),
    /// A full-resolution region of the preview, for zoom; `true` without the colour look, for
    /// the *Before* side of a compare view (FR-096).
    Region(PhotoId, Region, bool),
    /// Upright, uncropped T2 image for the crop editor; `true` turns it as the photo sits
    /// in its collage cell (FR-035).
    Source(PhotoId, bool),
    /// The latest live-view frame of the connected camera, from memory (FR-062).
    CameraLive,
    /// A collage sheet, small for the grid (FR-035), its photos' clipping marked (`clip=1`,
    /// FR-098, `J`).
    Sheet(crate::collage::SheetPair, bool, bool),
}

/// `region=x,y,w,h&px=n` from the query: `Ok(None)` without one, `Err` when it is malformed
/// or outside the photo (SECURITY.md §2).
fn region(query: Option<&str>) -> Result<Option<Region>, ()> {
    let params = || {
        query
            .unwrap_or_default()
            .split('&')
            .filter_map(|p| p.split_once('='))
    };
    let Some(rect) = params().find(|(k, _)| *k == "region").map(|(_, v)| v) else {
        return Ok(None);
    };
    let v: Vec<f64> = rect
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| ())?;
    let px = params()
        .find(|(k, _)| *k == "px")
        .and_then(|(_, v)| v.parse().ok())
        .ok_or(())?;
    match v.as_slice() {
        &[x, y, w, h] => Region::new(x, y, w, h, px).map(Some).ok_or(()),
        _ => Err(()),
    }
}

fn route(path: &str, query: Option<&str>) -> Option<Route> {
    if path.trim_matches('/') == "camera/live" {
        return Some(Route::CameraLive);
    }
    let mut parts = path.trim_matches('/').split('/');
    let kind = parts.next()?;
    if kind == "sheet" {
        let first = PhotoId(parts.next()?.parse().ok()?);
        let second = match parts.next() {
            Some(id) => Some(PhotoId(id.parse().ok()?)),
            None => None,
        };
        if parts.next().is_some() || second == Some(first) {
            return None;
        }
        let small = query.unwrap_or_default().split('&').any(|p| p == "small=1");
        let clip = query.unwrap_or_default().split('&').any(|p| p == "clip=1");
        return Some(Route::Sheet(
            crate::collage::SheetPair { first, second },
            small,
            clip,
        ));
    }
    let id = PhotoId(parts.next()?.parse().ok()?);
    if parts.next().is_some() {
        return None;
    }
    let flag = |name: &str| query.unwrap_or_default().split('&').any(|p| p == name);
    match kind {
        "thumb" => Some(Route::Thumb(id)),
        "preview" if flag("mask=1") => Some(Route::Mask(id)),
        "look" => Some(Route::Look(id)),
        "looktile" => {
            let look = query
                .unwrap_or_default()
                .split('&')
                .find_map(|p| p.strip_prefix("look="))?;
            fotollection_core::looks::valid_base(look).then(|| Route::LookTile(id, look.to_owned()))
        }
        "preview" => Some(match region(query).ok()? {
            Some(region) => Route::Region(id, region, flag("plain=1")),
            None => Route::Preview(
                id,
                if flag("plain=1") {
                    crate::commands::PreviewPart::Plain
                } else if flag("clip=1") {
                    crate::commands::PreviewPart::Clipping
                } else if flag("base=1") {
                    crate::commands::PreviewPart::Base
                } else {
                    crate::commands::PreviewPart::All
                },
            ),
        }),
        // `sheet=1`: as the photo sits in its collage cell, for the sheet's crop editor.
        "source" => Some(Route::Source(
            id,
            query.unwrap_or_default().split('&').any(|p| p == "sheet=1"),
        )),
        _ => None,
    }
}

/// A photo that could not be shown: its `IpcError` as JSON, so the UI can say why with
/// the `ERR-` code instead of waiting for an image that never comes.
fn fail(responder: UriSchemeResponder, error: &crate::state::IpcError) {
    let body = serde_json::to_vec(error).unwrap_or_default();
    let builder = Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONTENT_TYPE, "application/json");
    if let Ok(response) = builder.body(body) {
        responder.respond(response);
    }
}

fn respond(responder: UriSchemeResponder, status: StatusCode, body: Vec<u8>, jpeg: bool) {
    respond_as(responder, status, body, jpeg.then_some("image/jpeg"));
}

fn respond_as(
    responder: UriSchemeResponder,
    status: StatusCode,
    body: Vec<u8>,
    kind: Option<&str>,
) {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-cache");
    if let Some(kind) = kind {
        builder = builder.header(header::CONTENT_TYPE, kind);
    }
    if let Ok(response) = builder.body(body) {
        responder.respond(response);
    }
}

pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let Some(route) = route(request.uri().path(), request.uri().query()) else {
        respond(responder, StatusCode::NOT_FOUND, Vec::new(), false);
        return;
    };
    // A live-view frame is already a JPEG in memory: no decode, no pool. Never cached by
    // the webview, never written anywhere (FR-062).
    if matches!(route, Route::CameraLive) {
        match app.state::<crate::camera::CameraState>().live_frame() {
            Some(frame) => respond(responder, StatusCode::OK, frame.as_ref().clone(), true),
            None => respond(responder, StatusCode::NOT_FOUND, Vec::new(), false),
        }
        return;
    }
    // The photo the preview asks for is the one on screen from now: a RAW develop still
    // waiting for the photo before it stops going first (FR-091).
    let priority = priority_of(&route);
    if let Priority::Interactive(Some(photo)) = priority {
        fotollection_core::raw::show(photo);
    }
    // Decode work happens off the webview thread, on the interactive pool, so visible
    // thumbnails and previews never wait behind background thumbnail generation.
    let worker = app.clone();
    app.state::<AppState>().interactive_pool.spawn(move || {
        let app = worker;
        let state = app.state::<AppState>();
        fotollection_core::raw::with_priority(priority, || answer(&state, route, responder));
    });
}

/// Who waits for a route's RAW develops (FR-091): the photo on screen and the sheet shown go
/// first; thumbnails, small sheets of the grid and look tables wait behind them.
fn priority_of(route: &Route) -> Priority {
    match route {
        Route::Preview(id, _)
        | Route::Region(id, _, _)
        | Route::Mask(id)
        | Route::Source(id, _) => Priority::Interactive(Some(u64::from(id.0))),
        Route::Sheet(_, false, _) => Priority::Interactive(None),
        Route::Thumb(_)
        | Route::Sheet(_, true, _)
        | Route::Look(_)
        | Route::LookTile(..)
        | Route::CameraLive => Priority::Background,
    }
}

/// Answers `route` on a thread of the interactive pool.
fn answer(state: &AppState, route: Route, responder: UriSchemeResponder) {
    match route {
        Route::Thumb(id) => match thumbnail_bytes(state, id) {
            Some(bytes) => {
                crate::timing::mark(&crate::timing::FIRST_THUMB, "first thumbnail");
                respond(responder, StatusCode::OK, bytes, true);
            }
            None => respond(responder, StatusCode::NOT_FOUND, Vec::new(), false),
        },
        Route::Source(id, sheet) => match source_bytes(state, id, sheet) {
            Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
            Err(error) => fail(responder, &error),
        },
        Route::Region(id, region, plain) => match region_bytes(state, id, region, plain) {
            Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
            Err(error) => fail(responder, &error),
        },
        Route::CameraLive => {}
        Route::Sheet(sheet, small, clip) => {
            match crate::collage::sheet_bytes(state, sheet, small, clip) {
                Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
                Err(error) => fail(responder, &error),
            }
        }
        Route::Mask(id) => match crate::commands::preview_mask_bytes(state, id) {
            Ok(bytes) => respond_as(responder, StatusCode::OK, bytes, Some("image/png")),
            Err(error) => fail(responder, &error),
        },
        Route::Look(id) => match crate::commands::look_table_bytes(state, id) {
            Ok(bytes) => respond_as(
                responder,
                StatusCode::OK,
                bytes,
                Some("application/octet-stream"),
            ),
            Err(error) => fail(responder, &error),
        },
        Route::LookTile(id, look) => match crate::looks::look_tile_bytes(state, id, &look) {
            Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
            Err(error) => fail(responder, &error),
        },
        Route::Preview(id, plain) => match preview_bytes(state, id, plain) {
            Ok(bytes) => {
                crate::timing::mark(&crate::timing::FIRST_PREVIEW, "first preview");
                respond(responder, StatusCode::OK, bytes, true);
            }
            Err(error) => fail(responder, &error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_strict() {
        assert!(matches!(
            route("/thumb/7", None),
            Some(Route::Thumb(PhotoId(7)))
        ));
        assert!(matches!(
            route("/preview/0", Some("rev=3")),
            Some(Route::Preview(
                PhotoId(0),
                crate::commands::PreviewPart::All
            ))
        ));
        assert!(matches!(
            route("/preview/4", Some("rev=3&plain=1")),
            Some(Route::Preview(
                PhotoId(4),
                crate::commands::PreviewPart::Plain
            ))
        ));
        assert!(matches!(
            route("/preview/4", Some("rev=3&mask=1")),
            Some(Route::Mask(PhotoId(4)))
        ));
        assert!(matches!(
            route("/look/9", Some("rev=3")),
            Some(Route::Look(PhotoId(9)))
        ));
        assert!(matches!(
            route("/looktile/9", Some("look=builtin:clear&rev=2")),
            Some(Route::LookTile(PhotoId(9), ref l)) if l == "builtin:clear"
        ));
        assert!(route("/looktile/9", Some("look=../etc")).is_none());
        assert!(route("/looktile/9", None).is_none());
        assert!(route("/thumb/../etc", None).is_none());
        assert!(route("/thumb/7/extra", None).is_none());
        assert!(route("/other/1", None).is_none());
        assert!(route("/thumb/-1", None).is_none());
    }

    #[test]
    fn fr_035_sheet_routes_are_strict() {
        assert!(matches!(
            route("/sheet/3/4", Some("rev=2&small=1")),
            Some(Route::Sheet(
                crate::collage::SheetPair {
                    first: PhotoId(3),
                    second: Some(PhotoId(4))
                },
                true,
                false
            ))
        ));
        assert!(matches!(
            route("/sheet/3", Some("rev=2")),
            Some(Route::Sheet(
                crate::collage::SheetPair {
                    first: PhotoId(3),
                    second: None
                },
                false,
                false
            ))
        ));
        // FR-098: `J` on a sheet asks for its clipping.
        assert!(matches!(
            route("/sheet/3/4", Some("rev=2&clip=1")),
            Some(Route::Sheet(_, false, true))
        ));
        assert!(route("/sheet/3/3", None).is_none());
        assert!(route("/sheet/3/4/5", None).is_none());
        assert!(route("/sheet/a/4", None).is_none());
    }

    #[test]
    fn regions_are_checked() {
        assert!(matches!(
            route(
                "/preview/2",
                Some("rev=1&region=0.25,0.5,0.25,0.25&px=1600")
            ),
            Some(Route::Region(PhotoId(2), _, false))
        ));
        // The Before side of a compare view, sharp at 1:1 too (FR-096).
        assert!(matches!(
            route(
                "/preview/2",
                Some("rev=1&plain=1&region=0.25,0.5,0.25,0.25&px=1600")
            ),
            Some(Route::Region(PhotoId(2), _, true))
        ));
        for bad in [
            "region=0.25,0.5,0.25&px=1600",
            "region=0.25,0.5,0.25,0.25",
            "region=0.25,0.5,0.25,0.25&px=-1",
            "region=0.9,0,0.5,0.5&px=100",
            "region=NaN,0,0.5,0.5&px=100",
            "region=a,b,c,d&px=100",
        ] {
            assert!(route("/preview/2", Some(bad)).is_none(), "{bad}");
        }
    }
}
