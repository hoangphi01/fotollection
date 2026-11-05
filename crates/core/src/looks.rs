// SPDX-License-Identifier: GPL-3.0-only
//! Colour looks (FR-079, FR-070, ADR-0038): every per-pixel adjustment is baked into one
//! 33³ table and applied with tetrahedral interpolation, so a look costs the same per pixel
//! whatever is set. The table maps the photo's encoded values (ADR-0038), never linear light.

use std::sync::{Arc, LazyLock, Mutex};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

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

/// A photo's colour look (FR-079): the sliders, an optional imported LUT (FR-070) with its
/// strength, and an amount that scales all of them together, as a Lightroom preset's.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct LookSettings {
    pub adjust: Adjust,
    /// The content hash of an imported LUT in `looks/` (FR-070, session 12); none yet.
    pub lut: Option<String>,
    /// How much of the LUT, 0–100 %.
    pub lut_strength: u8,
    /// How much of the whole look, 0–100 %.
    pub amount: u8,
}

impl Default for LookSettings {
    fn default() -> Self {
        Self {
            adjust: Adjust::default(),
            lut: None,
            lut_strength: 100,
            amount: 100,
        }
    }
}

/// The sliders, each 0 when unset (FR-079). Signed ones run −100…100, the others 0…100.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Adjust {
    // Tone.
    pub brightness: i8,
    pub contrast: i8,
    pub highlights: i8,
    pub shadows: i8,
    // Colour.
    /// Blue (−) to amber (+).
    pub warmth: i8,
    /// Green (−) to magenta (+).
    pub tint: i8,
    pub saturation: i8,
    /// Saturation that spares skin and colours already strong.
    pub vibrance: i8,
    // Style.
    /// Lifted blacks, 0…100.
    pub fade: u8,
    pub shadow_tint: Tint,
    pub highlight_tint: Tint,
    /// Where shadows end and highlights begin for the tints: − more shadows, + more highlights.
    pub balance: i8,
    pub mono: Mono,
    // Effects, drawn after the table (session 13): kept, not yet rendered.
    pub grain: Grain,
    /// Darker (−) or lighter (+) corners round the cropped photo.
    pub vignette: i8,
}

/// A colour laid over the shadows or the highlights (split toning).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Tint {
    /// Degrees on the colour wheel, 0–359.
    pub hue: u16,
    /// 0…100.
    pub amount: u8,
}

/// Black and white, through a coloured filter as on film (FR-079, as the Lumix Monochrome
/// filter effect): yellow darkens blue skies a little, red a lot, green lightens leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Mono {
    #[default]
    Off,
    Neutral,
    Yellow,
    Orange,
    Red,
    Green,
}

/// Film grain, sized on the print (session 13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Grain {
    /// 0…100.
    pub amount: u8,
    /// 0…100, from fine to coarse.
    pub size: u8,
}

impl LookSettings {
    /// Within the sliders' ranges, whatever the file or the webview sent.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let a = &mut self.adjust;
        for v in [
            &mut a.brightness,
            &mut a.contrast,
            &mut a.highlights,
            &mut a.shadows,
            &mut a.warmth,
            &mut a.tint,
            &mut a.saturation,
            &mut a.vibrance,
            &mut a.balance,
            &mut a.vignette,
        ] {
            *v = (*v).clamp(-100, 100);
        }
        for v in [
            &mut a.fade,
            &mut a.shadow_tint.amount,
            &mut a.highlight_tint.amount,
            &mut a.grain.amount,
            &mut a.grain.size,
            &mut self.lut_strength,
            &mut self.amount,
        ] {
            *v = (*v).min(100);
        }
        a.shadow_tint.hue %= 360;
        a.highlight_tint.hue %= 360;
        // A LUT is named by its hash: anything else is dropped rather than looked up.
        if self
            .lut
            .as_ref()
            .is_some_and(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            self.lut = None;
        }
        self
    }

    /// True when the look changes nothing, so rendering skips it and a print stays the same,
    /// bit for bit (FR-079).
    pub fn is_identity(&self) -> bool {
        self.amount == 0 || (self.adjust == Adjust::default() && self.lut.is_none())
    }

    /// The table of this look's per-pixel part, from the app-wide cache; `None` when it
    /// changes nothing.
    pub fn table(&self) -> Option<Arc<Table>> {
        let colour_only = LookSettings {
            adjust: Adjust {
                grain: Grain::default(),
                vignette: 0,
                ..self.adjust
            },
            ..self.clone()
        };
        if colour_only.is_identity() {
            return None;
        }
        let key = blake3::hash(&serde_json::to_vec(&colour_only).unwrap_or_default());
        Some(TABLES.get(key, || bake(&colour_only)))
    }
}

