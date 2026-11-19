# Bundled places

The offline place data of the location stamp and place search ([FR-005](../../../../docs/PRD.md#fr-005--location-stamp-offline), [FR-032](../../../../docs/PRD.md#fr-032--manual-date-and-place), [ADR-0003](../../../../docs/adr/0003-offline-geocoding-geonames.md)). `places.bin` is compiled into the app, checked against the BLAKE3 sum below when it is loaded, and **never downloaded at runtime**. `scripts/geodata/fetch-and-pack.sh` rebuilds it.

| File | Contents | Source | Licence |
|------|----------|--------|---------|
| `places.bin` | 161,937 towns and cities with at least 1,000 people, 252 countries | [GeoNames](https://www.geonames.org/) `cities1000.zip` and `countryInfo.txt`, snapshot 2026-10-24 | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), © GeoNames |
| `zones.bin` | The IANA time zone of each place of `places.bin`, in its order (FR-145): 392 zones, 30 places without one | GeoNames `cities1000.zip` (column 18), download of 2026-11-19 | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), © GeoNames |
| `countries.bin` | 235 country outlines for RollSense's map (FR-141), 31,004 points | [Natural Earth](https://www.naturalearthdata.com/) 1:50m admin-0 countries, release v5.1.2 | Public domain |

Places that are not a town in their own right are left out: sections of a city (`PPLX`, such as "Praha 1") and historical, abandoned or destroyed places (`PPLH`, `PPLQ`, `PPLW`, `PPLCH`).

## Checksum

| File | Size | BLAKE3 (checked at load) |
|------|-----:|--------------------------|
| `places.bin` | 5,406,549 bytes | `b8b2c5496fb3b29ff077cfb924711927827ff2bd4efb4177e45f238d9babb069` |

Downloads used for this snapshot (SHA-256): `cities1000.zip` `5e1bf5a85957fd65602dd66fb02d3bdcffd3eaa377204769c5035487f57d95bc`, `countryInfo.txt` `c55f638cfaafb4fe45a9aa15120a0f1f78e8a34a8e1847d7b9320cf740fbc9ab`.

## Format (version 1, little-endian)

1. Header: `FGEO`, `u16` version, the snapshot date as 10 ASCII bytes, `u16` country count, `u32` place count.
2. Countries: 2-byte ISO code, `u8` length, English name (UTF-8).
3. Places, largest population first: `u32` GeoNames id, `i32` latitude and `i32` longitude in 1e-5 degrees, `u32` population, `u16` country index, `u8` 1 for a capital or first-level seat, `u8` length + name (UTF-8), `u8` length + ASCII name (empty when equal to the name).

The reader rejects any file whose checksum, magic, version or lengths do not match.

## Country outlines (`countries.bin`)

`crates/geodata`'s `countries` packer rebuilds it from Natural Earth's GeoJSON (`scripts/geodata/fetch-and-pack.sh countries`): each country's outer rings (lakes left out), simplified by Douglas–Peucker to half a pixel of a 275 px wide map, islands narrower than two pixels dropped, so a small country keeps its shape. Vietnam's map always holds the Hoàng Sa and Trường Sa archipelagos (owner, 2026-11-19), which 1:50m leaves out: a small mark for each main island, from its known position, in the packer's `ISLANDS` list. Checked against its BLAKE3 sum at load by `crates/core/src/geo_shapes.rs`.

| File | Size | BLAKE3 (checked at load) |
|------|-----:|--------------------------|
| `countries.bin` | 126,896 bytes | `f7fad8441a75a4241101c218aca64fe1f2fd8dbcb48e9046bb4ee43d2f9f170b` |

Download used (SHA-256): `ne_50m_admin_0_countries.geojson` (v5.1.2) `3e458fc036ad0a66411f2c1e6cac49c5d7bfb81cb1123bc513b22511a2b7fdeb`.

Format (version 1, little-endian): `FCTY`, `u16` version, `u16` country count; per country its 2-byte ISO 3166-1 alpha-2 code (Natural Earth's `ISO_A2`, or `ISO_A2_EH` where that is `-99`), `u16` ring count; per ring a `u16` point count, then each point as `i16` longitude and `i16` latitude in hundredths of a degree. The reader refuses a wrong magic or version, a length past the end, more than 1,000 rings or 20,000 points in one.

## Time zones (`zones.bin`)

`crates/geodata`'s `zones` packer (`scripts/geodata/fetch-and-pack.sh zones`) reads the GeoNames ids from the bundled `places.bin`, in its order, and each id's IANA zone from a fresh `cities1000.txt`, so `places.bin` is not packed again; a place gone from the newer dump gets no zone. The zones' rules, past summer times included, come from the IANA time zone database (public domain) compiled into `chrono-tz` (MIT OR Apache-2.0, tzdb 2025b); nothing is downloaded at runtime. Checked against its BLAKE3 sum at load by `crates/core/src/zones.rs`, and against `places.bin`: the file names the BLAKE3 of the `places.bin` it was packed for and its record count, and is refused when either differs.

| File | Size | BLAKE3 (checked at load) |
|------|-----:|--------------------------|
| `zones.bin` | 330,351 bytes | `60b1a138dd66b6d272ad03ed35c234ecab1f8493aaa3cac174c69b0ed84ad9d8` |

Download used (SHA-256): `cities1000.zip` (2026-11-19) `1dcfe6ea1731529fe53d54c8c524add5a8f4b198cff7ff0ab5e5b774d528db17`.

Format (version 1, little-endian): `FZON`, `u16` version, the 32-byte BLAKE3 of `places.bin`, `u32` place count (equal to `places.bin`'s), `u16` zone count (at most 1,024); per zone a `u8` length and its IANA name (ASCII); then one `u16` per place in `places.bin`'s order, the zone's index or `0xFFFF` for none. The reader refuses a wrong magic, version, hash or count, a zone index past the table, and a file shorter or longer than its counts; a name `chrono-tz` does not know gives no zone.
