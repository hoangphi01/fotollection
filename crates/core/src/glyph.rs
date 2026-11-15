// SPDX-License-Identifier: GPL-3.0-only
//! The font layer every text of the app is drawn through (FR-134): faces, their metrics,
//! glyphs and their coverage. One module, so the engine under it can change without touching
//! the stamp, the info frame or the postcards; `examples/text_parity.rs` compares two engines.
pub use ab_glyph::{Font, FontArc, FontRef, GlyphId, PxScale, ScaleFont, VariableFont, point};
