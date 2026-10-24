// SPDX-License-Identifier: GPL-3.0-only
//! Bundled camera-brand logos for the info frame (FR-040, ADR-0020). Each is a rendering of
//! a public-domain file on Wikimedia Commons; the marks remain their owners' trademarks
//! (`crates/core/assets/brands/BRANDS.md`). Logos are matched to a photo through the EXIF
//! make, and recoloured for a black frame or the mono style.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::logo::{LogoError, LogoImage, decode_png};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Brand {
    Apple,
    Canon,
    Fujifilm,
    Hasselblad,
    Leica,
    Lumix,
    Nikon,
    PhaseOne,
    Ricoh,
    Samsung,
    Sony,
}

/// How a brand logo is coloured (FR-040).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LogoStyle {
    /// Black wordmarks with the brands' small colour accents kept (Fujifilm's red "I"),
    /// and the colour badges and logos as they are (Leica, Nikon, Ricoh, Phase One), like
    /// a classic "Shot on" frame. Black parts turn white on a black frame.
    #[default]
    Classic,
    /// The brand's own colours; dark parts turn white on a black frame.
    Original,
    /// One colour like the frame text: black on white, white on black (badges kept).
    Mono,
}

/// How the classic style colours one brand.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Classic {
    /// Every pixel in the text colour (Canon's red and Samsung's blue wordmarks).
    WholeMark,
    /// Only neutral pixels (black, grey) in the text colour; colour accents stay.
    NeutralOnly,
    /// The brand's colours (Ricoh, Phase One, and the badges).
    Keep,
}

/// Below this luma (0–255) a colour is too dark to read on a black frame.
const DARK_LUMA: f32 = 64.0;
/// Mono logos on a white frame use the frame's text black.
const MONO_DARK: [u8; 3] = [17, 17, 17];

impl Brand {
    /// The brand of a camera from its EXIF make ("NIKON CORPORATION", "Panasonic", …).
    pub fn from_make(make: &str) -> Option<Self> {
        let m = make.to_ascii_lowercase();
        let has = |s: &str| m.contains(s);
        Some(if has("canon") {
            Self::Canon
        } else if has("nikon") {
            Self::Nikon
        } else if has("sony") {
            Self::Sony
        } else if has("fujifilm") || has("fuji photo") {
            Self::Fujifilm
        } else if has("panasonic") || has("lumix") {
            Self::Lumix
        } else if has("leica") {
            Self::Leica
        } else if has("hasselblad") {
            Self::Hasselblad
        } else if has("apple") {
            Self::Apple
        } else if has("samsung") {
            Self::Samsung
        } else if has("ricoh") {
            Self::Ricoh
        } else if has("phase one") {
            Self::PhaseOne
        } else {
            return None;
        })
    }

    fn png(self) -> &'static [u8] {
        match self {
            Self::Apple => include_bytes!("../assets/brands/apple.png"),
            Self::Canon => include_bytes!("../assets/brands/canon.png"),
            Self::Fujifilm => include_bytes!("../assets/brands/fujifilm.png"),
            Self::Hasselblad => include_bytes!("../assets/brands/hasselblad.png"),
            Self::Leica => include_bytes!("../assets/brands/leica.png"),
            Self::Lumix => include_bytes!("../assets/brands/lumix.png"),
            Self::Nikon => include_bytes!("../assets/brands/nikon.png"),
            Self::PhaseOne => include_bytes!("../assets/brands/phaseone.png"),
            Self::Ricoh => include_bytes!("../assets/brands/ricoh.png"),
            Self::Samsung => include_bytes!("../assets/brands/samsung.png"),
            Self::Sony => include_bytes!("../assets/brands/sony.png"),
        }
    }

    fn classic(self) -> Classic {
        match self {
            Self::Canon | Self::Samsung | Self::Hasselblad => Classic::WholeMark,
            Self::Apple | Self::Fujifilm | Self::Lumix | Self::Sony => Classic::NeutralOnly,
            Self::Leica | Self::Nikon | Self::PhaseOne | Self::Ricoh => Classic::Keep,
        }
    }

    /// Badges carry their own background (Leica's red disc, Nikon's yellow square), so they
    /// are never recoloured.
    fn is_badge(self) -> bool {
        matches!(self, Self::Leica | Self::Nikon)
    }
}

/// A pixel with little colour: black, white or grey.
const NEUTRAL_SATURATION: f32 = 0.2;

