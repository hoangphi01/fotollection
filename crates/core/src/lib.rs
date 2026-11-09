// SPDX-License-Identifier: GPL-3.0-only
//! Fotollection core: all image and domain logic, independent of the Tauri shell.
//!
//! See `docs/ARCHITECTURE.md` and ADR-0002 for the crate boundary.

pub mod atomic;
pub mod brands;
pub mod bundle;
pub mod cache_limit;
pub mod cancel;
pub mod codec;
pub mod colour;
pub mod colour_ops;
pub mod culling;
pub mod error_log;
pub mod errors;
pub mod exif_out;
pub mod export;
pub mod facts;
pub mod fonts;
pub mod frame;
pub mod geocode;
pub mod histogram;
pub mod history;
pub mod image;
pub mod import;
pub mod jpeg_layout;
pub mod library;
pub mod lock;
pub mod logo;
pub mod look_library;
pub mod look_thumbs;
pub mod looks;
pub mod lut;
pub mod machine;
pub mod metadata;
pub mod naming;
pub mod places;
pub mod plan;
pub mod pool;
pub mod postcard;
pub mod preset;
pub mod preset_look;
pub mod preview;
pub mod print_advice;
pub mod print_size;
pub mod project;
pub mod raw;
pub mod raw_finish;
pub mod render;
pub mod settings;
pub mod settings_bundle;
pub mod settings_store;
pub mod share;
pub mod sharpen;
pub mod sheet;
pub mod stamp;
pub mod straighten;
pub mod supervise;
pub mod text;
pub mod thumbs;
pub mod xmp;
pub mod zip_parts;

pub use print_size::PrintSize;

/// Version of the core crate, as declared in the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