static TABLES: LazyLock<TableCache> = LazyLock::new(TableCache::default);

/// The sliders as factors, scaled by the look's amount: −1…1 or 0…1.
struct Params {
    brightness: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    gains: [f32; 3],
    saturation: f32,
    vibrance: f32,
    mono: Option<[f32; 3]>,
    fade: f32,
    shadow_tint: [f32; 2],
    highlight_tint: [f32; 2],
    pivot: f32,
    /// A shadow or highlight tint is set.
    toned: bool,
}

impl Params {
    fn new(look: &LookSettings) -> Self {
        let a = look.adjust;
        let k = f32::from(look.amount) / 100.0;
        let s = |v: i8| f32::from(v) / 100.0 * k;
        let u = |v: u8| f32::from(v) / 100.0 * k;
        // Warmth and tint as channel gains in linear light, normalised so white keeps its
        // luminance (Rec. 709 weights).
        let (w, t) = (s(a.warmth), s(a.tint));
        // Tint moves one channel to warmth's two, and green weighs most in luminance: it
        // gets a larger step, so ±60 reads about as strong as warmth's.
        let raw = [(0.3 * w).exp2(), (-0.45 * t).exp2(), (-0.3 * w).exp2()];
        let y = 0.2126 * raw[0] + 0.7152 * raw[1] + 0.0722 * raw[2];
        let tint = |t: Tint| {
            let r = f32::from(t.hue).to_radians();
            let amount = f32::from(t.amount) / 100.0 * k * 0.08;
            [amount * r.cos(), amount * r.sin()]
        };
        Self {
            brightness: s(a.brightness),
            contrast: s(a.contrast),
            highlights: s(a.highlights),
            shadows: s(a.shadows),
            gains: raw.map(|g| g / y),
            saturation: s(a.saturation),
            vibrance: s(a.vibrance),
            mono: mono_weights(a.mono),
            fade: u(a.fade),
            shadow_tint: tint(a.shadow_tint),
            highlight_tint: tint(a.highlight_tint),
            pivot: 0.5 + 0.3 * s(a.balance),
            toned: k > 0.0 && (a.shadow_tint.amount > 0 || a.highlight_tint.amount > 0),
        }
    }
}

/// Linear-light channel weights of a black-and-white filter; they sum to 1, so white stays.
fn mono_weights(mono: Mono) -> Option<[f32; 3]> {
    match mono {
        Mono::Off => None,
        Mono::Neutral => Some([0.2126, 0.7152, 0.0722]),
        Mono::Yellow => Some([0.35, 0.60, 0.05]),
        Mono::Orange => Some([0.50, 0.45, 0.05]),
        Mono::Red => Some([0.70, 0.30, 0.00]),
        Mono::Green => Some([0.15, 0.80, 0.05]),
    }
}

/// Builds the table of `look`'s per-pixel part (ADR-0038). The order: warmth and tint, the
/// tone curve, saturation and vibrance, black and white, fade, then the shadow and highlight
/// tints, so a black-and-white photo can still be toned.
pub fn bake(look: &LookSettings) -> Table {
    let p = Params::new(look);
    Table::from_fn(|c| pixel(c, &p))
}

