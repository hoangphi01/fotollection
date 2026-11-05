// SPDX-License-Identifier: GPL-3.0-only
//! Colour looks (FR-079, FR-070, ADR-0038): every per-pixel adjustment is baked into one
//! 33³ table and applied with tetrahedral interpolation, so a look costs the same per pixel
//! whatever is set. The table maps the photo's encoded values (ADR-0038), never linear light.

use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::image::RgbImage;

/// Grid points a side: the size LUMIX Lab writes and the research's sweet spot for
/// tetrahedral interpolation (ADR-0036, ADR-0038).
pub const GRID: usize = 33;
const CELLS: usize = GRID - 1;

/// A 3D colour table: for each grid point, the output R, G, B (interleaved, red fastest).
#[derive(Clone, PartialEq, Eq)]
pub struct Table {
    data: Vec<[u16; 3]>,
}

impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field("grid", &GRID)
            .finish_non_exhaustive()
    }
}

/// Index steps of one grid point along red, green and blue.
const STEP_R: usize = 1;
const STEP_G: usize = GRID;
const STEP_B: usize = GRID * GRID;

/// The table's index of grid point (`r`, `g`, `b`).
const fn at(r: usize, g: usize, b: usize) -> usize {
    (b * GRID + g) * GRID + r
}

/// The value of grid step `i` on a 0–1 scale.
#[allow(clippy::cast_precision_loss)] // i ≤ 32
pub fn grid_value(i: usize) -> f32 {
    i as f32 / CELLS as f32
}

impl Table {
    /// Builds a table from `f`, which maps an encoded colour (each channel 0–1) to another.
    /// Outputs are clamped to 0–1.
    pub fn from_fn(f: impl Fn([f32; 3]) -> [f32; 3] + Sync) -> Self {
        let data = (0..GRID * GRID * GRID)
            .into_par_iter()
            .map(|i| {
                let (r, g, b) = (i % GRID, (i / GRID) % GRID, i / (GRID * GRID));
                let out = f([grid_value(r), grid_value(g), grid_value(b)]);
                out.map(to_u16)
            })
            .collect();
        Self { data }
    }

    /// The table that changes nothing.
    pub fn identity() -> Self {
        Self::from_fn(|c| c)
    }

    #[cfg(test)]
    fn get(&self, r: usize, g: usize, b: usize) -> [u16; 3] {
        self.data[at(r, g, b)]
    }
}

/// A 0–1 value as the table keeps it: u16, 65535 for full scale.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0–65535
fn to_u16(v: f32) -> u16 {
    (v.clamp(0.0, 1.0) * 65535.0).round() as u16
}

/// For each 8-bit input, its grid cell and the position inside it in 1/256ths.
struct Steps {
    cell: [u8; 256],
    frac: [u16; 256],
}

/// Fraction scale inside a cell: 256 steps, so the weights sum to 256 exactly.
const FRAC: u32 = 256;

#[allow(clippy::cast_possible_truncation)] // cell ≤ 31, frac ≤ 256
const fn steps() -> Steps {
    let mut cell = [0u8; 256];
    let mut frac = [0u16; 256];
    let mut v = 0;
    while v < 256 {
        // v / 255 of the way along 32 cells, in 1/256ths of a cell, rounded.
        let pos = (v as u32 * CELLS as u32 * FRAC * 2 + 255) / (255 * 2);
        let mut c = pos / FRAC;
        let mut f = pos % FRAC;
        if c >= CELLS as u32 {
            // The last value sits on the last grid point: the end of the last cell.
            c = CELLS as u32 - 1;
            f = FRAC;
        }
        cell[v] = c as u8;
        frac[v] = f as u16;
        v += 1;
    }
    Steps { cell, frac }
}

static STEPS: Steps = steps();

