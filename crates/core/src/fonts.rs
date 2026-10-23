// SPDX-License-Identifier: GPL-3.0-only
//! User fonts (FR-042): OTF/TTF files imported for the date stamp and the info frame.
//! Font files are untrusted input: magic bytes and a size cap are checked, and the file is
//! parsed by the pure-Rust `ab_glyph`/`ttf-parser` before use (SECURITY.md §3).

use ab_glyph::{Font, FontArc};
use serde::{Deserialize, Serialize};
use specta::Type;

pub const MAX_FONT_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_USER_FONTS: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FontFormat {
    Ttf,
    Otf,
}

impl FontFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ttf => "ttf",
            Self::Otf => "otf",
        }
    }
}

/// A row of the `fonts` table in `settings.json` (DATABASE.md §6). The file lives at
/// `config/fonts/<id>.<ttf|otf>`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserFont {
    pub id: u32,
    /// Display name (defaults to the file name).
    pub name: String,
    pub format: FontFormat,
}

/// A parsed font ready for rendering, compared by `id`.
#[derive(Clone)]
pub struct LoadedFont {
    pub id: u32,
    pub font: FontArc,
}

impl std::fmt::Debug for LoadedFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LoadedFont({})", self.id)
    }
}

impl PartialEq for LoadedFont {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FontError {
    #[error("only TrueType (.ttf) and OpenType (.otf) fonts are supported")]
    Unsupported,
    #[error("the font file is larger than 20 MB")]
    TooLarge,
    #[error("the font could not be read")]
    Invalid,
    #[error("the font has no digits, so it can't draw dates")]
    MissingDigits,
}

impl FontError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported => "font.unsupported",
            Self::TooLarge => "font.too_large",
            Self::Invalid => "font.invalid",
            Self::MissingDigits => "font.missing_digits",
        }
    }
}

/// Identifies TTF/OTF by magic bytes. Collections (`ttcf`) and WOFF are not accepted.
pub fn sniff(bytes: &[u8]) -> Option<FontFormat> {
    match bytes.get(..4)? {
        [0, 1, 0, 0] | b"true" => Some(FontFormat::Ttf),
        b"OTTO" => Some(FontFormat::Otf),
        _ => None,
    }
}

/// Validates and parses a font file.
pub fn load(bytes: Vec<u8>) -> Result<(FontFormat, FontArc), FontError> {
    let format = sniff(&bytes).ok_or(FontError::Unsupported)?;
    if bytes.len() > MAX_FONT_BYTES {
        return Err(FontError::TooLarge);
    }
    let font = FontArc::try_from_vec(bytes).map_err(|_| FontError::Invalid)?;
    if ('0'..='9').any(|c| font.glyph_id(c).0 == 0) {
        return Err(FontError::MissingDigits);
    }
    Ok((format, font))
}

#[cfg(test)]
mod tests {
    use super::*;

    static INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");

    #[test]
    fn fr_042_loads_a_real_ttf() {
        assert!(matches!(load(INTER.to_vec()), Ok((FontFormat::Ttf, _))));
    }

    #[test]
    fn fr_042_rejects_other_formats_and_garbage() {
        assert_eq!(
            load(b"wOFF0000".to_vec()).err(),
            Some(FontError::Unsupported)
        );
        assert_eq!(
            load(b"ttcf0000".to_vec()).err(),
            Some(FontError::Unsupported)
        );
        assert_eq!(
            load(b"OTTOgarbage".to_vec()).err(),
            Some(FontError::Invalid)
        );
    }
}
