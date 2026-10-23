# File Formats — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> The registry of **every file type Fotollection reads or writes**. For each type it lists the extension, MIME type, Uniform Type Identifier (UTI), how the file is recognized, the container, the current version, its limits and where it lives.
> **Field-level schemas live only in [DATABASE.md](DATABASE.md)**; this file links to them and never repeats them. Any change to a format updates this file and DATABASE.md in the same commit ([§6](#6-change-rules)).

## 1. Registry

| Type | Extension | MIME type | UTI | Recognized by | Container | Current version | Status |
|------|-----------|-----------|-----|---------------|-----------|-----------------|--------|
| Project | `.fotollection` | `application/vnd.fotollection.project+json` | `com.hoangphi01.fotollection.project` | `"format": "fotollection-project"` | JSON, UTF-8 | `schema_version` 1 | Implemented (session project) |
| Project bundle | `.fotollection` | `application/vnd.fotollection.project+zip` | `com.hoangphi01.fotollection.project` | ZIP magic `PK\x03\x04` + `manifest.json` | ZIP | — | Planned, [ROADMAP Stage 2](ROADMAP.md#phase-1-work-order-2026-10-23) |
| Preset | `.fotopreset` | `application/vnd.fotollection.preset+json` | `com.hoangphi01.fotollection.preset` | `"format": "fotollection-preset"` | JSON, UTF-8 | `schema_version` 1 (draft) | Planned ([FR-038](PRD.md#fr-038--print-presets)) |
| App settings | `settings.json` | `application/json` | — (internal) | `"format": "fotollection-settings"` | JSON, UTF-8 | `schema_version` 2 | Implemented |
| User font | `fonts/<id>.ttf`, `fonts/<id>.otf` | `font/ttf`, `font/otf` | — (internal copy) | Magic `00 01 00 00` / `true` (TTF), `OTTO` (OTF) | sfnt | — | Implemented ([FR-042](PRD.md#fr-042--custom-fonts)) |
| Camera logo | `logos/<id>.png` | `image/png` | — (internal copy) | PNG magic | PNG | — | Implemented ([FR-040](PRD.md#fr-040--exif-info-frame)) |
| Thumbnail cache | `thumbs/…` | `image/jpeg` | — (internal) | Path only | JPEG files | — | Implemented; disposable |

The `.fotollection` extension is shared by both project containers. **A reader always identifies a file by its content, never by its extension:** JSON with the `format` field, or ZIP with a `manifest.json`.

## 2. User-facing files

These files leave the app. Users copy, email and open them on other machines, so they must stay readable by later versions.

### 2.1 Project `.fotollection`

- **Today (v1, JSON):** folders and sparse per-photo edits (crop, rating, flag). It is written atomically, with a `.bak` copy. Schema: [DATABASE.md §4](DATABASE.md#4-project-file-schema).
- **Planned bundle (ZIP, like `.docx`):** one self-contained file carries everything needed to reproduce the prints on another machine. Photos themselves are referenced, not embedded.

  | Part | Content |
  |------|---------|
  | `manifest.json` | Format, version, and a list of parts with SHA-256 checksums |
  | `project.json` | The project tables |
  | `settings.json` | Render, frame, custom print sizes and camera profiles |
  | `fonts/`, `logos/` | Copies of the fonts and logos the project uses |
  | `presets/` | Copies of the user presets the project uses |

  When the bundle is opened, embedded fonts and logos are validated like any import, then installed. Moved photo folders are relinked through a dialog. The bundle lands in Stage 2, together with an ADR that supersedes [ADR-0010](adr/0010-project-file-format.md).
- **Limits:**
  - JSON v1: 64 MB per file.
  - Planned bundle: caps on the entry count, the total uncompressed size and the size of each part; entry names may not contain `..`, absolute paths or drive letters ([SECURITY.md §3](SECURITY.md#other-untrusted-inputs)).
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

## 4. Input photos

Photos are always identified by magic bytes, never by extension. Every input is untrusted ([SECURITY.md §3](SECURITY.md#3-input-and-metadata-defense)).

| Format | Extensions | Magic | Status | Notes |
|--------|------------|-------|--------|-------|
| JPEG | `.jpg`, `.jpeg`, `.jpe` | `FF D8 FF` | Supported | EXIF orientation, ICC profile, embedded thumbnail |
| PNG | `.png` | `89 50 4E 47 0D 0A 1A 0A` | Planned, [ROADMAP Stage 1](ROADMAP.md#phase-1-work-order-2026-10-23) | — |
| HEIC/HEIF | `.heic`, `.heif`, `.hif` | `ftyp` box at offset 4 | Planned, Stage 3 | Codec licence under review ([ADR-0005](adr/0005-input-formats-jpeg-heic-png.md)) |
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
- **File associations** (double-click to open `.fotollection` and `.fotopreset`) are registered in `tauri.conf.json` once Open exists (Stage 2). Until then, these MIME types and UTIs are reserved but not registered with the OS.
