// SPDX-License-Identifier: GPL-3.0-only
//! Start-up milestones for the cold-start budget (NFR-010), printed to stderr only when
//! `FOTOLLECTION_TIMING` is set, so `scripts/measure-startup.sh` can read them. Nothing is
//! recorded or sent anywhere.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static START: OnceLock<Option<Instant>> = OnceLock::new();

/// Starts the clock; the first call wins. Call it first thing in `run`.
pub fn start() {
    START.get_or_init(|| std::env::var_os("FOTOLLECTION_TIMING").map(|_| Instant::now()));
}

/// Prints `label` with the time since `start`, once per `flag`.
pub fn mark(flag: &AtomicBool, label: &str) {
    let Some(Some(start)) = START.get() else {
        return;
    };
    if !flag.swap(true, Ordering::Relaxed) {
        eprintln!(
            "timing: {label} {:.0} ms",
            start.elapsed().as_secs_f64() * 1e3
        );
    }
}

pub static PAGE_LOADED: AtomicBool = AtomicBool::new(false);
pub static SETUP_DONE: AtomicBool = AtomicBool::new(false);
pub static STARTUP_DONE: AtomicBool = AtomicBool::new(false);
pub static FIRST_THUMB: AtomicBool = AtomicBool::new(false);
pub static FIRST_PREVIEW: AtomicBool = AtomicBool::new(false);
