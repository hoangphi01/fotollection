// SPDX-License-Identifier: GPL-3.0-only
//! Each bundled place's time zone (FR-145, ADR-0058): `zones.bin`, packed by `crates/geodata`'s
//! `zones` from GeoNames (CC BY 4.0) beside `places.bin` and in its order, compiled in and
//! checked against a pinned BLAKE3 sum and against the places it names. The zones' rules
//! come from `chrono-tz`. Nothing is ever downloaded.

use std::str::FromStr;
use std::sync::OnceLock;

use chrono_tz::Tz;
use serde::Serialize;
use specta::Type;

use crate::geocode::{self, GeoError};

const ZONES: &[u8] = include_bytes!("../assets/geo/zones.bin");
const ZONES_BLAKE3: &str = "60b1a138dd66b6d272ad03ed35c234ecab1f8493aaa3cac174c69b0ed84ad9d8";
const MAGIC: &[u8; 4] = b"FZON";
const FORMAT_VERSION: u16 = 1;
const NO_ZONE: u16 = u16::MAX;
/// About 600 IANA names exist; more is not a zone table.
const MAX_ZONES: usize = 1024;
/// The most zones a search answers with.
pub const MAX_HITS: usize = 20;

/// The zone of each place, by record index.
struct ZoneIndex {
    /// Each zone of the table; `None` for a name `chrono-tz` does not know.
    zones: Vec<Option<Tz>>,
    of_place: Vec<u16>,
}

/// Reads `bytes`, refusing anything but a table for exactly `places` records whose data has
/// the BLAKE3 `places_hash` (hex). Every length is checked before it is read.
fn parse(bytes: &[u8], places: usize, places_hash: &str) -> Result<ZoneIndex, GeoError> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Result<&[u8], GeoError> {
        let end = at.checked_add(n).ok_or(GeoError::Malformed)?;
        let s = bytes.get(at..end).ok_or(GeoError::Malformed)?;
        at = end;
        Ok(s)
    };
    let le16 = |b: &[u8]| u16::from_le_bytes([b[0], b[1]]);
    if take(4)? != MAGIC || le16(take(2)?) != FORMAT_VERSION {
        return Err(GeoError::Malformed);
    }
    let named = blake3::Hash::from_hex(places_hash).map_err(|_| GeoError::Malformed)?;
    let hash = take(32)?;
    let count = take(4)?;
    let count = u32::from_le_bytes([count[0], count[1], count[2], count[3]]);
    if hash != named.as_bytes() || usize::try_from(count).ok() != Some(places) {
        return Err(GeoError::Malformed);
    }
    let n_zones = usize::from(le16(take(2)?));
    if n_zones > MAX_ZONES {
        return Err(GeoError::Malformed);
    }
    let mut zones = Vec::with_capacity(n_zones);
    for _ in 0..n_zones {
        let len = usize::from(take(1)?[0]);
        let name = std::str::from_utf8(take(len)?).map_err(|_| GeoError::Malformed)?;
        zones.push(Tz::from_str(name).ok());
    }
    let table = take(places.checked_mul(2).ok_or(GeoError::Malformed)?)?;
    let of_place: Vec<u16> = table
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&p| u16::from_le_bytes(p))
        .collect();
    if of_place
        .iter()
        .any(|&z| z != NO_ZONE && usize::from(z) >= n_zones)
        || take(1).is_ok()
    {
        return Err(GeoError::Malformed);
    }
    Ok(ZoneIndex { zones, of_place })
}

fn index() -> Result<&'static ZoneIndex, GeoError> {
    static CELL: OnceLock<Result<ZoneIndex, GeoError>> = OnceLock::new();
    CELL.get_or_init(|| {
        if blake3::hash(ZONES).to_hex().as_str() != ZONES_BLAKE3 {
            return Err(GeoError::Checksum);
        }
        let (places, hash) = geocode::records()?;
        parse(ZONES, places, hash)
    })
    .as_ref()
    .map_err(Clone::clone)
}

fn of_record(i: u32) -> Option<Tz> {
    let index = index().ok()?;
    let z = *index.of_place.get(usize::try_from(i).ok()?)?;
    index.zones.get(usize::from(z)).copied().flatten()
}

/// The time zone of the place with this GeoNames id; `None` for an unknown place or one
/// without a zone, or when the bundled data fails its checks.
pub fn of_place(geonames_id: u32) -> Option<Tz> {
    of_record(geocode::record_index(geonames_id).ok()??)
}

/// A zone found by a search: its IANA name and the largest place found in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ZoneHit {
    /// `Asia/Ho_Chi_Minh`.
    pub zone: String,
    /// `Ho Chi Minh City`.
    pub place: String,
    /// `Vietnam`.
    pub country: String,
}

