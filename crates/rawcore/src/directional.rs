// SPDX-License-Identifier: GPL-3.0-only
//! Directional demosaicing for Bayer sensors. At every pixel, how far the picture runs
//! vertically or horizontally is decided first, from the local energy of first and second
//! differences, as Luis Sanz Rodríguez describes for RCD. Green at red and blue sites then
//! comes from each of its four sides (the neighbouring green, corrected by half the change
//! of the site's own colour towards that side, as in Hamilton and Adams' interpolation),
//! weighted against the side's gradient, and the vertical and horizontal estimates are
//! blended by that decision. Red and blue follow from colour differences: along the
//! diagonals at blue and red sites, along their own axis at green ones.
//!
//! RCD's own ratio-corrected green was tried first and scored worse on the zone plate
//! (15.3 dB against Malvar's 17.3) and the Siemens star (32.5 against 37.9 dB here), so this
//! is the variant the score card kept (`examples/charts.rs`, RESULT §25). Written from the
//! descriptions, not from any implementation (ADR-0008).

use rayon::prelude::*;

use crate::demosaic::{CameraRgb, Plane};

/// Keeps the ratios and weights finite on black.
const EPS: f32 = 1e-5;

/// Runs `f(x, y)` for every pixel of rows `margin..height - margin` (columns likewise) into a
/// new plane, in parallel by rows; the border stays 0.
fn map(plane: &Plane, margin: usize, f: impl Fn(usize, usize) -> f32 + Sync) -> Vec<f32> {
    let (w, h) = (plane.width, plane.height);
    let mut out = vec![0f32; w * h];
    out.par_chunks_mut(w)
        .enumerate()
        .filter(|(y, _)| (margin..h - margin).contains(y))
        .for_each(|(y, row)| {
            for (x, v) in row.iter_mut().enumerate().take(w - margin).skip(margin) {
                *v = f(x, y);
            }
        });
    out
}

/// The weight of the vertical estimate at every pixel: how much more the picture changes
/// across columns than across rows around it, from first differences (same colour, one site
/// away on either side) and second differences (the site's own colour, two away), summed
/// over three pixels along each direction. High along a vertical edge.
fn decide(plane: &Plane) -> Vec<f32> {
    let w = plane.width;
    let at = |x: usize, y: usize| plane.data[y * w + x];
    let energy_v = |x: usize, y: usize| {
        let d1 = at(x, y - 1) - at(x, y + 1);
        let d2 = 2.0 * at(x, y) - at(x, y - 2) - at(x, y + 2);
        d1 * d1 + 0.25 * d2 * d2
    };
    let energy_h = |x: usize, y: usize| {
        let d1 = at(x - 1, y) - at(x + 1, y);
        let d2 = 2.0 * at(x, y) - at(x - 2, y) - at(x + 2, y);
        d1 * d1 + 0.25 * d2 * d2
    };
    map(plane, 3, |x, y| {
        let v = energy_v(x, y - 1) + energy_v(x, y) + energy_v(x, y + 1);
        let h = energy_h(x - 1, y) + energy_h(x, y) + energy_h(x + 1, y);
        (h + EPS) / (v + h + 2.0 * EPS)
    })
}

/// Green everywhere: at red and blue sites, each side's green corrected by half the change
/// of the site's own colour towards that side, the sides weighted against their gradients,
/// vertical and horizontal blended by `vertical`.
#[allow(clippy::many_single_char_names)]
fn green(plane: &Plane, vertical: &[f32]) -> Vec<f32> {
    let w = plane.width;
    let at = |x: usize, y: usize| plane.data[y * w + x];
    map(plane, 4, |x, y| {
        if plane.site(x, y) == 1 {
            return at(x, y);
        }
        let side = |g1: f32, c2: f32| g1 + 0.5 * (at(x, y) - c2);
        let n = side(at(x, y - 1), at(x, y - 2));
        let s = side(at(x, y + 1), at(x, y + 2));
        let e = side(at(x + 1, y), at(x + 2, y));
        let west = side(at(x - 1, y), at(x - 2, y));
        // Each side counts less where it changes more.
        let grad = |a: f32, b: f32, d: f32| EPS + (a - b).abs() + d.abs();
        let gn = grad(at(x, y), at(x, y - 2), at(x, y - 1) - at(x, y - 3));
        let gs = grad(at(x, y), at(x, y + 2), at(x, y + 1) - at(x, y + 3));
        let ge = grad(at(x, y), at(x + 2, y), at(x + 1, y) - at(x + 3, y));
        let gw = grad(at(x, y), at(x - 2, y), at(x - 1, y) - at(x - 3, y));
        let v_est = (gs * n + gn * s) / (gn + gs);
        let h_est = (gw * e + ge * west) / (ge + gw);
        // At a red or blue site the four diagonal greens' decisions are steadier than its
        // own; keep whichever is further from undecided.
        let own = vertical[y * w + x];
        let around = 0.25
            * (vertical[(y - 1) * w + x - 1]
                + vertical[(y - 1) * w + x + 1]
                + vertical[(y + 1) * w + x - 1]
                + vertical[(y + 1) * w + x + 1]);
        let k = if (own - 0.5).abs() < (around - 0.5).abs() {
            around
        } else {
            own
        };
        (k * v_est + (1.0 - k) * h_est).max(0.0)
    })
}

