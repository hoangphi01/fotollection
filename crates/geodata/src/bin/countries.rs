// SPDX-License-Identifier: GPL-3.0-only
//! Packs Natural Earth's 1:50m admin-0 countries (public domain) into `countries.bin`, the
//! offline country shapes of RollSense's map (FR-141; owner 2026-11-18: "a filter app, not
//! Google Maps"). Each country's outer rings, simplified by Douglas–Peucker to a tolerance
//! that follows the country's size, so a small one keeps its shape; islands too small to see
//! at 275 px are dropped. Read by `crates/core/src/geo_shapes.rs`, described in
//! `crates/core/assets/geo/GEODATA.md`.
//!
//! `countries <ne_50m_admin_0_countries.geojson> <out.bin>`

use std::io::Write;
use std::process::ExitCode;

use serde_json::Value;

/// The map's width in the panel, in px: what "too small to see" is measured against.
const VIEW_PX: f64 = 275.0;

/// Islands a country's map always shows, though too small for 1:50m (owner, 2026-11-19:
/// Vietnam's map has the Hoàng Sa and Trường Sa archipelagos): (code, latitude, longitude)
/// of each main island, drawn as a small mark that no size filter drops.
const ISLANDS: &[([u8; 2], f64, f64)] = &[
    // Hoàng Sa: Phú Lâm, Hoàng Sa, Linh Côn, Tri Tôn.
    (*b"VN", 16.834, 112.338),
    (*b"VN", 16.533, 111.608),
    (*b"VN", 16.669, 112.725),
    (*b"VN", 15.783, 111.200),
    // Trường Sa: Trường Sa Lớn, Ba Bình, Thị Tứ, Song Tử Tây, Nam Yết, Sinh Tồn, Sơn Ca,
    // Loại Ta, An Bang.
    (*b"VN", 8.644, 111.920),
    (*b"VN", 10.377, 114.365),
    (*b"VN", 11.054, 114.284),
    (*b"VN", 11.430, 114.330),
    (*b"VN", 10.181, 114.361),
    (*b"VN", 9.882, 114.329),
    (*b"VN", 10.375, 114.479),
    (*b"VN", 10.670, 114.420),
    (*b"VN", 7.893, 112.917),
];
/// An island's mark: a diamond this many degrees from its centre to each point (about 3 px).
const ISLAND_HALF: f64 = 0.08;

type Ring = Vec<(f64, f64)>;
/// A country's code and its rings in hundredths of a degree, as packed.
type Packed = ([u8; 2], Vec<Vec<(i16, i16)>>);

fn fail(message: &str) -> ExitCode {
    eprintln!("countries: {message}");
    ExitCode::FAILURE
}

/// A point's distance from the segment `a`–`b`, in degrees.
fn off(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.hypot(dy);
    if len == 0.0 {
        return (p.0 - a.0).hypot(p.1 - a.1);
    }
    ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / len
}

/// Douglas–Peucker, iterative: the points kept within `tol` of the ring.
fn simplify(ring: &[(f64, f64)], tol: f64) -> Ring {
    let n = ring.len();
    if n < 4 {
        return ring.to_vec();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let mut stack = vec![(0usize, n - 1)];
    while let Some((s, e)) = stack.pop() {
        let (mut far, mut at) = (0.0, 0);
        for i in s + 1..e {
            let d = off(ring[i], ring[s], ring[e]);
            if d > far {
                far = d;
                at = i;
            }
        }
        if far > tol {
            keep[at] = true;
            stack.push((s, at));
            stack.push((at, e));
        }
    }
    ring.iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(p, _)| *p)
        .collect()
}

fn ring(v: &Value) -> Option<Ring> {
    v.as_array()?
        .iter()
        .map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?)))
        .collect()
}

/// A feature's outer rings (holes, lakes inside a country, are left out at this size).
fn outer_rings(geometry: &Value) -> Vec<Ring> {
    let coords = &geometry["coordinates"];
    let polygons: Vec<&Value> = match geometry["type"].as_str() {
        Some("Polygon") => vec![coords],
        Some("MultiPolygon") => coords
            .as_array()
            .map(|a| a.iter().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    polygons
        .into_iter()
        .filter_map(|p| ring(p.get(0)?))
        .collect()
}

fn bbox(rings: &[Ring]) -> (f64, f64, f64, f64) {
    rings
        .iter()
        .flatten()
        .fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |b, &(x, y)| {
            (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y))
        })
}

