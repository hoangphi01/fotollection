// SPDX-License-Identifier: GPL-3.0-only
//! Photos by where they were taken (FR-141 S4; owner, 2026-11-17: "as the iPhone's Places
//! album"), apart from time: by named place (GeoNames, offline, FR-005), and as clusters on a
//! map at a zoom level, each with its centre, count and cover, for the view's bubbles (S7).
//! A photo with no location takes its event's place when the event holds one place only,
//! marked as borrowed.

use std::collections::HashMap;

/// A place and its photos, most photos first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceGroup {
    pub place_id: u32,
    pub members: Vec<usize>,
}

/// A bubble on the map at one zoom level.
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    pub lat: f32,
    pub lon: f32,
    pub count: usize,
    /// The first photo of the cluster, in the order given.
    pub cover: usize,
}

/// Where each photo is, and whether that place was borrowed from its event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Located {
    pub place_id: u32,
    pub borrowed: bool,
}

/// The photos of each place, most photos first.
pub fn by_place(places: &[Option<Located>]) -> Vec<PlaceGroup> {
    let mut groups: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, p) in places.iter().enumerate() {
        if let Some(p) = p {
            groups.entry(p.place_id).or_default().push(i);
        }
    }
    let mut out: Vec<PlaceGroup> = groups
        .into_iter()
        .map(|(place_id, members)| PlaceGroup { place_id, members })
        .collect();
    out.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then(a.place_id.cmp(&b.place_id))
    });
    out
}

/// A photo without a place takes its event's, when every located photo of the event agrees.
pub fn borrow_from_events(places: &mut [Option<Located>], events: &[Vec<usize>]) {
    for event in events {
        let mut ids = event
            .iter()
            .filter_map(|&i| places.get(i).copied().flatten());
        let Some(first) = ids.next() else {
            continue;
        };
        if ids.any(|p| p.place_id != first.place_id) {
            continue;
        }
        for &i in event {
            if let Some(slot @ None) = places.get_mut(i) {
                *slot = Some(Located {
                    place_id: first.place_id,
                    borrowed: true,
                });
            }
        }
    }
}

/// Clusters of located photos on a Web Mercator grid of `2^zoom` cells a side.
pub fn clusters(points: &[(usize, (f32, f32))], zoom: u8) -> Vec<Cluster> {
    let n = f64::from(1u32 << zoom.min(20));
    let tile = |(lat, lon): (f32, f32)| {
        let lat = f64::from(lat).clamp(-85.0, 85.0).to_radians();
        let x = (f64::from(lon) + 180.0) / 360.0 * n;
        let y = (1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
        #[allow(clippy::cast_possible_truncation)]
        (x.floor() as i64, y.floor() as i64)
    };
    let mut cells: HashMap<(i64, i64), (f64, f64, usize, usize)> = HashMap::new();
    let mut order = Vec::new();
    for &(i, p) in points {
        let key = tile(p);
        let c = cells.entry(key).or_insert_with(|| {
            order.push(key);
            (0.0, 0.0, 0, i)
        });
        c.0 += f64::from(p.0);
        c.1 += f64::from(p.1);
        c.2 += 1;
    }
    order
        .into_iter()
        .filter_map(|k| cells.get(&k))
        .map(|&(lat, lon, count, cover)| {
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let mean = |s: f64| (s / count as f64) as f32;
            Cluster {
                lat: mean(lat),
                lon: mean(lon),
                count,
                cover,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn at(id: u32) -> Located {
        Located {
            place_id: id,
            borrowed: false,
        }
    }

    #[test]
    fn fr_141_photos_gather_by_place_and_borrow_their_events() {
        let mut places = vec![
            Some(at(1)),
            Some(at(1)),
            None,
            Some(at(2)),
            None,
            Some(at(2)),
            Some(at(3)),
        ];
        // Event 0 is all place 1; event 1 mixes places 2 and 3, so lends nothing.
        borrow_from_events(&mut places, &[vec![0, 1, 2], vec![3, 4, 5, 6]]);
        assert_eq!(
            places[2],
            Some(Located {
                place_id: 1,
                borrowed: true
            })
        );
        assert_eq!(places[4], None);
        let g = by_place(&places);
        assert_eq!(
            g[0],
            PlaceGroup {
                place_id: 1,
                members: vec![0, 1, 2]
            }
        );
        assert_eq!(g.len(), 3);
    }

    #[test]
    fn fr_141_map_clusters_merge_when_zoomed_out() {
        // Budapest and Vienna, 214 km apart, and a photo in Hanoi.
        let pts = [
            (0, (47.49, 19.04)),
            (1, (47.50, 19.05)),
            (2, (48.21, 16.37)),
            (3, (21.03, 105.85)),
        ];
        let far = clusters(&pts, 3);
        assert_eq!(far.iter().map(|c| c.count).collect::<Vec<_>>(), vec![3, 1]);
        let near = clusters(&pts, 8);
        assert_eq!(
            near.iter().map(|c| c.count).collect::<Vec<_>>(),
            vec![2, 1, 1]
        );
        assert_eq!(near[0].cover, 0);
    }
}
