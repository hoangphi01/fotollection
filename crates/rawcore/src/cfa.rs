// SPDX-License-Identifier: GPL-3.0-only
//! The sensor's mosaic: raw values behind a 2×2 colour filter, with their black and white
//! levels, the active area and the camera's default crop (DNG 1.7 §4, TIFF/EP).

use crate::RawError;

/// Colour of a filter site: 0 red, 1 green, 2 blue.
pub type Site = u8;

#[derive(Debug, Clone)]
pub struct Mosaic {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u16>,
    /// The colours of the 2×2 pattern, row by row, at the image's top-left corner.
    pub pattern: [Site; 4],
    /// Black level per pattern position.
    pub black: [f32; 4],
    pub white: f32,
    /// `[top, left, bottom, right]` of the pixels that saw light.
    pub active: [usize; 4],
    /// `[x, y, width, height]` of the picture inside the active area.
    pub crop: [usize; 4],
}

impl Mosaic {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: usize,
        height: usize,
        data: Vec<u16>,
        pattern: [Site; 4],
        black: [f32; 4],
        white: f32,
        active: [usize; 4],
        crop: [usize; 4],
    ) -> Result<Self, RawError> {
        let [top, left, bottom, right] = active;
        if data.len() != width * height
            || bottom > height
            || right > width
            || top >= bottom
            || left >= right
        {
            return Err(RawError::Corrupt("active area"));
        }
        let [x, y, w, h] = crop;
        if w == 0 || h == 0 || left + x + w > right || top + y + h > bottom {
            return Err(RawError::Corrupt("default crop"));
        }
        let darkest = black.iter().copied().fold(0f32, f32::max);
        if white.is_nan() || white <= darkest {
            return Err(RawError::Corrupt("white level"));
        }
        Ok(Self {
            width,
            height,
            data,
            pattern,
            black,
            white,
            active,
            crop,
        })
    }

    /// The picture's size: the default crop.
    pub fn size(&self) -> (usize, usize) {
        (self.crop[2], self.crop[3])
    }

    /// The picture's top-left corner in the mosaic.
    pub fn origin(&self) -> (usize, usize) {
        (self.active[1] + self.crop[0], self.active[0] + self.crop[1])
    }

    /// The filter colour at mosaic position `(x, y)`.
    pub fn site(&self, x: usize, y: usize) -> Site {
        self.pattern[(y & 1) * 2 + (x & 1)]
    }

    /// The value at `(x, y)` scaled so black is 0 and white 1 (not clipped).
    pub fn level(&self, x: usize, y: usize) -> f32 {
        let i = (y & 1) * 2 + (x & 1);
        (f32::from(self.data[y * self.width + x]) - self.black[i]) / (self.white - self.black[i])
    }
}
