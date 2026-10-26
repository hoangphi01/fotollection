// SPDX-License-Identifier: GPL-3.0-only
//! The disk cache's size limit (DATABASE.md §8): thumbnails and best-shot analyses are
//! derived data, so the oldest are deleted when the total passes the limit, leftovers of an
//! interrupted write are removed, and *Clear cache* empties it. Every file here can be made
//! again from the photos, so deleting one only costs time.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use walkdir::WalkDir;

/// The cache stays under this (DATABASE.md §8): about 70,000 photos at ~27 KB each.
pub const LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// A prune goes down to this share of the limit, so it does not run again at once.
const AFTER_PRUNE: f64 = 0.9;
/// A `.part` file older than this is a write that never finished.
const STALE_PART: Duration = Duration::from_hours(1);
/// A hit refreshes a file's time only when it is older than this, so browsing does not write.
const TOUCH_AFTER: Duration = Duration::from_hours(24);

/// What the cache holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub bytes: u64,
    pub files: u64,
}

/// What a prune did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pruned {
    pub removed: u64,
    pub freed_bytes: u64,
    pub left: Usage,
}

struct Entry {
    path: PathBuf,
    bytes: u64,
    used: SystemTime,
}

fn entries(dirs: &[&Path]) -> Vec<Entry> {
    dirs.iter()
        .flat_map(|dir| WalkDir::new(dir).follow_links(false).into_iter())
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some(Entry {
                path: e.into_path(),
                bytes: meta.len(),
                used: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            })
        })
        .collect()
}

/// The size of everything in `dirs`.
pub fn usage(dirs: &[&Path]) -> Usage {
    entries(dirs).iter().fold(Usage::default(), |u, e| Usage {
        bytes: u.bytes + e.bytes,
        files: u.files + 1,
    })
}

/// Deletes unfinished `.part` writes, then, while the total is over `limit`, the files used
/// longest ago, down to 90 % of `limit`.
pub fn prune(dirs: &[&Path], limit: u64) -> Pruned {
    let now = SystemTime::now();
    let mut pruned = Pruned::default();
    let mut kept = Vec::new();
    for entry in entries(dirs) {
        let unfinished = entry.path.extension().is_some_and(|e| e == "part")
            && now.duration_since(entry.used).unwrap_or_default() > STALE_PART;
        if unfinished && std::fs::remove_file(&entry.path).is_ok() {
            pruned.removed += 1;
            pruned.freed_bytes += entry.bytes;
        } else {
            kept.push(entry);
        }
    }
    let mut total: u64 = kept.iter().map(|e| e.bytes).sum();
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let target = (limit as f64 * AFTER_PRUNE) as u64;
    if total > limit {
        kept.sort_by_key(|e| e.used);
        kept.retain(|entry| {
            if total <= target || std::fs::remove_file(&entry.path).is_err() {
                return true;
            }
            total -= entry.bytes;
            pruned.removed += 1;
            pruned.freed_bytes += entry.bytes;
            false
        });
    }
    pruned.left = Usage {
        bytes: kept.iter().map(|e| e.bytes).sum(),
        files: kept.len() as u64,
    };
    pruned
}

/// Empties `dirs` (*Clear cache*); the folders themselves are kept.
pub fn clear(dirs: &[&Path]) -> Usage {
    let mut freed = Usage::default();
    for entry in entries(dirs) {
        if std::fs::remove_file(&entry.path).is_ok() {
            freed.bytes += entry.bytes;
            freed.files += 1;
        }
    }
    for dir in dirs {
        if let Ok(subdirs) = std::fs::read_dir(dir) {
            for sub in subdirs.flatten() {
                let _ = std::fs::remove_dir(sub.path());
            }
        }
    }
    freed
}

/// Marks a cache file as used now, at most once a day, so the limit keeps what is browsed.
pub fn touch(path: &Path) {
    let Ok(file) = std::fs::File::options().write(true).open(path) else {
        return;
    };
    let old = file
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age > TOUCH_AFTER);
    if old {
        let _ = file.set_modified(SystemTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(dir: &Path, name: &str, bytes: usize, age_s: u64) -> std::io::Result<PathBuf> {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, vec![0u8; bytes])?;
        let when = SystemTime::now() - Duration::from_secs(age_s);
        std::fs::File::options()
            .write(true)
            .open(&path)?
            .set_modified(when)?;
        Ok(path)
    }

    #[test]
    fn prune_keeps_what_was_used_last_and_drops_unfinished_writes() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let old = file(dir.path(), "aa/old.t1.jpg", 400, 3 * 86_400)?;
        let mid = file(dir.path(), "bb/mid.t1.jpg", 400, 2 * 86_400)?;
        let new = file(dir.path(), "cc/new.t1.jpg", 400, 60)?;
        let stale = file(dir.path(), "aa/half.t1.part", 100, 7200)?;
        let writing = file(dir.path(), "bb/now.t1.part", 100, 10)?;
        assert_eq!(
            usage(&[dir.path()]),
            Usage {
                bytes: 1400,
                files: 5
            }
        );

        // Under the limit: only the stale `.part` goes.
        let pruned = prune(&[dir.path()], 10_000);
        assert_eq!((pruned.removed, pruned.freed_bytes), (1, 100));
        assert!(!stale.exists() && writing.exists());

        // Over it (1,300 > 800): oldest first, down to 90 % of the limit (720 bytes).
        let pruned = prune(&[dir.path()], 800);
        assert_eq!(pruned.removed, 2);
        assert!(!old.exists() && !mid.exists() && new.exists());
        assert_eq!(
            pruned.left,
            Usage {
                bytes: 500,
                files: 2
            }
        );
        Ok(())
    }

    #[test]
    fn touch_moves_an_old_file_to_the_back_of_the_queue() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let browsed = file(dir.path(), "aa/browsed.t1.jpg", 400, 5 * 86_400)?;
        let other = file(dir.path(), "bb/other.t1.jpg", 400, 2 * 86_400)?;
        touch(&browsed);
        prune(&[dir.path()], 500);
        assert!(browsed.exists() && !other.exists());
        Ok(())
    }

    #[test]
    fn clear_empties_the_cache_and_keeps_its_folders() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let thumbs = dir.path().join("thumbs");
        let analysis = dir.path().join("analysis");
        file(&thumbs, "aa/x.t1.jpg", 300, 0)?;
        file(&analysis, "bb/y.json", 50, 0)?;
        assert_eq!(
            clear(&[&thumbs, &analysis]),
            Usage {
                bytes: 350,
                files: 2
            }
        );
        assert_eq!(usage(&[&thumbs, &analysis]), Usage::default());
        assert!(thumbs.is_dir() && analysis.is_dir());
        Ok(())
    }
}
