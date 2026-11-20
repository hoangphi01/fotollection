// SPDX-License-Identifier: GPL-3.0-only
//! Packs each bundled place's IANA time zone into `zones.bin` (FR-145, ADR-0058), beside
//! `places.bin` and in the same order, so places.bin is never packed again for it. The ids are
//! read from the bundled `places.bin` itself; each id's zone from GeoNames `cities1000.txt`
//! (column 18, the IANA name). A place missing from the newer dump gets no zone. Read by
//! `crates/core/src/zones.rs`, described in `crates/core/assets/geo/GEODATA.md`.
//!
//! A place takes its own country's zone (owner, 2026-11-20: Hanoi listed on Asia/Bangkok):
//! GeoNames follows IANA's `zone1970.tab`, where a zone shared since 1970 covers several
//! countries (Asia/Bangkok: Thailand, Cambodia, Laos and most of Vietnam); when the place's
//! country has one zone in `zone.tab` and that shared zone covers the country, the country's
//! own zone is taken (Asia/Ho_Chi_Minh, Asia/Phnom_Penh). The clock is the same for the photos
//! people bring (Vietnam's two differ only from 1970 to 1975); countries with several zones
//! keep GeoNames' choice.
//!
//! `zones <places.bin> <cities1000.txt> <zone.tab> <zone1970.tab> <out.bin>`

use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::process::ExitCode;

const NO_ZONE: u16 = u16::MAX;
/// The longest IANA name has 32 characters (`America/Argentina/ComodRivadavia`); more is not a
/// zone.
const MAX_NAME: usize = 64;

fn fail(message: &str) -> ExitCode {
    eprintln!("zones: {message}");
    ExitCode::FAILURE
}

/// The GeoNames ids of `places.bin`, in its order (format in GEODATA.md).
fn place_ids(bytes: &[u8]) -> Option<Vec<u32>> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = bytes.get(at..at.checked_add(n)?)?;
        at += n;
        Some(s)
    };
    if take(4)? != b"FGEO" || take(2)? != [1, 0] {
        return None;
    }
    take(10)?;
    let countries = u16::from_le_bytes(take(2)?.try_into().ok()?);
    let records = u32::from_le_bytes(take(4)?.try_into().ok()?);
    for _ in 0..countries {
        take(2)?;
        let len = usize::from(take(1)?[0]);
        take(len)?;
    }
    let mut ids = Vec::with_capacity(usize::try_from(records).ok()?);
    for _ in 0..records {
        ids.push(u32::from_le_bytes(take(4)?.try_into().ok()?));
        take(4 + 4 + 4 + 2 + 1)?;
        let name = usize::from(take(1)?[0]);
        take(name)?;
        let ascii = usize::from(take(1)?[0]);
        take(ascii)?;
    }
    Some(ids)
}

fn valid_zone(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_+-".contains(&b))
}

/// Each country's zone where `zone.tab` gives it exactly one.
fn single_zones(text: &str) -> HashMap<&str, &str> {
    let mut all: HashMap<&str, Vec<&str>> = HashMap::new();
    for c in text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect::<Vec<_>>())
    {
        if let (Some(cc), Some(zone)) = (c.first(), c.get(2)) {
            all.entry(cc).or_default().push(zone);
        }
    }
    all.into_iter()
        .filter_map(|(cc, z)| (z.len() == 1).then(|| (cc, z[0])))
        .collect()
}

/// The countries each zone of `zone1970.tab` covers.
fn covered(text: &str) -> HashMap<&str, Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            Some((*c.get(2)?, c.first()?.split(',').collect()))
        })
        .collect()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let [_, places, cities, zone_tab, zone1970, out] = args.as_slice() else {
        return fail(
            "usage: zones <places.bin> <cities1000.txt> <zone.tab> <zone1970.tab> <out.bin>",
        );
    };
    let (Ok(zone_tab), Ok(zone1970)) = (
        std::fs::read_to_string(zone_tab),
        std::fs::read_to_string(zone1970),
    ) else {
        return fail("cannot read zone.tab or zone1970.tab");
    };
    let own = single_zones(&zone_tab);
    let shared = covered(&zone1970);
    let mut switched = 0usize;
    let Ok(places) = std::fs::read(places) else {
        return fail("cannot read places.bin");
    };
    let Some(ids) = place_ids(&places) else {
        return fail("places.bin is malformed");
    };
    let Ok(text) = std::fs::read_to_string(cities) else {
        return fail("cannot read cities1000.txt");
    };
    let mut zone_of: HashMap<u32, &str> = HashMap::new();
    for line in text.lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if let (Some(Ok(id)), Some(cc), Some(zone)) =
            (c.first().map(|s| s.parse::<u32>()), c.get(8), c.get(17))
            && valid_zone(zone)
        {
            let zone = match own.get(cc) {
                Some(&mine)
                    if mine != *zone && shared.get(zone).is_some_and(|cs| cs.contains(cc)) =>
                {
                    switched += 1;
                    mine
                }
                _ => zone,
            };
            zone_of.insert(id, zone);
        }
    }
    let names: Vec<&str> = ids
        .iter()
        .filter_map(|id| zone_of.get(id).copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let Ok(n_zones) = u16::try_from(names.len()) else {
        return fail("too many zones");
    };
    let index: HashMap<&str, u16> = names
        .iter()
        .enumerate()
        .filter_map(|(i, n)| Some((*n, u16::try_from(i).ok()?)))
        .collect();
    let Ok(n_places) = u32::try_from(ids.len()) else {
        return fail("too many places");
    };
    let mut bytes = Vec::with_capacity(48 + names.len() * 24 + ids.len() * 2);
    bytes.extend_from_slice(b"FZON");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(blake3::hash(&places).as_bytes());
    bytes.extend_from_slice(&n_places.to_le_bytes());
    bytes.extend_from_slice(&n_zones.to_le_bytes());
    for name in &names {
        let Ok(len) = u8::try_from(name.len()) else {
            return fail("zone name too long");
        };
        bytes.push(len);
        bytes.extend_from_slice(name.as_bytes());
    }
    let mut missing = 0usize;
    for id in &ids {
        let z = zone_of
            .get(id)
            .and_then(|n| index.get(n).copied())
            .unwrap_or_else(|| {
                missing += 1;
                NO_ZONE
            });
        bytes.extend_from_slice(&z.to_le_bytes());
    }
    if std::fs::File::create(out)
        .and_then(|mut f| f.write_all(&bytes))
        .is_err()
    {
        return fail("cannot write the output");
    }
    println!(
        "{} places, {} zones, {} without a zone, {} cities moved to their country's own zone, {} bytes, BLAKE3 {}",
        ids.len(),
        names.len(),
        missing,
        switched,
        bytes.len(),
        blake3::hash(&bytes).to_hex()
    );
    ExitCode::SUCCESS
}
