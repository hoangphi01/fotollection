// SPDX-License-Identifier: GPL-3.0-only
//! The develop view's histogram (FR-098): how many of the photo's pixels fall on each of the
//! 256 levels of red, green, blue and light, and how many are clipped, from the preview's
//! resized photo with its look, never the stamp, frame or border drawn on top.
//!
//! Every `step`th pixel of every `step`th row is counted (OPTIMIZATION §3: within 3 ms on a
//! 1600 px preview, the step chosen by `examples/measure_histogram.rs`). The look runs on the
//! sampled pixels only, through the same table as the preview, so the curves match what is
//! shown. Rows are counted on rayon; nothing is locked while they run.

use rayon::prelude::*;

use crate::image::RgbImage;
use crate::looks::{Table, lookup};

/// Levels per channel.
pub const BINS: usize = 256;
/// A pixel is blown when one of its channels is at or above this, blocked at or below
/// [`LOW`] (Lightroom marks a channel at its end, and a JPEG's noise leaves a level or so).
pub const HIGH: u8 = 254;
pub const LOW: u8 = 1;

/// Counts per level and the clipped pixels, of `samples` pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Histogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    /// Light: Rec. 709 weights on the encoded values, as a camera's histogram shows it.
    pub l: Vec<u32>,
    pub samples: u32,
    /// Pixels with a channel at or above [`HIGH`].
    pub high: u32,
    /// Pixels with a channel at or below [`LOW`].
    pub low: u32,
}

impl Histogram {
    fn empty() -> Self {
        Self {
            r: vec![0; BINS],
            g: vec![0; BINS],
            b: vec![0; BINS],
            l: vec![0; BINS],
            samples: 0,
            high: 0,
            low: 0,
        }
    }

    fn add(mut self, other: &Self) -> Self {
        for (a, b) in [
            (&mut self.r, &other.r),
            (&mut self.g, &other.g),
            (&mut self.b, &other.b),
            (&mut self.l, &other.l),
        ] {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
        self.samples += other.samples;
        self.high += other.high;
        self.low += other.low;
        self
    }

    fn count(&mut self, px: [u8; 3]) {
        let [r, g, b] = px;
        self.r[usize::from(r)] += 1;
        self.g[usize::from(g)] += 1;
        self.b[usize::from(b)] += 1;
        // 54 + 183 + 19 = 256: Rec. 709's 0.2126, 0.7152, 0.0722 in eighths of a level.
        let l = (54 * u32::from(r) + 183 * u32::from(g) + 19 * u32::from(b)) >> 8;
        self.l[l as usize] += 1;
        self.samples += 1;
        if r >= HIGH || g >= HIGH || b >= HIGH {
            self.high += 1;
        }
        if r <= LOW || g <= LOW || b <= LOW {
            self.low += 1;
        }
    }
}

/// Red where a pixel is blown (a channel at or above [`HIGH`]), blue where it is blocked (one
/// at or below [`LOW`]), as Lightroom's `J` shows them (FR-098); blown wins where both.
pub fn mark_clipping(image: &mut RgbImage) {
    let row = image.width as usize * 3;
    if row == 0 {
        return;
    }
    image.data.par_chunks_mut(row).for_each(|line| {
        for px in line.as_chunks_mut::<3>().0 {
            if px.iter().any(|&c| c >= HIGH) {
                *px = [255, 0, 0];
            } else if px.iter().any(|&c| c <= LOW) {
                *px = [0, 64, 255];
            }
        }
    });
}

/// The histogram of `image` seen through `look` (none: as it is), counting every `step`th
/// pixel of every `step`th row (`step` at least 1).
pub fn measure(image: &RgbImage, look: Option<&Table>, step: u32) -> Histogram {
    let step = step.max(1) as usize;
    let (width, height) = (image.width as usize, image.height as usize);
    if width == 0 || height == 0 {
        return Histogram::empty();
    }
    let row = width * 3;
    (0..height)
        .into_par_iter()
        .step_by(step)
        .fold(Histogram::empty, |mut h, y| {
            let line = &image.data[y * row..(y + 1) * row];
            for px in line.as_chunks::<3>().0.iter().step_by(step) {
                h.count(look.map_or(*px, |t| lookup(t, *px)));
            }
            h
        })
        .reduce(Histogram::empty, |a, b| a.add(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), &'static str>;

    fn image(
        width: u32,
        height: u32,
        px: impl Fn(u32, u32) -> [u8; 3],
    ) -> Result<RgbImage, &'static str> {
        let mut data = Vec::with_capacity(width as usize * height as usize * 3);
        for y in 0..height {
            for x in 0..width {
                data.extend_from_slice(&px(x, y));
            }
        }
        RgbImage::from_raw(width, height, data).ok_or("size")
    }

    /// FR-098: every sampled pixel lands once in each channel, and light follows Rec. 709.
    #[test]
    fn fr_098_histogram_counts_each_level() -> TestResult {
        let img = image(4, 2, |x, _| if x < 2 { [255, 0, 0] } else { [0, 0, 255] })?;
        let h = measure(&img, None, 1);
        assert_eq!(h.samples, 8);
        assert_eq!((h.r[255], h.r[0], h.b[255]), (4, 4, 4));
        assert_eq!(h.l[(54 * 255) >> 8], 4, "pure red is dark in light");
        assert_eq!(h.l[(19 * 255) >> 8], 4);
        Ok(())
    }

    /// FR-098: blown and blocked pixels are counted apart; a mid grey is neither.
    #[test]
    fn fr_098_histogram_sees_clipping() -> TestResult {
        let img = image(3, 1, |x, _| {
            [[255, 200, 180], [0, 30, 40], [128, 128, 128]][x as usize]
        })?;
        let h = measure(&img, None, 1);
        assert_eq!((h.high, h.low), (1, 1));
        Ok(())
    }

    /// FR-098: `J` marks blown pixels red and blocked ones blue, and leaves the rest alone.
    #[test]
    fn fr_098_clip_marks_only_clipped_pixels() -> TestResult {
        let mut img = image(3, 1, |x, _| {
            [[255, 200, 180], [0, 30, 40], [128, 128, 128]][x as usize]
        })?;
        mark_clipping(&mut img);
        assert_eq!(img.data, vec![255, 0, 0, 0, 64, 255, 128, 128, 128]);
        Ok(())
    }

    /// FR-098: with a step, only every step-th pixel of every step-th row is counted, and the
    /// look's table is applied to what is counted.
    #[test]
    fn fr_098_histogram_samples_and_follows_the_look() -> TestResult {
        let img = image(10, 10, |_, _| [100, 100, 100])?;
        assert_eq!(measure(&img, None, 2).samples, 25);
        let brighter = Table::from_fn(|[r, g, b]| {
            [(r * 1.5).min(1.0), (g * 1.5).min(1.0), (b * 1.5).min(1.0)]
        });
        let h = measure(&img, Some(&brighter), 1);
        assert_eq!(h.r[100], 0, "the look moved every pixel");
        assert_eq!(h.samples, 100);
        Ok(())
    }
}
