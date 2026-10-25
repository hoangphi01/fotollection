# File Formats — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> The registry of **every file type Fotollection reads or writes**. For each type it lists the extension, MIME type, Uniform Type Identifier (UTI), how the file is recognized, the container, the current version, its limits and where it lives.
> **Field-level schemas live only in [DATABASE.md](DATABASE.md)**; this file links to them and never repeats them. Any change to a format updates this file and DATABASE.md in the same commit ([§6](#6-change-rules)).

## 1. Registry

| Type | Extension | MIME type | UTI | Recognized by | Container | Current version | Status |
|------|-----------|-----------|-----|---------------|-----------|-----------------|--------|
| Project (legacy, and the session autosave) | `.fotollection` | `application/vnd.fotollection.project+json` | `com.hoangphi01.fotollection.project` | `"format": "fotollection-project"` | JSON, UTF-8 | `schema_version` 5 | Implemented: read always, written only for the session autosave |
| Project bundle | `.fotollection` | `application/vnd.fotollection.project+zip` | `com.hoangphi01.fotollection.project` | ZIP magic `PK\x03\x04` + `manifest.json` with `"format": "fotollection-bundle"` | ZIP (deflate) | manifest `schema_version` 1 | Implemented ([ADR-0016](adr/0016-self-contained-project-bundle.md)): what Save / Save As write |
| Preset | `.fotopreset` | `application/vnd.fotollection.preset+json` | `com.hoangphi01.fotollection.preset` | `"format": "fotollection-preset"` | JSON, UTF-8 | `schema_version` 1 (draft) | Planned ([FR-038](PRD.md#fr-038--print-presets)) |
| App settings | `settings.json` | `application/json` | — (internal) | `"format": "fotollection-settings"` | JSON, UTF-8 | `schema_version` 8 | Implemented |
| User font | `fonts/<id>.ttf`, `fonts/<id>.otf` | `font/ttf`, `font/otf` | — (internal copy) | Magic `00 01 00 00` / `true` (TTF), `OTTO` (OTF) | sfnt | — | Implemented ([FR-042](PRD.md#fr-042--custom-fonts)) |
| Camera logo | `logos/<id>.png` | `image/png` | — (internal copy) | PNG magic | PNG | — | Implemented ([FR-040](PRD.md#fr-040--exif-info-frame)) |
| Thumbnail cache | `thumbs/…` | `image/jpeg` | — (internal) | Path only | JPEG files | — | Implemented; disposable |
| Bundled places | `places.bin` (compiled in) | — | — (internal) | `FGEO` magic + version | Custom binary ([GEODATA.md](../crates/core/assets/geo/GEODATA.md)) | 1 | Implemented ([FR-005](PRD.md#fr-005--location-stamp-offline)) |
| Analysis cache | `analysis/<aa>/<key>.json` | `application/json` | — (internal) | Path only | JSON, UTF-8 | `version` = `ANALYSIS_VERSION` 2 | Implemented; disposable ([FR-041](PRD.md#fr-041--best-shot-suggestions)) |

The `.fotollection` extension is shared by both project containers. **A reader always identifies a file by its content, never by its extension:** JSON with the `format` field, or ZIP with a `manifest.json`.

## 2. User-facing files

These files leave the app. Users copy, email and open them on other machines, so they must stay readable by later versions.

### 2.1 Project `.fotollection`

- **Bundle (ZIP, like `.docx`, [ADR-0016](adr/0016-self-contained-project-bundle.md)):** one self-contained file carries everything needed to reproduce the prints on another machine. Photos themselves are referenced, not embedded. Any unzip tool can list it.

  | Part | Content | Cap |
  |------|---------|----:|
  | `manifest.json` | `format`, `schema_version`, `app_version`, and every other part with its byte count and BLAKE3 sum | 1 MB |
  | `project.json` | The project tables, schema v5 ([DATABASE.md §4](DATABASE.md#4-project-file-schema)). Folders are stored relative to the bundle when possible, with the absolute path as a fallback | 64 MB |
  | `settings.json` | Render (print size, custom sizes, stamp), frame, camera profiles and the fonts table, in the `settings.json` format without the per-machine `ui` block ([DATABASE.md §6](DATABASE.md#6-app-settings)) | 4 MB |
  | `fonts/<id>.ttf\|otf` | The fonts the project uses (date stamp and frame), stored uncompressed | 20 MB each |
  | `logos/<id>.png` | Logos of camera profiles set to an image logo, by profile id | 2 MB each |

  Presets are not part of version 1; adding them means a new allowlisted part and a manifest version bump.
- **Writing:** parts in a fixed order with the ZIP epoch (1980-01-01) as their time, so the same project gives the same bytes. The file is written atomically with a `.bak` copy.
- **Opening:** the whole bundle is validated before anything is applied. Then fonts and logos are installed (a font already present with the same content is reused), the print settings are applied, and folders are found relative to the bundle first, then by absolute path. A folder that is still missing is relinked by the user. Test fixture: `crates/core/tests/fixtures/bundle/v1.fotollection`.
- **JSON (legacy, schema v1–v2):** folders and sparse per-photo edits (crop, rating, flag), atomic with a `.bak`. It is still opened, but only the tables are read. The session autosave uses this format in the app data folder. Fixture: `crates/core/tests/fixtures/project/v1.fotollection`.
- **Limits:**
  - Bundle: at most 160 MB, 128 entries and a 100:1 compression ratio, checked before decompression; part names from a strict allowlist, so `..`, absolute paths, drive letters and unknown files are rejected ([SECURITY.md §3](SECURITY.md#other-untrusted-inputs)).
  - JSON: 64 MB per file.
- **Personal data:** the file contains folder paths, and later places and coordinates. Treat it like the photos.

### 2.2 Preset `.fotopreset`

A partial, stackable set of print or Photo Style settings ([DATABASE.md §7.1](DATABASE.md#71-preset-file-fotopreset)). Import is untrusted input, and a preset that fails to parse is skipped with a warning.

### 2.3 Exported prints

Exports are plain JPEG files that any lab accepts. They are not a Fotollection format, but their conventions are fixed:

| Property | Value |
|----------|-------|
| Path | `<destination>/YYYY-MM-DD/<source stem>.jpg`, or `undated/` when there is no capture date ([FR-009](PRD.md#fr-009--export-by-date)) |
| Name collisions | `_1`, `_2`, … appended; an existing file is never overwritten |
| Encoding | Baseline JPEG, quality 95, 4:4:4 chroma, optimized Huffman tables |
| Colour | The source ICC profile is copied unchanged (sRGB conversion: [FR-027](PRD.md#fr-027--srgb-conversion-for-labs)) |
| Metadata | No EXIF is written, so GPS is always stripped ([FR-010](PRD.md#fr-010--metadata-privacy-on-export)) |
| Write | Temp file, then rename; a cancelled export removes its partial files |

## 3. Internal files

These files live in the app's config and cache directories ([DATABASE.md §1](DATABASE.md#1-storage-map)). Users never need to open them, and their layout may change between versions, but always with a migration.

| File | Rules | Schema |
|------|-------|--------|
| `settings.json` | At most 4 MB. Version 1 files are read with defaults for the new fields. A newer version is refused, and defaults are used without overwriting the file. A corrupt file is moved aside | [DATABASE.md §6](DATABASE.md#6-app-settings) |
| `fonts/<id>.ttf\|otf` | At most 20 MB each and 50 fonts. TrueType collections (`ttcf`) and WOFF are rejected. The font must contain the digits 0–9 | [DATABASE.md §6](DATABASE.md#6-app-settings) |
| `logos/<id>.png` | At most 2 MB and 4096 px per edge. Decoded with a pure-Rust PNG decoder before it is copied | [DATABASE.md §6](DATABASE.md#6-app-settings) |
| `thumbs/` | Derived data, rebuilt on demand. Safe to delete at any time | [DATABASE.md §8](DATABASE.md#8-thumbnail-cache) |
| `analysis/` | At most 64 KB per file; another version, invalid JSON or out-of-range values are recomputed. Safe to delete at any time | [DATABASE.md §8.1](DATABASE.md#81-analysis-cache) |

## 4. Input photos

Photos are always identified by magic bytes, never by extension. Every input is untrusted ([SECURITY.md §3](SECURITY.md#3-input-and-metadata-defense)).

| Format | Extensions | Magic | Status | Notes |
|--------|------------|-------|--------|-------|
| JPEG | `.jpg`, `.jpeg`, `.jpe` | `FF D8 FF` | Supported | EXIF orientation, ICC profile, embedded thumbnail |
| PNG | `.png` | `89 50 4E 47 0D 0A 1A 0A` | Supported | 8/16-bit, grey and palette → RGB8; alpha flattened onto white; iCCP profile kept; EXIF from `eXIf` if present |
| HEIC/HEIF | `.heic`, `.heif`, `.hif` | `ftyp` box at offset 4 | Supported on macOS and Windows | Decoded by the OS (ImageIO / WIC; Windows needs the HEIF + HEVC Store extensions), primary image, alpha on white, ICC kept on macOS; EXIF read from the container ([ADR-0017](adr/0017-heic-through-os-codecs.md)) |
| RAW | `.dng`, `.rw2`, `.cr2`, … | — | Phase 3 ([FR-016](PRD.md#fr-016--raw-input)) | Rejected as unsupported today |

These limits apply to every input format:

- at most 200 MB per file;
- at most 30,000 px per edge and 200 megapixels, checked before any pixels are decoded;
- a file whose first 4 KB contain script or markup (`<script`, `<html`, `<svg`, `<?php`) is rejected as a possible polyglot.

## 5. Compatibility rules

1. **Version every user-facing and persisted format.** JSON formats carry a `format` string and an integer `schema_version`, and the planned bundle carries both in `manifest.json`.
2. **Read old files, migrate forward.** Each version step is a pure migration function with a fixture test ([DATABASE.md §4.3](DATABASE.md#43-versioning-and-migration)). A file is rewritten in the new version only on the next save, and the old one stays as `.bak`.
3. **Never open a newer file read-write.** An older app refuses a newer `schema_version` rather than damaging the file.
4. **Ignore unknown fields** within a known version, so that additive changes stay readable.
5. **Never reuse a `format` string or a `schema_version`.** Released versions are frozen.
6. **Store lengths in millimetres and colours as `#rrggbb`.** Units entered by the user, such as inches, are kept only as a display hint.

## 6. Change rules

- Any change to a format listed here updates this file, [DATABASE.md](DATABASE.md), the `schema_version`, a migration and a fixture, all in the same commit ([CLAUDE.md](../CLAUDE.md)).
- A new user-facing format gets an ADR and a row in §1 with its MIME type and UTI.
- **File associations:** `.fotollection` is registered in `tauri.conf.json` (`bundle.fileAssociations`): role Editor, rank Owner, MIME `application/vnd.fotollection.project+zip`, and the UTI `com.hoangphi01.fotollection.project` exported as conforming to `public.zip-archive`. macOS delivers double-clicked files as an `Opened` event, and Windows passes them on the command line. `.fotopreset` is registered once presets exist (FR-038).
