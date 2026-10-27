// SPDX-License-Identifier: GPL-3.0-only
//! Camera colour to sRGB, by the colour model of the DNG specification (1.7, chapter 6):
//! the camera's colour matrices for two calibration illuminants are blended by the colour
//! temperature of the shot's white, then camera RGB goes to CIE XYZ (D50) through the
//! forward matrix, or the inverse of the colour matrix and a Bradford adaptation, and on to
//! linear sRGB. Written from the specification.

use crate::tiff::{Ifd, Tiff};

pub type Mat3 = [[f64; 3]; 3];

pub mod tag {
    pub const COLOR_MATRIX_1: u16 = 50721;
    pub const COLOR_MATRIX_2: u16 = 50722;
    pub const CAMERA_CALIBRATION_1: u16 = 50723;
    pub const CAMERA_CALIBRATION_2: u16 = 50724;
    pub const ANALOG_BALANCE: u16 = 50727;
    pub const AS_SHOT_NEUTRAL: u16 = 50728;
    pub const BASELINE_EXPOSURE: u16 = 50730;
    pub const CALIBRATION_ILLUMINANT_1: u16 = 50778;
    pub const CALIBRATION_ILLUMINANT_2: u16 = 50779;
    pub const FORWARD_MATRIX_1: u16 = 50964;
    pub const FORWARD_MATRIX_2: u16 = 50965;
}

/// The colour tags of a raw image (DNG names; other formats fill what they know).
#[derive(Debug, Clone, Default)]
pub struct ColorTags {
    pub color_matrix: [Option<Mat3>; 2],
    pub forward_matrix: [Option<Mat3>; 2],
    pub camera_calibration: [Option<Mat3>; 2],
    /// Calibration illuminants as EXIF `LightSource` codes.
    pub illuminant: [u16; 2],
    pub analog_balance: [f64; 3],
    /// The camera's neutral (white) as camera RGB, from the white balance at the shot.
    pub as_shot_neutral: Option<[f64; 3]>,
    /// Stops to add for a normal exposure.
    pub baseline_exposure: f64,
}

fn mat3(v: &[f64]) -> Option<Mat3> {
    (v.len() >= 9).then(|| [[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]])
}

impl ColorTags {
    /// Reads the tags from the first directory, then from the raw image's (DNG 1.7 lets either
    /// hold them).
    pub fn read(tiff: &Tiff<'_>, first: &Ifd, raw: &Ifd) -> Self {
        let get = |t: u16| -> Vec<f64> {
            raw.get(t)
                .or_else(|| first.get(t))
                .and_then(|e| tiff.reals(e).ok())
                .unwrap_or_default()
        };
        let one = |t: u16| get(t).first().copied();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let illuminant = |t: u16| one(t).map_or(0, |v| v as u16);
        let neutral = get(tag::AS_SHOT_NEUTRAL);
        let balance = get(tag::ANALOG_BALANCE);
        Self {
            color_matrix: [
                mat3(&get(tag::COLOR_MATRIX_1)),
                mat3(&get(tag::COLOR_MATRIX_2)),
            ],
            forward_matrix: [
                mat3(&get(tag::FORWARD_MATRIX_1)),
                mat3(&get(tag::FORWARD_MATRIX_2)),
            ],
            camera_calibration: [
                mat3(&get(tag::CAMERA_CALIBRATION_1)),
                mat3(&get(tag::CAMERA_CALIBRATION_2)),
            ],
            illuminant: [
                illuminant(tag::CALIBRATION_ILLUMINANT_1),
                illuminant(tag::CALIBRATION_ILLUMINANT_2),
            ],
            analog_balance: match balance.as_slice() {
                [a, b, c, ..] => [*a, *b, *c],
                _ => [1.0; 3],
            },
            as_shot_neutral: match neutral.as_slice() {
                [a, b, c, ..] if *a > 0.0 && *b > 0.0 && *c > 0.0 => Some([*a, *b, *c]),
                _ => None,
            },
            baseline_exposure: one(tag::BASELINE_EXPOSURE).unwrap_or(0.0),
        }
    }
}

pub fn mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

