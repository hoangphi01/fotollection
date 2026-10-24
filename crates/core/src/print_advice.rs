// SPDX-License-Identifier: GPL-3.0-only
//! Print-size advice (FR-026): how large a photo prints well, and a verdict for every print
//! size the user can pick — sharp, still acceptable, or pixelated — from the effective DPI of
//! the size's centred crop.

use serde::Serialize;
use specta::Type;

use crate::plan::{DpiBand, effective_dpi};
use crate::print_size::PrintSize;

/// Sharp prints need 300 DPI; below 150 DPI pixels show (PRD §5).
const SHARP_DPI: f64 = 300.0;
const ACCEPTABLE_DPI: f64 = 150.0;
const MM_PER_IN: f64 = 25.4;

/// What a size will look like, for the green / yellow / red badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PrintVerdict {
    /// 300 DPI or more.
    Sharp,
    /// 150–299 DPI: fine at arm's length.
    Acceptable,
    /// Under 150 DPI: pixels show.
    Pixelated,
}

impl From<DpiBand> for PrintVerdict {
    fn from(band: DpiBand) -> Self {
        match band {
            DpiBand::Excellent => Self::Sharp,
            DpiBand::Good | DpiBand::Fair => Self::Acceptable,
            DpiBand::Poor => Self::Pixelated,
        }
    }
}

/// The verdict for one print size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SizeAdvice {
    /// The print size's id, built-in (`"20x30"`) or custom (`"custom-1"`).
    pub id: String,
    pub dpi: u32,
    pub verdict: PrintVerdict,
}

/// A print in millimetres, short edge first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrintMm {
    pub short_mm: u32,
    pub long_mm: u32,
}

/// How large one photo prints well, and the verdict for each size.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrintAdvice {
    /// Upright pixel size of the whole photo.
    pub width: u32,
    pub height: u32,
    pub megapixels: f32,
    /// The whole photo at 300 DPI: the largest sharp print.
    pub sharp_up_to: PrintMm,
    /// The whole photo at 150 DPI: the largest acceptable print.
    pub acceptable_up_to: PrintMm,
    pub sizes: Vec<SizeAdvice>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn at_dpi(width: u32, height: u32, dpi: f64) -> PrintMm {
    let mm = |px: u32| (f64::from(px) / dpi * MM_PER_IN).round() as u32;
    PrintMm {
        short_mm: mm(width.min(height)),
        long_mm: mm(width.max(height)),
    }
}

/// Advice for an upright `width` × `height` photo over `sizes` (id, size).
pub fn advise<'a>(
    width: u32,
    height: u32,
    sizes: impl IntoIterator<Item = (&'a str, PrintSize)>,
) -> PrintAdvice {
    #[allow(clippy::cast_possible_truncation)]
    let megapixels = (f64::from(width) * f64::from(height) / 1e6) as f32;
    PrintAdvice {
        width,
        height,
        megapixels,
        sharp_up_to: at_dpi(width, height, SHARP_DPI),
        acceptable_up_to: at_dpi(width, height, ACCEPTABLE_DPI),
        sizes: sizes
            .into_iter()
            .map(|(id, size)| {
                let (dpi, band) = effective_dpi(width, height, size);
                SizeAdvice {
                    id: id.to_owned(),
                    dpi,
                    verdict: band.into(),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(advice: &PrintAdvice, id: &str) -> Option<PrintVerdict> {
        advice.sizes.iter().find(|s| s.id == id).map(|s| s.verdict)
    }

    #[test]
    fn fr_026_a_26_mp_photo_prints_every_standard_size_sharp() {
        let advice = advise(6240, 4160, PrintSize::ALL.iter().map(|s| (s.id, *s)));
        assert!((advice.megapixels - 25.96).abs() < 0.01);
        // 6240 px at 300 DPI = 528 mm, at 150 DPI = 1057 mm.
        assert_eq!(
            advice.sharp_up_to,
            PrintMm {
                short_mm: 352,
                long_mm: 528
            }
        );
        assert_eq!(advice.acceptable_up_to.long_mm, 1057);
        assert!(
            advice
                .sizes
                .iter()
                .all(|s| s.verdict == PrintVerdict::Sharp)
        );
        assert_eq!(advice.sizes.len(), PrintSize::ALL.len());
    }

    #[test]
    fn fr_026_small_photos_get_yellow_then_red() {
        // 1600 × 1200 (2 MP): 10 × 15 crop 1600 × 1067 over 5.9 in → 271 DPI, acceptable;
        // 20 × 30 → 135 DPI, pixelated.
        let advice = advise(1600, 1200, PrintSize::ALL.iter().map(|s| (s.id, *s)));
        assert_eq!(verdict(&advice, "10x15"), Some(PrintVerdict::Acceptable));
        assert_eq!(verdict(&advice, "20x30"), Some(PrintVerdict::Pixelated));
        // 9 × 13: 1600 px over 5.1 in → 312 DPI, still sharp.
        assert_eq!(verdict(&advice, "9x13"), Some(PrintVerdict::Sharp));
        // Portrait photos give the same verdicts.
        let portrait = advise(1200, 1600, PrintSize::ALL.iter().map(|s| (s.id, *s)));
        assert_eq!(portrait.sizes, advice.sizes);
    }
}
