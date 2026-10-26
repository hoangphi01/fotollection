// SPDX-License-Identifier: GPL-3.0-only
//! Measures the disk cache's housekeeping (DATABASE.md §8): how long measuring and pruning
//! take for a cache of N files laid out like the real one (256 folders), to decide when the
//! app can run them. Files are 1 byte, since the walk costs per file, not per byte. Median
//! of 5.
//!
//! `cargo run --release -p fotollection-core --example measure_cache -- [N…]`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use fotollection_core::cache_limit::{prune, usage};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let sizes: Vec<usize> = std::env::args()
        .skip(1)
        .map(|a| a.parse().expect("a count"))
        .collect();
    let sizes = if sizes.is_empty() {
        vec![10_000, 70_000]
    } else {
        sizes
    };
    for n in sizes {
        let dir = tempfile::tempdir().expect("temp dir");
        for i in 0..n {
            let sub = dir.path().join(format!("{:02x}", i % 256));
            std::fs::create_dir_all(&sub).unwrap();
            std::fs::write(sub.join(format!("{i:08x}.t1.jpg")), [0u8]).unwrap();
        }
        let mut walk = Vec::new();
        let mut keep = Vec::new();
        for _ in 0..5 {
            let t = Instant::now();
            let u = usage(&[dir.path()]);
            walk.push(t.elapsed().as_secs_f64() * 1e3);
            assert_eq!(u.files, n as u64);
            // A prune under the limit: the walk plus the `.part` check, nothing deleted.
            let t = Instant::now();
            prune(&[dir.path()], u64::MAX);
            keep.push(t.elapsed().as_secs_f64() * 1e3);
        }
        // Over the limit: every file is sorted by age and 10 % + are deleted.
        let t = Instant::now();
        let pruned = prune(&[dir.path()], n as u64 / 2);
        let cut = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "{n:>6} files: usage {:.0} ms · prune under the limit {:.0} ms · prune to half {:.0} ms ({} removed)",
            median(walk),
            median(keep),
            cut,
            pruned.removed
        );
    }
}
