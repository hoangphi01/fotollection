// SPDX-License-Identifier: GPL-3.0-only
//! After demosaicing: false colour, the coloured dots and fringes a demosaic leaves on grey
//! detail. A 3 × 3 median of each pixel's colour differences (red − green, blue − green)
//! removes an isolated wrong colour and keeps a real colour edge, which spreads over more
//! than half of the window; green, which carries the detail, is left untouched.

use rayon::prelude::*;

use crate::demosaic::CameraRgb;

/// The median of nine values by a fixed network of 19 compare-exchanges (Paeth's), a few
/// times faster than sorting them.
fn median9(mut p: [f32; 9]) -> f32 {
    let mut s = |a: usize, b: usize| {
        if p[a] > p[b] {
            p.swap(a, b);
        }
    };
    for (a, b) in [
        (1, 2),
        (4, 5),
        (7, 8),
        (0, 1),
        (3, 4),
        (6, 7),
        (1, 2),
        (4, 5),
        (7, 8),
        (0, 3),
        (5, 8),
        (4, 7),
        (3, 6),
        (1, 4),
        (2, 5),
        (4, 7),
        (4, 2),
        (6, 4),
        (4, 2),
    ] {
        s(a, b);
    }
    p[4]
}

/// One pass of the median on colour differences, in place.
pub fn suppress(image: &mut CameraRgb) {
    let (w, h) = (image.width, image.height);
    if w < 3 || h < 3 {
        return;
    }
    // The two difference planes once, so each median reads nine values, not 27.
    let plane =
        |c: usize| -> Vec<f32> { image.rgb.par_chunks_exact(3).map(|p| p[c] - p[1]).collect() };
    let (red, blue) = (plane(0), plane(2));
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let ys = [y.saturating_sub(1) * w, y * w, (y + 1).min(h - 1) * w];
            for x in 0..w {
                let xs = [x.saturating_sub(1), x, (x + 1).min(w - 1)];
                let g = row[x * 3 + 1];
                for (c, diff) in [(0, &red), (2, &blue)] {
                    let v: [f32; 9] = std::array::from_fn(|k| diff[ys[k / 3] + xs[k % 3]]);
                    row[x * 3 + c] = (g + median9(v)).max(0.0);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_network_finds_the_median() {
        let mut seed = 7u32;
        for _ in 0..2000 {
            let v: [f32; 9] = std::array::from_fn(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                f32::from(u16::try_from(seed >> 20).unwrap_or(0))
            });
            let mut sorted = v;
            sorted.sort_by(f32::total_cmp);
            assert!((median9(v) - sorted[4]).abs() < f32::EPSILON, "{v:?}");
        }
    }

    /// A lone red pixel on grey goes grey; a red half stays red up to its edge.
    #[test]
    fn a_lone_colour_goes_an_edge_stays() {
        let (w, h) = (9, 9);
        let mut rgb = vec![0.5f32; w * h * 3];
        rgb[(4 * w + 4) * 3] = 0.9;
        let mut image = CameraRgb {
            width: w,
            height: h,
            rgb,
        };
        suppress(&mut image);
        assert!((image.rgb[(4 * w + 4) * 3] - 0.5).abs() < 1e-6);
        let rgb = (0..w * h)
            .flat_map(|i| if i % w < 4 { [0.8, 0.2, 0.2] } else { [0.5; 3] })
            .collect();
        let mut image = CameraRgb {
            width: w,
            height: h,
            rgb,
        };
        suppress(&mut image);
        assert!((image.rgb[(4 * w + 3) * 3] - 0.8).abs() < 1e-6);
        assert!((image.rgb[(4 * w + 4) * 3] - 0.5).abs() < 1e-6);
    }
}
