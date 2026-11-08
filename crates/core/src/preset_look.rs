// SPDX-License-Identifier: GPL-3.0-only
//! A Lightroom® preset compiled into a look (FR-089, ADR-0045): the settings our sliders have
//! go to them; the per-pixel ones they lack (point curves, whites and blacks, the eight-hue
//! mixer, colour grading, split toning, the black-and-white mixer) are baked into one 33³ LUT
//! saved as an imported LUT is, so a preset is one table as every look. What a table cannot
//! hold (clarity, texture, dehaze, sharpening, noise reduction, lens corrections) and what it
//! cannot know (an absolute white balance, Adobe's profiles) is named, never dropped silently.
//!
//! The mappings are written from what each setting is documented to do, not from Adobe's
//! code; their strengths are first guesses, 1:1, until `preset_bench` tunes them against
//! Lightroom's own renders (RESULT §36).

use std::fmt::Write as _;

use crate::looks::{Adjust, Grain, LookSettings, Mono};
use crate::lut::{Cube, Lut};
use crate::preset::{Preset, Value};

/// The eight hues of Lightroom's mixer, their centres on an HSV hue wheel (degrees).
const BANDS: [(&str, f32); 8] = [
    ("Red", 0.0),
    ("Orange", 30.0),
    ("Yellow", 60.0),
    ("Green", 120.0),
    ("Aqua", 180.0),
    ("Blue", 240.0),
    ("Purple", 270.0),
    ("Magenta", 300.0),
];

/// Points a side of the baked table.
const GRID: usize = 33;

/// A preset made a look.
#[derive(Debug, Clone, PartialEq)]
pub struct Compiled {
    pub look: LookSettings,
    /// The baked LUT as `.cube` text, to be kept by `lut::import`; `None` when the sliders
    /// hold everything.
    pub lut: Option<String>,
    /// The settings not applied, each with its reason, for the import's report.
    pub unapplied: Vec<String>,
}

fn number(p: &Preset, key: &str) -> f32 {
    match p.values.get(key) {
        Some(Value::Number(v)) => *v,
        _ => 0.0,
    }
}

fn flag(p: &Preset, key: &str) -> bool {
    matches!(p.values.get(key), Some(Value::Text(t)) if t.eq_ignore_ascii_case("true"))
}

#[allow(clippy::cast_possible_truncation)]
fn slider(v: f32) -> i8 {
    v.round().clamp(-100.0, 100.0) as i8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn amount(v: f32) -> u8 {
    v.round().clamp(0.0, 100.0) as u8
}

/// A point curve, 0–255, as a function on 0–1; `None` when it is the identity.
fn curve(p: &Preset, key: &str) -> Option<Vec<(f32, f32)>> {
    let Some(Value::Curve(points)) = p.values.get(key) else {
        return None;
    };
    let mut pts: Vec<(f32, f32)> = points
        .iter()
        .map(|&(x, y)| (x / 255.0, y / 255.0))
        .collect();
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6);
    let identity = pts.iter().all(|(x, y)| (x - y).abs() < 1e-3);
    (pts.len() >= 2 && !identity).then_some(pts)
}

/// A monotone cubic through `pts` (Fritsch and Carlson's condition, with Fritsch and
/// Butland's harmonic-mean tangents, 1980–84): no overshoot between points,
/// so a curve that never falls never makes a tone fall.
fn monotone(pts: &[(f32, f32)], x: f32) -> f32 {
    let n = pts.len();
    if x <= pts[0].0 {
        return pts[0].1;
    }
    if x >= pts[n - 1].0 {
        return pts[n - 1].1;
    }
    let slope = |i: usize| (pts[i + 1].1 - pts[i].1) / (pts[i + 1].0 - pts[i].0).max(1e-6);
    let tangent = |i: usize| -> f32 {
        if i == 0 {
            slope(0)
        } else if i == n - 1 {
            slope(n - 2)
        } else {
            let (a, b) = (slope(i - 1), slope(i));
            if a * b <= 0.0 {
                0.0
            } else {
                // The harmonic mean of the slopes either side: never steeper than three times
                // the gentler one, which keeps the cubic from overshooting.
                2.0 * a * b / (a + b)
            }
        }
    };
    let i = pts.windows(2).position(|w| x <= w[1].0).unwrap_or(n - 2);
    let h = (pts[i + 1].0 - pts[i].0).max(1e-6);
    let t = (x - pts[i].0) / h;
    let (m0, m1) = (tangent(i) * h, tangent(i + 1) * h);
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * pts[i].1
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * pts[i + 1].1
        + (t3 - t2) * m1
}

