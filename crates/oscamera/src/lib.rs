// SPDX-License-Identifier: GPL-3.0-only
//! Cameras over USB (FR-062, FR-063, ADR-0026): the operating system's own imaging stack
//! carries PTP (`ImageCaptureCore` on macOS, WPD on Windows), and Fotollection adds
//! its own drivers for each maker's operations, written from our protocol notes in
//! `docs/cameras/`. No driver is installed, and no LGPL library or maker SDK is linked.
//!
//! `ptp` is pure Rust and safe. The platform transport is the only place with `unsafe`
//! (FFI); each block carries a `// SAFETY:` comment (`docs/STYLE_RULE.md` §2).

pub mod ptp;
pub mod vendor;

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub mod macos;

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub mod windows;