/// One pixel through the table, tetrahedral interpolation in integers: the cell's corner,
/// the far corner, and the two between them on the path the fractions' order picks.
#[allow(clippy::cast_possible_truncation)] // the weighted sum is at most 65535 · 256
#[inline]
fn lookup(table: &Table, px: [u8; 3]) -> [u8; 3] {
    let s = &STEPS;
    let (r0, g0, b0) = (
        usize::from(s.cell[usize::from(px[0])]),
        usize::from(s.cell[usize::from(px[1])]),
        usize::from(s.cell[usize::from(px[2])]),
    );
    let (fr, fg, fb) = (
        u32::from(s.frac[usize::from(px[0])]),
        u32::from(s.frac[usize::from(px[1])]),
        u32::from(s.frac[usize::from(px[2])]),
    );
    let base = at(r0, g0, b0);
    let d = &table.data;
    let c000 = d[base];
    let c111 = d[base + STEP_R + STEP_G + STEP_B];
    // Steps along the edges, largest fraction first, give the tetrahedron.
    let (first, second, w) = if fr >= fg {
        if fg >= fb {
            (STEP_R, STEP_R + STEP_G, [fr, fg, fb]) // r ≥ g ≥ b
        } else if fr >= fb {
            (STEP_R, STEP_R + STEP_B, [fr, fb, fg]) // r ≥ b > g
        } else {
            (STEP_B, STEP_R + STEP_B, [fb, fr, fg]) // b > r ≥ g
        }
    } else if fb > fg {
        (STEP_B, STEP_G + STEP_B, [fb, fg, fr]) // b > g > r
    } else if fb > fr {
        (STEP_G, STEP_G + STEP_B, [fg, fb, fr]) // g ≥ b > r
    } else {
        (STEP_G, STEP_R + STEP_G, [fg, fr, fb]) // g > r ≥ b
    };
    let c1 = d[base + first];
    let c2 = d[base + second];
    // out = c000 + w0 (c1 − c000) + w1 (c2 − c1) + w2 (c111 − c2), as non-negative weights.
    let (k0, k1, k2, k3) = (FRAC - w[0], w[0] - w[1], w[1] - w[2], w[2]);
    let mut out = [0u8; 3];
    for ch in 0..3 {
        let sum = k0 * u32::from(c000[ch])
            + k1 * u32::from(c1[ch])
            + k2 * u32::from(c2[ch])
            + k3 * u32::from(c111[ch]);
        // From 65535 · 256 to 255: divide by 257 · 256, rounded.
        out[ch] = ((sum + 257 * 128) / (257 * 256)) as u8;
    }
    out
}

/// Maps every pixel of `image` through `table`, a row at a time on rayon.
pub fn apply(image: &mut RgbImage, table: &Table) {
    let row = image.width as usize * 3;
    if row == 0 {
        return;
    }
    image.data.par_chunks_mut(row).for_each(|line| {
        for px in line.as_chunks_mut::<3>().0 {
            let out = lookup(table, [px[0], px[1], px[2]]);
            *px = out;
        }
    });
}

/// The last tables built, by the hash of what built them, so dragging a slider back or
/// moving between photos with the same look builds nothing. Never held while rayon runs:
/// a table is cloned out as an `Arc` first (LESSONS).
pub struct TableCache {
    entries: Mutex<Vec<(blake3::Hash, Arc<Table>)>>,
}

/// Tables kept: about 210 KB each.
const CACHED: usize = 8;

impl Default for TableCache {
    fn default() -> Self {
        Self {
            entries: Mutex::new(Vec::with_capacity(CACHED)),
        }
    }
}

impl TableCache {
    /// The table for `key`, built by `build` (outside the lock) when it is not kept.
    pub fn get(&self, key: blake3::Hash, build: impl FnOnce() -> Table) -> Arc<Table> {
        if let Some(found) = self.find(key) {
            return found;
        }
        let table = Arc::new(build());
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|(k, _)| *k != key);
            if entries.len() >= CACHED {
                entries.remove(0);
            }
            entries.push((key, Arc::clone(&table)));
        }
        table
    }

    fn find(&self, key: blake3::Hash) -> Option<Arc<Table>> {
        let mut entries = self.entries.lock().ok()?;
        let i = entries.iter().position(|(k, _)| *k == key)?;
        // Most recently used last.
        let entry = entries.remove(i);
        let table = Arc::clone(&entry.1);
        entries.push(entry);
        Some(table)
    }
}

#[cfg(test)]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
mod tests {
    use super::*;