fn to_hsv(c: [f32; 3]) -> [f32; 3] {
    let (max, min) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    let d = max - min;
    // The channel that is the largest, by index: red first, then green, as HSV defines it.
    let top = if c[0] >= c[1] && c[0] >= c[2] {
        0
    } else if c[1] >= c[2] {
        1
    } else {
        2
    };
    let h = if d < 1e-6 {
        0.0
    } else if top == 0 {
        60.0 * ((c[1] - c[2]) / d).rem_euclid(6.0)
    } else if top == 1 {
        60.0 * ((c[2] - c[0]) / d + 2.0)
    } else {
        60.0 * ((c[0] - c[1]) / d + 4.0)
    };
    [h, if max > 0.0 { d / max } else { 0.0 }, max]
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn from_hsv([h, s, v]: [f32; 3]) -> [f32; 3] {
    let c = v * s;
    let hp = h.rem_euclid(360.0) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [r + m, g + m, b + m]
}

/// How much of each of the eight hues a hue belongs to: a triangle between neighbours, so a
/// colour between two bands takes some of both and the mixer has no seams.
fn band_weights(h: f32) -> [f32; 8] {
    let mut w = [0f32; 8];
    for i in 0..8 {
        let (c0, c1) = (BANDS[i].1, if i == 7 { 360.0 } else { BANDS[i + 1].1 });
        let h = h.rem_euclid(360.0);
        if h >= c0 && h <= c1 {
            let t = (h - c0) / (c1 - c0);
            w[i] += 1.0 - t;
            w[(i + 1) % 8] += t;
            break;
        }
    }
    w
}

/// A hue and saturation (0–100) as an RGB offset around grey, for colour grading's tints.
fn zero(t: &[f32; 3]) -> bool {
    t.iter().all(|v| *v == 0.0)
}

fn tint(hue: f32, sat: f32) -> [f32; 3] {
    let c = from_hsv([hue, 1.0, 1.0]);
    let k = sat / 100.0;
    [0, 1, 2].map(|i| (c[i] - 0.5) * k)
}

/// The per-pixel part of a preset that our sliders lack, as a function on encoded RGB.
struct Baked {
    whites: f32,
    blacks: f32,
    master: Option<Vec<(f32, f32)>>,
    channels: [Option<Vec<(f32, f32)>>; 3],
    hue: [f32; 8],
    sat: [f32; 8],
    lum: [f32; 8],
    grade: [([f32; 3], f32); 4],
    blending: f32,
    balance: f32,
    gray: Option<[f32; 8]>,
}

impl Baked {
    fn from(p: &Preset) -> Self {
        let per_band = |prefix: &str| BANDS.map(|(name, _)| number(p, &format!("{prefix}{name}")));
        let zone = |name: &str| {
            (
                tint(
                    number(p, &format!("ColorGrade{name}Hue")),
                    number(p, &format!("ColorGrade{name}Sat")),
                ),
                number(p, &format!("ColorGrade{name}Lum")) / 100.0,
            )
        };
        let mut grade = [
            zone("Shadow"),
            zone("Midtone"),
            zone("Highlight"),
            zone("Global"),
        ];
        // Split toning, the colour grading of older presets: shadows and highlights only.
        for (i, name) in [(0, "Shadow"), (2, "Highlight")] {
            let sat = number(p, &format!("SplitToning{name}Saturation"));
            if sat != 0.0 && zero(&grade[i].0) {
                grade[i].0 = tint(number(p, &format!("SplitToning{name}Hue")), sat);
            }
        }
        let balance = if p.values.contains_key("ColorGradeBalance") {
            number(p, "ColorGradeBalance")
        } else {
            number(p, "SplitToningBalance")
        };
        Self {
            whites: number(p, "Whites2012") / 100.0,
            blacks: number(p, "Blacks2012") / 100.0,
            master: curve(p, "ToneCurvePV2012"),
            channels: [
                curve(p, "ToneCurvePV2012Red"),
                curve(p, "ToneCurvePV2012Green"),
                curve(p, "ToneCurvePV2012Blue"),
            ],
            hue: per_band("HueAdjustment"),
            sat: per_band("SaturationAdjustment"),
            lum: per_band("LuminanceAdjustment"),
            grade,
            blending: if p.values.contains_key("ColorGradeBlending") {
                number(p, "ColorGradeBlending") / 100.0
            } else {
                0.5
            },
            balance: balance / 100.0,
            gray: flag(p, "ConvertToGrayscale")
                .then(|| per_band("GrayMixer"))
                .filter(|m| m.iter().any(|v| *v != 0.0)),
        }
    }

    fn is_identity(&self) -> bool {
        self.whites == 0.0
            && self.blacks == 0.0
            && self.master.is_none()
            && self.channels.iter().all(Option::is_none)
            && [self.hue, self.sat, self.lum]
                .iter()
                .all(|b| b.iter().all(|v| *v == 0.0))
            && self.grade.iter().all(|(t, l)| zero(t) && *l == 0.0)
            && self.gray.is_none()
    }

    fn apply(&self, c: [f32; 3]) -> [f32; 3] {
        let mut c = c;
        // Whites and blacks: the ends of the tone range moved, the middle kept.
        if self.whites != 0.0 || self.blacks != 0.0 {
            c = c.map(|v| {
                v + 0.25 * self.whites * v * v + 0.25 * self.blacks * (1.0 - v) * (1.0 - v)
            });
        }
        if let Some(m) = &self.master {
            c = c.map(|v| monotone(m, v.clamp(0.0, 1.0)));
        }
        for (i, ch) in self.channels.iter().enumerate() {
            if let Some(k) = ch {
                c[i] = monotone(k, c[i].clamp(0.0, 1.0));
            }
        }
        // The eight-hue mixer, weighted by how saturated a colour is, so greys stay put.
        let [h, s, v] = to_hsv(c.map(|x| x.clamp(0.0, 1.0)));
        if s > 1e-4 {
            let w = band_weights(h);
            let dot = |b: &[f32; 8]| w.iter().zip(b).map(|(w, b)| w * b).sum::<f32>() / 100.0;
            let (dh, ds, dl) = (dot(&self.hue) * 30.0, dot(&self.sat), dot(&self.lum));
            if dh != 0.0 || ds != 0.0 || dl != 0.0 {
                c = from_hsv([
                    h + dh,
                    (s * (1.0 + ds)).clamp(0.0, 1.0),
                    (v * (1.0 + 0.5 * dl * s)).clamp(0.0, 1.0),
                ]);
            }
        }
        // Black and white through the eight-hue mixer: luminance, each hue lighter or darker.
        if let Some(mix) = &self.gray {
            let y = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            let w = band_weights(h);
            let k = w.iter().zip(mix).map(|(w, m)| w * m).sum::<f32>() / 100.0;
            let g = (y * (1.0 + k * s)).clamp(0.0, 1.0);
            c = [g; 3];
        }
        // Colour grading: shadows, midtones and highlights by lightness, then everywhere.
        let y = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let width = 0.25 + 0.5 * self.blending;
        let pivot = 0.5 + 0.3 * self.balance;
        let shadow = (1.0 - y / pivot.max(1e-3))
            .clamp(0.0, 1.0)
            .powf(1.0 / width.max(0.1));
        let high = ((y - pivot) / (1.0 - pivot).max(1e-3))
            .clamp(0.0, 1.0)
            .powf(1.0 / width.max(0.1));
        let mid = (1.0 - shadow - high).max(0.0);
        for ((t, l), weight) in self.grade.iter().zip([shadow, mid, high, 1.0]) {
            if weight > 0.0 && (!zero(t) || *l != 0.0) {
                c = [0, 1, 2].map(|i| c[i] + weight * (t[i] * 0.5 + l * 0.25));
            }
        }
        c.map(|v| v.clamp(0.0, 1.0))
    }
}

/// A Kelvin temperature taken against daylight (5,500 K) when a preset gives only an
/// absolute white balance, as the legacy "Warm It Up" kind do: a guess, 100 for each 5,500 K
/// above, to be tuned with `preset_bench` (an absolute balance needs the photo's own).
const DAYLIGHT_K: f32 = 5500.0;

fn absolute_only(p: &Preset) -> bool {
    p.values.contains_key("Temperature") && !p.values.contains_key("IncrementalTemperature")
}

fn warmth(p: &Preset) -> f32 {
    if absolute_only(p) {
        (number(p, "Temperature") - DAYLIGHT_K) / DAYLIGHT_K * 100.0
    } else {
        number(p, "IncrementalTemperature")
    }
}

/// Lightroom's absolute tint runs −150…150, ours −100…100.
fn tint_of(p: &Preset) -> f32 {
    if absolute_only(p) {
        number(p, "Tint") * 100.0 / 150.0
    } else {
        number(p, "IncrementalTint")
    }
}

/// The settings a look cannot take, each named once with its reason, in the order read.
fn unapplied(p: &Preset) -> Vec<String> {
    let mut out = Vec::new();
    let set = |k: &str| match p.values.get(k) {
        Some(Value::Number(v)) => *v != 0.0,
        Some(Value::Text(t)) => t.eq_ignore_ascii_case("true"),
        _ => false,
    };
    for (key, reason) in [
        ("Clarity2012", "local contrast, not a colour of each pixel"),
        ("Texture", "local detail, not a colour of each pixel"),
        ("Dehaze", "local contrast, not a colour of each pixel"),
        ("Sharpness", "sharpening, not a colour"),
        ("LuminanceSmoothing", "noise reduction, not a colour"),
        ("ColorNoiseReduction", "noise reduction, not a colour"),
        ("LensProfileEnable", "lens corrections, not a colour"),
        ("ParametricShadows", "the parametric curve, not read yet"),
        ("ParametricDarks", "the parametric curve, not read yet"),
        ("ParametricLights", "the parametric curve, not read yet"),
        ("ParametricHighlights", "the parametric curve, not read yet"),
    ] {
        if set(key) {
            out.push(format!("{key}: {reason}"));
        }
    }
    if absolute_only(p) {
        out.push(
            "Temperature: an absolute white balance, taken against daylight (5,500 K), approximate"
                .into(),
        );
    }
    if set("GrainAmount") || set("PostCropVignetteAmount") {
        out.push("Grain and vignette: kept in the look, drawn once session 13 is built".into());
    }
    let old = p
        .process
        .as_deref()
        .and_then(|v| v.split('.').next())
        .and_then(|v| v.parse::<u32>().ok())
        .is_some_and(|major| major < 6);
    if old {
        out.push("ProcessVersion: older than Lightroom's 2012 process, mapped roughly".into());
    }
    out
}

/// The baked function as a 33³ `.cube`, red fastest.
#[allow(clippy::cast_precision_loss)]
fn cube_text(baked: &Baked) -> String {
    let mut s = String::with_capacity(GRID * GRID * GRID * 28 + 64);
    s.push_str("TITLE \"Fotollection preset\"\nLUT_3D_SIZE 33\n");
    let step = 1.0 / (GRID - 1) as f32;
    for b in 0..GRID {
        for g in 0..GRID {
            for r in 0..GRID {
                let o = baked.apply([r as f32 * step, g as f32 * step, b as f32 * step]);
                let _ = writeln!(s, "{:.6} {:.6} {:.6}", o[0], o[1], o[2]);
            }
        }
    }
    s
}

/// Compiles a preset into a look; the caller keeps `lut` with `lut::import` and puts its hash
/// in `look.lut`.
pub fn compile(p: &Preset) -> Compiled {
    let mut adjust = Adjust {
        brightness: slider(number(p, "Exposure2012") * 25.0),
        contrast: slider(number(p, "Contrast2012")),
        highlights: slider(number(p, "Highlights2012")),
        shadows: slider(number(p, "Shadows2012")),
        saturation: slider(number(p, "Saturation")),
        vibrance: slider(number(p, "Vibrance")),
        warmth: slider(warmth(p)),
        tint: slider(tint_of(p)),
        grain: Grain {
            amount: amount(number(p, "GrainAmount")),
            size: amount(number(p, "GrainSize")),
        },
        vignette: slider(number(p, "PostCropVignetteAmount")),
        ..Adjust::default()
    };
    let baked = Baked::from(p);
    // Grey without a mixer is our own neutral black and white; with one, the table does it.
    if flag(p, "ConvertToGrayscale") && baked.gray.is_none() {
        adjust.mono = Mono::Neutral;
    }
    Compiled {
        look: LookSettings {
            adjust,
            ..LookSettings::default()
        },
        lut: (!baked.is_identity()).then(|| cube_text(&baked)),
        unapplied: unapplied(p),
    }
}

/// The baked table itself, for tests and measurements, without writing `.cube` text.
#[must_use]
pub fn baked_lut(p: &Preset) -> Option<Lut> {
    let baked = Baked::from(p);
    if baked.is_identity() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let step = 1.0 / (GRID - 1) as f32;
    let mut data = Vec::with_capacity(GRID * GRID * GRID);
    for b in 0..GRID {
        for g in 0..GRID {
            for r in 0..GRID {
                #[allow(clippy::cast_precision_loss)]
                data.push(baked.apply([r as f32 * step, g as f32 * step, b as f32 * step]));
            }
        }
    }
    Some(Lut {
        title: None,
        shaper: None,
        cube: Some(Cube {
            size: GRID,
            domain: [[0.0; 3], [1.0; 3]],
            data,
        }),
        vlog: false,
    })
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)]
mod tests {
    use super::*;