fn pixel(c: [f32; 3], p: &Params) -> [f32; 3] {
    // Warmth and tint.
    let lin = c.map(to_linear);
    let lin = [0, 1, 2].map(|i| lin[i] * p.gains[i]);
    let mut e = lin.map(to_encoded);
    // The tone curve, on each channel's encoded value; 0 and 1 stay where they are.
    e = e.map(|v| tone(v.clamp(0.0, 1.0), p));
    let mut lab = oklab(e.map(to_linear));
    // Saturation and vibrance: chroma scaled, so a grey (no chroma) stays grey.
    let chroma = lab[1].hypot(lab[2]);
    if chroma > 0.0 {
        let mut factor = 1.0 + p.saturation;
        if p.vibrance != 0.0 {
            // Strongest on dull colours, and half as strong on skin (hues near 55° in OKLab).
            let dull = 1.0 - (chroma / 0.25).min(1.0);
            let hue = lab[2].atan2(lab[1]).to_degrees();
            let skin = (-((hue - 55.0) / 30.0).powi(2)).exp();
            let spare = if p.vibrance > 0.0 {
                1.0 - 0.5 * skin
            } else {
                1.0
            };
            factor *= 1.0 + p.vibrance * dull * spare;
        }
        let factor = factor.max(0.0);
        lab[1] *= factor;
        lab[2] *= factor;
    }
    let mut lin = from_oklab(lab).map(|v| v.clamp(0.0, 1.0));
    if let Some(w) = p.mono {
        let y = w[0] * lin[0] + w[1] * lin[1] + w[2] * lin[2];
        lin = [y; 3];
    }
    let mut e = lin.map(to_encoded);
    if p.fade > 0.0 {
        let lift = 0.12 * p.fade;
        e = e.map(|v| lift + v * (1.0 - lift));
    }
    if p.toned {
        let mut lab = oklab(e.map(to_linear));
        let l = lab[0].clamp(0.0, 1.0);
        let below = ((p.pivot - l) / p.pivot.max(1e-3)).clamp(0.0, 1.0);
        let above = ((l - p.pivot) / (1.0 - p.pivot).max(1e-3)).clamp(0.0, 1.0);
        let (ws, wh) = (below.sqrt(), above.sqrt());
        lab[1] += ws * p.shadow_tint[0] + wh * p.highlight_tint[0];
        lab[2] += ws * p.shadow_tint[1] + wh * p.highlight_tint[1];
        e = from_oklab(lab).map(|v| to_encoded(v.clamp(0.0, 1.0)));
    }
    e
}

/// Brightness, contrast, highlights and shadows on one encoded value. Each step is
/// monotonic and keeps 0 and 1, so the curve never folds over.
fn tone(x: f32, p: &Params) -> f32 {
    let mut y = x;
    if p.brightness != 0.0 {
        y = y.powf((-0.8 * p.brightness).exp2());
    }
    if p.contrast != 0.0 {
        let k = 0.7 * p.contrast;
        let target = if k > 0.0 {
            y * y * (3.0 - 2.0 * y)
        } else {
            0.5 - ((1.0 - 2.0 * y).asin() / 3.0).sin()
        };
        y += k.abs() * (target - y);
    }
    // Bumps of x³(1 − x) and x(1 − x)³: their slopes stay within −1…1 in the half they
    // move, so an amount of at most 1 keeps the curve monotonic.
    if p.highlights != 0.0 {
        y += p.highlights * y.powi(3) * (1.0 - y);
    }
    if p.shadows != 0.0 {
        y += p.shadows * y * (1.0 - y).powi(3);
    }
    y.clamp(0.0, 1.0)
}

