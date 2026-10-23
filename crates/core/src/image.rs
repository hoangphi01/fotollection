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