    fn preset(pairs: &[(&str, Value)]) -> Preset {
        Preset {
            values: pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
            ..Preset::default()
        }
    }

    #[test]
    fn fr_089_a_rising_curve_never_falls() {
        let pts = vec![
            (0.0, 0.0),
            (0.2, 0.05),
            (0.25, 0.6),
            (0.6, 0.62),
            (1.0, 1.0),
        ];
        let mut last = -1.0f32;
        for i in 0..=1000 {
            let y = monotone(&pts, i as f32 / 1000.0);
            assert!(y >= last - 1e-6, "falls at {i}: {y} < {last}");
            last = y;
        }
    }

    #[test]
    fn fr_089_sliders_go_to_ours_and_need_no_table() {
        let c = compile(&preset(&[
            ("Contrast2012", Value::Number(25.0)),
            ("Vibrance", Value::Number(-10.0)),
            ("IncrementalTemperature", Value::Number(12.0)),
            (
                "ToneCurvePV2012",
                Value::Curve(vec![(0.0, 0.0), (255.0, 255.0)]),
            ),
            ("HueAdjustmentBlue", Value::Number(0.0)),
        ]));
        assert_eq!(
            (
                c.look.adjust.contrast,
                c.look.adjust.vibrance,
                c.look.adjust.warmth
            ),
            (25, -10, 12)
        );
        assert_eq!(
            c.lut, None,
            "an identity curve and a zero mixer bake nothing"
        );
        assert_eq!(c.unapplied, Vec::<String>::new());
    }