pub fn apply(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

pub fn inverse(m: &Mat3) -> Option<Mat3> {
    let [[a, b, c], [d, e, f], [g, h, i]] = *m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det.abs() < 1e-12 || !det.is_finite() {
        return None;
    }
    Some([
        [
            (e * i - f * h) / det,
            (c * h - b * i) / det,
            (b * f - c * e) / det,
        ],
        [
            (f * g - d * i) / det,
            (a * i - c * g) / det,
            (c * d - a * f) / det,
        ],
        [
            (d * h - e * g) / det,
            (b * g - a * h) / det,
            (a * e - b * d) / det,
        ],
    ])
}

fn diag(v: [f64; 3]) -> Mat3 {
    [[v[0], 0.0, 0.0], [0.0, v[1], 0.0], [0.0, 0.0, v[2]]]
}

const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// D50 white, CIE XYZ (Y = 1), as the DNG specification uses.
pub const D50: [f64; 3] = [0.964_22, 1.0, 0.825_21];

/// XYZ (D50) to linear sRGB, with the Bradford adaptation from D50 to sRGB's D65 built in.
pub const XYZ_D50_TO_SRGB: Mat3 = [
    [3.133_856_1, -1.616_866_7, -0.490_614_6],
    [-0.978_768_4, 1.916_141_5, 0.033_454_0],
    [0.071_945_3, -0.228_991_4, 1.405_242_7],
];

/// The Bradford cone response matrix (Lam 1985).
const BRADFORD: Mat3 = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

/// The adaptation from a white `from` to a white `to` (both XYZ).
fn bradford(from: [f64; 3], to: [f64; 3]) -> Mat3 {
    let Some(inv) = inverse(&BRADFORD) else {
        return IDENTITY;
    };
    let (s, d) = (apply(&BRADFORD, from), apply(&BRADFORD, to));
    let scale = diag([d[0] / s[0], d[1] / s[1], d[2] / s[2]]);
    mul(&inv, &mul(&scale, &BRADFORD))
}

/// The colour temperature of an EXIF `LightSource` code (DNG uses these for calibration).
fn temperature(code: u16) -> Option<f64> {
    Some(match code {
        17 | 3 => 2856.0,
        24 => 3200.0,
        15 => 3450.0,
        2 | 14 => 4150.0,
        18 => 4874.0,
        13 => 5000.0,
        23 => 5003.0,
        1 | 4 | 9 => 5500.0,
        20 => 5503.0,
        12 => 6430.0,
        10 => 6500.0,
        21 => 6504.0,
        19 => 6774.0,
        11 => 7500.0,
        22 => 7504.0,
        _ => return None,
    })
}

/// The correlated colour temperature of a chromaticity, by the cubic formula of McCamy (1992).
#[allow(clippy::doc_markdown)]
fn cct(xyz: [f64; 3]) -> f64 {
    let sum = xyz[0] + xyz[1] + xyz[2];
    if sum <= 0.0 {
        return 5000.0;
    }
    let (x, y) = (xyz[0] / sum, xyz[1] / sum);
    let n = (x - 0.3320) / (0.1858 - y);
    (449.0 * n.powi(3) + 3525.0 * n.powi(2) + 6823.3 * n + 5520.33).clamp(2000.0, 50_000.0)
}

fn blend(a: Option<Mat3>, b: Option<Mat3>, w: f64) -> Option<Mat3> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let mut m = [[0.0; 3]; 3];
            for i in 0..3 {
                for j in 0..3 {
                    m[i][j] = w * a[i][j] + (1.0 - w) * b[i][j];
                }
            }
            Some(m)
        }
        (Some(m), None) | (None, Some(m)) => Some(m),
        (None, None) => None,
    }
}

/// The weight of the first calibration for a white of temperature `t`: by inverse
/// temperature, as the specification does, and held at the ends.
fn weight(tags: &ColorTags, t: f64) -> f64 {
    match (
        temperature(tags.illuminant[0]),
        temperature(tags.illuminant[1]),
    ) {
        (Some(t1), Some(t2)) if (t1 - t2).abs() > 1.0 => {
            ((1.0 / t - 1.0 / t2) / (1.0 / t1 - 1.0 / t2)).clamp(0.0, 1.0)
        }
        _ => 1.0,
    }
}

