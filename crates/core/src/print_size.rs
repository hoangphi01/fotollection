// SPDX-License-Identifier: GPL-3.0-only
//! Print sizes: Europe's (CEWE / dm / Rossmann, `docs/PRD.md` §5) and Japan's convenience-store
//! kiosks' (FR-129), each tagged with its market.

use serde::Serialize;
use specta::Type;

/// A physical print size. Dimensions are stored short edge first, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrintSize {
    /// Stable identifier used in settings and IPC, e.g. `"10x15"`.
    pub id: &'static str,
    pub short_mm: u32,
    pub long_mm: u32,
    /// The markets whose shops print it, for the size list's groups (FR-129, FR-132): a size
    /// two markets share to the millimetre keeps one id (Japan's L判 is Korea's 3.5 × 5 in).
    pub markets: &'static [Market],
}

/// Where a print size is sold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Market {
    /// dm, Rossmann, CEWE's kiosks.
    Europe,
    /// the multicopy machines of 7-Eleven, Lawson and `FamilyMart` (checked 2026-11-14 and 15).
    Japan,
    /// Printing Box at CU and GS25, and the labs (Snaps), in inches (checked 2026-11-15).
    Korea,
}

impl PrintSize {
    /// All supported sizes, in the order shown to the user.
    pub const ALL: [PrintSize; 21] = [
        PrintSize::new("9x13", 90, 130),
        PrintSize::new("10x15", 100, 150),
        PrintSize::new("11x17", 110, 170),
        PrintSize::new("13x18", 130, 180),
        PrintSize::new("15x20", 150, 200),
        PrintSize::new("20x30", 200, 300),
        PrintSize::new("10x10", 100, 100),
        PrintSize::new("13x13", 130, 130),
        PrintSize::new("15x15", 150, 150),
        PrintSize::new("20x20", 200, 200),
        PrintSize::new("5x15", 50, 150),
        // Japan's kiosks (FR-129): L判, 2L判, 7-Eleven's and Lawson's squares, the はがき.
        PrintSize::shared("l", 89, 127),
        PrintSize::shared("2l", 127, 178),
        PrintSize::japan("sq89", 89, 89),
        PrintSize::japan("sq127", 127, 127),
        PrintSize::HAGAKI,
        // Korea's (FR-132): Printing Box's 4 × 6 and 6 × 8 in, the labs' 8 × 10 and 4 × 4, the
        // 2 × 6 strip; 3.5 × 5 and 5 × 7 are L判 and 2L判 above.
        PrintSize::KR_4X6,
        PrintSize::korea("6x8", 152, 203),
        PrintSize::korea("8x10", 203, 254),
        PrintSize::KR_4X4,
        PrintSize::korea("2x6", 51, 152),
    ];

    /// Korea's everyday print, 4 × 6 in: a Korean card's landscape size (FR-132).
    pub const KR_4X6: PrintSize = PrintSize::korea("4x6", 102, 152);
    /// The square 4 × 4 in: a Korean card's square size (FR-132).
    pub const KR_4X4: PrintSize = PrintSize::korea("4x4", 102, 102);

    /// Japan Post's はがき, 100 × 148 mm: every Japanese card prints at it (FR-130).
    pub const HAGAKI: PrintSize = PrintSize::japan("hagaki", 100, 148);

    const fn new(id: &'static str, short_mm: u32, long_mm: u32) -> Self {
        Self {
            id,
            short_mm,
            long_mm,
            markets: &[Market::Europe],
        }
    }

    const fn japan(id: &'static str, short_mm: u32, long_mm: u32) -> Self {
        Self {
            id,
            short_mm,
            long_mm,
            markets: &[Market::Japan],
        }
    }

    const fn korea(id: &'static str, short_mm: u32, long_mm: u32) -> Self {
        Self {
            id,
            short_mm,
            long_mm,
            markets: &[Market::Korea],
        }
    }

    const fn shared(id: &'static str, short_mm: u32, long_mm: u32) -> Self {
        Self {
            id,
            short_mm,
            long_mm,
            markets: &[Market::Japan, Market::Korea],
        }
    }

    /// A user-defined size (FR-043); `id` is only a marker, settings hold the real one.
    pub const fn custom(short_mm: u32, long_mm: u32) -> Self {
        Self {
            id: "custom",
            short_mm,
            long_mm,
            markets: &[Market::Europe],
        }
    }

