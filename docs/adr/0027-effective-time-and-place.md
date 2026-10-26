# ADR-0027: Effective time and place, with the camera's own values kept

- **Status:** Accepted (time: FR-030, done in v0.8.0; place: FR-031, FR-032, FR-005, being built)
- **Date:** 2026-10-26
- **Related:** FR-030, FR-031, FR-032, FR-005, NFR-020, ADR-0003, [DATABASE.md §3.2](../DATABASE.md#32-effective-value-resolution)

## Context

Photos from one trip disagree about when and where they were taken: a camera's clock is set to the wrong hour or time zone, a Lumix GX9 has no GPS, a scanned print has no EXIF at all. Sorting, best-shot groups, day folders, the date stamp, the info frame and GPS matching all read the capture time; the date line and the info frame read the place. Until v0.8.0 they read the raw EXIF time, and the place came from GPS only.

Measured first ([RESULT.md §14](../RESULT.md#14-places-and-time-v080-measured-first)): sorting 70,000 photos takes 0.9 ms and recomputing a corrected-time column 0.05 ms, so correcting times costs nothing measurable.

## Options considered

1. **Rewrite the times and places into the files.** Rejected: Fotollection never writes into source photos (§9 of the PRD), and the user would lose what the camera recorded.
2. **Correct per photo**, stored with each photo's edits. Simple, but one wrong clock touches hundreds of rows, and photos added later stay wrong.
3. **Correct per camera body**, plus per-photo values only where the user sets them by hand, resolved in one fixed order at read time. The camera's own values stay in memory for reference.

## Decision

Option 3.

- **Effective time** = a manual date set on the photo (FR-032) → otherwise the camera's time plus its body's clock offset (FR-030) → otherwise the file's time (never shifted). A body is (make, model, serial number when the camera writes one), so two bodies of one model keep their own clocks.
- **Effective place** = a manual place (FR-032) → a place borrowed from another device's photo of the same scene, applied automatically only at high confidence (FR-031) → the photo's GPS reverse-geocoded offline → none; a renamed place (FR-005) replaces the dataset's name everywhere.
- Clock offsets live in the **project** (schema v7, `cameras[]`), because a wrong clock belongs to a trip, not to the machine. They are limited to a month and time zones to ±14 h, checked in Rust on every change and on load.
- `PhotoRecord.capture` is the effective time and `PhotoRecord.camera_time` the camera's own, so every reader of the capture time follows the correction without knowing about it; the export writes the effective time into the copy's EXIF.

## Consequences

- One correction moves every photo of the body at once, including photos imported later.
- A clock correction is not a step in the edit history (it is not a per-photo edit); the Camera clocks card sets it back to zero in one click.
- The capture time shown in the grid and the preview is the effective one, with the camera's own time under it when they differ.
- Places follow the same pattern in FR-031 and FR-032, as fields of the photo's edits (so they are undoable) and a rename table in the project.
