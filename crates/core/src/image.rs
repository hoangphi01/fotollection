// SPDX-License-Identifier: GPL-3.0-only
//! Minimal 8-bit RGB raster shared by every pipeline stage.

/// Interleaved RGB, 3 bytes per pixel, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl RgbImage {
    /// A black image of the given size.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; pixel_bytes(width, height)],
        }
    }

    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        (data.len() == pixel_bytes(width, height)).then_some(Self {
            width,
            height,
            data,
        })
    }

    /// Long edge in pixels.
    pub fn long_edge(&self) -> u32 {
        self.width.max(self.height)
    }
}

fn pixel_bytes(width: u32, height: u32) -> usize {
    width as usize * height as usize * 3
}

/// The single EXIF orientation that equals applying `orientation` and then turning the
/// result `quarter_turns` × 90° clockwise (FR-006): one pass over the pixels instead of two.
pub fn compose_orientation(orientation: u8, quarter_turns: u8) -> u8 {
    // Rows: EXIF orientation 1–8. Columns: 0–3 clockwise quarter turns.
    const TABLE: [[u8; 4]; 8] = [
        [1, 6, 3, 8],
        [2, 7, 4, 5],
        [3, 8, 1, 6],
        [4, 5, 2, 7],
        [5, 2, 7, 4],
        [6, 3, 8, 1],
        [7, 4, 5, 2],
        [8, 1, 6, 3],
    ];
    let row = if (1..=8).contains(&orientation) {
        orientation - 1
    } else {
        0
    };
    TABLE[usize::from(row)][usize::from(quarter_turns % 4)]
}

/// Applies an EXIF orientation (1–8) so the result is upright. Unknown values are treated as 1.
pub fn apply_orientation(image: RgbImage, orientation: u8) -> RgbImage {
    if !(2..=8).contains(&orientation) {
        return image;
    }
    let (w, h) = (image.width as usize, image.height as usize);
    let swaps = orientation >= 5;
    let (dw, dh) = if swaps { (h, w) } else { (w, h) };
    let mut out = vec![0u8; image.data.len()];

    for dy in 0..dh {
        for dx in 0..dw {
            // Source pixel that lands on (dx, dy) after the transform.
            let (sx, sy) = match orientation {
                2 => (w - 1 - dx, dy),
                3 => (w - 1 - dx, h - 1 - dy),
                4 => (dx, h - 1 - dy),
                5 => (dy, dx),
                6 => (dy, h - 1 - dx),
                7 => (w - 1 - dy, h - 1 - dx),
                _ => (w - 1 - dy, dx), // 8
            };
            let s = (sy * w + sx) * 3;
            let d = (dy * dw + dx) * 3;
            out[d..d + 3].copy_from_slice(&image.data[s..s + 3]);
        }
    }

    // INVARIANT: dimensions are a permutation of the source's, so the length matches.
    RgbImage {
        width: u32::try_from(dw).unwrap_or(image.width),
        height: u32::try_from(dh).unwrap_or(image.height),
        data: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_006_quarter_turns_compose_with_every_exif_orientation() {
        // A 3 × 2 image with distinct pixels.
        let image =
            RgbImage::from_raw(3, 2, (0..18).collect()).unwrap_or_else(|| RgbImage::new(1, 1));
        for orientation in 1..=8 {
            for turns in 0..4 {
                let mut stepwise = apply_orientation(image.clone(), orientation);
                for _ in 0..turns {
                    stepwise = apply_orientation(stepwise, 6);
                }
                let once =
                    apply_orientation(image.clone(), compose_orientation(orientation, turns));
                assert_eq!(once, stepwise, "orientation {orientation}, {turns} turns");
            }
            // Four quarter turns are no turn at all.
            assert_eq!(compose_orientation(orientation, 4), orientation);
        }
    }

    const A: [u8; 3] = [1, 1, 1];
    const B: [u8; 3] = [2, 2, 2];

    /// A 2×1 image: A on the left, B on the right.
    fn ab() -> RgbImage {
        RgbImage::from_raw(2, 1, [A, B].concat()).unwrap_or_else(|| RgbImage::new(0, 0))
    }

    #[test]
    fn fr_002_orientation_6_rotates_clockwise() {
        let out = apply_orientation(ab(), 6);
        assert_eq!((out.width, out.height), (1, 2));
        assert_eq!(out.data, [A, B].concat(), "A ends up on top");
    }

    #[test]
    fn fr_002_orientation_8_rotates_counter_clockwise() {
        let out = apply_orientation(ab(), 8);
        assert_eq!((out.width, out.height), (1, 2));
        assert_eq!(out.data, [B, A].concat(), "B ends up on top");
    }

    #[test]
    fn fr_002_orientation_3_and_2_flip() {
        assert_eq!(apply_orientation(ab(), 3).data, [B, A].concat());
        assert_eq!(apply_orientation(ab(), 2).data, [B, A].concat());
        assert_eq!(apply_orientation(ab(), 4).data, [A, B].concat());
    }

    #[test]
    fn unknown_orientation_is_identity() {
        assert_eq!(apply_orientation(ab(), 0), ab());
        assert_eq!(apply_orientation(ab(), 9), ab());
    }
}