    #[test]
    fn fr_089_a_curve_and_the_mixer_are_baked() {
        let s_curve = preset(&[(
            "ToneCurvePV2012",
            Value::Curve(vec![
                (0.0, 0.0),
                (64.0, 40.0),
                (192.0, 215.0),
                (255.0, 255.0),
            ]),
        )]);
        let lut = baked_lut(&s_curve).unwrap_or_else(|| panic!("an S curve bakes"));
        let dark = lut.apply([0.25; 3])[0];
        let light = lut.apply([0.75; 3])[0];
        assert!(dark < 0.22 && light > 0.8, "contrast: {dark} {light}");
        let mid = lut.apply([0.5, 0.5, 0.5]);
        assert!(
            (mid[0] - mid[1]).abs() < 1e-3 && (mid[1] - mid[2]).abs() < 1e-3,
            "grey stays grey: {mid:?}"
        );
        // Blue darker: a blue loses lightness, an orange none.
        let mixer = preset(&[("LuminanceAdjustmentBlue", Value::Number(-80.0))]);
        let lut = baked_lut(&mixer).unwrap_or_else(|| panic!("the mixer bakes"));
        assert!(lut.apply([0.1, 0.2, 0.9])[2] < 0.85);
        let orange = lut.apply([0.9, 0.5, 0.1]);
        assert!((orange[0] - 0.9).abs() < 0.01, "{orange:?}");
        assert!(
            compile(&mixer)
                .lut
                .is_some_and(|t| t.starts_with("TITLE") && t.lines().count() == 2 + 33 * 33 * 33)
        );
    }

