// SPDX-License-Identifier: GPL-3.0-only
//! Before demosaicing, on the balanced plane: hot pixels and the green imbalance. Both are
//! faults of one site that demosaicing would spread to its neighbours (a coloured cross, a
//! maze), so they are fixed while each site still holds only what it measured.

use rayon::prelude::*;

use crate::demosaic::Plane;

/// A site this many times brighter than every neighbour, plus [`HOT_FLOOR`], is hot.
const HOT_RATIO: f32 = 2.0;
/// Of white: below this, a bright speck is noise, not a fault worth fixing.
const HOT_FLOOR: f32 = 0.03;

/// The 8 same-colour neighbours of a site, two away.
const SAME: [(isize, isize); 8] = [
    (-2, -2),
    (0, -2),
    (2, -2),
    (-2, 0),
    (2, 0),
    (-2, 2),
    (0, 2),
    (2, 2),
];
/// The 8 touching neighbours, of the other colours.
const TOUCH: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn offset(i: usize, w: usize, (dx, dy): (isize, isize)) -> usize {
    (i as isize + dy * w as isize + dx) as usize
}

/// Replaces each hot pixel with the median of its 8 same-colour neighbours. A site is hot
/// when it outshines both its same-colour neighbours and the 8 it touches: a real point of
/// light (a star, a glint) also reaches the touching sites, a stuck site does not. Returns
/// how many were fixed.
pub fn hot_pixels(plane: &mut Plane) -> usize {
    let (w, h) = (plane.width, plane.height);
    let data = &plane.data;
    let hot: Vec<(usize, f32)> = (2..h - 2)
        .into_par_iter()
        .flat_map_iter(|y| {
            (2..w - 2).filter_map(move |x| {
                let i = y * w + x;
                let v = data[i];
                let max = |set: &[(isize, isize); 8]| {
                    set.iter()
                        .map(|&d| data[offset(i, w, d)])
                        .fold(0f32, f32::max)
                };
                if v <= HOT_RATIO * max(&SAME) + HOT_FLOOR
                    || v <= HOT_RATIO * max(&TOUCH) + HOT_FLOOR
                {
                    return None;
                }
                let mut same = SAME.map(|d| data[offset(i, w, d)]);
                same.sort_by(f32::total_cmp);
                Some((i, f32::midpoint(same[3], same[4])))
            })
        })
        .collect();
    for &(i, v) in &hot {
        plane.data[i] = v;
    }
    hot.len()
}

/// Two greens closer than this, relative to their level, are taken as one surface.
const GREEN_GAP: f32 = 0.06;

/// Green equilibration: the two greens of a Bayer square sit on rows with different
/// neighbours and often read a percent or two apart, which demosaicing turns into a maze.
/// Where the two greens around a site agree within [`GREEN_GAP`] (a flat area, not an edge),
/// each green site moves halfway towards the other green's local mean.
pub fn green_balance(plane: &mut Plane) {
    let (w, h) = (plane.width, plane.height);
    let data = &plane.data;
    let mut fixed = data.clone();
    fixed
        .par_chunks_mut(w)
        .enumerate()
        .filter(|(y, _)| (2..h - 2).contains(y))
        .for_each(|(y, row)| {
            for (x, out) in row.iter_mut().enumerate().take(w - 2).skip(2) {
                if plane.site(x, y) != 1 {
                    continue;
                }
                let i = y * w + x;
                let own = (data[i]
                    + [(0, -2), (0, 2), (-2, 0), (2, 0)]
                        .iter()
                        .map(|&d| data[offset(i, w, d)])
                        .sum::<f32>())
                    / 5.0;
                let other = [(-1, -1), (1, -1), (-1, 1), (1, 1)]
                    .iter()
                    .map(|&d| data[offset(i, w, d)])
                    .sum::<f32>()
                    / 4.0;
                let level = own.max(other);
                if level > 0.0 && (own - other).abs() < GREEN_GAP * level {
                    *out = (data[i] - 0.5 * (own - other)).max(0.0);
                }
            }
        });
    plane.data = fixed;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfa::Mosaic;
    use crate::demosaic::balance;

    fn plane(data: Vec<u16>, w: usize, h: usize) -> Plane {
        let mosaic = Mosaic::new(
            w,
            h,
            data,
            [0, 1, 1, 2],
            [0.0; 4],
            1000.0,
            [0, 0, h, w],
            [0, 0, w, h],
        );
        let Ok(mosaic) = mosaic else {
            panic!("a mosaic");
        };
        balance(&mosaic, [1.0; 3])
    }

    /// A lone stuck site on grey is fixed; a 3 × 3 point of light (a star) is kept.
    #[test]
    fn fr_072_a_lone_spike_goes_a_bright_spot_stays() {
        let (w, h) = (24, 24);
        let mut data = vec![200u16; w * h];
        data[10 * w + 10] = 1000;
        for y in 15..18 {
            for x in 15..18 {
                data[y * w + x] = 1000;
            }
        }
        let mut p = plane(data, w, h);
        assert_eq!(hot_pixels(&mut p), 1);
        let at = |x: usize, y: usize| p.data[(y + p.pad) * p.width + x + p.pad];
        assert!((at(10, 10) - 0.2).abs() < 1e-6);
        assert!((at(16, 16) - 1.0).abs() < 1e-6);
    }

    /// Greens 2 % apart on flat grey become one; across a strong edge they are left alone.
    #[test]
    fn fr_072_greens_meet_on_flat_grey_not_across_an_edge() {
        let (w, h) = (24, 24);
        let green_b = |x: usize, y: usize| !y.is_multiple_of(2) && x.is_multiple_of(2);
        let data = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let base: u16 = if x < 12 { 400 } else { 900 };
                if green_b(x, y) {
                    base + base / 50
                } else {
                    base
                }
            })
            .collect();
        let mut p = plane(data, w, h);
        let before = p.data.clone();
        green_balance(&mut p);
        let idx = |x: usize, y: usize| (y + p.pad) * p.width + x + p.pad;
        // Inside the flat left half, a Gr and a Gb site now read the same.
        assert!((p.data[idx(5, 5)] - p.data[idx(4, 4)]).abs() < 0.002);
        // Next to the edge, nothing moved.
        assert!((p.data[idx(11, 5)] - before[idx(11, 5)]).abs() < 1e-6);
    }
}
