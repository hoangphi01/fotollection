// SPDX-License-Identifier: GPL-3.0-only
//! Folder discovery and file validation: SECURITY.md §3, Layer 1 (FR-001).

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;
use specta::Type;
use walkdir::WalkDir;

/// Files larger than this are rejected before any parsing.
pub const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;

/// Extensions we look at at all. Other files (sidecars, videos, documents) are ignored silently.
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "heic", "heif", "hif", "png", "tif", "tiff", "webp", "dng", "rw2", "cr2",
    "cr3", "nef", "arw", "raf", "orf",
];

/// Byte patterns that must never appear in the head of an image (polyglot detection).
const SUSPICIOUS_MARKERS: &[&[u8]] = &[b"<script", b"<html", b"<svg", b"<?php"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sniffed {
    Jpeg,
    Png,
    Heif,
    Unknown,
}

/// Identifies a file by its magic bytes, never by its extension.
pub fn sniff(head: &[u8]) -> Sniffed {
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Sniffed::Jpeg
    } else if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Sniffed::Png
    } else if head.len() >= 12 && &head[4..8] == b"ftyp" {
        Sniffed::Heif
    } else {
        Sniffed::Unknown
    }
}

fn looks_like_polyglot(head: &[u8]) -> bool {
    let lower: Vec<u8> = head.iter().map(u8::to_ascii_lowercase).collect();
    SUSPICIOUS_MARKERS
        .iter()
        .any(|marker| lower.windows(marker.len()).any(|w| w == *marker))
}

/// An accepted file. Paths are absolute, sizes in bytes, `mtime` in Unix seconds.
#[derive(Debug, Clone)]
pub struct ImportedFile {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RejectReason {
    /// Not a format this build can read yet (HEIC, RAW…).
    Unsupported,
    TooLarge,
    Unreadable,
    /// Magic bytes don't match, or the head contains script/markup (possible polyglot).
    Suspicious,
}

impl RejectReason {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unsupported => "import.unsupported",
            Self::TooLarge => "import.too_large",
            Self::Unreadable => "import.unreadable",
            Self::Suspicious => "import.suspicious",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RejectedFile {
    pub path: PathBuf,
    pub reason: RejectReason,
}

#[derive(Debug, Default)]
pub struct ScanResult {
    pub accepted: Vec<ImportedFile>,
    pub rejected: Vec<RejectedFile>,
}

fn has_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| IMAGE_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|n| n.starts_with('.'))
}

/// Checks one file: size, magic bytes and polyglot markers.
pub fn check_file(path: &Path) -> Result<ImportedFile, RejectReason> {
    let metadata = std::fs::metadata(path).map_err(|_| RejectReason::Unreadable)?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(RejectReason::TooLarge);
    }
    let mut head = [0u8; 4096];
    let n = File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .map_err(|_| RejectReason::Unreadable)?;
    let head = &head[..n];
    if looks_like_polyglot(head) {
        return Err(RejectReason::Suspicious);
    }
    match sniff(head) {
        Sniffed::Jpeg | Sniffed::Png => {}
        Sniffed::Heif => return Err(RejectReason::Unsupported),
        Sniffed::Unknown => {
            // An image extension whose bytes are not an image we know: RAW, or spoofed.
            return Err(if is_raw_extension(path) {
                RejectReason::Unsupported
            } else {
                RejectReason::Suspicious
            });
        }
    }
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    Ok(ImportedFile {
        path: path.to_path_buf(),
        size: metadata.len(),
        mtime,
    })
}

fn is_raw_extension(path: &Path) -> bool {
    const RAW: &[&str] = &[
        "dng", "rw2", "cr2", "cr3", "nef", "arw", "raf", "orf", "tif", "tiff",
    ];
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| RAW.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// Recursively scans `root`, skipping hidden files and folders and not following symlinks.
pub fn scan(root: &Path) -> ScanResult {
    let mut result = ScanResult::default();
    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_hidden(e.file_name()));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file() || !has_image_extension(path) {
            continue;
        }
        match check_file(path) {
            Ok(file) => result.accepted.push(file),
            Err(reason) => result.rejected.push(RejectedFile {
                path: path.to_path_buf(),
                reason,
            }),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap_or_default();
        path
    }

    #[test]
    fn fr_001_sniffs_by_magic_bytes() {
        assert_eq!(sniff(&[0xFF, 0xD8, 0xFF, 0xE1]), Sniffed::Jpeg);
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\n...."), Sniffed::Png);
        assert_eq!(sniff(b"\0\0\0\x18ftypheic"), Sniffed::Heif);
        assert_eq!(sniff(b"<html>"), Sniffed::Unknown);
    }

    #[test]
    fn fr_001_accepts_jpeg_and_png_by_magic_bytes() -> Result<(), std::io::Error> {
        let dir = tempfile::tempdir()?;
        write(dir.path(), "real.jpg", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]);
        write(dir.path(), "screen.png", b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR");
        write(dir.path(), "fake.jpg", b"<html><script>alert(1)</script>");
        write(dir.path(), "phone.heic", b"\0\0\0\x18ftypheic\0\0\0\0");
        write(dir.path(), "notes.txt", b"ignored entirely");
        write(dir.path(), ".hidden.jpg", &[0xFF, 0xD8, 0xFF]);

        let result = scan(dir.path());
        let mut names: Vec<_> = result
            .accepted
            .iter()
            .filter_map(|f| f.path.file_name()?.to_str())
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["real.jpg", "screen.png"]);

        let mut rejected: Vec<_> = result
            .rejected
            .iter()
            .filter_map(|r| Some((r.path.file_name()?.to_str()?, r.reason)))
            .collect();
        rejected.sort_unstable();
        assert_eq!(
            rejected,
            [
                ("fake.jpg", RejectReason::Suspicious),
                ("phone.heic", RejectReason::Unsupported)
            ]
        );
        Ok(())
    }

    #[test]
    fn fr_001_detects_polyglot_jpeg() -> Result<(), std::io::Error> {
        let dir = tempfile::tempdir()?;
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xFE];
        bytes.extend_from_slice(b"<ScRiPt>steal()</script>");
        let path = write(dir.path(), "poly.jpg", &bytes);
        assert_eq!(check_file(&path).err(), Some(RejectReason::Suspicious));
        Ok(())
    }
}