/// Recolours the logo's pixels in place for the style and the frame colour; alpha is kept.
fn recolor(logo: &mut LogoImage, brand: Brand, dark_frame: bool, style: LogoStyle) {
    let text = if dark_frame { [255; 3] } else { MONO_DARK };
    let whole = match style {
        LogoStyle::Mono => true,
        LogoStyle::Classic => brand.classic() == Classic::WholeMark,
        LogoStyle::Original => false,
    };
    let neutral_only = style == LogoStyle::Classic && brand.classic() == Classic::NeutralOnly;
    for px in logo.rgba.as_chunks_mut::<4>().0 {
        let [r, g, b, _] = *px;
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let saturation = if max == 0 {
            0.0
        } else {
            f32::from(max - min) / f32::from(max)
        };
        let luma = 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
        let recolour = whole
            || (neutral_only && saturation < NEUTRAL_SATURATION)
            // Whatever stays in colour must still read on a black frame.
            || (dark_frame && luma < DARK_LUMA);
        if recolour {
            px[..3].copy_from_slice(&text);
        }
    }
}

type Cache = Mutex<HashMap<(Brand, bool, LogoStyle), Arc<LogoImage>>>;

/// The brand's logo ready to draw on a white (`dark_frame = false`) or black frame,
/// decoded and recoloured once per process.
pub fn logo(brand: Brand, dark_frame: bool, style: LogoStyle) -> Result<Arc<LogoImage>, LogoError> {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cache = CACHE.get_or_init(Cache::default);
    let key = (brand, dark_frame, style);
    if let Some(hit) = cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
        return Ok(hit);
    }
    let mut image = decode_png(brand.png())?;
    if !brand.is_badge() {
        recolor(&mut image, brand, dark_frame, style);
    }
    let image = Arc::new(image);
    if let Ok(mut c) = cache.lock() {
        c.insert(key, Arc::clone(&image));
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Brand; 11] = [
        Brand::Apple,
        Brand::Canon,
        Brand::Fujifilm,
        Brand::Hasselblad,
        Brand::Leica,
        Brand::Lumix,
        Brand::Nikon,
        Brand::PhaseOne,
        Brand::Ricoh,
        Brand::Samsung,
        Brand::Sony,
    ];

    #[test]
    fn fr_040_exif_makes_map_to_brands() {
        for (make, brand) in [
            ("Canon", Brand::Canon),
            ("NIKON CORPORATION", Brand::Nikon),
            ("SONY", Brand::Sony),
            ("FUJIFILM", Brand::Fujifilm),
            ("Panasonic", Brand::Lumix),
            ("LEICA CAMERA AG", Brand::Leica),
            ("Hasselblad", Brand::Hasselblad),
            ("Apple", Brand::Apple),
            ("samsung", Brand::Samsung),
            ("RICOH IMAGING COMPANY, LTD.", Brand::Ricoh),
            ("Phase One", Brand::PhaseOne),
        ] {
            assert_eq!(Brand::from_make(make), Some(brand), "{make}");
        }
        assert_eq!(Brand::from_make("OLYMPUS IMAGING CORP."), None);
    }

    #[test]
    fn fr_040_every_bundled_logo_decodes() {
        for brand in ALL {
            let logo = logo(brand, false, LogoStyle::Original).map_err(|e| e.to_string());
            assert!(
                logo.as_ref().is_ok_and(|l| l.width > 0 && l.height > 0),
                "{brand:?} {logo:?}"
            );
        }
    }

    /// The darkest visible pixel's luma.
    fn darkest(logo: &LogoImage) -> f32 {
        logo.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 128)
            .map(|p| 0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2]))
            .fold(255.0, f32::min)
    }

    /// Some clearly coloured, visible pixel.
    fn has_colour(logo: &LogoImage) -> bool {
        logo.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[3] > 200 && p[0].max(p[1]).max(p[2]) - p[0].min(p[1]).min(p[2]) > 80)
    }

    #[test]
    fn fr_040_dark_frames_get_readable_logos_and_badges_stay() {
        // Sony is black: white on a black frame, black when mono on white.
        let sony_dark =
            logo(Brand::Sony, true, LogoStyle::Original).unwrap_or_else(|_| unreachable!());
        assert!(darkest(&sony_dark) > 200.0);
        let sony_mono =
            logo(Brand::Sony, false, LogoStyle::Mono).unwrap_or_else(|_| unreachable!());
        assert!(darkest(&sony_mono) < 30.0);
        // Classic: Canon's red wordmark turns black, Fujifilm keeps its red accent.
        let canon =
            logo(Brand::Canon, false, LogoStyle::Classic).unwrap_or_else(|_| unreachable!());
        assert!(!has_colour(&canon));
        let fuji =
            logo(Brand::Fujifilm, false, LogoStyle::Classic).unwrap_or_else(|_| unreachable!());
        assert!(has_colour(&fuji));
        let canon_original =
            logo(Brand::Canon, false, LogoStyle::Original).unwrap_or_else(|_| unreachable!());
        assert!(has_colour(&canon_original));
        // Nikon's black lettering on its yellow square is kept on any frame.
        let nikon = logo(Brand::Nikon, true, LogoStyle::Mono).unwrap_or_else(|_| unreachable!());
        assert!(darkest(&nikon) < 30.0);
    }
}