    #[test]
    fn fr_089_grading_tints_shadows_and_a_mixer_makes_grey() {
        // Teal shadows: a dark grey gets more blue than red, a light grey stays nearly grey.
        let teal = preset(&[
            ("ColorGradeShadowHue", Value::Number(190.0)),
            ("ColorGradeShadowSat", Value::Number(60.0)),
        ]);
        let lut = baked_lut(&teal).unwrap_or_else(|| panic!("grading bakes"));
        let dark = lut.apply([0.15; 3]);
        assert!(dark[2] > dark[0] + 0.02, "{dark:?}");
        let light = lut.apply([0.9; 3]);
        assert!((light[2] - light[0]).abs() < 0.02, "{light:?}");
        let bw = preset(&[
            ("ConvertToGrayscale", Value::Text("True".into())),
            ("GrayMixerRed", Value::Number(40.0)),
        ]);
        let out = baked_lut(&bw)
            .map(|l| l.apply([0.8, 0.2, 0.2]))
            .unwrap_or_default();
        assert!(
            (out[0] - out[1]).abs() < 1e-4 && (out[1] - out[2]).abs() < 1e-4,
            "grey: {out:?}"
        );
        let plain_bw = compile(&preset(&[(
            "ConvertToGrayscale",
            Value::Text("True".into()),
        )]));
        assert_eq!(
            (plain_bw.look.adjust.mono, plain_bw.lut),
            (Mono::Neutral, None)
        );
    }

