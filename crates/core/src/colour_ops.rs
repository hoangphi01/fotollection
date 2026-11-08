// SPDX-License-Identifier: GPL-3.0-only
//! Colour operations as functions of one pixel in `f32` (FR-079, ADR-0043): the eight-hue
//! colour mixer and the hue wheel it works on. Each takes and gives RGB in `f32`, assumes no bit
//! depth and clamps no top, so today's 33³ table calls it for each of its points
//! (`looks::bake`) and the 32-bit path of ADR-0043 can call it on every pixel of a float image
//! unchanged. The sliders stay integers (−100…100) where they are stored and shown;
//! [`MixerParams::from`] is the one place they become factors.
//!
//! Lightroom presets' mixer settings land here too (`preset_look`), so a preset and the user's
//! own sliders colour a photo the same way.

use serde::{Deserialize, Serialize};
use specta::Type;

/// The eight hues of the mixer, as Lightroom's Color Mixer names them, their centres on an
/// HSV hue wheel (degrees).
pub const BANDS: [(&str, f32); 8] = [
    ("Red", 0.0),
    ("Orange", 30.0),
    ("Yellow", 60.0),
    ("Green", 120.0),
    ("Aqua", 180.0),
    ("Blue", 240.0),
    ("Purple", 270.0),
    ("Magenta", 300.0),
];

/// The most a hue slider turns its colour, in degrees, at ±100.
const HUE_RANGE: f32 = 30.0;

/// HSV of an RGB colour: hue in degrees, saturation 0–1, value the largest channel (any size).
pub fn to_hsv(c: [f32; 3]) -> [f32; 3] {
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

/// RGB of an HSV colour, the inverse of [`to_hsv`].
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn from_hsv([h, s, v]: [f32; 3]) -> [f32; 3] {
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
pub fn band_weights(h: f32) -> [f32; 8] {
    let mut w = [0f32; 8];
    let h = h.rem_euclid(360.0);
    for i in 0..8 {
        let (c0, c1) = (BANDS[i].1, if i == 7 { 360.0 } else { BANDS[i + 1].1 });
        if h >= c0 && h <= c1 {
            let t = (h - c0) / (c1 - c0);
            w[i] += 1.0 - t;
            w[(i + 1) % 8] += t;
            break;
        }
    }
    w
}

/// The colour mixer's sliders (FR-079): for each of the eight hues, its hue turned, its
/// saturation and its lightness, −100…100, 0 untouched. Stored with the look (project v12,
/// settings v18); absent means untouched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Mixer {
    pub hue: [i8; 8],
    pub sat: [i8; 8],
    pub lum: [i8; 8],
}

impl Mixer {
    /// True when no slider is moved.
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Every slider within −100…100.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        for v in self
            .hue
            .iter_mut()
            .chain(&mut self.sat)
            .chain(&mut self.lum)
        {
            *v = (*v).clamp(-100, 100);
        }
        self
    }
}

/// The mixer as factors, scaled by the look's amount: the one place sliders become `f32`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixerParams {
    /// Degrees each hue turns.
    hue: [f32; 8],
    /// −1…1: saturation scaled by `1 + sat`.
    sat: [f32; 8],
    /// −1…1: lightness scaled by `1 + lum/2`, on saturated colours only.
    lum: [f32; 8],
}

impl MixerParams {
    /// `mixer` at `amount` (0–1, the look's); `None` when it changes nothing.
    pub fn from(mixer: &Mixer, amount: f32) -> Option<Self> {
        if mixer.is_identity() || amount <= 0.0 {
            return None;
        }
        let k = |v: i8| f32::from(v) / 100.0 * amount;
        Some(Self {
            hue: mixer.hue.map(|v| k(v) * HUE_RANGE),
            sat: mixer.sat.map(k),
            lum: mixer.lum.map(k),
        })
    }

