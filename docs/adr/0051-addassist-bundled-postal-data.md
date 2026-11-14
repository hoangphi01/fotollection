# ADR-0051: AddAssist: each post's own data, bundled and looked up offline in Rust

- **Status:** Accepted
- **Date:** 2026-11-15
- **Related:** FR-133, FR-130, FR-132, ADR-0003, ADR-0044

## Context

The Japanese and Korean postcards need an address written as each country's post standardises it, and the owner asked for a switch between typing by hand and typing with help where the post's data is open (FR-133, "AddAssist"). The app is offline: the photos and the user's typing never leave the computer, and the core makes no network call beyond the update check (CLAUDE.md, rule 4). Japan Post's postal-code data claims no copyright and may be redistributed freely; it is 18.4 MB of CSV, 124,527 rows.

## Options considered

1. **An online address API** (Japan Post's, Google's). Always current, nothing bundled; but it sends what the user types to a service, needs a network and a key, and breaks the offline promise.
2. **The raw CSV in the app, parsed at start.** Simple; but 18 MB on disk, and a parse of 124,527 rows each launch.
3. **A packed table per country, compiled into the binary, as the places are (ADR-0003).** A build-time crate (`crates/addassistdata`, never shipped) packs each post's data into a small binary format, every name stored once; the core checks its BLAKE3 sum and parses it on first use (`OnceLock`), with no `unsafe` and every count and id bounded.

## Decision

Option 3. One module, `addassist`, owns every country behind one interface (`countries`, `lookup`, `search`), so the postcard and any later form only call it. AddAssist is versioned as a format is: **1.0** first (Japan and Korea; Korea's table, from the Ministry of the Interior's road-name addresses, keeps each road's runs of building numbers per code, so a road and a number give the code), a minor version for a country, a field or a newer edition of a country's data, a major one for a change of interface; the version is one constant, shown with each country's data date and kept with an address it filled. Latin names are made in Rust from the post's own kana (one Hepburn rule set) rather than from a separate romaji edition. The tables are stored uncompressed: the installers compress them already (Japan's 5.5 MB table is 1.3 MB with xz), and no decompression runs on first use.

## Consequences

- Nothing typed leaves the computer; AddAssist works without a network.
- The installed app grows by each table's size (Japan 5.5 MB, Korea 6.5 MB); its installer by about a quarter of that (1.3 MB each with xz), within NFR-008. First use costs 6 ms (checksum and parse) and 40 ms more on the first search (every name folded once); a search runs in about 4 ms (OPTIMIZATION.md §5).
- Each release rebuilds the tables from the posts' current data (`scripts/addassist/fetch-and-pack.sh`), so a code added since stays typable by hand until the next release.
- A country is added as one entry and its table, its source and terms in `crates/core/assets/addassist/ADDASSIST.md`; a post whose data is not open (Korea Post's own file states no terms, Deutsche Post's is paid) is not used.
