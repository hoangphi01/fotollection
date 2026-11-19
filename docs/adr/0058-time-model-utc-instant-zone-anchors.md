# ADR-0058: Time model: a UTC instant per photo, the place's zone, anchors in the photo

- **Status:** Accepted
- **Date:** 2026-11-19
- **Related:** FR-145, FR-030, FR-141, FR-083, FR-049, ADR-0027

## Context

Photo times were camera-local seconds stored as if they were UTC, while a photo dated from its file took the file's real UTC time; the GPS time was not read, a photo's own offset only told one source from another, and each camera's zone (`tz_minutes`, FR-030) was saved but unused. RollSense had also read the raw clock instead of ADR-0027's corrected time (fixed 2026-11-19). The owner asked for one standard time at +0, a home time from the user's computer, and times that stay right on a laptop taken abroad, and asked what anchors the time if the computer's clock is wrong.

## Options considered

1. **Keep local seconds, add an offset where known.** Least work; mixed bases stay, a day abroad is wrong.
2. **Ask a time server (time.microsoft) for a country's current time.** Needs the network, gives only the time now, not the rules of past years, and anchors on something outside the photo.
3. **A UTC instant per photo from anchors in the photo, shown in the place's zone, with a home zone for clocks that carry none, rules from the IANA database compiled in.** Offline, true for past years, independent of the computer's clock.

## Decision

Option 3. Each photo's instant is resolved in the core from, in order: the EXIF GPS time; the photo's offset; the camera's clock plus its FR-030 correction read in that camera's zone or else the home zone; the file's time. It is shown in the zone of its place when it has one, else in the offset it came with. The home zone is an IANA name kept in the settings, suggested from the system at first run and changed only by the user. `jiff` (Unlicense or MIT) holds the rules, always from the IANA database it bundles, never the system's, and reads the system's zone name (amended 2026-11-19: `chrono-tz` was unmaintained, RUSTSEC-2026-0555, and `iana-time-zone`'s Windows path was the suspect of a crash in CI); a zone index beside `places.bin` gives each GeoNames place its zone.

## Consequences

- One local day (instant plus the place's offset) for RollSense, the calendar, the trips and later the journal; the trips' saved edits keep days since 1970.
- The computer's clock is never used to place a photo, only to suggest the home zone and as the file's own time.
- Larger binary by the zone rules and index (measured against NFR-008); the resolve runs once per photo in the roll's pass (measured, OPTIMIZATION §5).
- Settings gain the home zone (a schema bump); the GPS time parser is bounded and mutation-tested.