    /// Tetrahedral interpolation in f64 straight from its definition, for checking the
    /// integer one.
    fn reference(table: &Table, px: [u8; 3]) -> [f64; 3] {
        let pos = px.map(|v| f64::from(v) / 255.0 * CELLS as f64);
        let base = pos.map(|p| (p.floor() as usize).min(CELLS - 1));
        let f = [0, 1, 2].map(|i| pos[i] - base[i] as f64);
        let corner = |d: [usize; 3]| {
            let v = table.get(base[0] + d[0], base[1] + d[1], base[2] + d[2]);
            v.map(|x| f64::from(x) / 65535.0 * 255.0)
        };
        // Order the axes by fraction, largest first.
        let mut axes = [0usize, 1, 2];
        axes.sort_by(|&a, &b| f[b].total_cmp(&f[a]));
        let mut d = [0usize; 3];
        let mut prev = corner(d);
        let mut out = prev;
        for &axis in &axes {
            d[axis] = 1;
            let next = corner(d);
            for ch in 0..3 {
                out[ch] += f[axis] * (next[ch] - prev[ch]);
            }
            prev = next;
        }
        out
    }

    /// A smooth, strong look to test against: channels mixed, a curve, an offset.
    fn wild() -> Table {
        Table::from_fn(|[r, g, b]| {
            [
                (0.7 * r + 0.3 * g).powf(0.8),
                (g * g * 0.9 + 0.1 * b).sqrt(),
                (0.2 + 0.6 * b + 0.2 * r * g).min(1.0),
            ]
        })
    }

    #[test]
    fn fr_079_steps_cover_every_input_and_end_on_the_last_grid_point() {
        let s = &STEPS;
        assert_eq!((s.cell[0], s.frac[0]), (0, 0));
        assert_eq!((s.cell[255], s.frac[255]), (31, 256));
        for v in 0..256 {
            assert!(
                usize::from(s.cell[v]) < CELLS && u32::from(s.frac[v]) <= FRAC,
                "{v}"
            );
        }
    }

    #[test]
    fn fr_079_the_identity_table_changes_no_pixel() {
        let t = Table::identity();
        for v in 0..=255u8 {
            assert_eq!(lookup(&t, [v, v, v]), [v, v, v], "grey {v}");
        }
        for r in (0..=255u8).step_by(5) {
            for g in (0..=255u8).step_by(7) {
                for b in (0..=255u8).step_by(11) {
                    assert_eq!(lookup(&t, [r, g, b]), [r, g, b]);
                }
            }
        }
        for px in [
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [255, 255, 0],
            [1, 254, 128],
        ] {
            assert_eq!(lookup(&t, px), px);
        }
    }

    #[test]
    fn fr_079_integer_tetrahedral_matches_the_reference_within_one_level() {
        let t = wild();
        let mut seed = 0x2545_f491_u32;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed & 0xff) as u8
        };
        let mut worst = 0.0f64;
        for _ in 0..200_000 {
            let px = [next(), next(), next()];
            let got = lookup(&t, px);
            let want = reference(&t, px);
            for ch in 0..3 {
                worst = worst.max((f64::from(got[ch]) - want[ch]).abs());
            }
        }
        assert!(worst <= 1.0, "worst {worst}");
    }

    #[test]
    fn fr_079_apply_maps_every_row() {
        let mut image = RgbImage::new(3, 2);
        image.data = vec![
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180,
        ];
        let invert = Table::from_fn(|c| c.map(|v| 1.0 - v));
        apply(&mut image, &invert);
        let want: Vec<u8> = [
            10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180,
        ]
        .iter()
        .map(|v| 255 - v)
        .collect();
        assert_eq!(image.data, want);
    }

    #[test]
    fn fr_079_the_cache_builds_once_per_key_and_keeps_eight() {
        let cache = TableCache::default();
        let key = |n: u8| blake3::hash(&[n]);
        let builds = std::cell::Cell::new(0);
        let build = || {
            builds.set(builds.get() + 1);
            Table::identity()
        };
        let a = cache.get(key(1), build);
        let b = cache.get(key(1), || unreachable!("kept"));
        assert!(Arc::ptr_eq(&a, &b));
        for n in 2..=9 {
            cache.get(key(n), Table::identity);
        }
        assert_eq!(cache.entries.lock().map_or(0, |e| e.len()), CACHED);
        // Key 1 was the least recently used: it went.
        assert!(cache.find(key(1)).is_none());
        assert_eq!(builds.get(), 1);
    }
}
