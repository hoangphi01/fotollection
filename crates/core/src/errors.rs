// SPDX-License-Identifier: GPL-3.0-only
//! The `ERR-<KIND>-<NNNN>` code of every error the app can show (`docs/ERROR_CONTROL.md`,
//! the single source of truth). The app is offline and sends nothing, so this code is the
//! only clue a user can give us: it travels with every IPC error and goes into the log.
//! Numbers are permanent; a code no longer raised keeps its row as retired.

/// Each stable Rust error code (the key the interface translates) and its `ERR-` code, in
/// the order of the dictionary.
pub const CODES: &[(&str, &str)] = &[
    ("font.invalid", "ERR-CFG-0001"),
    ("font.missing_digits", "ERR-CFG-0002"),
    ("font.too_large", "ERR-CFG-0003"),
    ("font.too_many", "ERR-CFG-0004"),
    ("font.unsupported", "ERR-CFG-0005"),
    ("logo.decode_failed", "ERR-CFG-0006"),
    ("logo.not_png", "ERR-CFG-0007"),
    ("logo.too_large", "ERR-CFG-0008"),
    ("settings.invalid", "ERR-CFG-0009"),
    ("camera.bad_answer", "ERR-DRV-0001"),
    ("camera.failed", "ERR-DRV-0002"),
    ("camera.refused", "ERR-DRV-0003"),
    ("camera.unavailable_platform", "ERR-DRV-0004"),
    ("camera.unsupported", "ERR-DRV-0005"),
    ("export.cancelled", "ERR-EXP-0001"),
    ("export.empty", "ERR-EXP-0002"),
    ("export.no_destination", "ERR-EXP-0003"),
    ("import.heic_codec_missing", "ERR-EXT-0001"),
    ("import.heic_linux", "ERR-EXT-0002"),
    ("import.raw_codec_missing", "ERR-EXT-0003"),
    ("camera.delete_failed", "ERR-FS-0001"),
    ("camera.not_imported", "ERR-FS-0002"),
    ("camera.save_failed", "ERR-FS-0003"),
    ("export.write_failed", "ERR-FS-0004"),
    ("font.unreadable", "ERR-FS-0005"),
    ("font.write_failed", "ERR-FS-0006"),
    ("import.unreadable", "ERR-FS-0007"),
    ("io.read_failed", "ERR-FS-0008"),
    ("logo.unreadable", "ERR-FS-0009"),
    ("logo.write_failed", "ERR-FS-0010"),
    ("project.rename_failed", "ERR-FS-0011"),
    ("project.write_failed", "ERR-FS-0012"),
    ("camera.already_connected", "ERR-HDW-0001"),
    ("camera.busy", "ERR-HDW-0002"),
    ("camera.finishing", "ERR-HDW-0003"),
    ("camera.no_shot", "ERR-HDW-0004"),
    ("camera.not_connected", "ERR-HDW-0005"),
    ("camera.not_found", "ERR-HDW-0006"),
    ("camera.still_connected", "ERR-HDW-0007"),
    ("camera.timeout", "ERR-HDW-0008"),
    ("camera.unplugged", "ERR-HDW-0009"),
    ("culling.analysis_failed", "ERR-IMG-0001"),
    ("decode.failed", "ERR-IMG-0002"),
    ("import.suspicious", "ERR-IMG-0003"),
    ("import.too_large", "ERR-IMG-0004"),
    ("import.unsupported", "ERR-IMG-0005"),
    ("decode.truncated", "ERR-IMG-0006"),
    ("camera.bad_destination", "ERR-IPC-0001"),
    ("camera.bad_format", "ERR-IPC-0002"),
    ("camera.bad_step", "ERR-IPC-0003"),
    ("camera.bad_time_zone", "ERR-IPC-0004"),
    ("camera.destination_open", "ERR-IPC-0005"),
    ("camera.not_session", "ERR-IPC-0006"),
    ("camera.offset_too_large", "ERR-IPC-0007"),
    ("camera.same_clock", "ERR-IPC-0008"),
    ("camera.unknown", "ERR-IPC-0009"),
    ("culling.empty", "ERR-IPC-0010"),
    ("culling.invalid_groups", "ERR-IPC-0011"),
    ("culling.unknown_group", "ERR-IPC-0012"),
    ("folder.unknown", "ERR-IPC-0013"),
    ("photo.invalid_date", "ERR-IPC-0014"),
    ("photo.invalid_rating", "ERR-IPC-0015"),
    ("photo.invalid_selection", "ERR-IPC-0016"),
    ("photo.unknown", "ERR-IPC-0017"),
    ("place.bad_window", "ERR-IPC-0018"),
    ("place.query_too_long", "ERR-IPC-0019"),
    ("place.unknown", "ERR-IPC-0020"),
    ("project.unknown_folder", "ERR-IPC-0021"),
    ("postcard.text_invalid", "ERR-IPC-0022"),
    ("update.none", "ERR-NET-0001"),
    ("update.offline", "ERR-NET-0002"),
    ("update.unreadable", "ERR-NET-0003"),
    ("project.corrupt", "ERR-PRJ-0001"),
    ("project.invalid_name", "ERR-PRJ-0002"),
    ("project.name_taken", "ERR-PRJ-0003"),
    ("project.newer_version", "ERR-PRJ-0004"),
    ("project.not_a_project", "ERR-PRJ-0005"),
    ("project.too_large", "ERR-PRJ-0006"),
    ("culling.model_failed", "ERR-RUS-0001"),
    ("encode.failed", "ERR-RUS-0002"),
    ("internal", "ERR-RUS-0003"),
    ("render.failed", "ERR-RUS-0004"),
    ("panic", "ERR-RUS-0005"),
];

