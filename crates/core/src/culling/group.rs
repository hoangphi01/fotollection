// SPDX-License-Identifier: GPL-3.0-only
//! Groups near-identical shots (bursts, a few frames of the same scene) for best-shot
//! suggestions (FR-041): consecutive in time, same camera and lens, and visually similar
//! (allowing for a zoom between the shots).

use super::similar::{self, Signature};
use crate::library::{PhotoId, PhotoRecord};

/// Most photos in one group; longer runs are split.
pub const MAX_GROUP: usize = 30;
/// A shot joins the previous one when their layouts correlate at least this much and their
/// colours overlap at least this much (`similar`). Tuned on the labelled reference set
/// (bench/bestshot): same-moment pairs mostly score 0.7–1.0 / 0.78+, different scenes with
/// a similar layout fall below the colour bound.
pub const MIN_STRUCTURE: f32 = 0.6;
pub const MIN_COLOR: f32 = 0.72;
/// Looser bound against the group's first shot, so a slow pan never chains on.
pub const MIN_STRUCTURE_FIRST: f32 = 0.4;
/// Gap threshold bounds, in seconds: at least 2 s, at most 30 s.
const MIN_GAP: i64 = 2;
const MAX_GAP: i64 = 30;
/// GPS moves above this many metres start a new group.
const MAX_MOVE_M: f32 = 30.0;

/// What grouping needs to know about one photo.
#[derive(Debug, Clone)]
pub struct Shot {
    pub id: PhotoId,
    /// Capture time in seconds; ties keep file-name order (bursts count up).
    pub time: i64,
    pub name: String,
    /// Make and model.
    pub camera: (Option<String>, Option<String>),
    pub lens: Option<String>,
    pub focal_mm: Option<f32>,
    pub gps: Option<(f32, f32)>,
    /// Visual signature of the photo (see `similar`).
    pub signature: Signature,
}

/// Layout and colour similarity of two photos (see `similar`).
pub fn similarity(a: &super::PhotoAnalysis, b: &super::PhotoAnalysis) -> (f32, f32) {
    (
        similar::structure(&a.signature, &b.signature),
        similar::color(&a.signature, &b.signature),
    )
}

impl Shot {
    /// The grouping facts of a library photo, with its signature from the analysis.
    pub fn from_record(record: &PhotoRecord, signature: Signature) -> Self {
        Self {
            id: record.id,
            time: record.capture.and_utc().timestamp(),
            name: record.file_name.clone(),
            camera: (record.shot.make.clone(), record.shot.model.clone()),
            lens: record.shot.lens.clone(),
            focal_mm: record.shot.focal_mm,
            gps: record.gps,
            signature,
        }
    }
}

fn metres(a: (f32, f32), b: (f32, f32)) -> f32 {
    // Equirectangular approximation: plenty for tens of metres.
    let lat = f32::to_radians(f32::midpoint(a.0, b.0));
    let dx = (b.1 - a.1).to_radians() * lat.cos();
    let dy = (b.0 - a.0).to_radians();
    6_371_000.0 * (dx * dx + dy * dy).sqrt()
}

/// Layout similarity, allowing for a zoom between the shots: re-shooting the same moment
/// closer is common, so a focal-length change alone never splits a group.
fn structure(a: &Shot, b: &Shot) -> f32 {
    let zoom = match (a.focal_mm, b.focal_mm) {
        (Some(fa), Some(fb)) if fa > 0.0 && fb > 0.0 => fb / fa,
        _ => 1.0,
    };
    similar::structure_zoomed(&a.signature, &b.signature, zoom)
}

/// The gap that splits groups: 4 × the median gap between consecutive shots, kept within
/// 2–30 s, so a fast burst and a slow walk both group sensibly.
fn gap_threshold(shots: &[Shot]) -> i64 {
    let mut gaps: Vec<i64> = shots
        .windows(2)
        .map(|w| w[1].time - w[0].time)
        .filter(|g| *g > 0)
        .collect();
    if gaps.is_empty() {
        return MIN_GAP;
    }
    gaps.sort_unstable();
    (4 * gaps[gaps.len() / 2]).clamp(MIN_GAP, MAX_GAP)
}

