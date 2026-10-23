// SPDX-License-Identifier: GPL-3.0-only
//! `fotothumb://` serves thumbnails and previews as JPEG bytes, so pixels never cross
//! IPC as JSON or base64 (ARCHITECTURE.md §6).
//!
//! - `fotothumb://localhost/thumb/{id}`
//! - `fotothumb://localhost/preview/{id}?rev={settings revision}`
//!
//! On Windows the webview addresses it as `http://fotothumb.localhost/…`.

// The handler signature is fixed by Tauri.
#![allow(clippy::needless_pass_by_value)]

use fotollection_core::library::PhotoId;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::commands::{preview_bytes, thumbnail_bytes};
use crate::state::AppState;

pub const SCHEME: &str = "fotothumb";

enum Route {
    Thumb(PhotoId),
    Preview(PhotoId),
}

fn route(path: &str) -> Option<Route> {
    let mut parts = path.trim_matches('/').split('/');
    let kind = parts.next()?;
    let id = PhotoId(parts.next()?.parse().ok()?);
    if parts.next().is_some() {
        return None;
    }
    match kind {
        "thumb" => Some(Route::Thumb(id)),
        "preview" => Some(Route::Preview(id)),
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
    let Some(route) = route(request.uri().path()) else {
        respond(responder, StatusCode::NOT_FOUND, Vec::new(), false);
        return;
    };
    // Decode work happens off the webview thread, on the interactive pool, so visible
    // thumbnails and previews never wait behind background thumbnail generation.
    let worker = app.clone();
    app.state::<AppState>().interactive_pool.spawn(move || {
        let app = worker;
        let state = app.state::<AppState>();
        match route {
            Route::Thumb(id) => match thumbnail_bytes(&state, id) {
                Some(bytes) => respond(responder, StatusCode::OK, bytes, true),
                None => respond(responder, StatusCode::NOT_FOUND, Vec::new(), false),
            },
            Route::Preview(id) => match preview_bytes(&state, id) {
                Ok(bytes) => respond(responder, StatusCode::OK, bytes, true),
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
        assert!(matches!(route("/thumb/7"), Some(Route::Thumb(PhotoId(7)))));
        assert!(matches!(
            route("/preview/0"),
            Some(Route::Preview(PhotoId(0)))
        ));
        assert!(route("/thumb/../etc").is_none());
        assert!(route("/thumb/7/extra").is_none());
        assert!(route("/other/1").is_none());
        assert!(route("/thumb/-1").is_none());
    }
}
