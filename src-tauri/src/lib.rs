// SPDX-License-Identifier: GPL-3.0-only
//! Thin Tauri shell: adapts IPC commands, events and the image protocol to
//! `fotollection-core` (ADR-0002).

mod cache;
mod camera;
mod collage;
mod commands;
mod culling;
#[cfg(debug_assertions)]
mod demo;
mod document;
mod events;
mod i18n;
mod looks;
mod menu;
mod open_with;
mod places;
mod prefetch;
mod protocol;
mod settings_copy;
mod state;
mod timing;
mod updates;

use tauri::{DragDropEvent, Manager, WindowEvent};
use tauri_specta::{Builder, Event as _, collect_commands, collect_events};

/// Path of the generated TypeScript bindings, relative to this crate.
pub const BINDINGS_PATH: &str = "../ui/src/ipc/bindings.ts";

/// Builds the IPC surface. Shared by the app and the bindings export test.
// One line per command: the list grows with the app, and splitting it would hide it.
#[allow(clippy::too_many_lines)]
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_info,
            commands::startup_finished,
            open_with::take_photo_request,
            open_with::take_startup_notices,
            document::document_requested,
            updates::check_for_update,
            updates::open_update_download,
            updates::check_for_update_at_start,
            updates::open_update_page,
            updates::open_issue_page,
            updates::open_website,
            updates::diagnostics,
            updates::report_ui_error,
            cache::cache_usage,
            cache::clear_cache,
            cache::memory_info,
            cache::machine_info,
            cache::suggest_memory_settings,
            commands::list_print_sizes,
            commands::pick_and_import_folder,
            commands::refresh_library,
            commands::list_photos,
            commands::get_settings,
            commands::set_settings,
            commands::pick_export_folder,
            commands::start_export,
            commands::export_name_example,
            commands::open_export_folder,
            commands::estimate_export,
            commands::prefetch_previews,
            commands::cancel_job,
            commands::photo_details,
            commands::print_advice,
            commands::get_frame_config,
            commands::set_frame_settings,
            commands::set_camera_profile,
            commands::list_cameras,
            commands::set_camera_clock,
            commands::clock_offset_between,
            commands::clock_match_candidates,
            commands::pick_camera_logo,
            commands::clear_camera_logo,
            commands::get_crop_geometry,
            commands::set_photo_crop,
            commands::get_ui_settings,
            document::get_document,
            document::new_document,
            document::use_settings_as_defaults,
            document::remove_folder,
            document::move_folder,
            document::open_document,
            document::open_requested_document,
            document::save_document,
            document::save_document_as,
            document::relink_folder,
            document::rename_document,
            commands::set_ui_settings,
            commands::set_photo_rating,
            commands::set_photo_flag,
            commands::set_photo_queue,
            collage::list_sheets,
            collage::set_collage_order,
            collage::turn_sheet,
            collage::set_collage_crop,
            collage::rotate_collage_photo,
            collage::auto_turn_collage,
            collage::sheet_info,
            collage::postcard_cuts,
            collage::postcard_text_gaps,
            collage::apply_postcard_text,
            collage::recent_addresses,
            collage::remember_postcard_address,
            collage::forget_address,
            places::search_places,
            places::set_photo_date,
            places::set_photo_place,
            places::list_places,
            places::rename_place,
            places::find_borrowed_places,
            places::accept_borrowed_places,
            commands::rotate_photos,
            commands::get_photo_stamp,
            commands::set_photo_stamp,
            commands::set_photo_caption,
            commands::set_postcard_text,
            commands::get_postcard_text,
            commands::apply_stamp_to_all,
            commands::get_photo_look,
            commands::set_photo_look,
            commands::apply_look_to_all,
            commands::set_sheet_look,
            looks::list_looks,
            looks::save_look,
            looks::delete_look,
            looks::import_lut,
            looks::import_presets,
            looks::look_batch_plan,
            looks::apply_look_to_photos,
            looks::rename_look,
            looks::update_look,
            looks::move_look,
            commands::set_raw_look,
            commands::apply_raw_look_to_all,
            commands::reset_photo_stamps,
            commands::undo,
            commands::redo,
            commands::photo_histogram,
            commands::pick_white_balance,
            collage::pick_white_balance_on_sheet,
            commands::photo_history,
            commands::restore_photo_state,
            settings_copy::copy_photo_settings,
            settings_copy::copied_settings,
            settings_copy::paste_photo_settings,
            settings_copy::reset_photo_settings,
            commands::list_fonts,
            commands::import_font,
            commands::remove_font,
            culling::analyze_library,
            culling::list_groups,
            culling::keep_best,
            culling::keep_one,
            camera::camera_connect,
            camera::camera_disconnect,
            camera::camera_capture,
            camera::camera_step,
            camera::camera_set_format,
            camera::camera_sessions,
            camera::save_camera_session,
            camera::discard_camera_session,
            camera::quit_app,
        ])
        .events(collect_events![
            events::LibraryChanged,
            events::ThumbReady,
            events::ExportProgress,
            events::ExportFinished,
            events::TaskProgress,
            events::RevisionChanged,
            events::DocumentChanged,
            events::MenuAction,
            events::AnalysisFinished,
            events::StartupFinished,
            open_with::OpenPhoto,
            camera::CameraStatus,
            camera::QuitBlocked,
            camera::CameraFrame,
            camera::CameraSettingsChanged,
            camera::CameraShot,
            camera::CameraShotFailed,
        ])
}

