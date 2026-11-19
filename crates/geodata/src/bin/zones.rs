// SPDX-License-Identifier: GPL-3.0-only
//! Packs each bundled place's IANA time zone into `zones.bin` (FR-145, ADR-0058), beside
//! `places.bin` and in the same order, so places.bin is never packed again for it. The ids are
//! read from the bundled `places.bin` itself; each id's zone from GeoNames `cities1000.txt`
//! (column 18, the IANA name). A place missing from the newer dump gets no zone. Read by
//! `crates/core/src/zones.rs`, described in `crates/core/assets/geo/GEODATA.md`.
//!
//! `zones <places.bin> <cities1000.txt> <out.bin>`

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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let [_, places, cities, out] = args.as_slice() else {
        return fail("usage: zones <places.bin> <cities1000.txt> <out.bin>");
    };
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
        if let (Some(Ok(id)), Some(zone)) = (c.first().map(|s| s.parse::<u32>()), c.get(17))
            && valid_zone(zone)
        {
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
        "{} places, {} zones, {} without a zone, {} bytes, BLAKE3 {}",
        ids.len(),
        names.len(),
        missing,
        bytes.len(),
        blake3::hash(&bytes).to_hex()
    );
    ExitCode::SUCCESS
}
