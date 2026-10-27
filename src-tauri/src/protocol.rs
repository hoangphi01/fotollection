// SPDX-License-Identifier: GPL-3.0-only
//! `fotothumb://` serves thumbnails and previews as JPEG bytes, so pixels never cross
//! IPC as JSON or base64 (ARCHITECTURE.md §6).
//!
//! - `fotothumb://localhost/thumb/{id}`
//! - `fotothumb://localhost/preview/{id}?rev={settings revision}`
//! - `fotothumb://localhost/preview/{id}?rev=…&region={x},{y},{w},{h}&px={width}`: one part
//!   of the full-resolution render, for zoom (FR-057); fractions of the photo, checked here
//! - `fotothumb://localhost/source/{id}?rev=…`
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

pub const SCHEME: &str = "fotothumb";

enum Route {
    Thumb(PhotoId),
    Preview(PhotoId),
    /// A full-resolution region of the preview, for zoom.
    Region(PhotoId, Region),
    /// Upright, uncropped T2 image for the crop editor.
    Source(PhotoId),
    /// The latest live-view frame of the connected camera, from memory (FR-062).
    CameraLive,
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
    let id = PhotoId(parts.next()?.parse().ok()?);
    if parts.next().is_some() {
        return None;
    }
    match kind {
        "thumb" => Some(Route::Thumb(id)),
        "preview" => Some(match region(query).ok()? {
            Some(region) => Route::Region(id, region),
            None => Route::Preview(id),
        }),
        "source" => Some(Route::Source(id)),
        _ => None,
    }
}

fn respond(responder: UriSchemeResponder, status: StatusCode, body: Vec<u8>, jpeg: bool) {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-cache");
    if jpeg {
        builder = builder.header(header::CONTENT_TYPE, "image/jpeg");
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
    // Decode work happens off the webview thread, on the interactive pool, so visible
    // thumbnails and previews never wait behind background thumbnail generation.
    let worker = app.clone();
    app.state::<AppState>().interactive_pool.spawn(move || {
        let app = worker;
        let state = app.state::<AppState>();
        match route {
            Route::Thumb(id) => match thumbnail_bytes(&state, id) {
                Some(bytes) => {
                    crate::timing::mark(&crate::timing::FIRST_THUMB, "first thumbnail");
                    respond(responder, StatusCode::OK, bytes, true);
                }
                None => respond(responder, StatusCode::NOT_FOUND, Vec::new(), false),
            },
            Route::Source(id) => match source_bytes(&state, id) {
                Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
                Err(message) => respond(
                    responder,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message.into_bytes(),
                    false,
                ),
            },
            Route::Region(id, region) => match region_bytes(&state, id, region) {
                Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
                Err(message) => respond(
                    responder,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message.into_bytes(),
                    false,
                ),
            },
            Route::CameraLive => {}
            Route::Preview(id) => match preview_bytes(&state, id) {
                Ok(bytes) => {
                    crate::timing::mark(&crate::timing::FIRST_PREVIEW, "first preview");
                    respond(responder, StatusCode::OK, bytes, true);
                }
                Err(message) => respond(
                    responder,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message.into_bytes(),
                    false,
                ),
            },
        }
    });
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
            Some(Route::Preview(PhotoId(0)))
        ));
        assert!(route("/thumb/../etc", None).is_none());
        assert!(route("/thumb/7/extra", None).is_none());
        assert!(route("/other/1", None).is_none());
        assert!(route("/thumb/-1", None).is_none());
    }

    #[test]
    fn regions_are_checked() {
        assert!(matches!(
            route(
                "/preview/2",
                Some("rev=1&region=0.25,0.5,0.25,0.25&px=1600")
            ),
            Some(Route::Region(PhotoId(2), _))
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
