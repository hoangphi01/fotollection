// SPDX-License-Identifier: GPL-3.0-only
//! Colour looks (FR-079, FR-070, ADR-0038): every per-pixel adjustment is baked into one
//! 33³ table and applied with tetrahedral interpolation, so a look costs the same per pixel
//! whatever is set. The table maps the photo's encoded values (ADR-0038), never linear light.

use crate::lock::Mutex;
use std::sync::{Arc, LazyLock};

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

    /// The table as the webview's shader reads it (ADR-0039): RGBA8, red fastest, alpha 255,
    /// `GRID`³ × 4 bytes. Rounded from u16, half a level at most at a grid point.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() * 4);
        for px in &self.data {
            for &v in px {
                // 65535 → 255 rounded: (v + 128) / 257, at most 255.
                out.push(u8::try_from((u32::from(v) + 128) / 257).unwrap_or(u8::MAX));
            }
            out.push(u8::MAX);
        }
        out
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
pub(crate) fn lookup(table: &Table, px: [u8; 3]) -> [u8; 3] {
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
    /// The name of the look it came from, shown in the info frame's photo-style line
    /// (FR-040); never part of the table.
    pub name: Option<String>,
    /// Which look of the library it came from (`builtin:<slug>`, `user:<id>`, `lut:<hash>`),
    /// so the Looks card marks it; never part of the table.
    pub base: Option<String>,
}

/// The longest look name kept, in characters.
pub const MAX_LOOK_NAME: usize = 60;

/// A look's name as shown: no control characters, trimmed, at most [`MAX_LOOK_NAME`]
/// characters; `None` when nothing is left.
#[must_use]
pub fn clean_name(name: &str) -> Option<String> {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(MAX_LOOK_NAME)
        .collect();
    let cleaned = cleaned.trim_end().to_owned();
    (!cleaned.is_empty()).then_some(cleaned)
}

/// A library reference: a known kind, then 1–64 lowercase letters, digits or dashes.
#[must_use]
pub fn valid_base(base: &str) -> bool {
    let Some((kind, id)) = base.split_once(':') else {
        return false;
    };
    matches!(kind, "builtin" | "user" | "lut")
        && (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Default for LookSettings {
    fn default() -> Self {
        Self {
            adjust: Adjust::default(),
            lut: None,
            lut_strength: 100,
            amount: 100,
            name: None,
            base: None,
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
    // Effects, drawn after the table (FR-079, session 13, `effects`): sized on the print.
    pub grain: Grain,
    /// Darker (−) or lighter (+) corners round the cropped photo.
    pub vignette: i8,
    /// The vignette's shape, from a Lightroom preset's post-crop vignette (FR-118); no slider
    /// of its own. Absent before v0.14.1: Lightroom's defaults.
    pub vignette_shape: VignetteShape,
    /// Haze taken out (+) or laid on (−), before the table (FR-115, `dehaze`).
    pub dehaze: i8,
    /// The eight-hue colour mixer (FR-079, Lightroom's Color Mixer): each hue turned,
    /// saturated and lightened. Absent before it: untouched.
    pub mixer: crate::colour_ops::Mixer,
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

/// Film grain, sized on the print (session 13, `effects`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Grain {
    /// 0…100.
    pub amount: u8,
    /// 0…100, from fine to coarse.
    pub size: u8,
    /// 0…100, from even to rough (Lightroom's Roughness, `GrainFrequency`); 50 by default.
    pub roughness: u8,
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            amount: 0,
            size: 0,
            roughness: 50,
        }
    }
}

/// The post-crop vignette's shape (FR-118, Lightroom's Midpoint, Roundness, Feather and
/// Highlights), Lightroom's defaults when a look has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct VignetteShape {
    /// 0…100, where the fall starts: near the centre to near the corners.
    pub midpoint: u8,
    /// −100 (a rounded rectangle)…100 (a circle).
    pub roundness: i8,
    /// 0…100, from a hard edge to a soft one.
    pub feather: u8,
    /// 0…100, how much of the highlights a darkening keeps.
    pub highlights: u8,
}

impl Default for VignetteShape {
    fn default() -> Self {
        Self {
            midpoint: 50,
            roundness: 0,
            feather: 50,
            highlights: 0,
        }
    }
}

/// The sliders a camera RAW takes in scene-linear light before its tone (FR-088): brightness
/// as exposure, highlights, shadows, warmth and tint, scaled by the look's amount. Kept as
/// the slider values, so it can key a cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RawSliders {
    pub brightness: i8,
    pub highlights: i8,
    pub shadows: i8,
    pub warmth: i8,
    pub tint: i8,
    pub amount: u8,
}