    #[test]
    fn fr_089_an_absolute_white_balance_is_taken_against_daylight() {
        let warm = compile(&preset(&[
            ("Temperature", Value::Number(6375.0)),
            ("Tint", Value::Number(-3.0)),
        ]));
        assert_eq!((warm.look.adjust.warmth, warm.look.adjust.tint), (16, -2));
        let relative = compile(&preset(&[
            ("Temperature", Value::Number(6375.0)),
            ("IncrementalTemperature", Value::Number(5.0)),
        ]));
        assert_eq!(relative.look.adjust.warmth, 5, "a relative one wins");
    }

    #[test]
    fn fr_089_what_no_look_can_hold_is_named() {
        let c = compile(&Preset {
            process: Some("5.0".into()),
            ..preset(&[
                ("Clarity2012", Value::Number(30.0)),
                ("Dehaze", Value::Number(0.0)),
                ("Temperature", Value::Number(6375.0)),
                ("GrainAmount", Value::Number(20.0)),
            ])
        });
        let text = c.unapplied.join("\n");
        assert!(
            text.contains("Clarity2012") && !text.contains("Dehaze"),
            "{text}"
        );
        assert!(
            text.contains("Temperature")
                && text.contains("Grain")
                && text.contains("ProcessVersion"),
            "{text}"
        );
        assert_eq!(c.look.adjust.grain.amount, 20, "kept for session 13");
    }
}
