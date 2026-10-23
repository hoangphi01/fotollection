// SPDX-License-Identifier: GPL-3.0-only
//! Native menu (macOS) with the File commands of FR-023, in the interface language. On
//! Windows the same shortcuts are handled by the UI, so no menu bar is added to the window.

use fotollection_core::settings::Language;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

use crate::events::MenuAction;
use crate::i18n::strings;

/// File commands the UI runs when their menu item is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    New,
    Open,
    Save,
    SaveAs,
    /// The OS handed over a `.fotollection` file (not a menu item).
    OpenRequested,
}

impl Action {
    const ALL: [(Self, &'static str); 4] = [
        (Self::New, "new"),
        (Self::Open, "open"),
        (Self::Save, "save"),
        (Self::SaveAs, "save_as"),
    ];
}

pub fn build<R: Runtime>(app: &AppHandle<R>, language: Language) -> tauri::Result<Menu<R>> {
    let t = strings(language);
    let item = |id: &str, text: &str, accel: &str| {
        MenuItemBuilder::with_id(id, text)
            .accelerator(accel)
            .build(app)
    };
    let file = SubmenuBuilder::new(app, t.file)
        .item(&item("new", t.new, "CmdOrCtrl+N")?)
        .item(&item("open", t.open, "CmdOrCtrl+O")?)
        .separator()
        .item(&item("save", t.save, "CmdOrCtrl+S")?)
        .item(&item("save_as", t.save_as, "CmdOrCtrl+Shift+S")?)
        .build()?;
    // The same predefined items as Tauri's default menu, so text fields keep native
    // undo and copy/paste while the app's own undo stays on Cmd+Z in the library.
    let edit = SubmenuBuilder::new(app, t.edit)
        .undo_with_text(t.undo)
        .redo_with_text(t.redo)
        .separator()
        .cut_with_text(t.cut)
        .copy_with_text(t.copy)
        .paste_with_text(t.paste)
        .select_all_with_text(t.select_all)
        .build()?;
    let window = SubmenuBuilder::new(app, t.window)
        .minimize_with_text(t.minimize)
        .close_window_with_text(t.close_window)
        .build()?;
    let app_menu = SubmenuBuilder::new(app, "Fotollection")
        .about_with_text(t.about, None)
        .separator()
        .services()
        .separator()
        .hide_with_text(t.hide)
        .hide_others_with_text(t.hide_others)
        .show_all_with_text(t.show_all)
        .separator()
        .quit_with_text(t.quit)
        .build()?;
    MenuBuilder::new(app)
        .items(&[&app_menu, &file, &edit, &window])
        .build()
}

/// Rebuilds the menu after a language change. macOS only (see module docs).
pub fn refresh<R: Runtime>(app: &AppHandle<R>, language: Language) {
    if cfg!(target_os = "macos") {
        match build(app, language) {
            Ok(menu) => {
                let _ = app.set_menu(menu);
            }
            Err(e) => eprintln!("could not build the menu: {e}"),
        }
    }
}

/// Forwards File menu clicks to the UI.
pub fn on_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    if let Some((action, _)) = Action::ALL.iter().find(|(_, id)| event.id() == *id) {
        let _ = MenuAction { action: *action }.emit(app);
    }
}
