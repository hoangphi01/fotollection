// SPDX-License-Identifier: GPL-3.0-only
//! Highlight recovery (FR-112): where a sensor channel clipped, its value is rebuilt from the
//! channels that did not, with the colour the clipped areas' edges show, and kept above
//! white, so lowering the exposure or *Highlights* brings a sky, a lamp or a cloud back
//! instead of a flat grey patch.
//!
//! Read first (owner, 2026-11-09; RESULT §41), never their code: darktable's highlight
//! reconstruction (its default since 4.2, *inpaint opposed*: the clipped channels from the
//! unclipped ones with the average colour of the pixels next to the clipped areas; its
//! *segmentation*, a colour of each clipped area's own), `RawTherapee`'s *Color Propagation*
//! and its *inpaint opposed* (5.10), and Masood, Zhu and Tappen (2009): the ratios between
//! a pixel's channels change slowly across a surface. Rouf, Lau and Heidrich (2012) is
//! patented and left out.
//!
//! Runs on white-balanced camera RGB after the demosaic, where a channel's clip is its gain
//! (its sensor's 1.0 times the gain). `highlights_bench` compares the [`Method`]s against
//! known truth; [`DEFAULT`] is the one it chose: one colour for the whole frame, which beat a
//! colour per area or per tile on real frames and never turned a sky magenta.
//!
//! - **The colour:** over the unclipped pixels along every clipped area's edge, the mean of
//!   `ln v_c − mean ln v` (chroma in logarithms, which holds whatever channels clipped).
//! - **A pixel with some channels clipped:** its brightness from its unclipped channels
//!   (`L = mean(ln v_u − k_u)`), each clipped channel `exp(L + k_c)`, never lower than it
//!   was; where that colour would put a clipped channel under its clip, the colour leans to
//!   neutral (a clipped channel is at least its clip).
//! - **A pixel with all three clipped:** neutral, at least as bright as the brightest rebuilt
//!   pixel of its own area, so its core is never darker than its rim; never held at white.
//! - **An area with no edge to read:** the fade to white, as before (FR-088).

use rayon::prelude::*;

use crate::demosaic::CameraRgb;

/// The ways compared (`highlights_bench`, RESULT §41).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// The fade toward a neutral at the brightest channel (FR-088), as before.
    White,
    /// One colour for every clipped area, from all their edges (darktable's *inpaint
    /// opposed*, in essence).
    Global,
    /// Each clipped area its own colour, from its own edge (as *segmentation*).
    Regions,
    /// Each pixel the colour of the edge nearest it: the edge's colour gathered in tiles,
    /// empty tiles filled from coarser ones (pull-push), read back smoothly.
    Local,
}

/// The method's knobs, swept by `highlights_bench`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// A channel is clipped at this share of its clip.
    pub threshold: f32,
    /// How far the edge reaches out of an area, in pixels, across a row.
    pub band: usize,
    /// An edge pixel darker than this share of green's clip says nothing of the area's colour.
    pub dark: f32,
    /// [`Method::Local`]'s tile, in pixels.
    pub tile: usize,
    /// An edge pixel says the area's colour only when every channel is under this share of
    /// its clip: nearer the clip, the demosaic has already mixed the clipped sites into it.
    pub edge: f32,
    /// [`Method::Local`]: a tile with this many edge pixels keeps its own colour; one with
    /// fewer leans on the coarser tile above it.
    pub enough: f32,
}

/// The measured method and its knobs (RESULT §41).
pub const DEFAULT: (Method, Params) = (
    Method::Global,
    Params {
        threshold: 0.9,
        band: 10,
        dark: 0.1,
        tile: 32,
        edge: 0.9,
        enough: 8.0,
    },
);

/// Rebuilds the clipped highlights of `camera`, whose channels clip at `clip` (the white
/// balance's gains).
pub fn recover(camera: &mut CameraRgb, clip: [f32; 3], method: Method, params: &Params) {
    match method {
        Method::White => crate::highlights_to_white(&mut camera.rgb, clip),
        Method::Global | Method::Regions | Method::Local => rebuild(camera, clip, params, method),
    }
}

