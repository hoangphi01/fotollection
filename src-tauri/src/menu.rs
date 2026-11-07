// SPDX-License-Identifier: GPL-3.0-only
//! Native menu (macOS) with the File commands of FR-023 and the three modes of FR-048, in
//! the interface language. On Windows and Linux the same shortcuts are handled by the UI, so
//! no menu bar is added to the window.

use fotollection_core::settings::Language;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

use crate::events::MenuAction;
use crate::i18n::strings;

/// Commands the UI runs when their menu item is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    New,
    Open,
    Save,
    SaveAs,
    /// Simple, Collage or Pro (FR-048).
    ModeSimple,
    ModeCollage,
    ModeProfessional,
    /// The OS handed over a `.fotollection` file (not a menu item).
    OpenRequested,
    /// The Settings window (FR-086), Cmd+, on macOS.
    Settings,
    /// Copy, paste and reset a photo's develop settings (FR-097), ⇧⌘C, ⇧⌘V, ⌥⇧⌘V (to the
    /// ticked photos) and ⇧⌘R.
    CopySettings,
    PasteSettings,
    PasteSettingsTicked,
    ResetSettings,
    /// The manual inside the app (FR-110), from the Help menu.
    Manual,
}

impl Action {
    const ALL: [(Self, &'static str); 13] = [
        (Self::New, "new"),
        (Self::Open, "open"),
        (Self::Save, "save"),
        (Self::SaveAs, "save_as"),
        (Self::ModeSimple, "mode_simple"),
        (Self::ModeCollage, "mode_collage"),
        (Self::ModeProfessional, "mode_professional"),
        (Self::Settings, "settings"),
        (Self::CopySettings, "copy_settings"),
        (Self::PasteSettings, "paste_settings"),
        (Self::PasteSettingsTicked, "paste_settings_ticked"),
        (Self::ResetSettings, "reset_settings"),
        (Self::Manual, "manual"),
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
        .separator()
        .item(&item(
            "copy_settings",
            t.copy_settings,
            "CmdOrCtrl+Shift+C",
        )?)
        .item(&item(
            "paste_settings",
            t.paste_settings,
            "CmdOrCtrl+Shift+V",
        )?)
        .item(&item(
            "paste_settings_ticked",
            t.paste_settings_ticked,
            "CmdOrCtrl+Alt+Shift+V",
        )?)
        .item(&item(
            "reset_settings",
            t.reset_settings,
            "CmdOrCtrl+Shift+R",
        )?)
        .build()?;
    let view = SubmenuBuilder::new(app, t.view)
        .item(&item("mode_simple", t.mode_simple, "CmdOrCtrl+1")?)
        .item(&item("mode_collage", t.mode_collage, "CmdOrCtrl+2")?)
        .item(&item(
            "mode_professional",
            t.mode_professional,
            "CmdOrCtrl+3",
        )?)
        .build()?;
    let window = SubmenuBuilder::new(app, t.window)
        .minimize_with_text(t.minimize)
        .close_window_with_text(t.close_window)
        .build()?;
    // The Help menu (FR-110): the manual, offline, in the app's language.
    let help = SubmenuBuilder::new(app, t.help)
        .item(&item("manual", t.manual, "CmdOrCtrl+?")?)
        .build()?;
    let app_menu = SubmenuBuilder::new(app, "Fotollection")
        .about_with_text(t.about, None)
        .separator()
        .item(&item("settings", t.settings, "CmdOrCtrl+,")?)
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
        .items(&[&app_menu, &file, &edit, &view, &window, &help])
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

/// Forwards File and View menu clicks to the UI.
pub fn on_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    if let Some((action, _)) = Action::ALL.iter().find(|(_, id)| event.id() == *id) {
        let _ = MenuAction { action: *action }.emit(app);
    }
}

#[cfg(test)]
mod tests {
    use super::Action;

    /// FR-110: the Help menu's *Fotollection Manual* reaches the UI as `manual`, and every
    /// menu id names one action.
    #[test]
    fn fr_110_the_help_menu_opens_the_manual() {
        assert!(Action::ALL.contains(&(Action::Manual, "manual")));
        let mut ids: Vec<_> = Action::ALL.iter().map(|(_, id)| *id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), Action::ALL.len());
    }
}