    /// One pixel through the mixer: each colour by how much it belongs to each hue, so greys
    /// stay put and neighbouring hues blend without a seam. Any `f32` RGB: nothing above 1 is
    /// clamped (a 32-bit pixel keeps its range); negatives count as 0 for the colour's hue.
    pub fn apply(&self, c: [f32; 3]) -> [f32; 3] {
        let [h, s, v] = to_hsv(c.map(|x| x.max(0.0)));
        if s <= 1e-4 {
            return c;
        }
        let w = band_weights(h);
        let dot = |b: &[f32; 8]| w.iter().zip(b).map(|(w, b)| w * b).sum::<f32>();
        let (dh, ds, dl) = (dot(&self.hue), dot(&self.sat), dot(&self.lum));
        if dh == 0.0 && ds == 0.0 && dl == 0.0 {
            return c;
        }
        from_hsv([
            h + dh,
            (s * (1.0 + ds)).clamp(0.0, 1.0),
            (v * (1.0 + 0.5 * dl * s)).max(0.0),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixer(band: usize, hue: i8, sat: i8, lum: i8) -> MixerParams {
        let mut m = Mixer::default();
        m.hue[band] = hue;
        m.sat[band] = sat;
        m.lum[band] = lum;
        let Some(p) = MixerParams::from(&m, 1.0) else {
            panic!("a moved slider gives factors");
        };
        p
    }

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    /// FR-079: an untouched mixer gives no factors at all, so the table is not changed.
    #[test]
    fn fr_079_an_untouched_mixer_changes_nothing() {
        assert!(MixerParams::from(&Mixer::default(), 1.0).is_none());
        let mut m = Mixer::default();
        m.sat[0] = -50;
        assert!(MixerParams::from(&m, 0.0).is_none());
    }

    /// FR-079: red's saturation at −100 makes red grey and leaves blue alone; greys never move.
    #[test]
    fn fr_079_red_desaturated_blue_and_greys_kept() {
        let p = mixer(0, 0, -100, 0);
        let red = p.apply([0.8, 0.1, 0.1]);
        assert!(
            (red[0] - red[1]).abs() < 1e-4 && (red[1] - red[2]).abs() < 1e-4,
            "{red:?}"
        );
        assert!(close(p.apply([0.1, 0.2, 0.9]), [0.1, 0.2, 0.9]));
        assert!(close(p.apply([0.4, 0.4, 0.4]), [0.4, 0.4, 0.4]));
    }

    /// FR-079: orange's hue at +100 turns orange toward yellow.
    #[test]
    fn fr_079_orange_turned_toward_yellow() {
        let p = mixer(1, 100, 0, 0);
        let before = to_hsv([0.9, 0.45, 0.0])[0];
        let after = to_hsv(p.apply([0.9, 0.45, 0.0]))[0];
        assert!(after > before + 20.0, "{before} → {after}");
    }

    /// FR-079: walking round the wheel, the output moves smoothly: no seam between bands.
    #[test]
    fn fr_079_no_seam_between_hues() {
        let p = mixer(2, 0, 80, -40);
        let mut last = p.apply(from_hsv([0.0, 0.8, 0.8]));
        for step in 1..=720 {
            let h = f32::from(u16::try_from(step).unwrap_or(0)) * 0.5;
            let now = p.apply(from_hsv([h, 0.8, 0.8]));
            let jump = now
                .iter()
                .zip(last)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max);
            assert!(jump < 0.03, "a jump of {jump} at {h}°");
            last = now;
        }
    }

    /// ADR-0043: a pixel above 1 (a 32-bit photo's highlight) keeps its range, not clipped.
    #[test]
    fn fr_079_a_float_pixel_keeps_its_range() {
        let p = mixer(5, 0, 40, 30);
        let bright = p.apply([0.5, 1.2, 3.0]);
        assert!(bright[2] > 3.0, "{bright:?}");
        assert!(bright.iter().all(|v| v.is_finite() && *v >= 0.0));
    }
}