impl RawSliders {
    /// The adjustment in linear light: brightness ±100 is ±2 stops; highlights and shadows
    /// ±100 are rawcore's ±1; warmth and tint the same luminance-kept gains as on a JPEG
    /// (`Params`), so each slider means the same on both.
    pub fn adjust(&self) -> fotollection_rawcore::RawAdjust {
        let k = f32::from(self.amount) / 100.0;
        let s = |v: i8| f32::from(v) / 100.0 * k;
        let (w, t) = (s(self.warmth), s(self.tint));
        let raw = [(0.3 * w).exp2(), (-0.45 * t).exp2(), (-0.3 * w).exp2()];
        let y = 0.2126 * raw[0] + 0.7152 * raw[1] + 0.0722 * raw[2];
        fotollection_rawcore::RawAdjust {
            exposure: 2.0 * s(self.brightness),
            highlights: s(self.highlights),
            shadows: s(self.shadows),
            gains: raw.map(|g| g / y),
        }
    }
}

impl LookSettings {
    /// For a camera RAW (FR-088): the five sliders it takes in linear light, and the look
    /// left for the table, those five at zero, so none is applied twice.
    #[must_use]
    pub fn split_for_raw(&self) -> (RawSliders, Self) {
        let a = self.adjust;
        let raw = RawSliders {
            brightness: a.brightness,
            highlights: a.highlights,
            shadows: a.shadows,
            warmth: a.warmth,
            tint: a.tint,
            amount: self.amount,
        };
        let mut rest = self.clone();
        rest.adjust.brightness = 0;
        rest.adjust.highlights = 0;
        rest.adjust.shadows = 0;
        rest.adjust.warmth = 0;
        rest.adjust.tint = 0;
        let raw = if a.brightness == 0
            && a.highlights == 0
            && a.shadows == 0
            && a.warmth == 0
            && a.tint == 0
        {
            RawSliders::default()
        } else {
            raw
        };
        (raw, rest)
    }

    /// Only the five sliders a RAW takes in linear light, the rest at rest: a RAW's base
    /// for the webview's look while dragging (FR-088), whose table holds the rest.
    #[must_use]
    pub fn raw_only(&self) -> Self {
        let mut only = Self {
            amount: self.amount,
            ..Self::default()
        };
        only.adjust.brightness = self.adjust.brightness;
        only.adjust.highlights = self.adjust.highlights;
        only.adjust.shadows = self.adjust.shadows;
        only.adjust.warmth = self.adjust.warmth;
        only.adjust.tint = self.adjust.tint;
        only
    }

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
            &mut a.dehaze,
            &mut a.vignette_shape.roundness,
        ] {
            *v = (*v).clamp(-100, 100);
        }
        for v in [
            &mut a.fade,
            &mut a.shadow_tint.amount,
            &mut a.highlight_tint.amount,
            &mut a.grain.amount,
            &mut a.grain.size,
            &mut a.grain.roughness,
            &mut a.vignette_shape.midpoint,
            &mut a.vignette_shape.feather,
            &mut a.vignette_shape.highlights,
            &mut self.lut_strength,
            &mut self.amount,
        ] {
            *v = (*v).min(100);
        }
        a.mixer = a.mixer.sanitized();
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
        self.name = self.name.as_deref().and_then(clean_name);
        if self.base.as_deref().is_some_and(|b| !valid_base(b)) {
            self.base = None;
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
        let colour_only = self.colour_only();
        let key = colour_only.colour_hash()?;
        Some(TABLES.get(key, || bake(&colour_only)))
    }

    /// The dehaze this look gives a photo, scaled by its amount (FR-115): it is drawn
    /// before the table, never in it.
    #[must_use]
    pub fn dehaze(&self) -> i8 {
        let d =
            i16::from(self.adjust.dehaze.clamp(-100, 100)) * i16::from(self.amount.min(100)) / 100;
        i8::try_from(d).unwrap_or(0)
    }

    /// The look's grain and vignette on a print at `px_per_mm` (FR-079, session 13), scaled
    /// by its amount; `None` when they draw nothing. `seed` keeps a photo's grain the same at
    /// every export.
    #[must_use]
    pub fn effects(&self, px_per_mm: f32, seed: u32) -> Option<crate::effects::Effects> {
        crate::effects::Effects::of(&self.adjust, self.amount, px_per_mm, seed)
    }

    /// The look on `image` as every path draws it: the dehaze, then the table.
    pub fn draw(&self, image: &mut RgbImage) {
        crate::dehaze::apply(image, self.dehaze());
        if let Some(table) = self.table() {
            apply(image, &table);
        }
    }

    /// What the table holds: the colour, without grain, vignette, dehaze, name or base.
    fn colour_only(&self) -> LookSettings {
        LookSettings {
            adjust: Adjust {
                grain: Grain::default(),
                vignette: 0,
                vignette_shape: VignetteShape::default(),
                dehaze: 0,
                ..self.adjust
            },
            name: None,
            base: None,
            ..self.clone()
        }
    }

    fn colour_hash(&self) -> Option<blake3::Hash> {
        (!self.is_identity()).then(|| blake3::hash(&serde_json::to_vec(self).unwrap_or_default()))
    }

    /// A short key of this look's colour (FR-114, ADR-0047): the same for two looks that
    /// colour a photo alike whatever their names, 0 for a look that changes nothing. A
    /// thumbnail's address carries it, so the grid fetches a photo again when its colour
    /// changes.
    #[must_use]
    pub fn colour_key(&self) -> u32 {
        let colour = self.colour_only().colour_hash();
        let hash = match (colour, self.dehaze()) {
            (colour, 0) => colour,
            // The dehaze changes the thumbnail too, so it changes the key.
            (colour, d) => {
                let mut h = blake3::Hasher::new();
                h.update(colour.as_ref().map_or(&[0u8; 32], |c| c.as_bytes()));
                h.update(&d.to_le_bytes());
                Some(h.finalize())
            }
        };
        hash.map_or(0, |h| {
            let b = h.as_bytes();
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]).max(1)
        })
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
    /// The colour mixer at the look's amount; `None` when untouched.
    mixer: Option<crate::colour_ops::MixerParams>,
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
            mixer: crate::colour_ops::MixerParams::from(&a.mixer, k),
        }
    }
}

