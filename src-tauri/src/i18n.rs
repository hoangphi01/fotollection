// SPDX-License-Identifier: GPL-3.0-only
//! The few strings the shell shows itself: native menu, window title, file dialogs
//! (FR-020). Everything else is translated in the UI (ADR-0015).

use fotollection_core::settings::Language;

pub struct Strings {
    pub file: &'static str,
    pub new: &'static str,
    pub open: &'static str,
    pub save: &'static str,
    pub save_as: &'static str,
    pub edit: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub cut: &'static str,
    pub copy: &'static str,
    pub paste: &'static str,
    pub select_all: &'static str,
    pub window: &'static str,
    pub minimize: &'static str,
    pub close_window: &'static str,
    pub about: &'static str,
    pub hide: &'static str,
    pub hide_others: &'static str,
    pub show_all: &'static str,
    pub quit: &'static str,
    pub untitled: &'static str,
    pub edited: &'static str,
    pub project_filter: &'static str,
    pub open_title: &'static str,
    pub save_title: &'static str,
    pub relink_title: &'static str,
    /// Save all of a camera session (FR-063).
    pub camera_save_title: &'static str,
}

const EN: Strings = Strings {
    file: "File",
    new: "New Project",
    open: "Open…",
    save: "Save",
    save_as: "Save As…",
    edit: "Edit",
    undo: "Undo",
    redo: "Redo",
    cut: "Cut",
    copy: "Copy",
    paste: "Paste",
    select_all: "Select All",
    window: "Window",
    minimize: "Minimize",
    close_window: "Close Window",
    about: "About Fotollection",
    hide: "Hide Fotollection",
    hide_others: "Hide Others",
    show_all: "Show All",
    quit: "Quit Fotollection",
    untitled: "Untitled",
    edited: "Edited",
    project_filter: "Fotollection project",
    open_title: "Open a project",
    save_title: "Save the project",
    relink_title: "Locate the folder",
    camera_save_title: "Save the photos to",
};

const VI: Strings = Strings {
    file: "Tệp",
    new: "Dự án mới",
    open: "Mở…",
    save: "Lưu",
    save_as: "Lưu thành…",
    edit: "Sửa",
    undo: "Hoàn tác",
    redo: "Làm lại",
    cut: "Cắt",
    copy: "Sao chép",
    paste: "Dán",
    select_all: "Chọn tất cả",
    window: "Cửa sổ",
    minimize: "Thu nhỏ",
    close_window: "Đóng cửa sổ",
    about: "Giới thiệu Fotollection",
    hide: "Ẩn Fotollection",
    hide_others: "Ẩn ứng dụng khác",
    show_all: "Hiện tất cả",
    quit: "Thoát Fotollection",
    untitled: "Chưa đặt tên",
    edited: "Đã sửa",
    project_filter: "Dự án Fotollection",
    open_title: "Mở dự án",
    save_title: "Lưu dự án",
    relink_title: "Tìm thư mục",
    camera_save_title: "Lưu ảnh vào",
};

pub fn strings(language: Language) -> &'static Strings {
    match language {
        Language::En => &EN,
        Language::Vi => &VI,
    }
}
