// SPDX-License-Identifier: GPL-3.0-only
//! Groups near-identical shots (bursts, a few frames of the same scene) for best-shot
//! suggestions (FR-041): consecutive in time, same camera setup, and visually similar.

use crate::library::{PhotoId, PhotoRecord};

/// Most photos in one group; longer runs are split.
pub const MAX_GROUP: usize = 30;
/// Largest Hamming distance between dHashes still counted as the same scene. Hand-held
/// re-framings of one moment measured 11–16 on the reference set, different scenes 18+.
pub const MAX_HASH_DISTANCE: u32 = 16;
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
    /// dHash of the photo (see `metrics::dhash`).
    pub hash: u64,
}

impl Shot {
    /// The grouping facts of a library photo, with its dHash from the analysis.
    pub fn from_record(record: &PhotoRecord, hash: u64) -> Self {
        Self {
            id: record.id,
            time: record.capture.and_utc().timestamp(),
            name: record.file_name.clone(),
            camera: (record.shot.make.clone(), record.shot.model.clone()),
            lens: record.shot.lens.clone(),
            focal_mm: record.shot.focal_mm,
            gps: record.gps,
            hash,
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
            let focal_ok = match (prev.focal_mm, shot.focal_mm) {
                (Some(a), Some(b)) if a > 0.0 => (b - a).abs() / a <= 0.2,
                _ => true,
            };
            let moved = match (prev.gps, shot.gps) {
                (Some(a), Some(b)) => metres(a, b) > MAX_MOVE_M,
                _ => false,
            };
            shot.time - prev.time <= threshold
                && shot.camera == prev.camera
                && shot.lens == prev.lens
                && focal_ok
                && !moved
                // Compared with the group's first photo, so a slow pan never chains on.
                && (shot.hash ^ first.hash).count_ones() <= MAX_HASH_DISTANCE
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

    fn shot(id: u32, time: i64, hash: u64) -> Shot {
        Shot {
            id: PhotoId(id),
            time,
            name: format!("IMG_{id:04}.JPG"),
            camera: (Some("Panasonic".into()), Some("DC-GX9".into())),
            lens: None,
            focal_mm: Some(20.0),
            gps: None,
            hash,
        }
    }

    const A: u64 = 0xF0F0_F0F0_0F0F_0F0F;
    const B: u64 = !A; // a completely different scene

    #[test]
    fn fr_041_groups_bursts_and_splits_on_time_scene_and_setup() {
        let mut shots = vec![
            shot(1, 100, A),
            shot(2, 100, A ^ 0b11),  // same second, almost the same image
            shot(3, 101, A ^ 0b111), // a burst of three
            shot(4, 160, A),         // a minute later: new moment
            shot(5, 161, A ^ 1),
            shot(6, 162, B), // same moment, different scene
            shot(7, 300, A),
        ];
        // A zoom change breaks the group too.
        shots.push(Shot {
            focal_mm: Some(45.0),
            ..shot(8, 301, A)
        });
        let groups = group(shots);
        let ids: Vec<Vec<u32>> = groups
            .iter()
            .map(|g| g.iter().map(|p| p.0).collect())
            .collect();
        assert_eq!(ids, [vec![1, 2, 3], vec![4, 5]]);
    }

    #[test]
    fn fr_041_long_runs_are_capped() {
        let shots: Vec<_> = (0..45).map(|i| shot(i, 1000 + i64::from(i), A)).collect();
        let sizes: Vec<usize> = group(shots).iter().map(Vec::len).collect();
        assert_eq!(sizes, [MAX_GROUP, 15]);
    }

    #[test]
    fn fr_041_gps_moves_split_groups() {
        let mut a = shot(1, 10, A);
        a.gps = Some((38.7, -9.14));
        let mut b = shot(2, 11, A);
        b.gps = Some((38.7005, -9.14)); // ~55 m north
        assert_eq!(group(vec![a, b]).len(), 0);
    }
}