/// Exports TypeScript bindings for all commands, events and types.
pub fn export_bindings() -> Result<(), String> {
    specta_builder()
        .export(
            specta_typescript::Typescript::default()
                .header("// GENERATED by tauri-specta. Do not edit; run `cargo test -p fotollection`.\n// SPDX-License-Identifier: GPL-3.0-only\n"),
            BINDINGS_PATH,
        )
        .map_err(|e| e.to_string())
}

/// Menu language, theme and, on Windows, the app-drawn title bar (FR-020, FR-052).
fn prepare_window(app: &tauri::AppHandle) {
    let ui = app
        .state::<state::AppState>()
        .ui
        .read()
        .map(|u| u.clone())
        .unwrap_or_default();
    menu::refresh(app, ui.language);
    commands::apply_window_theme(app, ui.theme);
    // Windows: the app draws its own title bar in its colours, like VS Code.
    #[cfg(windows)]
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(true);
    }
}

/// Autosave (NFR-013): the project at most ~1 s after the last edit, the settings within a
/// quarter second, atomically.
fn spawn_autosave(handle: tauri::AppHandle) {
    // For the app's life, started again if it panics (ADR-0048): a dead autosave saved
    // nothing for the rest of the run, without a word.
    fotollection_core::supervise::forever("autosave", move || {
        let mut tick = 0u8;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));
            tick = tick.wrapping_add(1);
            let state = handle.state::<state::AppState>();
            state.flush_settings();
            if tick.is_multiple_of(4)
                && state
                    .project_dirty
                    .swap(false, std::sync::atomic::Ordering::Relaxed)
            {
                state.save_project();
            }
        }
    });
}

/// The app's context, generated by Tauri; its own code starts threads clippy would refuse.
#[allow(clippy::disallowed_methods)]
fn context() -> tauri::Context {
    tauri::generate_context!()
}

/// Every command inside `catch_unwind` (ADR-0048): a panic in one, many run on the main
/// thread, answers that call with `ERR-RUS-0005` instead of quitting the app. The panic hook
/// has already logged it.
fn contained<R: tauri::Runtime>(
    handler: impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    move |invoke| {
        let resolver = invoke.resolver.clone();
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(invoke))).unwrap_or_else(
            |_| {
                resolver.reject(crate::state::IpcError::new("panic", "the command failed"));
                true
            },
        )
    }
}

/// What follows the session's restore: the start finished, the cache pruned, the memory
/// watched, then the command-line paths and those macOS handed over before setup ran.
fn start_the_rest(handle: &tauri::AppHandle, requested: Vec<std::path::PathBuf>) {
    commands::finish_startup(handle);
    cache::prune_in_background(handle);
    // The previews give their memory back when the machine runs short (FR-091).
    cache::watch_memory(handle);
    for path in requested.into_iter().chain(open_with::take_early()) {
        open_with::route(handle, path);
    }
}

/// Every panic leaves an `ERR-RUS-0005` line in the error log before the default hook runs
/// (STATUS S7). Release builds are stripped, so the line holds the panic's message and
/// where it happened in the source, its last folders joined by `::` so the log's path
/// redaction keeps it (it is the build machine's path, never the user's).
fn log_panics() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let at = info.location().map_or_else(String::new, |l| {
            let parts: Vec<&str> = l.file().split(['/', '\\']).collect();
            let tail = parts[parts.len().saturating_sub(4)..].join("::");
            format!("{tail}:{}", l.line())
        });
        let payload = info.payload();
        let what = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_owned();
        fotollection_core::error_log::record("panic", &format!("at {at} ({thread}): {what}"));
        default(info);
    }));
}