/// The `ERR-` code for a stable Rust error code. Every code the app raises has a row (the
/// tests below hold the code and the dictionary together); an unknown one reads as an
/// internal error.
pub fn err_code(code: &str) -> &'static str {
    CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or("ERR-RUS-0003", |(_, e)| e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The dictionary's rows: `ERR-` code → Rust code, retired rows left out.
    fn dictionary() -> BTreeMap<String, String> {
        let text =
            std::fs::read_to_string(root().join("docs/ERROR_CONTROL.md")).unwrap_or_default();
        text.lines()
            .filter(|l| l.starts_with("| `ERR-") && !l.contains("Retired"))
            .filter_map(|l| {
                let cells: Vec<&str> = l.split(" | ").collect();
                let err = cells.first()?.trim_start_matches("| ").trim_matches('`');
                let code = cells.get(2)?.trim_matches('`');
                Some((err.to_owned(), code.to_owned()))
            })
            .collect()
    }

    #[test]
    fn every_code_has_its_row_in_the_dictionary_and_back() {
        let dict = dictionary();
        let table: BTreeMap<String, String> = CODES
            .iter()
            .map(|(c, e)| ((*e).to_owned(), (*c).to_owned()))
            .collect();
        assert_eq!(table.len(), CODES.len(), "an ERR- code is used twice");
        let codes: std::collections::BTreeSet<_> = CODES.iter().map(|(c, _)| c).collect();
        assert_eq!(codes.len(), CODES.len(), "a Rust code has two ERR- codes");
        assert_eq!(table, dict, "errors.rs and docs/ERROR_CONTROL.md differ");
    }

    #[test]
    fn every_error_raised_in_the_source_has_a_code() {
        // The literals that name an error: `IpcError::new("…"` and `CameraIssue::new("…"`
        // (the code may sit on the next line), and the `=> "kind.name"` arms of the core's
        // `code()` functions. This file names them in prose, so it is skipped.
        let mut missing = Vec::new();
        for dir in ["crates", "src-tauri/src"] {
            visit(&root().join(dir), &mut |path, text| {
                if path.ends_with("core/src/errors.rs") {
                    return;
                }
                for code in error_literals(text) {
                    if !CODES.iter().any(|(c, _)| *c == code) {
                        missing.push(format!("{}: {code}", path.display()));
                    }
                }
            });
        }
        assert!(
            missing.is_empty(),
            "errors without an ERR- code (add a row to docs/ERROR_CONTROL.md and CODES): {missing:#?}"
        );
    }

    fn error_literals(text: &str) -> Vec<String> {
        let mut found = Vec::new();
        for marker in ["IpcError::new(", "CameraIssue::new(", "=> "] {
            let mut rest = text;
            while let Some(at) = rest.find(marker) {
                let tail = rest[at + marker.len()..].trim_start();
                if let Some(quoted) = tail.strip_prefix('"') {
                    let code: String = quoted.chars().take_while(|c| *c != '"').collect();
                    let is_code = code.contains('.')
                        && code
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_');
                    if is_code {
                        found.push(code);
                    }
                }
                rest = &rest[at + marker.len()..];
            }
        }
        found
    }

    fn visit(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|n| n != "target" && n != "fixtures")
                {
                    visit(&path, f);
                }
            } else if path.extension().is_some_and(|e| e == "rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                f(&path, &text);
            }
        }
    }

    #[test]
    fn an_unknown_code_reads_as_internal() {
        assert_eq!(err_code("decode.truncated"), "ERR-IMG-0006");
        assert_eq!(err_code("no.such_code"), "ERR-RUS-0003");
        assert_eq!(err_code("internal"), "ERR-RUS-0003");
    }
}
