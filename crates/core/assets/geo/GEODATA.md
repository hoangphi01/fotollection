# Bundled places

The offline place data of the location stamp and place search ([FR-005](../../../../docs/PRD.md#fr-005--location-stamp-offline), [FR-032](../../../../docs/PRD.md#fr-032--manual-date-and-place), [ADR-0003](../../../../docs/adr/0003-offline-geocoding-geonames.md)). `places.bin` is compiled into the app, checked against the BLAKE3 sum below when it is loaded, and **never downloaded at runtime**. `scripts/geodata/fetch-and-pack.sh` rebuilds it.

| File | Contents | Source | Licence |
|------|----------|--------|---------|
| `places.bin` | 161,937 towns and cities with at least 1,000 people, 252 countries | [GeoNames](https://www.geonames.org/) `cities1000.zip` and `countryInfo.txt`, snapshot 2026-10-24 | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), © GeoNames |

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
