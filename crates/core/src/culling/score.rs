// SPDX-License-Identifier: GPL-3.0-only
//! Ranks the photos of one group and explains the pick (FR-041). Every measure is turned into
//! a robust z-score **within the group** (median and MAD), so only differences between the
//! shots matter, not the scene itself.

use serde::Serialize;
use specta::Type;

use super::PhotoAnalysis;
use crate::library::PhotoId;

/// Weights of the measures (PRD FR-041).
const W_SHARP: f32 = 0.35;
const W_EYES: f32 = 0.20;
const W_SHAKE: f32 = 0.15;
const W_EXPOSURE: f32 = 0.15;
const W_HIGHLIGHTS: f32 = 0.10;
const W_NOISE: f32 = 0.05;
/// Two scores closer than half a sigma of subject sharpness are a toss-up.
const TOO_CLOSE: f32 = 0.5 * W_SHARP;
/// An eye counts as closed below this eye aspect ratio, or below 75 % of the same
/// person's widest-open eye in the group.
const EAR_CLOSED: f32 = 0.2;
const EAR_RELATIVE: f32 = 0.75;

/// Why the best photo won, compared with the runner-up. The UI words these (FR-020).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum Reason {
    /// Sharper: edges this many percent stronger than the runner-up's (gradient magnitude,
    /// the square root of the Tenengrad energy). A percentage reads better than a sigma, and
    /// small groups have no meaningful sigma.
    Sharper {
        percent: f32,
    },
    /// Less camera shake.
    SteadierShot,
    /// Everyone's eyes are open (and not in the runner-up).
    EyesOpen,
    /// Closer to a good exposure.
    BetterExposure,
    /// Fewer blown highlights, as shares of the image.
    FewerHighlights {
        best: f32,
        runner_up: f32,
    },
    LessNoise,
}

/// One photo's result inside its group.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoScore {
    pub id: PhotoId,
    pub score: f32,
    /// The weighted parts of the score, for the compare view's bars.
    pub sharpness: f32,
    pub steadiness: f32,
    pub exposure: f32,
    pub highlights: f32,
    pub noise: f32,
    pub eyes: f32,
    /// Faces whose eyes were found closed.
    pub closed_eyes: u32,
    pub faces: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupResult {
    /// Best first.
    pub photos: Vec<PhotoScore>,
    pub reasons: Vec<Reason>,
    /// The top two are within half a sigma of sharpness: the user should look.
    pub too_close: bool,
}

/// Smallest spread each measure is scaled by, in its own units: differences below these are
/// not visible in a print, so near-identical values must not look like many sigmas apart.
const MIN_SPREAD_SHARP: f32 = 0.1; // ln of Tenengrad energy, about 10 %
const MIN_SPREAD_SHAKE: f32 = 0.08; // coherence
const MIN_SPREAD_EV: f32 = 0.15; // stops
const MIN_SPREAD_HIGHLIGHTS: f32 = 0.02; // share of the image
const MIN_SPREAD_NOISE: f32 = 1.0; // luma sigma

/// Robust z-scores (median / MAD, at least `min_spread`); fewer than 4 values fall back to
/// ranks in −1…1, still ignoring differences below `min_spread`.
fn standardize(values: &[f32], min_spread: f32) -> Vec<f32> {
    let n = values.len();
    if n < 2 {
        return vec![0.0; n];
    }
    if n < 4 {
        #[allow(clippy::cast_precision_loss)]
        return values
            .iter()
            .map(|v| {
                let below = values.iter().filter(|o| **o < v - min_spread).count() as f32;
                let equal = values
                    .iter()
                    .filter(|o| (**o - v).abs() <= min_spread)
                    .count() as f32
                    - 1.0;
                2.0 * (below + equal / 2.0) / (n - 1) as f32 - 1.0
            })
            .collect();
    }
    let median = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    let m = median(&mut values.to_vec());
    let mad = median(&mut values.iter().map(|v| (v - m).abs()).collect());
    let scale = (1.4826 * mad).max(min_spread);
    values
        .iter()
        .map(|v| ((v - m) / scale).clamp(-4.0, 4.0))
        .collect()
}