pub fn run() {
    timing::start();
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    if let Err(e) = export_bindings() {
        eprintln!("failed to export TypeScript bindings: {e}");
    }

    let result = tauri::Builder::default()
        // First, as the plugin asks: a second launch hands its paths to this instance (FR-054).
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            open_with::focus(app);
            for path in open_with::from_args(argv) {
                open_with::route(app, path);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        // Opens the update installer in the browser, from Rust only (FR-061); the webview gets
        // no opener permission.
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(contained(builder.invoke_handler()))
        .register_asynchronous_uri_scheme_protocol(protocol::SCHEME, protocol::handle)
        .setup(move |app| {
            builder.mount_events(app);
            // The offline error log, first, so every later error and panic is in it.
            fotollection_core::error_log::init(
                app.path().app_data_dir()?.join("logs"),
                env!("CARGO_PKG_VERSION"),
            );
            log_panics();
            let thumb_dir = app.path().app_cache_dir()?.join("thumbs");
            let config_dir = app.path().app_config_dir()?;
            let project_path = app
                .path()
                .app_data_dir()?
                .join("projects")
                .join("session.fotollection");
            // Dev-only demo dump for the website: its own state, never the user's (demo.rs).
            #[cfg(debug_assertions)]
            let demo = demo::requested();
            #[cfg(debug_assertions)]
            let (thumb_dir, config_dir, project_path) = match &demo {
                Some((_, out)) => demo::state_dirs(out),
                None => (thumb_dir, config_dir, project_path),
            };
            app.manage(state::AppState::new(thumb_dir, &config_dir, project_path)?);
            // Camera sessions live next to the session project (ADR-0028).
            app.manage(camera::CameraState::new(
                app.path().app_data_dir()?.join("camera"),
            ));
            prepare_window(app.handle());

            spawn_autosave(app.handle().clone());
            // Reopen the last session: re-scan folders and restore edits (FR-023). Files
            // passed on the command line (Windows/Linux "Open with") come afterwards: a
            // project through the UI's unsaved-changes flow, a photo as its folder's tab.
            #[cfg(debug_assertions)]
            if let Some((folder, out)) = demo {
                commands::finish_startup(app.handle());
                let handle = app.handle().clone();
                fotollection_core::supervise::spawn(
                    "demo",
                    move || demo::run(&handle, folder, &out),
                    || {},
                );
                return Ok(());
            }
            let handle = app.handle().clone();
            let requested = open_with::from_args(std::env::args_os());
            // A restore that panics still finishes the start (ADR-0048): before, the splash
            // left on its timer and the memory watcher, the prune and Open with never started.
            let ended = handle.clone();
            fotollection_core::supervise::spawn(
                "startup",
                move || {
                    commands::restore_session(&handle);
                    start_the_rest(&handle, requested);
                },
                move || start_the_rest(&ended, Vec::new()),
            );
            // Dev-only: FOTOLLECTION_IMPORT=<folder> imports at startup, for quick iteration.
            #[cfg(debug_assertions)]
            if let Some(folder) = std::env::var_os("FOTOLLECTION_IMPORT") {
                let handle = app.handle().clone();
                fotollection_core::supervise::spawn(
                    "dev-import",
                    move || {
                        let _ =
                            commands::import_paths(&handle, vec![std::path::PathBuf::from(folder)]);
                    },
                    || {},
                );
            }
            timing::mark(&timing::SETUP_DONE, "setup done");
            Ok(())
        })
        .on_page_load(|_, payload| {
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) {
                timing::mark(&timing::PAGE_LOADED, "page loaded");
            }
        })
        .on_menu_event(|app, event| menu::on_event(app, &event))
        .on_window_event(on_window_event)
        .build(context());

    match result {
        Ok(app) => app.run(on_run_event),
        Err(e) => {
            eprintln!("fotollection failed to start: {e}");
            std::process::exit(1);
        }
    }
}

/// Window events: dropped folders, and the closing question (ADR-0028).
fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    // Dropped paths come from the OS, not from the webview (SECURITY.md §2).
    if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
        let app = window.app_handle().clone();
        let paths = paths.clone();
        fotollection_core::supervise::spawn(
            "drop-import",
            move || {
                let _ = commands::import_paths(&app, paths);
            },
            || {},
        );
    }
    // Shots only Fotollection holds: ask before the window closes (ADR-0028).
    if let WindowEvent::CloseRequested { api, .. } = event
        && let Some(blocked) = camera::should_block_quit(window.app_handle())
    {
        api.prevent_close();
        let _ = blocked.emit(window.app_handle());
    }
}

/// App events: documents opened from Finder (macOS) and the closing question on Cmd+Q.
fn on_run_event(handle: &tauri::AppHandle, event: tauri::RunEvent) {
    // macOS delivers double-clicked documents as an event (file association).
    #[cfg(target_os = "macos")]
    if let tauri::RunEvent::Opened { urls } = &event {
        for url in urls {
            if let Ok(path) = url.to_file_path() {
                open_with::route(handle, path);
            }
        }
    }
    // Cmd+Q and the Quit menu: the same question as closing the window.
    if let tauri::RunEvent::ExitRequested { api, code, .. } = &event
        && code.is_none()
        && let Some(blocked) = camera::should_block_quit(handle)
    {
        api.prevent_exit();
        let _ = blocked.emit(handle);
    }
    // A setting changed in the last quarter second is written before the app goes.
    if let tauri::RunEvent::Exit = &event {
        handle.state::<state::AppState>().flush_settings();
    }
    let _ = (handle, event);
}

#[cfg(test)]
mod tests {
    /// CI runs this and then `git diff --exit-code` to catch stale bindings.
    #[test]
    fn export_typescript_bindings() {
        assert_eq!(super::export_bindings(), Ok(()));
    }
}
