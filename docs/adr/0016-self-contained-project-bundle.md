# ADR-0016: Self-contained project bundle (ZIP)

- **Status:** Accepted. Supersedes [ADR-0010](0010-project-file-format.md) for the container; the table schema stays in [DATABASE.md §4](../DATABASE.md#4-project-file-schema)
- **Date:** 2026-10-24
- **Related:** FR-020, FR-023, FR-040, FR-042, FR-043, NFR-013, ADR-0014, ADR-0015

## Context

A `.fotollection` file was a single JSON document with folders and per-photo edits ([ADR-0010](0010-project-file-format.md)). The owner wants it to work like a `.docx`: save it, open it on another machine, and get the same prints. A JSON file cannot do that, because the look of a print also depends on files that live outside it:

- the print settings, the info frame and the camera profiles, which live in the app's `settings.json`;
- user fonts and camera logos, which are copied into the app's config folder ([FR-042](../PRD.md#fr-042--custom-fonts), [ADR-0014](0014-user-supplied-brand-logos.md)).

A project also had to work when the project file and its photos are copied together to a new place.

## Options considered

1. **Keep JSON and inline fonts and logos as base64.** One file, but it grows by a third, the fonts become hard to inspect, and a 20 MB font makes every autosave slow.
2. **A folder package (`Trip.fotollection/`)** with the parts as files. That is natural on macOS, but Windows and email treat it as a folder, so it is easy to break.
3. **A ZIP container like `.docx`/OPC**, holding a manifest and typed parts. One file everywhere, inspectable with any unzip tool, and deflate keeps the JSON small.

## Decision

Use a **ZIP bundle** ([EXTENSION.md §2.1](../EXTENSION.md#21-project-fotollection)) with exactly these parts:

- `manifest.json`: format `fotollection-bundle`, schema version 1, and every part with its size and BLAKE3 sum;
- `project.json`: the project tables, schema v2;
- `settings.json`: render, frame, camera profiles and fonts, in the `settings.json` format, without the per-machine `ui` block ([ADR-0015](0015-ui-localization-i18next.md));
- `fonts/<id>.ttf|otf`: only the fonts the project uses;
- `logos/<id>.png`: the logos of cameras set to an image logo.

Photos are **referenced, not embedded**. A folder is stored relative to the bundle when possible, with its absolute path as a fallback (project schema v2), so a bundle copied together with its photos opens anywhere. A folder that cannot be found is relinked by the user.

Reading treats the bundle as untrusted:

- part names come from an **allowlist** (no `..`, no absolute paths, no stray files);
- there are at most 128 entries and 160 MB uncompressed, each part has its own size cap, and no part may be compressed more than 100:1 — all checked **before** decompression;
- reads are bounded by the real byte count, not by the header;
- every BLAKE3 sum must match the manifest;
- every part goes through its typed reader: project validation, settings validation, the font and PNG validators.

Writing is deterministic (fixed order, fixed ZIP epoch) and atomic with a `.bak`. The crate is `zip` (MIT), with deflate only, on `flate2` with the pure-Rust `miniz_oxide` backend. A JSON v1/v2 project file is still recognised by its content and opened.

## Consequences

- Opening a bundle installs its fonts and logos after validation, deduplicated by content, and applies its print settings. The interface language stays with each machine.
- The session autosave stays a plain JSON file in the app data folder, so autosave never has to rewrite fonts. The user's `.fotollection` document is written by Save and Save As (FR-023).
- New parts (presets, places) need a new allowlist pattern, a manifest version bump and a fixture (`crates/core/tests/fixtures/bundle/`).
