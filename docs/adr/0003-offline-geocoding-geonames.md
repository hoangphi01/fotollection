# ADR-0003: Offline reverse geocoding with GeoNames

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** FR-005, NFR-006

## Context

Location stamps need GPS coordinates turned into a place name. The original plan used the OpenStreetMap Nominatim API. That requires a network connection, is limited to 1 request per second, and sends the user's coordinates to a third party. Both of these conflict with the offline and privacy goals.

## Options considered

1. **Nominatim API:** accurate, but online, rate-limited, and leaks coordinates.
2. **Self-hosted geocoder:** needs a server, which contradicts ADR-0001.
3. **Bundled GeoNames `cities1000` + k-d tree:** about 150k places with a population of 1,000 or more, about 5 MB packed. Nearest-city lookup takes microseconds and needs no network.

## Decision

Bundle a packed GeoNames `cities1000` dataset (CC BY 4.0) with country and admin-1 names, indexed with a `kiddo` k-d tree. It is built at compile time by `crates/geodata`.

## Consequences

- Fully offline and private. Results are city-level, which is right for a print stamp.
- Remote locations snap to the nearest town. The user can override the name per group ([FR-005](../PRD.md#fr-005--location-stamp-offline)).
- GeoNames attribution is required in the About dialog and README.
- The dataset is refreshed occasionally, as part of a release.

## Implementation (2026-10-24)

- **No `kiddo`:** a 1° × 1° grid built at first use (25 ms, together with the checksum) gives about 5 µs per lookup, so no extra dependency is needed.
- **Packed by `crates/geodata`** into `crates/core/assets/geo/places.bin`: 161,937 places, 5.4 MB, version header. It is pinned by a BLAKE3 sum like the face models ([ADR-0018](0018-face-models-and-runtime.md)); GeoNames changes its dump daily, so the committed file is the reference ([GEODATA.md](../../crates/core/assets/geo/GEODATA.md)). `scripts/geodata/fetch-and-pack.sh` rebuilds it.
- **Town level only:** sections of a city (`PPLX`, such as "Praha 1") and historical places are left out.
- **Choosing the place:** each place reaches `1 km + 0.012 km × √population` (about 6 km for Debrecen, 46 km for Ho Chi Minh City), and a photo goes to the place whose reach it is deepest in, within 30 km. Some countries list city districts as places of their own (Vietnam's "Quận 4"), so in the inner 30 % of the reach of a capital, a provincial seat or a city of a million, the larger place wins when it is at least 5 times bigger. Tests pin Prague, Debrecen, Districts 1 and 7 of Ho Chi Minh City, Hanoi and Paris, and check that Budaörs (next to Budapest) and Sintra keep their own names.
