// SPDX-License-Identifier: GPL-3.0-only
//! Exports for sharing (FR-053): lighter files at the size a platform shows best, in sRGB,
//! with the camera's EXIF and Fotollection's name but never GPS. Prints are untouched.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::codec::{Chroma, EncodeOptions};

/// Smallest and largest long edge of a custom share size, in pixels.
pub const CUSTOM_EDGE_RANGE: (u32, u32) = (640, 6000);
/// Facebook shows regular photos best at 2048 px on the long edge and does not resize them.
const FACEBOOK_LONG_EDGE: u32 = 2048;
/// Instagram stores 1080 px wide, and portraits at most 3:4 (1080 × 1440).
const INSTAGRAM_WIDTH: u32 = 1080;
const INSTAGRAM_MAX_HEIGHT: u32 = 1440;

/// Where the exported photos are going.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind", content = "longEdge")]
pub enum SharePreset {
    /// Same width and height, only a lighter file (about half a compact-camera JPEG, RESULT §10).
    #[default]
    Original,
    Facebook,
    Instagram,
    /// A long edge chosen by the user, clamped to [`CUSTOM_EDGE_RANGE`].
    Custom(u32),
}

/// What an export makes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind", content = "preset")]
pub enum ExportTarget {
    /// Full-quality prints (FR-008): the print size, the original's profile, no metadata.
    #[default]
    Print,
    Share(SharePreset),
}

impl ExportTarget {
    /// The target with a custom share edge kept in range (files and IPC are not trusted).
    #[must_use]
    pub fn sanitized(self) -> Self {
        match self {
            Self::Share(preset) => Self::Share(preset.sanitized()),
            Self::Print => Self::Print,
        }
    }
}

impl SharePreset {
    /// The preset with a custom edge kept in range.
    #[must_use]
    pub fn sanitized(self) -> Self {
        match self {
            Self::Custom(edge) => {
                Self::Custom(edge.clamp(CUSTOM_EDGE_RANGE.0, CUSTOM_EDGE_RANGE.1))
            }
            other => other,
        }
    }

    /// JPEG settings, chosen by `bench/quality/share_audit.py` (RESULT §10): 4:2:0, which
    /// the platforms recompress to anyway, at quality 88 for resized photos (SSIM 0.979,
    /// below that it drops faster than the file shrinks) and 80 for Original, which keeps
    /// ΔE2000 under 1 at half the file size.
    pub const fn encode_options(self) -> EncodeOptions {
        EncodeOptions {
            quality: match self {
                Self::Original => 80,
                Self::Facebook | Self::Instagram | Self::Custom(_) => 88,
            },
            chroma: Chroma::Half,
            optimize_huffman: true,
        }
    }
}

/// The size, in pixels, at which a `width` × `height` photo is shared. Never larger than
/// the photo itself, and the photo's own aspect ratio is kept.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn target_size(width: u32, height: u32, preset: SharePreset) -> (u32, u32) {
    let (w, h) = (width.max(1), height.max(1));
    let scale = match preset.sanitized() {
        SharePreset::Original => 1.0,
        SharePreset::Facebook => f64::from(FACEBOOK_LONG_EDGE) / f64::from(w.max(h)),
        SharePreset::Custom(edge) => f64::from(edge) / f64::from(w.max(h)),
        SharePreset::Instagram => (f64::from(INSTAGRAM_WIDTH) / f64::from(w))
            .min(f64::from(INSTAGRAM_MAX_HEIGHT) / f64::from(h)),
    }
    .min(1.0);
    let fit = |v: u32| ((f64::from(v) * scale).round() as u32).max(1);
    (fit(w), fit(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_053_facebook_long_edge_is_2048_and_ratio_is_kept() {
        assert_eq!(target_size(6240, 4160, SharePreset::Facebook), (2048, 1365));
        assert_eq!(target_size(4160, 6240, SharePreset::Facebook), (1365, 2048));
        // Never enlarged.
        assert_eq!(target_size(1600, 1200, SharePreset::Facebook), (1600, 1200));
    }

    #[test]
    fn fr_053_instagram_is_1080_wide_and_portraits_stop_at_3_4() {
        assert_eq!(target_size(6000, 4000, SharePreset::Instagram), (1080, 720));
        assert_eq!(
            target_size(3000, 3000, SharePreset::Instagram),
            (1080, 1080)
        );
        // 3:4 portrait fits exactly; a taller 2:3 one is limited by its height.
        assert_eq!(
            target_size(3000, 4000, SharePreset::Instagram),
            (1080, 1440)
        );
        assert_eq!(target_size(4000, 6000, SharePreset::Instagram), (960, 1440));
    }

    #[test]
    fn fr_053_original_keeps_the_size_and_custom_is_clamped() {
        assert_eq!(target_size(6240, 4160, SharePreset::Original), (6240, 4160));
        assert_eq!(
            target_size(6000, 4000, SharePreset::Custom(1600)),
            (1600, 1067)
        );
        assert_eq!(
            SharePreset::Custom(10).sanitized(),
            SharePreset::Custom(640)
        );
        assert_eq!(
            SharePreset::Custom(99_999).sanitized(),
            SharePreset::Custom(6000)
        );
    }

    #[test]
    fn fr_053_original_is_lighter_and_resized_photos_keep_q88() {
        assert_eq!(SharePreset::Original.encode_options().quality, 80);
        for p in [
            SharePreset::Facebook,
            SharePreset::Instagram,
            SharePreset::Custom(1600),
        ] {
            let o = p.encode_options();
            assert_eq!((o.quality, o.chroma), (88, Chroma::Half), "{p:?}");
        }
    }
}
