// SPDX-License-Identifier: GPL-3.0-only
//! Offline places (FR-005, FR-032, ADR-0003): GPS → the town or city it is in, and search by
//! name. The data is GeoNames `cities1000` (CC BY 4.0), packed by `crates/geodata`, compiled
//! into the binary and checked against a pinned BLAKE3 sum. Nothing is ever downloaded.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use specta::Type;

const PLACES: &[u8] = include_bytes!("../assets/geo/places.bin");
const PLACES_BLAKE3: &str = "b8b2c5496fb3b29ff077cfb924711927827ff2bd4efb4177e45f238d9babb069";
const MAGIC: &[u8; 4] = b"FGEO";
const FORMAT_VERSION: u16 = 1;

/// Places further than this from the photo are never suggested.
const MAX_DISTANCE_KM: f64 = 30.0;
/// A place "reaches" `1 km + REACH_PER_SQRT_POP × √population`: about 6 km for a city of
/// 200,000 and 35 km for one of 8 million. A photo goes to the place whose reach it is
/// deepest in, so a city's outskirts get the city's name, not the nearest village's.
const REACH_BASE_KM: f64 = 1.0;
const REACH_PER_SQRT_POP: f64 = 0.012;
const EARTH_KM: f64 = 6371.0;
/// Districts can be places of their own (Vietnam's "Quận 4" has 180,000 people). When the
/// photo is in the inner 30 % of the reach of a major place (a capital or provincial seat, or a
/// city of a million; GeoNames lists Ho Chi Minh City as a plain PPL) at least this many
/// times larger, that place's name wins: District 1 is "Ho Chi Minh City", while a town
/// 20 km from Hanoi keeps its own name.
const MAJOR_POPULATION: u32 = 1_000_000;
const MAJOR_MIN_RATIO: u32 = 5;
const MAJOR_MAX_SCORE: f64 = 0.3;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GeoError {
    #[error("bundled place data does not match its checksum")]
    Checksum,
    #[error("bundled place data is malformed")]
    Malformed,
}

/// A town or city.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub geonames_id: u32,
    pub name: String,
    /// ISO 3166-1 alpha-2, e.g. `HU`.
    pub country_code: String,
    /// English country name, e.g. `Hungary`.
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub population: u32,
}

struct Record {
    id: u32,
    lat: f64,
    lon: f64,
    population: u32,
    country: u16,
    /// A capital or first-level administrative seat (GeoNames PPLC, PPLA, PPLG).
    seat: bool,
    name: &'static str,
    /// The ASCII spelling, or `""` when it equals `name`.
    ascii: &'static str,
}

struct Dataset {
    snapshot: &'static str,
    countries: Vec<(&'static str, &'static str)>,
    /// Largest population first.
    records: Vec<Record>,
    /// Record indices per 1° × 1° cell.
    grid: HashMap<(i16, i16), Vec<u32>>,
}

struct Reader {
    bytes: &'static [u8],
    at: usize,
}

