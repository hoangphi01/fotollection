// SPDX-License-Identifier: GPL-3.0-only
//! Standard EU print sizes (CEWE / dm / Rossmann), as listed in `docs/PRD.md` §5.

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
}

impl PrintSize {
    /// All supported sizes, in the order shown to the user.
    pub const ALL: [PrintSize; 8] = [
        PrintSize::new("9x13", 90, 130),
        PrintSize::new("10x15", 100, 150),
        PrintSize::new("11x17", 110, 170),
        PrintSize::new("13x18", 130, 180),
        PrintSize::new("15x20", 150, 200),
        PrintSize::new("20x30", 200, 300),
        PrintSize::new("15x15", 150, 150),
        PrintSize::new("5x15", 50, 150),
    ];

    const fn new(id: &'static str, short_mm: u32, long_mm: u32) -> Self {
        Self {
            id,
            short_mm,
            long_mm,
        }
    }

    /// A user-defined size (FR-043); `id` is only a marker, settings hold the real one.
    pub const fn custom(short_mm: u32, long_mm: u32) -> Self {
        Self {
            id: "custom",
            short_mm,
            long_mm,
        }
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
    fn unknown_print_size_is_none() {
        assert_eq!(PrintSize::by_id("A4"), None);
    }
}