/// The warmth and tint, in slider steps at full amount, that turn the encoded colour `rgb`
/// (0–255, as the photo is before its look) into a neutral grey (FR-098, `W`): the exact inverse
/// of [`Params::new`]'s gains, `2^(0.3·w)` on red, `2^(−0.45·t)` on green, `2^(−0.3·w)` on blue,
/// whose ratios the luminance normalisation leaves alone. Not clamped: the caller adds what the
/// photo already has and keeps the result within the sliders' range.
pub fn white_balance_from(rgb: [f32; 3]) -> (f32, f32) {
    let [r, g, b] = rgb.map(|v| to_linear((v / 255.0).clamp(0.0, 1.0)).max(1e-4));
    let warmth = 100.0 * (b / r).log2() / 0.6;
    let tint = 100.0 * (g / (r * b).sqrt()).log2() / 0.45;
    (warmth, tint)
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
    // The imported LUT first (FR-070, ADR-0038), mixed by its strength and the look's amount,
    // then the sliders on what it gave. A LUT whose file is missing is left out.
    let lut = look.lut.as_deref().and_then(crate::lut::get);
    let mix = f32::from(look.lut_strength) / 100.0 * f32::from(look.amount) / 100.0;
    Table::from_fn(|c| {
        let c = match &lut {
            Some(lut) => {
                let l = lut.apply(c);
                [0, 1, 2].map(|i| (c[i] + (l[i] - c[i]) * mix).clamp(0.0, 1.0))
            }
            None => c,
        };
        pixel(c, &p)
    })
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
    // The colour mixer, as Lightroom's after the tone and saturation, on encoded values as
    // a preset's mixer is read (`colour_ops`, the same function the 32-bit path will call).
    if let Some(m) = &p.mixer {
        lin = m
            .apply(lin.map(to_encoded))
            .map(|v| to_linear(v.clamp(0.0, 1.0)));
    }
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

    /// FR-088: on a RAW, brightness, highlights, shadows, warmth and tint leave the table for
    /// linear light, the rest stays; warmth's gains are the table's own, white kept.
    #[test]
    fn fr_088_five_sliders_leave_the_table_for_a_raw() {
        let mut look = LookSettings::default();
        look.adjust.brightness = 50;
        look.adjust.warmth = 40;
        look.adjust.saturation = 30;
        let (raw, rest) = look.split_for_raw();
        assert_eq!((raw.brightness, raw.warmth, raw.amount), (50, 40, 100));
        assert_eq!(
            (
                rest.adjust.brightness,
                rest.adjust.warmth,
                rest.adjust.saturation
            ),
            (0, 0, 30)
        );
        let a = raw.adjust();
        assert!((a.exposure - 1.0).abs() < 1e-6);
        let y = 0.2126 * a.gains[0] + 0.7152 * a.gains[1] + 0.0722 * a.gains[2];
        assert!((y - 1.0).abs() < 1e-5 && a.gains[0] > a.gains[2]);
        // Only the five set: nothing left for the table.
        look.adjust.saturation = 0;
        assert!(look.split_for_raw().1.table().is_none());
        assert_eq!(
            LookSettings::default().split_for_raw().0,
            RawSliders::default()
        );
    }

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

    /// FR-098: a grey patch tinted by known warmth and tint is picked back to them, so the
    /// picker's correction cancels the cast to within a step.
    #[test]
    fn fr_098_white_balance_round_trips() {
        for (w, t) in [(30i8, 0i8), (-40, 20), (0, -35), (55, 45)] {
            let mut look = LookSettings::default();
            (look.adjust.warmth, look.adjust.tint) = (w, t);
            let p = Params::new(&look);
            // A mid grey seen through the cast, as a camera with the wrong white balance would.
            let lin = [0.2f32; 3];
            let cast = [0, 1, 2].map(|i| to_encoded(lin[i] / p.gains[i]) * 255.0);
            let (pw, pt) = white_balance_from(cast);
            assert!((pw - f32::from(w)).abs() < 1.0, "warmth {w}: picked {pw}");
            assert!((pt - f32::from(t)).abs() < 1.0, "tint {t}: picked {pt}");
        }
        let (w, t) = white_balance_from([128.0; 3]);
        assert!(w.abs() < 0.01 && t.abs() < 0.01, "a grey needs nothing");
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

    /// FR-115: a look from before Dehaze has none; one out of range is brought within; it
    /// never enters the table, it changes the thumbnail's key, and the amount scales it.
    #[test]
    fn fr_115_a_looks_dehaze_is_read_clamped_and_kept_out_of_the_table()
    -> Result<(), serde_json::Error> {
        let old: LookSettings = serde_json::from_str(r#"{"adjust":{"saturation":10}}"#)?;
        assert_eq!(old.adjust.dehaze, 0);
        let wild: LookSettings = serde_json::from_str(r#"{"adjust":{"dehaze":-128}}"#)?;
        assert_eq!(wild.sanitized().adjust.dehaze, -100);
        let hazy = LookSettings {
            adjust: Adjust {
                dehaze: 40,
                ..Adjust::default()
            },
            ..LookSettings::default()
        };
        assert!(hazy.table().is_none(), "no table for a dehaze alone");
        assert_ne!(hazy.colour_key(), 0, "a thumbnail is fetched again");
        assert_eq!(LookSettings { amount: 50, ..hazy }.dehaze(), 20);
        Ok(())
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
                    roughness: 200,
                },
                ..Adjust::default()
            },
            lut: Some("../../etc/passwd".into()),
            lut_strength: 250,
            amount: 101,
            ..LookSettings::default()
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

    #[test]
    fn fr_079_a_looks_name_and_base_never_change_its_table() {
        let plain = with(|a| a.warmth = 30);
        let named = LookSettings {
            name: Some("Warm old film".into()),
            base: Some("builtin:warm-film".into()),
            ..plain.clone()
        };
        let (Some(a), Some(b)) = (plain.table(), named.table()) else {
            panic!("both looks warm the photo");
        };
        assert!(Arc::ptr_eq(&a, &b), "one cached table for both");
        let bare = LookSettings {
            name: Some("Nothing".into()),
            ..LookSettings::default()
        };
        assert!(bare.table().is_none(), "a name alone changes no pixel");
    }

    #[test]
    fn fr_079_a_looks_name_and_base_are_sanitized() {
        let long = "x".repeat(MAX_LOOK_NAME + 10);
        let look = LookSettings {
            name: Some(format!("  Film\u{7}\n{long}")),
            base: Some("builtin:../etc".into()),
            ..LookSettings::default()
        }
        .sanitized();
        let name = look.name.unwrap_or_default();
        assert!(name.starts_with("Film") && name.chars().count() == MAX_LOOK_NAME);
        assert!(!name.chars().any(char::is_control));
        assert_eq!(look.base, None, "an unknown reference is dropped");
        assert_eq!(clean_name(" \t "), None);
        for ok in ["builtin:c200-natural", "user:3f9a", "lut:0a1b"] {
            assert!(valid_base(ok), "{ok}");
        }
        for bad in ["c200", "other:x", "user:", "user:UPPER", "lut:a/b"] {
            assert!(!valid_base(bad), "{bad}");
        }
    }
}
