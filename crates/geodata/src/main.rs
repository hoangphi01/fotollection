// SPDX-License-Identifier: GPL-3.0-only
//! Packs GeoNames `cities1000.txt` + `countryInfo.txt` into `places.bin`, the offline place
//! data of FR-005 / FR-032 ([ADR-0003]). Run by `scripts/geodata/fetch-and-pack.sh`; the
//! format is read by `crates/core/src/geocode.rs` and described in
//! `crates/core/assets/geo/GEODATA.md`.
//!
//! `fotollection-geodata <cities1000.txt> <countryInfo.txt> <snapshot YYYY-MM-DD> <out.bin>`

use std::collections::HashMap;
use std::io::Write;
use std::process::ExitCode;

/// Feature codes that are not a town in their own right: sections of a city (PPLX),
/// historical, abandoned or destroyed places.
const SKIPPED: [&str; 5] = ["PPLX", "PPLH", "PPLQ", "PPLW", "PPLCH"];

struct Record {
    id: u32,
    lat: i32,
    lon: i32,
    population: u32,
    country: u16,
    seat: bool,
    name: String,
    ascii: String,
}

fn fail(message: &str) -> ExitCode {
    eprintln!("geodata: {message}");
    ExitCode::FAILURE
}

#[allow(clippy::cast_possible_truncation)]
fn micro(value: &str) -> Option<i32> {
    let v: f64 = value.parse().ok()?;
    (-180.0..=180.0)
        .contains(&v)
        .then(|| (v * 1e5).round() as i32)
}

/// ISO code → index, and (code, English name) rows in file order.
type Countries = (HashMap<String, u16>, Vec<(String, String)>);

fn read_countries(text: &str) -> Countries {
    let mut index: HashMap<String, u16> = HashMap::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let cols: Vec<&str> = line.split('\t').collect();
        if let (Some(code), Some(name)) = (cols.first(), cols.get(4))
            && code.len() == 2
            && let Ok(i) = u16::try_from(rows.len())
        {
            index.insert((*code).to_owned(), i);
            rows.push(((*code).to_owned(), (*name).to_owned()));
        }
    }
    (index, rows)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [cities, countries, snapshot, out] = args.as_slice() else {
        return fail(
            "usage: fotollection-geodata <cities1000.txt> <countryInfo.txt> <YYYY-MM-DD> <out.bin>",
        );
    };
    if snapshot.len() != 10 {
        return fail("snapshot must be YYYY-MM-DD");
    }
    let Ok(country_text) = std::fs::read_to_string(countries) else {
        return fail("cannot read countryInfo.txt");
    };
    let (country_index, country_rows) = read_countries(&country_text);

    let Ok(city_text) = std::fs::read_to_string(cities) else {
        return fail("cannot read cities1000.txt");
    };
    let mut records = Vec::new();
    for line in city_text.lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 15 || SKIPPED.contains(&c[7]) {
            continue;
        }
        let (Ok(id), Some(lat), Some(lon), Some(&country)) = (
            c[0].parse::<u32>(),
            micro(c[4]),
            micro(c[5]),
            country_index.get(c[8]),
        ) else {
            continue;
        };
        let (name, ascii) = (c[1].trim(), c[2].trim());
        if name.is_empty() || name.len() > 255 || ascii.len() > 255 {
            continue;
        }
        records.push(Record {
            id,
            lat,
            lon,
            population: c[14].parse().unwrap_or(0),
            country,
            seat: matches!(c[7], "PPLC" | "PPLA" | "PPLG"),
            name: name.to_owned(),
            ascii: if ascii == name {
                String::new()
            } else {
                ascii.to_owned()
            },
        });
    }
    // Largest first: search results come out in population order for free.
    records.sort_by(|a, b| b.population.cmp(&a.population).then(a.id.cmp(&b.id)));

    let mut bin = Vec::with_capacity(records.len() * 40);
    bin.extend_from_slice(b"FGEO");
    bin.extend_from_slice(&1u16.to_le_bytes());
    bin.extend_from_slice(snapshot.as_bytes());
    let (Ok(n_countries), Ok(n_records)) = (
        u16::try_from(country_rows.len()),
        u32::try_from(records.len()),
    ) else {
        return fail("too many rows");
    };
    bin.extend_from_slice(&n_countries.to_le_bytes());
    bin.extend_from_slice(&n_records.to_le_bytes());
    for (code, name) in &country_rows {
        bin.extend_from_slice(code.as_bytes());
        let Ok(len) = u8::try_from(name.len()) else {
            return fail("country name too long");
        };
        bin.push(len);
        bin.extend_from_slice(name.as_bytes());
    }
    for r in &records {
        bin.extend_from_slice(&r.id.to_le_bytes());
        bin.extend_from_slice(&r.lat.to_le_bytes());
        bin.extend_from_slice(&r.lon.to_le_bytes());
        bin.extend_from_slice(&r.population.to_le_bytes());
        bin.extend_from_slice(&r.country.to_le_bytes());
        bin.push(u8::from(r.seat));
        // Lengths were checked above (≤ 255 bytes).
        bin.push(u8::try_from(r.name.len()).unwrap_or(u8::MAX));
        bin.extend_from_slice(r.name.as_bytes());
        bin.push(u8::try_from(r.ascii.len()).unwrap_or(u8::MAX));
        bin.extend_from_slice(r.ascii.as_bytes());
    }
    let written = std::fs::File::create(out).and_then(|mut f| f.write_all(&bin));
    if written.is_err() {
        return fail("cannot write the output");
    }
    eprintln!(
        "geodata: {} places, {} countries, {} bytes, BLAKE3 {}",
        records.len(),
        country_rows.len(),
        bin.len(),
        blake3::hash(&bin).to_hex()
    );
    ExitCode::SUCCESS
}