impl Reader {
    fn take(&mut self, n: usize) -> Result<&'static [u8], GeoError> {
        let end = self.at.checked_add(n).ok_or(GeoError::Malformed)?;
        let slice = self.bytes.get(self.at..end).ok_or(GeoError::Malformed)?;
        self.at = end;
        Ok(slice)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], GeoError> {
        self.take(N)?.try_into().map_err(|_| GeoError::Malformed)
    }
    fn u8(&mut self) -> Result<u8, GeoError> {
        Ok(self.array::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, GeoError> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32, GeoError> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn i32(&mut self) -> Result<i32, GeoError> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    fn str(&mut self, len: usize) -> Result<&'static str, GeoError> {
        std::str::from_utf8(self.take(len)?).map_err(|_| GeoError::Malformed)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn cell(lat: f64, lon: f64) -> (i16, i16) {
    (lat.floor() as i16, lon.floor() as i16)
}

fn parse(bytes: &'static [u8]) -> Result<Dataset, GeoError> {
    let mut r = Reader { bytes, at: 0 };
    if &r.array::<4>()? != MAGIC || r.u16()? != FORMAT_VERSION {
        return Err(GeoError::Malformed);
    }
    let snapshot = r.str(10)?;
    let n_countries = r.u16()?;
    let n_records = r.u32()?;
    let mut countries = Vec::with_capacity(usize::from(n_countries));
    for _ in 0..n_countries {
        let code = r.str(2)?;
        let len = usize::from(r.u8()?);
        countries.push((code, r.str(len)?));
    }
    let mut records = Vec::with_capacity(n_records as usize);
    let mut grid: HashMap<(i16, i16), Vec<u32>> = HashMap::new();
    for i in 0..n_records {
        let id = r.u32()?;
        let lat = f64::from(r.i32()?) / 1e5;
        let lon = f64::from(r.i32()?) / 1e5;
        let population = r.u32()?;
        let country = r.u16()?;
        let seat = r.u8()? != 0;
        let name_len = usize::from(r.u8()?);
        let name = r.str(name_len)?;
        let ascii_len = usize::from(r.u8()?);
        let ascii = r.str(ascii_len)?;
        if usize::from(country) >= countries.len() || !(-90.0..=90.0).contains(&lat) {
            return Err(GeoError::Malformed);
        }
        grid.entry(cell(lat, lon)).or_default().push(i);
        records.push(Record {
            id,
            lat,
            lon,
            population,
            country,
            seat,
            name,
            ascii,
        });
    }
    if r.at != bytes.len() {
        return Err(GeoError::Malformed);
    }
    Ok(Dataset {
        snapshot,
        countries,
        records,
        grid,
    })
}

fn dataset() -> Result<&'static Dataset, GeoError> {
    static CELL: OnceLock<Result<Dataset, GeoError>> = OnceLock::new();
    CELL.get_or_init(|| {
        if blake3::hash(PLACES).to_hex().as_str() != PLACES_BLAKE3 {
            return Err(GeoError::Checksum);
        }
        parse(PLACES)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// The GeoNames snapshot date of the bundled data, for the About dialog.
pub fn snapshot() -> Result<&'static str, GeoError> {
    Ok(dataset()?.snapshot)
}

fn place(d: &Dataset, r: &Record) -> Place {
    let (code, country) = d.countries[usize::from(r.country)];
    Place {
        geonames_id: r.id,
        name: r.name.to_owned(),
        country_code: code.to_owned(),
        country: country.to_owned(),
        lat: r.lat,
        lon: r.lon,
        population: r.population,
    }
}

/// Great-circle distance in km.
fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = p2 - p1;
    let dl = (lon2 - lon1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * EARTH_KM * a.sqrt().asin()
}

/// The town or city a GPS position is in; `None` far from any place (at sea, in the wild).
pub fn reverse(lat: f64, lon: f64) -> Result<Option<Place>, GeoError> {
    let d = dataset()?;
    if !lat.is_finite() || !lon.is_finite() || !(-90.0..=90.0).contains(&lat) {
        return Ok(None);
    }
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    // Cells within reach: one degree of latitude is 111 km; a degree of longitude shrinks
    // towards the poles.
    #[allow(clippy::cast_possible_truncation)]
    let lon_cells = (1.0 / lat.to_radians().cos().max(0.1)).ceil().min(10.0) as i16;
    let (cy, cx) = cell(lat, lon);
    let mut best: Option<(f64, &Record)> = None;
    let mut majors: Vec<(f64, &Record)> = Vec::new();
    for dy in -1..=1 {
        for dx in -lon_cells..=lon_cells {
            let wrapped = (cx + dx + 180).rem_euclid(360) - 180;
            let Some(ids) = d.grid.get(&(cy + dy, wrapped)) else {
                continue;
            };
            for &i in ids {
                let r = &d.records[i as usize];
                let km = distance_km(lat, lon, r.lat, r.lon);
                let reach = REACH_BASE_KM + REACH_PER_SQRT_POP * f64::from(r.population).sqrt();
                if km > MAX_DISTANCE_KM.max(reach) {
                    continue;
                }
                let score = km / reach;
                if best.is_none_or(|(s, _)| score < s) {
                    best = Some((score, r));
                }
                if (r.seat || r.population >= MAJOR_POPULATION) && score <= MAJOR_MAX_SCORE {
                    majors.push((score, r));
                }
            }
        }
    }
    let best = best.map(|(score, r)| {
        majors
            .iter()
            .filter(|(_, m)| m.population >= r.population.saturating_mul(MAJOR_MIN_RATIO))
            .max_by_key(|(_, s)| s.population)
            .copied()
            .unwrap_or((score, r))
    });
    Ok(best.map(|(_, r)| place(d, r)))
}

/// The place with this GeoNames id, for a place stored in a project (FR-032).
pub fn by_id(geonames_id: u32) -> Result<Option<Place>, GeoError> {
    static INDEX: OnceLock<HashMap<u32, u32>> = OnceLock::new();
    let d = dataset()?;
    let index = INDEX.get_or_init(|| {
        d.records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| Some((r.id, u32::try_from(i).ok()?)))
            .collect()
    });
    Ok(index
        .get(&geonames_id)
        .and_then(|&i| d.records.get(i as usize))
        .map(|r| place(d, r)))
}

/// Lower-case ASCII with diacritics removed and spaces dropped, so "Hà Nội", "ha noi" and
/// "Hanoi" all become "hanoi".
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c)
            } else {
                crate::text::fold_char(c)
            }
        })
        .collect()
}

