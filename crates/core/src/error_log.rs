// SPDX-License-Identifier: GPL-3.0-only
//! The offline error log (`docs/ERROR_CONTROL.md` §3, STATUS S7): the app sends nothing,
//! so every error it shows and every panic leaves one line here, for the user to copy into
//! an issue with *Copy diagnostics*. Five files of 1 MB at most, rotated.
//!
//! A line holds the time, the `ERR-` code, the stable code, the version, the system and the
//! error's text with every path cut down to its extension. **Never** image data, GPS or a
//! full path: a file name can say where someone was, so not even that.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// One file grows to this size before it is rotated.
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;
/// `errors.log` and `errors.1.log` … `errors.4.log`.
pub const FILES: usize = 5;
/// The longest error text kept on a line.
const MAX_TEXT: usize = 300;

/// The log of this run, once `init` has named its folder.
static LOG: OnceLock<Log> = OnceLock::new();

struct Log {
    dir: PathBuf,
    version: String,
    /// One writer at a time, so lines never interleave and a rotation is never raced. It
    /// holds the last code and text: the same error again (a camera polled every second)
    /// is written once.
    lock: Mutex<Option<(String, String)>>,
}

/// Opens the log in `dir` for this run, with the app's `version`. A second call is ignored.
pub fn init(dir: PathBuf, version: &str) {
    let _ = LOG.set(Log {
        dir,
        version: version.to_owned(),
        lock: Mutex::new(None),
    });
}

/// The folder the log is kept in, once `init` named it (for *Open folder*, FR-116).
pub fn dir() -> Option<PathBuf> {
    LOG.get().map(|log| log.dir.clone())
}

/// Writes one line for the error `code` (its `ERR-` code from [`crate::errors`]), unless
/// it repeats the previous one. Logging never fails the caller: a log that cannot be
/// written is skipped.
pub fn record(code: &str, text: &str) {
    let Some(log) = LOG.get() else {
        return;
    };
    // A poisoned lock only means a writer panicked mid-line; the next line is still whole.
    let mut previous = log
        .lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if previous
        .as_ref()
        .is_some_and(|(c, t)| c == code && t == text)
    {
        return;
    }
    *previous = Some((code.to_owned(), text.to_owned()));
    let line = line(
        &chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        code,
        &log.version,
        text,
    );
    let _ = append(&log.dir, &line);
}

/// The text *Copy diagnostics* puts on the clipboard: version, system and the last `last`
/// lines, newest last.
pub fn diagnostics(last: usize) -> String {
    let (version, lines) = LOG.get().map_or_else(
        || (String::new(), Vec::new()),
        |log| (log.version.clone(), recent(&log.dir, last)),
    );
    let mut out = format!(
        "Fotollection {version} · {} {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    if lines.is_empty() {
        out.push_str("No errors logged.\n");
    } else {
        out.push_str("Recent errors:\n");
        for l in lines {
            out.push_str(&l);
            out.push('\n');
        }
    }
    out
}

fn line(time: &str, code: &str, version: &str, text: &str) -> String {
    format!(
        "{time} {} {code} v{version} {}/{} {}",
        crate::errors::err_code(code),
        std::env::consts::OS,
        std::env::consts::ARCH,
        redact(text)
    )
}

fn path(dir: &Path, n: usize) -> PathBuf {
    if n == 0 {
        dir.join("errors.log")
    } else {
        dir.join(format!("errors.{n}.log"))
    }
}

fn append(dir: &Path, line: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let current = path(dir, 0);
    let size = fs::metadata(&current).map_or(0, |m| m.len());
    if size > 0 && size + line.len() as u64 + 1 > MAX_FILE_BYTES {
        // The oldest goes; each other file moves one place down.
        let _ = fs::remove_file(path(dir, FILES - 1));
        for n in (0..FILES - 1).rev() {
            let _ = fs::rename(path(dir, n), path(dir, n + 1));
        }
    }
    let mut file = OpenOptions::new().create(true).append(true).open(current)?;
    writeln!(file, "{line}")
}