/// One clipped run of a row: its first pixel and the one after its last.
#[derive(Debug, Clone, Copy)]
struct Run {
    start: usize,
    end: usize,
}

/// The sums an area's edge gives: its chroma in logarithms and how many pixels said it.
#[derive(Debug, Clone, Copy, Default)]
struct Edge {
    k: [f64; 3],
    n: u64,
}

fn find(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        let up = parent[parent[i as usize] as usize];
        parent[i as usize] = up;
        i = up;
    }
    i
}

/// The channels of a pixel at or over their clip, as bits.
fn clipped(p: &[f32], limit: [f32; 3]) -> u8 {
    u8::from(p[0] >= limit[0])
        | (u8::from(p[1] >= limit[1]) << 1)
        | (u8::from(p[2] >= limit[2]) << 2)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
fn rebuild(camera: &mut CameraRgb, clip: [f32; 3], params: &Params, method: Method) {
    let global = method == Method::Global;
    let (w, h) = (camera.width, camera.height);
    if w == 0 || h == 0 {
        return;
    }
    let limit = clip.map(|c| c * params.threshold);
    // The clipped runs of each row.
    let runs: Vec<Vec<Run>> = camera
        .rgb
        .par_chunks_exact(w * 3)
        .map(|row| {
            let mut out = Vec::new();
            let mut start = None;
            for (x, p) in row.as_chunks::<3>().0.iter().enumerate() {
                match (clipped(p, limit) != 0, start) {
                    (true, None) => start = Some(x),
                    (false, Some(s)) => {
                        out.push(Run { start: s, end: x });
                        start = None;
                    }
                    _ => {}
                }
            }
            if let Some(s) = start {
                out.push(Run { start: s, end: w });
            }
            out
        })
        .collect();
    let total: usize = runs.iter().map(Vec::len).sum();
    // Nothing clipped: nothing to do, and the image is as it was, bit for bit.
    if total == 0 {
        return;
    }
    // Areas: runs that touch across rows joined (union-find over runs, row by row).
    let first: Vec<usize> = runs
        .iter()
        .scan(0usize, |n, r| {
            let at = *n;
            *n += r.len();
            Some(at)
        })
        .collect();
    let mut parent: Vec<u32> = (0..total as u32).collect();
    for y in 1..h {
        let (above, here) = (&runs[y - 1], &runs[y]);
        let (mut i, mut j) = (0, 0);
        while i < above.len() && j < here.len() {
            let (a, b) = (above[i], here[j]);
            if a.start < b.end && b.start < a.end {
                let (ra, rb) = (
                    find(&mut parent, (first[y - 1] + i) as u32),
                    find(&mut parent, (first[y] + j) as u32),
                );
                if ra != rb {
                    parent[ra.max(rb) as usize] = ra.min(rb);
                }
            }
            if a.end <= b.end {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    let mut area = vec![0u32; total];
    let mut areas = 0u32;
    let mut index = vec![u32::MAX; total];
    for (r, slot) in area.iter_mut().enumerate() {
        let root = find(&mut parent, r as u32) as usize;
        if index[root] == u32::MAX {
            index[root] = areas;
            areas += 1;
        }
        *slot = index[root];
    }
    let areas = areas as usize;

    // Each area's colour from its unclipped edge: across its runs' ends, `band` pixels out,
    // and the pixels just above and below them.
    let rgb = &camera.rgb;
    let floor = clip[1] * params.dark;
    let trusted = clip.map(|c| c * params.edge.min(params.threshold));
    let good = |x: usize, y: usize| -> Option<[f32; 3]> {
        let p = &rgb[(y * w + x) * 3..][..3];
        (clipped(p, trusted) == 0 && p.iter().all(|&v| v > floor * 0.25) && p[1] >= floor)
            .then(|| [p[0], p[1], p[2]])
    };
    let tile = params.tile.max(1);
    let grid = if method == Method::Local {
        (w.div_ceil(tile), h.div_ceil(tile))
    } else {
        (0, 0)
    };
    let (edges, tiles) = (0..h)
        .into_par_iter()
        .fold(
            || {
                (
                    vec![Edge::default(); areas],
                    vec![Edge::default(); grid.0 * grid.1],
                )
            },
            |(mut acc, mut tiles), y| {
                let mut add = |a: usize, (x, y): (usize, usize), p: [f32; 3]| {
                    let l = p.map(f32::ln);
                    let mean = (l[0] + l[1] + l[2]) / 3.0;
                    let k = l.map(|v| f64::from(v - mean));
                    let put = |e: &mut Edge| {
                        for (sum, v) in e.k.iter_mut().zip(k) {
                            *sum += v;
                        }
                        e.n += 1;
                    };
                    put(&mut acc[a]);
                    if !tiles.is_empty() {
                        put(&mut tiles[(y / tile) * grid.0 + x / tile]);
                    }
                };
                for (j, run) in runs[y].iter().enumerate() {
                    let a = area[first[y] + j] as usize;
                    for d in 1..=params.band {
                        if run.start >= d
                            && let Some(p) = good(run.start - d, y)
                        {
                            add(a, (run.start - d, y), p);
                        }
                        if run.end + d - 1 < w
                            && let Some(p) = good(run.end + d - 1, y)
                        {
                            add(a, (run.end + d - 1, y), p);
                        }
                    }
                    for x in run.start..run.end {
                        for d in (1..=params.band).step_by(3) {
                            for yy in [y.wrapping_sub(d), y + d] {
                                if yy < h
                                    && let Some(p) = good(x, yy)
                                {
                                    add(a, (x, yy), p);
                                }
                            }
                        }
                    }
                }
                (acc, tiles)
            },
        )
        .reduce(
            || {
                (
                    vec![Edge::default(); areas],
                    vec![Edge::default(); grid.0 * grid.1],
                )
            },
            |(mut a, mut at), (b, bt)| {
                for (x, y) in a.iter_mut().zip(b).chain(at.iter_mut().zip(bt)) {
                    for c in 0..3 {
                        x.k[c] += y.k[c];
                    }
                    x.n += y.n;
                }
                (a, at)
            },
        );
    let local = pull_push(&tiles, grid, params.enough);
    // Global: every area the colour of all their edges together; an area with no edge of
    // its own to read still goes to white.
    let all = edges.iter().fold(Edge::default(), |mut a, e| {
        for c in 0..3 {
            a.k[c] += e.k[c];
        }
        a.n += e.n;
        a
    });
    let chroma: Vec<Option<[f32; 3]>> = edges
        .iter()
        .map(|e| {
            let from = if global { &all } else { e };
            (e.n > 0).then(|| from.k.map(|k| (k / from.n as f64) as f32))
        })
        .collect();

    // Partly clipped pixels rebuilt; each area's brightest rebuilt pixel noted for its
    // fully clipped core.
    let peaks = camera
        .rgb
        .par_chunks_exact_mut(w * 3)
        .enumerate()
        .fold(
            || vec![0f32; areas],
            |mut peak, (y, row)| {
                for (j, run) in runs[y].iter().enumerate() {
                    let a = area[first[y] + j] as usize;
                    for x in run.start..run.end {
                        let p = &mut row[x * 3..x * 3 + 3];
                        let bits = clipped(p, limit);
                        if bits == 0b111 {
                            continue;
                        }
                        let Some(k) = chroma[a] else {
                            continue;
                        };
                        let k = if local.is_empty() {
                            k
                        } else {
                            sample(&local, grid, tile, x, y)
                        };
                        let Some((level, k)) = fit(p, bits, k) else {
                            continue;
                        };
                        for c in 0..3 {
                            if bits & (1 << c) != 0 {
                                p[c] = p[c].max((level + k[c]).exp());
                            }
                        }
                        peak[a] = peak[a].max(p[0].max(p[1]).max(p[2]));
                    }
                }
                peak
            },
        )
        .reduce(
            || vec![0f32; areas],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(b) {
                    *x = x.max(y);
                }
                a
            },
        );
    // Fully clipped pixels: neutral, as bright as their area's brightest rebuilt pixel; and
    // where an area had no edge to read, the fade to white as before.
    camera
        .rgb
        .par_chunks_exact_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            for (j, run) in runs[y].iter().enumerate() {
                let a = area[first[y] + j] as usize;
                for x in run.start..run.end {
                    let p = &mut row[x * 3..x * 3 + 3];
                    let m = p[0].max(p[1]).max(p[2]);
                    if chroma[a].is_none() {
                        p.fill(m);
                    } else if clipped(p, limit) == 0b111 {
                        p.fill(m.max(peaks[a]));
                    }
                }
            }
        });
}

/// How far, in natural logarithms, the edge's colour may put a clipped channel under its clip
/// before it is not used at all.
const DOUBT: f32 = 0.25;

/// A partly clipped pixel's brightness (`ln`) and the colour it is rebuilt with: the edge's
/// colour `k`, pulled toward neutral as far as it would put a clipped channel under the clip
/// it already reached (a white sky by brown branches is not brown: a clipped channel is at
/// least its clip). `None` when no channel is left to say how bright it is.
fn fit(p: &[f32], bits: u8, k: [f32; 3]) -> Option<(f32, [f32; 3])> {
    let level = |k: [f32; 3]| {
        let (mut sum, mut n) = (0.0f32, 0.0f32);
        for c in 0..3 {
            if bits & (1 << c) == 0 && p[c] > 0.0 {
                sum += p[c].ln() - k[c];
                n += 1.0;
            }
        }
        (n > 0.0).then(|| sum / n)
    };
    let l = level(k)?;
    let short = (0..3)
        .filter(|&c| bits & (1 << c) != 0)
        .map(|c| p[c].ln() - (l + k[c]))
        .fold(0f32, f32::max);
    if short <= 0.0 {
        return Some((l, k));
    }
    let k = k.map(|v| v * (1.0 - (short / DOUBT).min(1.0)));
    Some((level(k)?, k))
}

/// Each tile's colour, from its own edge pixels where it has enough and from coarser tiles
/// where it has fewer (pull-push: sums pulled up a pyramid of 2×2 tiles, colours pushed
/// back down). Empty when there are no tiles.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn pull_push(tiles: &[Edge], grid: (usize, usize), enough: f32) -> Vec<[f32; 3]> {
    if tiles.is_empty() {
        return Vec::new();
    }
    let mut levels = vec![(tiles.to_vec(), grid)];
    while let Some((below, (gw, gh))) = levels.last()
        && (*gw > 1 || *gh > 1)
    {
        let (nw, nh) = (gw.div_ceil(2), gh.div_ceil(2));
        let mut up = vec![Edge::default(); nw * nh];
        for y in 0..*gh {
            for x in 0..*gw {
                let (e, u) = (below[y * gw + x], &mut up[(y / 2) * nw + x / 2]);
                for c in 0..3 {
                    u.k[c] += e.k[c];
                }
                u.n += e.n;
            }
        }
        levels.push((up, (nw, nh)));
    }
    let mut colour: Vec<[f64; 3]> = Vec::new();
    let mut size = (0, 0);
    for (sums, (gw, gh)) in levels.into_iter().rev() {
        let mut next = vec![[0f64; 3]; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let e = sums[y * gw + x];
                let parent = if colour.is_empty() {
                    [0.0; 3]
                } else {
                    colour[(y / 2) * size.0 + x / 2]
                };
                let own = e.n as f64;
                let t = (own / f64::from(enough)).min(1.0);
                next[y * gw + x] = std::array::from_fn(|c| {
                    let mine = if e.n > 0 { e.k[c] / own } else { 0.0 };
                    mine * t + parent[c] * (1.0 - t)
                });
            }
        }
        colour = next;
        size = (gw, gh);
    }
    colour.iter().map(|k| k.map(|v| v as f32)).collect()
}