/// The sRGB transfer curve, encoded → linear.
fn to_linear(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// The sRGB transfer curve, linear → encoded.
fn to_encoded(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Linear sRGB → `OKLab` (Björn Ottosson, 2020).
fn oklab([r, g, b]: [f32; 3]) -> [f32; 3] {
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// `OKLab` → linear sRGB.
fn from_oklab([l, a, b]: [f32; 3]) -> [f32; 3] {
    let l_ = (l + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m_ = (l - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s_ = (l - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    [
        4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_,
        -1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_,
    ]
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

    fn with(f: impl FnOnce(&mut Adjust)) -> LookSettings {
        let mut look = LookSettings::default();
        f(&mut look.adjust);
        look
    }

    fn through(look: &LookSettings, px: [u8; 3]) -> [u8; 3] {
        lookup(&bake(look), px)
    }

    const GREYS: [u8; 9] = [0, 16, 40, 64, 100, 128, 180, 230, 255];

    #[test]
    fn fr_079_a_look_that_changes_nothing_has_no_table() {
        assert!(LookSettings::default().is_identity());
        assert!(LookSettings::default().table().is_none());
        let none = LookSettings {
            amount: 0,
            ..with(|a| a.contrast = 80)
        };
        assert!(none.table().is_none());
        // Grain and vignette are drawn later: on their own they need no table.
        assert!(with(|a| a.vignette = -40).table().is_none());
        assert!(with(|a| a.saturation = 10).table().is_some());
    }

    #[test]
    fn fr_079_every_slider_at_zero_bakes_the_identity() {
        let t = bake(&LookSettings::default());
        for v in 0..=255u8 {
            assert_eq!(lookup(&t, [v, v, v]), [v, v, v]);
        }
        for px in [[200, 30, 60], [10, 120, 250], [255, 255, 0]] {
            let out = lookup(&t, px);
            for ch in 0..3 {
                assert!(out[ch].abs_diff(px[ch]) <= 1, "{px:?} → {out:?}");
            }
        }
    }

    #[test]
    fn fr_079_tone_curves_stay_monotonic_and_keep_black_and_white() {
        for v in [-100i8, -50, 50, 100] {
            for (name, look) in [
                ("brightness", with(|a| a.brightness = v)),
                ("contrast", with(|a| a.contrast = v)),
                ("highlights", with(|a| a.highlights = v)),
                ("shadows", with(|a| a.shadows = v)),
            ] {
                let t = bake(&look);
                let mut last = 0u8;
                for g in 0..=255u8 {
                    let out = lookup(&t, [g, g, g])[0];
                    assert!(out >= last, "{name} {v}: {g} → {out} after {last}");
                    last = out;
                }
                assert_eq!(lookup(&t, [0, 0, 0]), [0, 0, 0], "{name} {v}");
                assert_eq!(lookup(&t, [255, 255, 255]), [255, 255, 255], "{name} {v}");
            }
        }
    }

    #[test]
    fn fr_079_each_slider_moves_the_right_way() {
        let mid = [128, 128, 128];
        assert!(through(&with(|a| a.brightness = 50), mid)[0] > 140);
        assert!(through(&with(|a| a.brightness = -50), mid)[0] < 116);
        // Contrast pulls a dark grey down and a light one up.
        let c = with(|a| a.contrast = 60);
        assert!(through(&c, [60; 3])[0] < 58 && through(&c, [196; 3])[0] > 198);
        assert!(through(&with(|a| a.highlights = 80), [200; 3])[0] > 205);
        assert!(through(&with(|a| a.shadows = 80), [50; 3])[0] > 55);
        let warm = through(&with(|a| a.warmth = 60), mid);
        assert!(warm[0] > warm[2] + 10, "{warm:?}");
        let magenta = through(&with(|a| a.tint = 60), mid);
        assert!(magenta[1] + 10 < magenta[0], "{magenta:?}");
        // Saturation: a muted red gets redder, and −100 leaves no colour.
        let red = [180, 90, 80];
        let more = through(&with(|a| a.saturation = 60), red);
        assert!(more[0] > red[0] && more[2] < red[2], "{more:?}");
        let none = through(&with(|a| a.saturation = -100), red);
        assert!(
            none[0].abs_diff(none[1]) <= 1 && none[1].abs_diff(none[2]) <= 1,
            "{none:?}"
        );
        assert!(through(&with(|a| a.fade = 100), [0; 3])[0] > 20);
    }

    #[test]
    fn fr_079_greys_stay_grey_under_saturation_and_vibrance() {
        for look in [
            with(|a| a.saturation = 100),
            with(|a| a.saturation = -60),
            with(|a| a.vibrance = 100),
            with(|a| a.vibrance = -100),
        ] {
            let t = bake(&look);
            for g in GREYS {
                let out = lookup(&t, [g, g, g]);
                assert!(
                    out[0].abs_diff(out[1]) <= 1 && out[1].abs_diff(out[2]) <= 1,
                    "{g} → {out:?}"
                );
            }
        }
    }

    #[test]
    fn fr_079_vibrance_lifts_dull_colours_more_than_strong_ones() {
        let look = with(|a| a.vibrance = 100);
        let gain = |px: [u8; 3]| {
            let out = through(&look, px);
            let spread = |p: [u8; 3]| {
                i32::from(*p.iter().max().unwrap_or(&0)) - i32::from(*p.iter().min().unwrap_or(&0))
            };
            spread(out) - spread(px)
        };
        assert!(
            gain([110, 130, 150]) > gain([20, 60, 230]),
            "a dull blue gains more"
        );
    }

    #[test]
    fn fr_079_black_and_white_has_no_colour_and_the_filter_matters() {
        for mono in [
            Mono::Neutral,
            Mono::Yellow,
            Mono::Orange,
            Mono::Red,
            Mono::Green,
        ] {
            let t = bake(&with(|a| a.mono = mono));
            for px in [[200, 40, 40], [30, 90, 220], [250, 250, 250]] {
                let out = lookup(&t, px);
                assert!(
                    out[0] == out[1] && out[1] == out[2],
                    "{mono:?} {px:?} → {out:?}"
                );
            }
        }
        // A red filter darkens a blue sky more than no filter.
        let sky = [70, 130, 230];
        assert!(
            through(&with(|a| a.mono = Mono::Red), sky)[0]
                < through(&with(|a| a.mono = Mono::Neutral), sky)[0]
        );
    }

    #[test]
    fn fr_079_split_toning_tints_the_shadows_and_the_highlights_apart() {
        // Teal shadows (OKLab hue ~200°), amber highlights (~70°).
        let look = with(|a| {
            a.shadow_tint = Tint {
                hue: 200,
                amount: 100,
            };
            a.highlight_tint = Tint {
                hue: 70,
                amount: 100,
            };
        });
        let dark = through(&look, [50; 3]);
        let light = through(&look, [210; 3]);
        assert!(dark[2] > dark[0], "shadows bluer {dark:?}");
        assert!(light[0] > light[2], "highlights warmer {light:?}");
    }

    #[test]
    fn fr_079_sanitized_brings_every_value_into_range() {
        let wild = LookSettings {
            adjust: Adjust {
                brightness: 127,
                contrast: -128,
                fade: 255,
                shadow_tint: Tint {
                    hue: 725,
                    amount: 200,
                },
                grain: Grain {
                    amount: 101,
                    size: 255,
                },
                ..Adjust::default()
            },
            lut: Some("../../etc/passwd".into()),
            lut_strength: 250,
            amount: 101,
        }
        .sanitized();
        assert_eq!(
            (
                wild.adjust.brightness,
                wild.adjust.contrast,
                wild.adjust.fade
            ),
            (100, -100, 100)
        );
        assert_eq!(
            (wild.adjust.shadow_tint.hue, wild.adjust.shadow_tint.amount),
            (5, 100)
        );
        assert_eq!(
            (wild.adjust.grain.amount, wild.adjust.grain.size),
            (100, 100)
        );
        assert_eq!((wild.lut, wild.lut_strength, wild.amount), (None, 100, 100));
    }

    #[test]
    fn fr_079_the_amount_scales_the_whole_look() {
        let full = with(|a| a.brightness = 80);
        let half = LookSettings {
            amount: 50,
            ..full.clone()
        };
        let (f, h) = (through(&full, [100; 3])[0], through(&half, [100; 3])[0]);
        assert!(h > 100 && h < f, "half {h}, full {f}");
    }
}
