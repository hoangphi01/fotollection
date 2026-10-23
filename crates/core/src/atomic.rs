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
    std::fs::rename(&tmp, path)
}