/// A two-letter ISO code: Natural Earth's own, or its "EH" one where the first is `-99`
/// (France, Norway and a few more).
fn code(props: &Value) -> Option<[u8; 2]> {
    ["ISO_A2", "ISO_A2_EH"].iter().find_map(|k| {
        let c = props[*k].as_str()?.as_bytes();
        (c.len() == 2 && c.iter().all(u8::is_ascii_uppercase)).then(|| [c[0], c[1]])
    })
}

#[allow(clippy::cast_possible_truncation)]
fn centi(v: f64) -> i16 {
    (v * 100.0).round().clamp(-32_767.0, 32_767.0) as i16
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [source, out] = args.as_slice() else {
        return fail("usage: countries <ne_50m_admin_0_countries.geojson> <out.bin>");
    };
    let Ok(text) = std::fs::read_to_string(source) else {
        return fail("cannot read the GeoJSON");
    };
    let Ok(doc) = serde_json::from_str::<Value>(&text) else {
        return fail("not JSON");
    };
    let Some(features) = doc["features"].as_array() else {
        return fail("no features");
    };
    let mut countries: Vec<Packed> = Vec::new();
    let mut points = 0usize;
    for f in features {
        let Some(code) = code(&f["properties"]) else {
            continue;
        };
        let rings = outer_rings(&f["geometry"]);
        let (x0, y0, x1, y1) = bbox(&rings);
        // Degrees per pixel at the map's width; the longer side sets the scale.
        let span = (x1 - x0).max(y1 - y0).max(0.01);
        let px = span / VIEW_PX;
        let kept: Vec<Vec<(i16, i16)>> = rings
            .iter()
            .filter(|r| {
                let (a, b, c, d) = bbox(std::slice::from_ref(r));
                (c - a).max(d - b) >= 2.0 * px
            })
            .map(|r| simplify(r, 0.5 * px))
            .filter(|r| r.len() >= 4)
            .map(|r| r.into_iter().map(|(x, y)| (centi(x), centi(y))).collect())
            .collect();
        let mut kept = kept;
        for &(_, lat, lon) in ISLANDS.iter().filter(|(c, _, _)| *c == code) {
            let h = ISLAND_HALF;
            let diamond = [
                (lon, lat + h),
                (lon + h, lat),
                (lon, lat - h),
                (lon - h, lat),
                (lon, lat + h),
            ];
            kept.push(diamond.iter().map(|&(x, y)| (centi(x), centi(y))).collect());
        }
        if kept.is_empty() || countries.iter().any(|(c, _)| *c == code) {
            continue;
        }
        points += kept.iter().map(Vec::len).sum::<usize>();
        countries.push((code, kept));
    }
    countries.sort_by_key(|(c, _)| *c);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"FCTY");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(
        &u16::try_from(countries.len())
            .unwrap_or(u16::MAX)
            .to_le_bytes(),
    );
    for (code, rings) in &countries {
        bytes.extend_from_slice(code);
        bytes.extend_from_slice(&u16::try_from(rings.len()).unwrap_or(u16::MAX).to_le_bytes());
        for r in rings {
            bytes.extend_from_slice(&u16::try_from(r.len()).unwrap_or(u16::MAX).to_le_bytes());
            for (x, y) in r {
                bytes.extend_from_slice(&x.to_le_bytes());
                bytes.extend_from_slice(&y.to_le_bytes());
            }
        }
    }
    let written = std::fs::File::create(out).and_then(|mut f| f.write_all(&bytes));
    if written.is_err() {
        return fail("cannot write the output");
    }
    println!(
        "{} countries, {points} points, {} bytes, BLAKE3 {}",
        countries.len(),
        bytes.len(),
        blake3::hash(&bytes).to_hex()
    );
    ExitCode::SUCCESS
}
