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
