// SPDX-License-Identifier: GPL-3.0-only
//! Crash-safe file replacement (DATABASE.md §4.4): temp file in the same directory, fsync,
//! keep the previous version as `.bak`, then an atomic rename.

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn write_with_backup(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    if path.exists() {
        let mut bak = path.as_os_str().to_owned();
        bak.push(".bak");
        std::fs::copy(path, bak)?;
    }
    std::fs::rename(&tmp, path)?;
    sync_dir(path);
    Ok(())
}

/// A file the user made (a font, a logo, a LUT) written whole or not at all, and on the
/// disk before this returns (ADR-0048): a `.part` beside it, fsynced, renamed, then the folder
/// fsynced, so a crash or a power cut never leaves half a file under its real name. No `.bak`:
/// such a file is written once under its name and never changed. Thumbnails keep
/// `thumbs::write_atomic`, without the fsyncs: a cache can be made again.
pub fn write_durable(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = std::path::PathBuf::from(part);
    {
        let mut f = File::create(&part)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&part, path)?;
    sync_dir(path);
    Ok(())
}

/// The folder holding `path`, fsynced, so a rename in it survives a power cut. Best effort:
/// Windows does not open folders as files, and NTFS journals the rename itself.
fn sync_dir(path: &Path) {
    if let Some(dir) = path.parent()
        && let Ok(handle) = File::open(dir)
    {
        let _ = handle.sync_all();
    }
}

/// Removes what a write cut short left behind (`.tmp`, `.part`) in `dir`, at start-up,
/// before anything writes there (DATABASE.md §4.4).
pub fn sweep_leftovers(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name.ends_with(".tmp") || name.ends_with(".part"))
            && entry.file_type().is_ok_and(|t| t.is_file())
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A durable write leaves the file whole and no `.part`; a sweep removes what a cut
    /// write left and keeps the rest.
    #[test]
    fn a_durable_write_and_a_sweep() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let font = dir.path().join("3.ttf");
        write_durable(&font, b"font bytes")?;
        assert_eq!(std::fs::read(&font)?, b"font bytes");
        assert!(!dir.path().join("3.ttf.part").exists());
        std::fs::write(dir.path().join("4.ttf.part"), b"cut")?;
        std::fs::write(dir.path().join("settings.json.tmp"), b"cut")?;
        sweep_leftovers(dir.path());
        assert!(!dir.path().join("4.ttf.part").exists());
        assert!(!dir.path().join("settings.json.tmp").exists());
        assert!(font.exists());
        Ok(())
    }
}
