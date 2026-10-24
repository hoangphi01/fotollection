// SPDX-License-Identifier: GPL-3.0-only
//! Fotollection core: all image and domain logic, independent of the Tauri shell.
//!
//! See `docs/ARCHITECTURE.md` and ADR-0002 for the crate boundary.

pub mod atomic;
pub mod brands;
pub mod bundle;
pub mod cancel;
pub mod codec;
pub mod culling;
pub mod export;
pub mod fonts;
pub mod frame;
pub mod geocode;
pub mod history;
pub mod image;
pub mod import;
pub mod library;
pub mod logo;
pub mod metadata;
pub mod plan;
pub mod preview;
pub mod print_size;
pub mod project;
pub mod render;
pub mod settings;
pub mod settings_store;
pub mod stamp;
pub mod straighten;
pub mod text;
pub mod thumbs;

pub use print_size::PrintSize;

/// Version of the core crate, as declared in the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
