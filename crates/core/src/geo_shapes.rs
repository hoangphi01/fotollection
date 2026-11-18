// SPDX-License-Identifier: GPL-3.0-only
//! Country shapes for RollSense's map (FR-141; owner 2026-11-18: "a filter app, not Google
//! Maps"): one country's outline and its photos' places as pins, in a 275 px wide frame.
//! The outlines are Natural Earth's 1:50m countries (public domain), simplified and packed by
//! `crates/geodata` into `assets/geo/countries.bin`, compiled in and checked against a pinned
//! BLAKE3 sum; nothing is ever downloaded.

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::OnceLock;

use serde::Serialize;
use specta::Type;

const SHAPES: &[u8] = include_bytes!("../assets/geo/countries.bin");
const SHAPES_BLAKE3: &str = "f7fad8441a75a4241101c218aca64fe1f2fd8dbcb48e9046bb4ee43d2f9f170b";

/// The map's width in the panel, in px; its height follows the country's shape.
pub const MAP_WIDTH: f32 = 275.0;
/// Pins nearer than this, in px, show as one (two towns 20 km apart in a 500 km country).
const PIN_MERGE_PX: f32 = 16.0;
/// Most points one ring may hold, most rings one country: the reader's bounds.
const MAX_POINTS: usize = 20_000;
const MAX_RINGS: usize = 1_000;

/// One country's outer rings, in hundredths of a degree (longitude, latitude).
type Rings = Vec<Vec<(i16, i16)>>;

/// One country drawn for the panel: its outline as an SVG path (only `M`, `L` and `Z` with
/// numbers) and its pins, all in a frame `width` × `height` px.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CountryMap {
    pub width: f32,
    pub height: f32,
    pub path: String,
    pub pins: Vec<MapPin>,
}

/// A pin: the places it stands for (one, or several close together), their photos, and where.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MapPin {
    pub ids: Vec<u32>,
    pub x: f32,
    pub y: f32,
    pub count: u32,
}

/// Reads the packed shapes; `None` for anything malformed (bounded, never panics).
fn parse(bytes: &[u8]) -> Option<HashMap<[u8; 2], Rings>> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = bytes.get(at..at.checked_add(n)?)?;
        at += n;
        Some(s)
    };
    let u16_at = |s: &[u8]| u16::from_le_bytes([s[0], s[1]]);
    if take(4)? != b"FCTY" || u16_at(take(2)?) != 1 {
        return None;
    }
    let countries = usize::from(u16_at(take(2)?));
    let mut out = HashMap::with_capacity(countries);
    for _ in 0..countries {
        let code = take(2)?;
        let code = [code[0], code[1]];
        let rings = usize::from(u16_at(take(2)?));
        if rings > MAX_RINGS {
            return None;
        }
        let mut list = Vec::with_capacity(rings);
        for _ in 0..rings {
            let points = usize::from(u16_at(take(2)?));
            if points > MAX_POINTS {
                return None;
            }
            let raw = take(points.checked_mul(4)?)?;
            list.push(
                raw.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| {
                        (
                            i16::from_le_bytes([c[0], c[1]]),
                            i16::from_le_bytes([c[2], c[3]]),
                        )
                    })
                    .collect(),
            );
        }
        out.insert(code, list);
    }
    Some(out)
}

fn shapes() -> &'static HashMap<[u8; 2], Rings> {
    static SHAPES_MAP: OnceLock<HashMap<[u8; 2], Rings>> = OnceLock::new();
    SHAPES_MAP.get_or_init(|| {
        if blake3::hash(SHAPES).to_hex().as_str() != SHAPES_BLAKE3 {
            return HashMap::new();
        }
        parse(SHAPES).unwrap_or_default()
    })
}

/// A longitude moved by 360° when the country crosses the 180th meridian (Russia, Fiji), so
/// its east and west stay side by side.
fn wrap(lon: f32, crosses: bool) -> f32 {
    if crosses && lon < 0.0 {
        lon + 360.0
    } else {
        lon
    }
}