/// The zones of the places, and of the countries, whose name starts with `query`, each zone
/// once, the largest place first (FR-145: *Choose a place…* for the home zone), at most
/// [`MAX_HITS`].
pub fn search(query: &str) -> Result<Vec<ZoneHit>, GeoError> {
    index()?;
    let places = geocode::search(query, 200)?
        .into_iter()
        .filter_map(|p| Some((geocode::record_index(p.geonames_id).ok()??, p)));
    let countries = geocode::in_countries(query, 2_000)?;
    let mut hits: Vec<ZoneHit> = Vec::new();
    for (i, p) in places.chain(countries) {
        let Some(zone) = of_record(i) else { continue };
        if hits.iter().any(|h| h.zone == zone.name()) {
            continue;
        }
        hits.push(ZoneHit {
            zone: zone.name().to_owned(),
            place: p.name,
            country: p.country,
        });
        if hits.len() == MAX_HITS {
            break;
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A zone table in the bundled format, for places whose data hashes to `ab…ab`.
    fn build(names: &[&str], of: &[u16]) -> Vec<u8> {
        let mut b = MAGIC.to_vec();
        b.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        b.extend_from_slice(&[0xAB; 32]);
        b.extend_from_slice(&u32::try_from(of.len()).unwrap_or(0).to_le_bytes());
        b.extend_from_slice(&u16::try_from(names.len()).unwrap_or(0).to_le_bytes());
        for n in names {
            b.push(u8::try_from(n.len()).unwrap_or(0));
            b.extend_from_slice(n.as_bytes());
        }
        for z in of {
            b.extend_from_slice(&z.to_le_bytes());
        }
        b
    }

    fn places() -> (usize, &'static str) {
        geocode::records().unwrap_or((0, ""))
    }

    /// FR-145: the bundled index matches places.bin, every zone in it is one `chrono-tz` knows,
    /// and almost every place has one.
    #[test]
    fn fr_145_the_bundled_zones_match_the_places() -> Result<(), GeoError> {
        let (n, hash) = places();
        let index = parse(ZONES, n, hash)?;
        assert!(
            index.zones.iter().all(Option::is_some),
            "a zone chrono-tz lacks"
        );
        let missing = index.of_place.iter().filter(|&&z| z == NO_ZONE).count();
        assert!(missing < 100, "{missing} places without a zone");
        Ok(())
    }

    /// FR-145: places find their zone, by GeoNames id.
    #[test]
    fn fr_145_places_have_their_zone() -> Result<(), GeoError> {
        let zone = |name: &str, cc: &str| -> Result<Option<Tz>, GeoError> {
            let p = geocode::search(name, 20)?
                .into_iter()
                .find(|p| p.country_code == cc);
            Ok(p.and_then(|p| of_place(p.geonames_id)))
        };
        assert_eq!(zone("Seoul", "KR")?, Some(Tz::Asia__Seoul));
        assert_eq!(zone("Debrecen", "HU")?, Some(Tz::Europe__Budapest));
        assert_eq!(of_place(u32::MAX), None);
        Ok(())
    }

    /// FR-145: a search by city or by country gives each zone once, a long country several.
    #[test]
    fn fr_145_zone_search_dedupes_and_finds_countries() -> Result<(), GeoError> {
        let budapest = search("Budapest")?;
        assert_eq!(
            budapest.first().map(|h| h.zone.as_str()),
            Some("Europe/Budapest")
        );
        let vietnam = search("Vietnam")?;
        assert!(
            vietnam.iter().any(|h| h.zone == "Asia/Ho_Chi_Minh"),
            "{vietnam:?}"
        );
        let russia = search("Russia")?;
        assert!(russia.len() > 5, "{russia:?}");
        let mut zones: Vec<&str> = russia.iter().map(|h| h.zone.as_str()).collect();
        zones.dedup();
        assert_eq!(zones.len(), russia.len());
        assert_eq!(search("")?, Vec::new());
        Ok(())
    }

    /// FR-145: a damaged index is refused, never read past its end: a wrong magic, version,
    /// places hash or count, a zone index past the table, a cut or a longer file, and
    /// 20,000 random mutations never panic.
    #[test]
    fn fr_145_malformed_zones_are_refused() {
        let (n, hash) = places();
        let at_table = 4 + 2 + 32 + 4 + 2;
        let mut bad = Vec::new();
        for (at, byte) in [
            (0, b'X'),
            (4, 9),
            (6, 1),
            (38, 0xFF),
            (42, 0xFF),
            (43, 0xFF),
        ] {
            let mut b = ZONES.to_vec();
            b[at] ^= byte;
            bad.push(b);
        }
        let mut past = ZONES.to_vec();
        let last = past.len() - 2;
        past[last..].copy_from_slice(&1000u16.to_le_bytes());
        bad.push(past);
        bad.push(ZONES[..ZONES.len() - 1].to_vec());
        bad.push([ZONES, &[0]].concat());
        bad.push(ZONES[..at_table].to_vec());
        bad.push(Vec::new());
        for b in &bad {
            assert!(parse(b, n, hash).is_err());
        }
        assert!(parse(ZONES, n + 1, hash).is_err());
        assert!(parse(ZONES, n, "00").is_err());
        // A small table of its own, mutated anywhere: refused or read, never a panic.
        let small = build(
            &["Asia/Seoul", "Europe/Budapest", "Not/AZone"],
            &[0, 1, 2, NO_ZONE],
        );
        assert!(parse(&small, 4, &"ab".repeat(32)).is_ok_and(|z| z.zones[2].is_none()));
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        for _ in 0..20_000 {
            let mut b = small.clone();
            for _ in 0..=state % 3 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let at = usize::try_from(state % (b.len() as u64)).unwrap_or(0);
                b[at] = u8::try_from(state >> 56).unwrap_or(0);
            }
            if let Ok(z) = parse(&b, 4, &"ab".repeat(32)) {
                assert_eq!(z.of_place.len(), 4);
            }
        }
    }
}