    /// `print_size` value that keeps each photo's own aspect ratio (FR-045).
    pub const ORIGINAL_ID: &'static str = "original";
    /// Short edge of an Original print. A 4:3 photo prints at 10 × 13.3 cm, like a lab's
    /// "digital" format.
    pub const ORIGINAL_SHORT_MM: u32 = 100;

    /// The Original print of a `width` × `height` photo (FR-045): no crop, the photo's own
    /// ratio, 10 cm on the short edge. The long edge is rounded to whole millimetres; the
    /// render plan uses the exact pixel ratio, so the rounding never crops a pixel.
    pub fn original(width: u32, height: u32) -> Self {
        let (short, long) = (width.min(height).max(1), width.max(height).max(1));
        let long_mm = (u64::from(Self::ORIGINAL_SHORT_MM) * u64::from(long) + u64::from(short) / 2)
            / u64::from(short);
        Self {
            id: Self::ORIGINAL_ID,
            short_mm: Self::ORIGINAL_SHORT_MM,
            // Clamped to the custom-size ceiling, so a panorama stays a sane print.
            long_mm: u32::try_from(long_mm)
                .unwrap_or(u32::MAX)
                .clamp(Self::ORIGINAL_SHORT_MM, 1000),
            markets: &[Market::Europe],
        }
    }

    pub fn is_original(self) -> bool {
        self.id == Self::ORIGINAL_ID
    }

    /// Looks up a size by its identifier.
    pub fn by_id(id: &str) -> Option<PrintSize> {
        Self::ALL.iter().copied().find(|size| size.id == id)
    }

    /// Long edge divided by short edge (always ≥ 1).
    pub fn aspect_ratio(self) -> f64 {
        f64::from(self.long_mm) / f64::from(self.short_mm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_006_print_sizes_match_prd_ratios() {
        let ratio = |id| PrintSize::by_id(id).map(PrintSize::aspect_ratio);
        assert_eq!(ratio("10x15"), Some(1.5));
        assert_eq!(ratio("20x30"), Some(1.5));
        assert_eq!(ratio("15x15"), Some(1.0));
        // Square sizes of the CEWE kiosks at dm and Rossmann (PRD §5).
        for id in ["10x10", "13x13", "20x20"] {
            assert_eq!(ratio(id), Some(1.0), "{id}");
        }
        assert_eq!(ratio("5x15"), Some(3.0));
        assert!((ratio("15x20").unwrap_or_default() - 4.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn fr_006_print_sizes_are_unique_and_short_edge_first() {
        for (i, size) in PrintSize::ALL.iter().enumerate() {
            assert!(
                size.short_mm <= size.long_mm,
                "{} is not short-edge first",
                size.id
            );
            assert!(
                PrintSize::ALL[i + 1..]
                    .iter()
                    .all(|other| other.id != size.id),
                "{} is duplicated",
                size.id
            );
        }
    }

    #[test]
    fn fr_129_japans_kiosk_sizes_are_listed() {
        let size = |id| PrintSize::by_id(id).map(|s| (s.short_mm, s.long_mm, s.markets));
        assert_eq!(
            size("l"),
            Some((89, 127, &[Market::Japan, Market::Korea][..]))
        );
        assert_eq!(
            size("2l"),
            Some((127, 178, &[Market::Japan, Market::Korea][..]))
        );
        assert_eq!(size("sq89"), Some((89, 89, &[Market::Japan][..])));
        assert_eq!(size("sq127"), Some((127, 127, &[Market::Japan][..])));
        assert_eq!(size("hagaki"), Some((100, 148, &[Market::Japan][..])));
        assert_eq!(size("10x15").map(|s| s.2), Some(&[Market::Europe][..]));
    }

    #[test]
    fn fr_132_koreas_sizes_are_listed() {
        let size = |id| PrintSize::by_id(id).map(|s| (s.short_mm, s.long_mm, s.markets));
        for (id, short, long) in [
            ("4x6", 102, 152),
            ("6x8", 152, 203),
            ("8x10", 203, 254),
            ("4x4", 102, 102),
            ("2x6", 51, 152),
        ] {
            assert_eq!(size(id), Some((short, long, &[Market::Korea][..])), "{id}");
        }
    }

    #[test]
    fn unknown_print_size_is_none() {
        assert_eq!(PrintSize::by_id("A4"), None);
    }
}