/// How the white-balanced camera colours of a shot become linear sRGB.
#[derive(Debug, Clone, Copy)]
pub struct Rendering {
    /// Gains for the camera channels that make the white neutral (green = 1).
    pub gains: [f32; 3],
    /// White-balanced camera RGB to linear sRGB.
    pub to_srgb: Mat3,
}

/// The rendering for these tags, or `None` when they have no colour matrix.
pub fn rendering(tags: &ColorTags) -> Option<Rendering> {
    let neutral = tags.as_shot_neutral.unwrap_or([1.0; 3]);
    let ab = diag(tags.analog_balance);
    // Find the white's temperature: the matrix depends on it, and it on the matrix.
    let mut t = 5000.0;
    let mut xyz_to_camera = IDENTITY;
    for _ in 0..12 {
        let w = weight(tags, t);
        let cm = blend(tags.color_matrix[0], tags.color_matrix[1], w)?;
        let cc =
            blend(tags.camera_calibration[0], tags.camera_calibration[1], w).unwrap_or(IDENTITY);
        xyz_to_camera = mul(&ab, &mul(&cc, &cm));
        let white = apply(&inverse(&xyz_to_camera)?, neutral);
        let next = cct(white);
        if (next - t).abs() < 1.0 {
            t = next;
            break;
        }
        t = next;
    }
    let w = weight(tags, t);
    // Forward matrices map white-balanced camera RGB straight to XYZ D50.
    let camera_to_xyz = if let Some(fm) = blend(tags.forward_matrix[0], tags.forward_matrix[1], w) {
        let cc =
            blend(tags.camera_calibration[0], tags.camera_calibration[1], w).unwrap_or(IDENTITY);
        let reference = apply(&inverse(&mul(&ab, &cc))?, neutral);
        let balance = inverse(&diag(reference))?;
        mul(&fm, &mul(&balance, &inverse(&mul(&ab, &cc))?))
    } else {
        // Otherwise: invert the colour matrix and adapt the shot's white to D50.
        let to_xyz = inverse(&xyz_to_camera)?;
        let white = apply(&to_xyz, neutral);
        let scale = 1.0 / white[1].max(1e-9);
        let adapted = mul(
            &bradford([white[0] * scale, 1.0, white[2] * scale], D50),
            &to_xyz,
        );
        adapted.map(|row| row.map(|v| v * scale))
    };
    // Work in white-balanced camera RGB: gains bring the neutral to (1, 1, 1).
    let g = neutral[1];
    #[allow(clippy::cast_possible_truncation)]
    let gains = [(g / neutral[0]) as f32, 1.0, (g / neutral[2]) as f32];
    let unbalance = diag([neutral[0] / g, 1.0, neutral[2] / g]);
    let exposure = 2f64.powf(tags.baseline_exposure);
    let to_srgb = mul(&XYZ_D50_TO_SRGB, &mul(&camera_to_xyz, &unbalance))
        .map(|row| row.map(|v| v * exposure));
    Some(Rendering { gains, to_srgb })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With the identity as colour matrix (camera RGB = XYZ) and a D50 neutral, white stays
    /// white in sRGB.
    #[test]
    fn a_neutral_white_maps_to_srgb_white() {
        let tags = ColorTags {
            color_matrix: [Some(IDENTITY), None],
            illuminant: [23, 0],
            analog_balance: [1.0; 3],
            as_shot_neutral: Some(D50),
            ..ColorTags::default()
        };
        let Some(r) = rendering(&tags) else {
            panic!("a rendering");
        };
        let white = [1.0, 1.0, 1.0];
        let out = apply(&r.to_srgb, white);
        for v in out {
            assert!((v - 1.0).abs() < 0.01, "{out:?}");
        }
    }

    #[test]
    fn inverse_undoes_the_matrix() {
        let m = [[0.6, -0.27, -0.03], [-0.38, 1.0, 0.45], [-0.01, 0.04, 0.72]];
        let Some(inv) = inverse(&m) else {
            panic!("invertible");
        };
        let id = mul(&m, &inv);
        for (i, row) in id.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                assert!((v - if i == j { 1.0 } else { 0.0 }).abs() < 1e-9);
            }
        }
    }
}