/// Places whose name, or a word of it, starts with `query` (diacritics and spaces ignored),
/// largest first.
pub fn search(query: &str, limit: usize) -> Result<Vec<Place>, GeoError> {
    let d = dataset()?;
    let q = fold(query);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    // The name from any word on, so "chi minh" finds "Ho Chi Minh City".
    let matches = |name: &str| {
        !name.is_empty()
            && std::iter::once(0)
                .chain(name.match_indices([' ', '-']).map(|(i, _)| i + 1))
                .any(|start| fold(&name[start..]).starts_with(&q))
    };
    Ok(d.records
        .iter()
        .filter(|r| matches(r.name) || matches(r.ascii))
        .take(limit)
        .map(|r| place(d, r))
        .collect())
}

/// How a place is written on a print (FR-005).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PlaceFormat {
    /// `Debrecen`
    #[default]
    City,
    /// `Debrecen, HU`
    CityCountryCode,
    /// `Debrecen, Hungary`
    CityCountry,
}

/// The text printed for a place.
pub fn label(name: &str, country_code: &str, country: &str, format: PlaceFormat) -> String {
    let suffix = match format {
        PlaceFormat::City => "",
        PlaceFormat::CityCountryCode => country_code,
        PlaceFormat::CityCountry => country,
    };
    if suffix.is_empty() {
        name.to_owned()
    } else {
        format!("{name}, {suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_at(lat: f64, lon: f64) -> Option<String> {
        reverse(lat, lon).ok().flatten().map(|p| p.name)
    }

    #[test]
    fn fr_005_known_places_resolve_to_their_town() {
        // Old Town Square, Prague: the city, not a district.
        assert_eq!(name_at(50.0875, 14.4213).as_deref(), Some("Prague"));
        // Debrecen, the Great Church (photographed in the reference set).
        assert_eq!(name_at(47.5316, 21.6252).as_deref(), Some("Debrecen"));
        // District 1, Ho Chi Minh City, about 7 km from the GeoNames point.
        assert_eq!(
            name_at(10.7769, 106.7009).as_deref(),
            Some("Ho Chi Minh City")
        );
        // District 7, about 12 km out.
        assert_eq!(
            name_at(10.7340, 106.7218).as_deref(),
            Some("Ho Chi Minh City")
        );
        // Hoan Kiem Lake, Hanoi.
        assert_eq!(name_at(21.0287, 105.8524).as_deref(), Some("Hanoi"));
        // Towns next to a big city keep their own name.
        assert_eq!(name_at(47.4621, 18.9580).as_deref(), Some("Budaörs")); // 12 km from Budapest
        assert_eq!(name_at(38.7975, -9.3906).as_deref(), Some("Sintra")); // 25 km from Lisbon
        // Central Paris.
        assert_eq!(name_at(48.8584, 2.2945).as_deref(), Some("Paris")); // the Eiffel Tower
    }

    #[test]
    fn fr_005_open_sea_has_no_place() {
        assert_eq!(name_at(0.0, -30.0), None);
        assert_eq!(name_at(f64::NAN, 10.0), None);
        assert_eq!(name_at(95.0, 10.0), None);
    }

    #[test]
    fn fr_032_search_ignores_diacritics_and_ranks_by_size() {
        let first = |q: &str| {
            search(q, 5)
                .ok()
                .and_then(|v| v.into_iter().next())
                .map(|p| p.name)
        };
        assert_eq!(first("debre").as_deref(), Some("Debrecen"));
        assert_eq!(first("Hà Nội").as_deref(), Some("Hanoi"));
        assert_eq!(first("chi minh").as_deref(), Some("Ho Chi Minh City"));
        assert_eq!(search("", 5).map(|v| v.len()), Ok(0));
    }

    #[test]
    fn fr_005_labels_follow_the_format() {
        assert_eq!(
            label("Debrecen", "HU", "Hungary", PlaceFormat::City),
            "Debrecen"
        );
        assert_eq!(
            label("Debrecen", "HU", "Hungary", PlaceFormat::CityCountryCode),
            "Debrecen, HU"
        );
        assert_eq!(
            label("Debrecen", "HU", "Hungary", PlaceFormat::CityCountry),
            "Debrecen, Hungary"
        );
        assert_eq!(fold("Hà Nội"), "hanoi");
    }

    #[test]
    fn damaged_data_is_rejected() {
        assert_eq!(parse(b"FGEO").err(), Some(GeoError::Malformed));
        assert_eq!(parse(b"NOPE\x01\x00").err(), Some(GeoError::Malformed));
        assert!(snapshot().is_ok_and(|s| s.len() == 10));
    }
}