/// The last `last` lines across the files, oldest first.
fn recent(dir: &Path, last: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for n in 0..FILES {
        let Ok(file) = File::open(path(dir, n)) else {
            break;
        };
        let mut these: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();
        these.append(&mut lines);
        lines = these;
        if lines.len() >= last {
            break;
        }
    }
    let skip = lines.len().saturating_sub(last);
    lines.split_off(skip)
}

/// File kinds whose names are cut to the extension even without a folder in front.
const FILE_KINDS: &[&str] = &[
    "jpg",
    "jpeg",
    "png",
    "heic",
    "heif",
    "dng",
    "nef",
    "cr2",
    "cr3",
    "arw",
    "raf",
    "orf",
    "rw2",
    "pef",
    "srw",
    "tif",
    "tiff",
    "xmp",
    "json",
    "fotollection",
    "ttf",
    "otf",
];

/// The error's text on one line, at most [`MAX_TEXT`] characters, with every path and file
/// name replaced by `<path>` and its extension. A path may hold spaces, so everything from
/// the first word with a `/` or `\` to the last one goes, words between them included.
pub fn redact(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let has_sep = |w: &&str| w.contains('/') || w.contains('\\');
    let ext = |w: &str| -> Option<String> {
        let w = w.trim_end_matches(|c: char| !c.is_alphanumeric());
        let (_, e) = w.rsplit_once('.')?;
        FILE_KINDS
            .contains(&e.to_ascii_lowercase().as_str())
            .then(|| e.to_owned())
    };
    let hidden = |w: &str| ext(w).map_or_else(|| "<path>".to_owned(), |e| format!("<path>.{e}"));
    let first = words.iter().position(has_sep);
    let last = words.iter().rposition(has_sep);
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    for (i, w) in words.iter().enumerate() {
        match (first, last) {
            (Some(a), Some(b)) if (a..b).contains(&i) => {}
            (Some(_), Some(b)) if i == b => out.push(hidden(w)),
            _ if ext(w).is_some() => out.push(hidden(w)),
            _ => out.push((*w).to_owned()),
        }
    }
    out.join(" ").chars().take(MAX_TEXT).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_names_keep_only_their_extension() {
        assert_eq!(
            redact("cannot open /Users/an/My Trip Paris/IMG_1.JPG: denied"),
            "cannot open <path>.JPG denied"
        );
        assert_eq!(
            redact(r"read C:\Users\an\trip\ failed"),
            "read <path> failed"
        );
        assert_eq!(
            redact("IMG_1010.jpg could not be decoded"),
            "<path>.jpg could not be decoded"
        );
        assert_eq!(
            redact("version 0.9.1, layout 4:2:0"),
            "version 0.9.1, layout 4:2:0"
        );
        assert_eq!(redact("line one\nline two"), "line one line two");
        assert_eq!(redact(&"x".repeat(1000)).len(), MAX_TEXT);
    }

    #[test]
    fn a_line_names_both_codes_the_version_and_the_system() {
        let l = line(
            "2026-11-02T10:00:00Z",
            "decode.failed",
            "0.9.1",
            "bad /a/b.png",
        );
        assert!(l.starts_with("2026-11-02T10:00:00Z ERR-IMG-0002 decode.failed v0.9.1 "));
        assert!(l.ends_with(" bad <path>.png"));
        assert!(!l.contains("/a/"));
    }

    /// Five files of 1 MB at most: the oldest lines go, the newest stay readable.
    #[test]
    fn the_log_rotates_and_keeps_the_newest() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let filler = "x".repeat(1000);
        for i in 0..6000 {
            append(dir.path(), &format!("{i} {filler}"))?;
        }
        let total: u64 = (0..=FILES)
            .filter_map(|n| fs::metadata(path(dir.path(), n)).ok())
            .map(|m| m.len())
            .sum();
        assert!(total <= MAX_FILE_BYTES * FILES as u64, "{total}");
        assert!(!path(dir.path(), FILES).exists());
        let last = recent(dir.path(), 3);
        assert_eq!(last.len(), 3);
        assert!(last[2].starts_with("5999 "), "{}", &last[2][..10]);
        assert!(last[0].starts_with("5997 "));
        Ok(())
    }
}
