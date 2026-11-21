// SPDX-License-Identifier: GPL-3.0-only
//! NFR-004, ADR-0060: macOS's RAW develop (`ImageIO`) keeps about 120 MB of dead memory a photo
//! that nothing in process frees, so it runs in a child. Its own `main` (`harness = false`):
//! this executable is also the child, as the app's is.
//!
//! Needs `FOTOLLECTION_RAW_SAMPLES` naming a folder with a Panasonic (`.rw2`) or Olympus
//! (`.orf`) file, the formats macOS develops first; without one it says it skipped.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

#[cfg(target_os = "macos")]
fn main() {
    use fotollection_core::raw::{self, RawAdjust, RawLook};

    if fotollection_core::raw_child::serve() {
        return;
    }
    fotollection_core::raw_child::use_child(std::env::current_exe().unwrap());
    let Some(path) = sample() else {
        println!("os_decode_child: skipped, no .rw2 or .orf in FOTOLLECTION_RAW_SAMPLES");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let kind = raw::kind(&bytes).unwrap();
    let develop = || {
        raw::decode(&bytes, kind, RawLook::Neutral, &RawAdjust::default()).unwrap();
    };

    // (a) No growth: after one develop to warm up, seven more leave this process where it was.
    develop();
    let first = footprint_mb(std::process::id());
    for _ in 0..7 {
        develop();
    }
    let last = footprint_mb(std::process::id());
    println!("os_decode_child: this process {first:.0} MB after 1 develop, {last:.0} MB after 8");
    // It fails, too, if the develops fell back in process.
    assert_eq!(
        fotollection_core::raw_child::served(),
        8,
        "the child did not serve every develop"
    );
    assert!(
        last <= first * 1.10,
        "the develop left memory behind: {first:.0} MB after 1, {last:.0} MB after 8"
    );
    println!("os_decode_child: ok");
}

#[cfg(not(target_os = "macos"))]
fn main() {}

/// The first Panasonic or Olympus file of `FOTOLLECTION_RAW_SAMPLES`.
#[cfg(target_os = "macos")]
fn sample() -> Option<std::path::PathBuf> {
    let dir = std::env::var_os("FOTOLLECTION_RAW_SAMPLES")?;
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("rw2") || e.eq_ignore_ascii_case("orf"))
        })
        .collect();
    files.sort();
    files.into_iter().next()
}

/// A process's memory as macOS counts it (`phys_footprint`, compressed pages included), in MB.
#[cfg(target_os = "macos")]
fn footprint_mb(pid: u32) -> f64 {
    let out = std::process::Command::new("/usr/bin/footprint")
        .args(["-p", &pid.to_string()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("phys_footprint:"))
        .expect("footprint prints phys_footprint");
    let mut parts = line.split_whitespace();
    let value: f64 = parts.next().unwrap().parse().unwrap();
    match parts.next() {
        Some("KB") => value / 1024.0,
        Some("GB") => value * 1024.0,
        Some("B") => value / 1024.0 / 1024.0,
        _ => value,
    }
}