/// The country `code` (ISO 3166-1 alpha-2) drawn with pins for `places` (id, latitude,
/// longitude, photos); `None` when the shapes hold no such country.
pub fn country_map(code: &str, places: &[(u32, f64, f64, u32)]) -> Option<CountryMap> {
    let key: [u8; 2] = code.as_bytes().try_into().ok()?;
    let rings = shapes().get(&key)?;
    let points = || {
        rings
            .iter()
            .flatten()
            .map(|&(x, y)| (f32::from(x) / 100.0, f32::from(y) / 100.0))
    };
    let (lo, hi) = points().fold((f32::MAX, f32::MIN), |(a, b), (x, _)| (a.min(x), b.max(x)));
    let crosses = hi - lo > 180.0;
    let (x0, y0, x1, y1) = points().fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |b, (x, y)| {
        let x = wrap(x, crosses);
        (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y))
    });
    // Equirectangular, longitude shrunk by the cosine of the middle latitude: true enough
    // for one country.
    let k = y0.midpoint(y1).to_radians().cos().max(0.2);
    let scale = MAP_WIDTH / ((x1 - x0) * k).max(0.01);
    let at = |lon: f32, lat: f32| ((wrap(lon, crosses) - x0) * k * scale, (y1 - lat) * scale);
    let mut path = String::new();
    for ring in rings {
        for (i, &(x, y)) in ring.iter().enumerate() {
            let (px, py) = at(f32::from(x) / 100.0, f32::from(y) / 100.0);
            let _ = write!(path, "{}{px:.1},{py:.1}", if i == 0 { "M" } else { "L" });
        }
        path.push('Z');
    }
    let mut pins: Vec<MapPin> = Vec::new();
    let mut sorted: Vec<&(u32, f64, f64, u32)> = places.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.3));
    for &&(id, lat, lon, count) in &sorted {
        #[allow(clippy::cast_possible_truncation)]
        let (x, y) = at(lon as f32, lat as f32);
        match pins
            .iter_mut()
            .find(|p| (p.x - x).hypot(p.y - y) < PIN_MERGE_PX)
        {
            Some(p) => {
                p.ids.push(id);
                p.count += count;
            }
            None => pins.push(MapPin {
                ids: vec![id],
                x,
                y,
                count,
            }),
        }
    }
    Some(CountryMap {
        width: MAP_WIDTH,
        height: (y1 - y0) * scale,
        path,
        pins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_141_the_bundled_shapes_load_and_match_their_checksum() {
        assert_eq!(blake3::hash(SHAPES).to_hex().as_str(), SHAPES_BLAKE3);
        assert!(shapes().len() > 200);
    }

    #[test]
    fn fr_141_a_country_is_drawn_with_its_pins_and_close_towns_share_one() {
        // Praha, and Most and Litvínov 10 km apart.
        let places = [
            (1, 50.088, 14.421, 300),
            (2, 50.503, 13.636, 58),
            (3, 50.601, 13.611, 5),
        ];
        let map = country_map("CZ", &places).unwrap_or_else(|| panic!("CZ is bundled"));
        assert!((map.width - MAP_WIDTH).abs() < f32::EPSILON);
        assert!(
            map.height > 100.0 && map.height < 200.0,
            "Czechia is wider than tall"
        );
        assert!(map.path.starts_with('M') && map.path.ends_with('Z'));
        assert!(map.path.chars().all(|c| "MLZ.,-0123456789".contains(c)));
        assert_eq!(map.pins.len(), 2);
        assert_eq!(map.pins[1].ids, vec![2, 3]);
        assert_eq!(map.pins[1].count, 63);
        let praha = &map.pins[0];
        assert!(praha.x > 0.0 && praha.x < map.width && praha.y > 0.0 && praha.y < map.height);
        assert_eq!(country_map("XX", &places), None);
        assert_eq!(country_map("CZE", &places), None);
    }

    #[test]
    fn fr_141_vietnams_map_holds_hoang_sa_and_truong_sa() {
        // Trường Sa Lớn and Phú Lâm (Hoàng Sa) fall inside Vietnam's frame (owner, 2026-11-19).
        let places = [(1, 8.644, 111.920, 1), (2, 16.834, 112.338, 1)];
        let map = country_map("VN", &places).unwrap_or_else(|| panic!("VN is bundled"));
        for pin in &map.pins {
            assert!(pin.x > 0.0 && pin.x <= map.width && pin.y > 0.0 && pin.y <= map.height);
        }
        assert!(map.path.matches('M').count() > 13, "the islands are drawn");
    }

    #[test]
    fn fr_141_a_country_across_the_180th_meridian_stays_whole() {
        let map = country_map("RU", &[]).unwrap_or_else(|| panic!("RU is bundled"));
        assert!(
            map.height < map.width,
            "Russia is far wider than tall once kept whole"
        );
    }

    #[test]
    fn fr_141_malformed_shapes_are_refused_without_a_panic() {
        assert!(parse(b"").is_none());
        assert!(parse(b"FCTY\x01\x00\x01\x00CZ\x01\x00\xff\xff").is_none());
        let mut cut = SHAPES.to_vec();
        cut.truncate(SHAPES.len() / 2);
        assert!(parse(&cut).is_none());
    }
}