/// Groups of at least two similar shots, each in capture order. Photos that match nothing
/// are left out: there is nothing to choose between.
pub fn group(mut shots: Vec<Shot>) -> Vec<Vec<PhotoId>> {
    shots.sort_by(|a, b| (a.time, &a.name).cmp(&(b.time, &b.name)));
    let threshold = gap_threshold(&shots);
    let mut groups: Vec<Vec<PhotoId>> = Vec::new();
    let mut current: Vec<&Shot> = Vec::new();
    for shot in &shots {
        let joins = current.last().is_some_and(|prev| {
            let first = current[0];
            let moved = match (prev.gps, shot.gps) {
                (Some(a), Some(b)) => metres(a, b) > MAX_MOVE_M,
                _ => false,
            };
            shot.time - prev.time <= threshold
                && shot.camera == prev.camera
                && shot.lens == prev.lens
                && !moved
                && structure(prev, shot) >= MIN_STRUCTURE
                && similar::color(&prev.signature, &shot.signature) >= MIN_COLOR
                && structure(first, shot) >= MIN_STRUCTURE_FIRST
                && current.len() < MAX_GROUP
        });
        if !joins {
            if current.len() >= 2 {
                groups.push(current.iter().map(|s| s.id).collect());
            }
            current.clear();
        }
        current.push(shot);
    }
    if current.len() >= 2 {
        groups.push(current.iter().map(|s| s.id).collect());
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::culling::metrics::tests::{blur, scene};
    use crate::image::RgbImage;

    /// A synthetic photo's signature: the test scene, optionally blurred (another frame of a
    /// burst), mirrored (another view) and tinted.
    fn signature_of(blur_radius: usize, mirrored: bool, tint: [f32; 3]) -> Signature {
        let mut g = scene(320, 240, 0);
        if blur_radius > 0 {
            g = blur(&g, blur_radius, true, true);
        }
        if mirrored {
            g.data
                .as_chunks_mut::<320>()
                .0
                .iter_mut()
                .for_each(|row| row.reverse());
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let data = g
            .data
            .iter()
            .flat_map(|v| tint.map(|t| (v * t).clamp(0.0, 255.0) as u8))
            .collect();
        similar::signature(
            &RgbImage::from_raw(320, 240, data).unwrap_or_else(|| RgbImage::new(1, 1)),
        )
    }

    const WARM: [f32; 3] = [1.0, 0.75, 0.5];
    const COOL: [f32; 3] = [0.5, 0.75, 1.0];

    fn shot(id: u32, time: i64, signature: &Signature) -> Shot {
        Shot {
            id: PhotoId(id),
            time,
            name: format!("IMG_{id:04}.JPG"),
            camera: (Some("Panasonic".into()), Some("DC-GX9".into())),
            lens: None,
            focal_mm: Some(20.0),
            gps: None,
            signature: signature.clone(),
        }
    }

    #[test]
    fn fr_041_groups_bursts_and_splits_on_time_scene_and_setup() {
        let a = signature_of(0, false, WARM);
        let a_soft = signature_of(1, false, WARM);
        let a_softer = signature_of(2, false, WARM);
        let other_view = signature_of(0, true, WARM);
        let mut shots = vec![
            shot(1, 100, &a),
            shot(2, 100, &a_soft),   // same second, almost the same image
            shot(3, 101, &a_softer), // a burst of three
            shot(4, 160, &a),        // a minute later: new moment
            shot(5, 161, &a_soft),
            shot(6, 162, &other_view), // same moment, different view
            shot(7, 300, &a),
        ];
        // Another camera breaks the group too.
        shots.push(Shot {
            camera: (Some("Canon".into()), None),
            ..shot(8, 301, &a)
        });
        let groups = group(shots);
        let ids: Vec<Vec<u32>> = groups
            .iter()
            .map(|g| g.iter().map(|p| p.0).collect())
            .collect();
        assert_eq!(ids, [vec![1, 2, 3], vec![4, 5]]);
    }

    #[test]
    fn fr_041_same_layout_in_other_colours_is_another_scene() {
        let warm = signature_of(0, false, WARM);
        let cool = signature_of(0, false, COOL);
        assert!(similar::structure(&warm, &cool) > 0.95);
        assert!(similar::color(&warm, &cool) < MIN_COLOR);
        assert_eq!(group(vec![shot(1, 10, &warm), shot(2, 11, &cool)]).len(), 0);
    }

    #[test]
    fn fr_041_long_runs_are_capped() {
        let a = signature_of(0, false, WARM);
        let shots: Vec<_> = (0..45).map(|i| shot(i, 1000 + i64::from(i), &a)).collect();
        let sizes: Vec<usize> = group(shots).iter().map(Vec::len).collect();
        assert_eq!(sizes, [MAX_GROUP, 15]);
    }

    #[test]
    fn fr_041_gps_moves_split_groups() {
        let s = signature_of(0, false, WARM);
        let mut a = shot(1, 10, &s);
        a.gps = Some((38.7, -9.14));
        let mut b = shot(2, 11, &s);
        b.gps = Some((38.7005, -9.14)); // ~55 m north
        assert_eq!(group(vec![a, b]).len(), 0);
    }
}