/// Red at blue sites and blue at red ones, from the colour differences of the four diagonal
/// neighbours, weighted towards the diagonal that changes less.
fn opposite(plane: &Plane, green: &[f32]) -> Vec<f32> {
    let w = plane.width;
    let at = |x: usize, y: usize| plane.data[y * w + x];
    let g = |x: usize, y: usize| green[y * w + x];
    map(plane, 5, |x, y| {
        if plane.site(x, y) == 1 {
            return 0.0;
        }
        let diff = |xx: usize, yy: usize| at(xx, yy) - g(xx, yy);
        let p = EPS
            + (at(x - 1, y - 1) - at(x + 1, y + 1)).abs()
            + (2.0 * g(x, y) - g(x - 1, y - 1) - g(x + 1, y + 1)).abs();
        let q = EPS
            + (at(x + 1, y - 1) - at(x - 1, y + 1)).abs()
            + (2.0 * g(x, y) - g(x + 1, y - 1) - g(x - 1, y + 1)).abs();
        let along_p = f32::midpoint(diff(x - 1, y - 1), diff(x + 1, y + 1));
        let along_q = f32::midpoint(diff(x + 1, y - 1), diff(x - 1, y + 1));
        g(x, y) + (q * along_p + p * along_q) / (p + q)
    })
}

/// Demosaics the default crop of a balanced [`Plane`], clipping at `clip`.
pub fn directional(plane: &Plane, clip: f32) -> CameraRgb {
    let w = plane.width;
    let at = |x: usize, y: usize| plane.data[y * w + x];
    let green = green(plane, &decide(plane));
    let g = |x: usize, y: usize| green[y * w + x];
    let opposite = opposite(plane, &green);
    // Each red or blue site's own colour and the one estimated there.
    let red_blue = |x: usize, y: usize| -> (f32, f32) {
        match plane.site(x, y) {
            0 => (at(x, y), opposite[y * w + x]),
            2 => (opposite[y * w + x], at(x, y)),
            _ => (0.0, 0.0),
        }
    };
    let [_, _, cw, ch] = plane.crop;
    let mut rgb = vec![0f32; cw * ch * 3];
    rgb.par_chunks_mut(cw * 3).enumerate().for_each(|(r, row)| {
        for col in 0..cw {
            let i = plane.index_of_crop(col, r);
            let (x, y) = (i % w, i / w);
            let gc = g(x, y);
            let (red, blue) = if plane.site(x, y) == 1 {
                // Red and blue at green sites from the colour differences of the two
                // neighbours on the axis that holds each colour.
                let diffs = |xx: usize, yy: usize| {
                    let (rr, bb) = red_blue(xx, yy);
                    (rr - g(xx, yy), bb - g(xx, yy))
                };
                let (n, s) = (diffs(x, y - 1), diffs(x, y + 1));
                let (e, west) = (diffs(x + 1, y), diffs(x - 1, y));
                let (rd, bd) = if plane.site(x + 1, y) == 0 {
                    (f32::midpoint(e.0, west.0), f32::midpoint(n.1, s.1))
                } else {
                    (f32::midpoint(n.0, s.0), f32::midpoint(e.1, west.1))
                };
                (gc + rd, gc + bd)
            } else {
                red_blue(x, y)
            };
            let o = &mut row[col * 3..col * 3 + 3];
            o[0] = red.clamp(0.0, clip);
            o[1] = gc.clamp(0.0, clip);
            o[2] = blue.clamp(0.0, clip);
        }
    });
    CameraRgb {
        width: cw,
        height: ch,
        rgb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfa::Mosaic;
    use crate::demosaic::balance;

    fn develop(w: usize, h: usize, data: Vec<u16>, active: [usize; 4]) -> CameraRgb {
        let Ok(mosaic) = Mosaic::new(
            w,
            h,
            data,
            [0, 1, 1, 2],
            [0.0; 4],
            1000.0,
            active,
            [2, 2, 12, 8],
        ) else {
            panic!("a mosaic");
        };
        directional(&balance(&mosaic, [1.0; 3]), 1.0)
    }

    #[test]
    fn a_flat_field_stays_flat() {
        let out = develop(16, 12, vec![500; 16 * 12], [0, 0, 12, 16]);
        assert!(out.rgb.iter().all(|v| (v - 0.5).abs() < 1e-5));
    }

    /// Only the red sites lit, the active area from row 1 and column 1: still red, so the
    /// pattern counts from the active area (LESSONS: a repeating pattern has an origin).
    #[test]
    fn a_red_field_is_red_behind_an_odd_active_area() {
        let (w, h) = (17, 13);
        let mut data = vec![0u16; w * h];
        for y in (1..h).step_by(2) {
            for x in (1..w).step_by(2) {
                data[y * w + x] = 1000;
            }
        }
        let out = develop(w, h, data, [1, 1, h, w]);
        for px in out.rgb.as_chunks::<3>().0 {
            assert!(px[0] > 0.9 && px[1] < 0.1 && px[2] < 0.1, "{px:?}");
        }
    }
}
