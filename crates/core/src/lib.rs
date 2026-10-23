// SPDX-License-Identifier: GPL-3.0-only
//! Fotollection core: all image and domain logic, independent of the Tauri shell.
//!
//! See `docs/ARCHITECTURE.md` and ADR-0002 for the crate boundary.

pub mod print_size;

pub use print_size::PrintSize;

/// Version of the core crate, as declared in the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