/// Closed eyes per photo, using each face's own widest-open eye in the group as reference.
fn closed_eyes(photos: &[(PhotoId, &PhotoAnalysis)]) -> Vec<(u32, u32)> {
    photos
        .iter()
        .map(|(_, a)| {
            let mut closed = 0;
            let mut seen = 0;
            for face in &a.faces {
                let Some(ear) = face.ear else { continue };
                seen += 1;
                // The same person in the other shots: overlapping face boxes.
                let reference = photos
                    .iter()
                    .flat_map(|(_, o)| &o.faces)
                    .filter(|o| o.overlaps(face))
                    .filter_map(|o| o.ear)
                    .fold(ear, f32::max);
                if ear < EAR_CLOSED || ear < EAR_RELATIVE * reference {
                    closed += 1;
                }
            }
            (closed, seen)
        })
        .collect()
}

/// Scores one group and explains the winner.
pub fn rank(photos: &[(PhotoId, &PhotoAnalysis)]) -> GroupResult {
    // Faces are the subject when every shot has one; otherwise all shots are compared on the
    // same whole-image measure (eye-area and whole-image sharpness are not comparable).
    let face_sharpness =
        |a: &PhotoAnalysis| a.faces.iter().map(|f| f.sharpness).fold(0.0, f32::max);
    let all_faces = photos.iter().all(|(_, a)| face_sharpness(a) > 0.0);
    let sharp: Vec<f32> = photos
        .iter()
        .map(|(_, a)| {
            if all_faces {
                face_sharpness(a).ln_1p()
            } else {
                a.metrics.subject_sharpness.ln_1p()
            }
        })
        .collect();
    let steady: Vec<f32> = photos.iter().map(|(_, a)| -a.metrics.coherence).collect();
    let exposure: Vec<f32> = photos
        .iter()
        .map(|(_, a)| -a.metrics.exposure_ev.abs())
        .collect();
    let highlights: Vec<f32> = photos.iter().map(|(_, a)| -a.metrics.highlights).collect();
    let noise: Vec<f32> = photos.iter().map(|(_, a)| -a.metrics.noise).collect();
    let eyes = closed_eyes(photos);
    // Most faces whose eyes could be checked in any one shot: a shot where nobody faces the
    // camera scores 0, below one where everyone's eyes are open.
    let most_faces = eyes.iter().map(|(_, seen)| *seen).max().unwrap_or(0);

    let (zs, zt, ze, zh, zn) = (
        standardize(&sharp, MIN_SPREAD_SHARP),
        standardize(&steady, MIN_SPREAD_SHAKE),
        standardize(&exposure, MIN_SPREAD_EV),
        standardize(&highlights, MIN_SPREAD_HIGHLIGHTS),
        standardize(&noise, MIN_SPREAD_NOISE),
    );
    let mut scored: Vec<PhotoScore> = photos
        .iter()
        .enumerate()
        .map(|(i, (id, _))| {
            let (closed, faces) = eyes[i];
            // Each open pair of eyes adds 1, each closed pair costs 2, and each face turned
            // away or hidden (fewer than the most in any shot) costs 0.5, relative to the most
            // faces in the group: closed eyes rarely win, and neither does a lost face.
            #[allow(clippy::cast_precision_loss)]
            let eyes_part = if most_faces == 0 {
                0.0
            } else {
                let missing = most_faces - faces;
                (faces as f32 - 3.0 * closed as f32 - 0.5 * missing as f32) / most_faces as f32
            };
            let parts = [
                W_SHARP * zs[i],
                W_SHAKE * zt[i],
                W_EXPOSURE * ze[i],
                W_HIGHLIGHTS * zh[i],
                W_NOISE * zn[i],
                W_EYES * eyes_part,
            ];
            PhotoScore {
                id: *id,
                score: parts.iter().sum(),
                sharpness: parts[0],
                steadiness: parts[1],
                exposure: parts[2],
                highlights: parts[3],
                noise: parts[4],
                eyes: parts[5],
                closed_eyes: closed,
                faces,
            }
        })
        .collect();
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));

    let mut reasons = Vec::new();
    let mut too_close = false;
    if let [best, runner_up, ..] = scored.as_slice() {
        too_close = best.score - runner_up.score < TOO_CLOSE;
        let index = |id: PhotoId| photos.iter().position(|(p, _)| *p == id).unwrap_or(0);
        let (b, r) = (index(best.id), index(runner_up.id));
        let mut candidates: Vec<(f32, Reason)> = vec![
            (
                best.sharpness - runner_up.sharpness,
                Reason::Sharper {
                    percent: 100.0 * (((sharp[b] - sharp[r]) / 2.0).exp() - 1.0),
                },
            ),
            (best.steadiness - runner_up.steadiness, Reason::SteadierShot),
            (best.exposure - runner_up.exposure, Reason::BetterExposure),
            (
                best.highlights - runner_up.highlights,
                Reason::FewerHighlights {
                    best: photos[b].1.metrics.highlights,
                    runner_up: photos[r].1.metrics.highlights,
                },
            ),
            (best.noise - runner_up.noise, Reason::LessNoise),
        ];
        if best.closed_eyes == 0 && runner_up.closed_eyes > 0 {
            candidates.push((best.eyes - runner_up.eyes, Reason::EyesOpen));
        }
        candidates.retain(|(gain, _)| *gain > 0.05);
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        reasons = candidates.into_iter().take(3).map(|(_, r)| r).collect();
    }
    GroupResult {
        photos: scored,
        reasons,
        too_close,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::culling::metrics::measure;
    use crate::culling::metrics::tests::{blur, scene};
    use crate::culling::{ANALYSIS_VERSION, FaceSummary};

    fn analysis_of(g: &crate::culling::metrics::Gray) -> PhotoAnalysis {
        PhotoAnalysis {
            version: ANALYSIS_VERSION,
            metrics: measure(g),
            signature: crate::culling::similar::Signature::default(),
            faces: Vec::new(),
        }
    }

    #[test]
    fn fr_041_sharp_photo_wins_and_says_why() {
        let sharp = analysis_of(&scene(320, 240, 0));
        let soft = analysis_of(&blur(&scene(320, 240, 0), 2, true, true));
        let softer = analysis_of(&blur(&scene(320, 240, 0), 4, true, true));
        let shaken = analysis_of(&blur(&scene(320, 240, 0), 5, false, true));
        let group = [
            (PhotoId(1), &soft),
            (PhotoId(2), &sharp),
            (PhotoId(3), &softer),
            (PhotoId(4), &shaken),
        ];
        let result = rank(&group);
        assert_eq!(result.photos[0].id, PhotoId(2), "{result:?}");
        assert!(
            matches!(result.reasons.first(), Some(Reason::Sharper { .. })),
            "{result:?}"
        );
        assert!(!result.too_close, "{result:?}");
    }

    #[test]
    fn fr_041_closed_eyes_lose_against_open_ones() {
        let base = analysis_of(&scene(320, 240, 0));
        let face = |ear: f32| FaceSummary {
            x: 0.4,
            y: 0.3,
            w: 0.2,
            h: 0.25,
            ear: Some(ear),
            sharpness: 900.0,
        };
        let open = PhotoAnalysis {
            faces: vec![face(0.31)],
            ..base.clone()
        };
        let blink = PhotoAnalysis {
            faces: vec![face(0.12)],
            ..base.clone()
        };
        let half = PhotoAnalysis {
            faces: vec![face(0.21)], // above 0.2 but well below this person's 0.31
            ..base
        };
        let result = rank(&[
            (PhotoId(1), &blink),
            (PhotoId(2), &half),
            (PhotoId(3), &open),
        ]);
        assert_eq!(result.photos[0].id, PhotoId(3));
        assert!(result.reasons.contains(&Reason::EyesOpen), "{result:?}");
        let closed: Vec<u32> = result.photos.iter().map(|p| p.closed_eyes).collect();
        assert_eq!(closed, [0, 1, 1]);
    }

    #[test]
    fn fr_041_identical_shots_are_too_close_to_call() {
        let a = analysis_of(&scene(200, 150, 0));
        let result = rank(&[(PhotoId(1), &a), (PhotoId(2), &a.clone())]);
        assert!(result.too_close);
    }

    #[test]
    fn robust_z_and_rank_fallback() {
        assert_eq!(standardize(&[1.0, 2.0, 3.0], 0.1), [-1.0, 0.0, 1.0]);
        assert_eq!(standardize(&[1.0, 1.05, 3.0], 0.1), [-0.5, -0.5, 1.0]);
        let z = standardize(&[10.0, 11.0, 12.0, 13.0, 100.0], 0.1);
        assert!(
            z[4] >= 3.9,
            "an outlier is clamped, not allowed to dominate: {z:?}"
        );
        let z = standardize(&[0.0, 0.0, 0.0, 0.008], 0.02);
        assert!(
            z[3] < 1.0,
            "a tiny difference stays small even when the MAD is 0: {z:?}"
        );
    }
}