/// The tiles' colour at a pixel, read smoothly between the four nearest tile centres.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn sample(
    colour: &[[f32; 3]],
    (gw, gh): (usize, usize),
    tile: usize,
    x: usize,
    y: usize,
) -> [f32; 3] {
    let at = |v: usize, n: usize| {
        let f = ((v as f32 + 0.5) / tile as f32 - 0.5).clamp(0.0, (n - 1) as f32);
        let i = (f as usize).min(n.saturating_sub(2));
        (i, (i + 1).min(n - 1), f - i as f32)
    };
    let (x0, x1, fx) = at(x, gw);
    let (y0, y1, fy) = at(y, gh);
    let c = |x: usize, y: usize| colour[y * gw + x];
    let (a, b, d, e) = (c(x0, y0), c(x1, y0), c(x0, y1), c(x1, y1));
    std::array::from_fn(|i| {
        let top = a[i] + (b[i] - a[i]) * fx;
        let bottom = d[i] + (e[i] - d[i]) * fx;
        top + (bottom - top) * fy
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAINS: [f32; 3] = [2.0, 1.0, 1.6];

    /// A frame of `truth`, each channel clipped at its gain as a sensor would.
    fn clipped_frame(
        w: usize,
        h: usize,
        truth: impl Fn(usize, usize) -> [f32; 3],
    ) -> (CameraRgb, Vec<f32>) {
        let mut seen = Vec::with_capacity(w * h * 3);
        let mut real = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                let t = truth(x, y);
                for c in 0..3 {
                    real.push(t[c]);
                    seen.push(t[c].min(GAINS[c]));
                }
            }
        }
        (
            CameraRgb {
                width: w,
                height: h,
                rgb: seen,
            },
            real,
        )
    }

    /// A sky: a blue gradient whose blue clips towards the top, red and green under theirs.
    fn sky(x: usize, y: usize) -> [f32; 3] {
        #[allow(clippy::cast_precision_loss)]
        let lift = 1.0 + 1.5 * (1.0 - y as f32 / 100.0) + 0.002 * x as f32;
        [0.35 * lift, 0.55 * lift, 1.25 * lift]
    }

    /// The mean error of `ln(B/G)` where the truth's blue clipped.
    fn ratio_error(got: &[f32], want: &[f32]) -> f32 {
        let (mut sum, mut n) = (0.0f32, 0.0f32);
        for (g, t) in got.as_chunks::<3>().0.iter().zip(want.as_chunks::<3>().0) {
            if t[2] > GAINS[2] {
                sum += ((g[2] / g[1]).ln() - (t[2] / t[1]).ln()).abs();
                n += 1.0;
            }
        }
        sum / n.max(1.0)
    }

    /// FR-112: a sky whose blue clipped keeps its blue, where the fade made it white.
    #[test]
    fn fr_112_a_clipped_sky_keeps_its_blue() {
        let (w, h) = (120, 100);
        let (mut new, truth) = clipped_frame(w, h, sky);
        let mut old = new.clone();
        recover(&mut new, GAINS, DEFAULT.0, &DEFAULT.1);
        recover(&mut old, GAINS, Method::White, &DEFAULT.1);
        let (e_new, e_old) = (ratio_error(&new.rgb, &truth), ratio_error(&old.rgb, &truth));
        assert!(e_new < 0.05, "blue against green, new: {e_new}");
        assert!(e_old > e_new * 4.0, "old {e_old}, new {e_new}");
    }

    /// FR-112: what clipped is kept above its clip, so lowering the exposure finds it again.
    #[test]
    fn fr_112_brightness_above_white_is_kept() {
        let (mut camera, _) = clipped_frame(120, 100, sky);
        recover(&mut camera, GAINS, DEFAULT.0, &DEFAULT.1);
        let top = &camera.rgb[..120 * 3];
        assert!(
            top.as_chunks::<3>().0.iter().all(|p| p[2] > GAINS[2] * 1.2),
            "the top's blue above its clip"
        );
        // The gradient survives: the top row is brighter than a lower clipped row.
        let row = |y: usize| camera.rgb[(y * 120 + 60) * 3 + 2];
        assert!(row(0) > row(20), "{} against {}", row(0), row(20));
    }

    /// FR-112: a warm lamp beside a blue light, side by side: one colour serves both (the
    /// measured method, RESULT §41), yet each ends nearer its own than the fade to white.
    #[test]
    fn fr_112_two_colours_side_by_side_beat_the_fade() {
        let (w, h) = (160, 60);
        let (new, truth) = clipped_frame(w, h, |x, y| {
            let bright = if (20..40).contains(&y) { 2.2 } else { 0.6 };
            if x < 80 {
                [0.9 * bright, 0.5 * bright, 0.2 * bright] // a warm lamp, red clipping
            } else {
                [0.2 * bright, 0.45 * bright, 1.0 * bright] // a blue light, blue clipping
            }
        });
        let error = |method: Method| {
            let mut camera = new.clone();
            recover(&mut camera, GAINS, method, &DEFAULT.1);
            let off = |x: usize, c: usize| {
                let (p, t) = (&camera.rgb[(30 * w + x) * 3..], &truth[(30 * w + x) * 3..]);
                ((p[c] / p[1]).ln() - (t[c] / t[1]).ln()).abs()
            };
            off(40, 0) + off(120, 2)
        };
        let (ours, fade) = (error(DEFAULT.0), error(Method::White));
        assert!(ours * 2.0 < fade, "ours {ours}, the fade {fade}");
    }

    /// FR-112: where every channel clipped, the pixel is neutral and kept at least as bright as
    /// the brightest rebuilt pixel round it, never at white.
    #[test]
    fn fr_112_all_clipped_stays_neutral() {
        let (w, h) = (100, 100);
        let (mut camera, _) = clipped_frame(w, h, |x, y| {
            #[allow(clippy::cast_precision_loss)]
            let d = ((x as f32 - 50.0).hypot(y as f32 - 50.0)) / 50.0;
            let v = 4.0 * (1.0 - d).max(0.0) + 0.3;
            [0.8 * v, 0.7 * v, 0.9 * v]
        });
        recover(&mut camera, GAINS, DEFAULT.0, &DEFAULT.1);
        let p = &camera.rgb[(50 * w + 50) * 3..][..3];
        assert!(
            (p[0] - p[1]).abs() < 1e-6 && (p[1] - p[2]).abs() < 1e-6,
            "neutral: {p:?}"
        );
        assert!(p[0] > GAINS[0], "above every clip: {p:?}");
    }

    /// FR-112: a white sky between brown branches stays white, not magenta: the branches'
    /// colour would put the clipped blue under its clip, so it is not used.
    #[test]
    fn fr_112_a_white_sky_by_branches_stays_white() {
        let (w, h) = (120, 100);
        let (mut camera, _) = clipped_frame(w, h, |x, y| {
            if (30..90).contains(&x) && (20..80).contains(&y) {
                [1.75, 1.75, 1.75] // the sky: green and blue clip, red does not
            } else {
                [0.3, 0.2, 0.1] // the branches
            }
        });
        recover(&mut camera, GAINS, DEFAULT.0, &DEFAULT.1);
        let p = &camera.rgb[(50 * w + 60) * 3..][..3];
        let spread =
            p.iter().copied().fold(0f32, f32::max) / p.iter().copied().fold(f32::MAX, f32::min);
        assert!(spread < 1.1, "neutral within 10 %: {p:?}");
    }

    /// FR-112: a frame with nothing clipped is left as it was, bit for bit.
    #[test]
    fn fr_112_nothing_clipped_costs_nothing() {
        let (mut camera, _) = clipped_frame(80, 60, |x, _| {
            #[allow(clippy::cast_precision_loss)]
            let v = 0.2 + 0.005 * x as f32;
            [v, v * 0.9, v * 1.1]
        });
        let before = camera.rgb.clone();
        recover(&mut camera, GAINS, DEFAULT.0, &DEFAULT.1);
        assert_eq!(camera.rgb, before);
    }
}
