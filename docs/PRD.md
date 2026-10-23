# Product Requirements Document — Fotollection

> **Owner:** product · **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> This document is the single source of truth for **what** Fotollection does and **why**.
> *How* it is built lives in [ARCHITECTURE.md](ARCHITECTURE.md); *when* lives in [ROADMAP.md](ROADMAP.md).
> Requirement IDs (`FR-xxx`, `NFR-xxx`) are permanent: never renumber or reuse them. Retire an ID by marking it **Withdrawn**.

## 1. Problem

People shoot thousands of photos but print almost none. Preparing a print order is tedious and lossy:

- Printed photos carry no date or place, so a few years later nobody remembers where or when a photo was taken.
- Print labs (CEWE, dm, Rossmann, Boots…) use fixed paper ratios (2:3, 3:4, 1:1…). Camera images get auto-cropped by the lab, which can cut off heads and horizons.
- Photos from a trip are scattered across days and devices, with no grouping.
- The tools that exist either upload your photos to a server (a privacy risk, since GPS reveals your home) or each solve only one of these problems.

## 2. Product vision

**Fotollection is a fast, private, offline desktop app (macOS and Windows) that turns a folder of photos into a print-ready set.** It stamps date and location from EXIF, crops to standard EU print sizes, groups photos by trip, and keeps the original quality. Photos never leave the user's machine.

### Market gap

Nobody combines EXIF date/location stamping, trip auto-organization, and batch cropping to EU print sizes in one app.

| Tool | Type | Date stamp | Trip grouping | CEWE ratios | Privacy |
|------|------|-----------|---------------|-------------|---------|
| BatchPhoto | Desktop (paid) | Yes | No | No | Local |
| Timestamp It | Web | Yes (3 free) | No | No | Server-side |
| BulkPicTools | Web | No | No | No | Client-side |
| RowPix | Web | No | No | No | Client-side |
| CropMaster | Web | No | No | No | Client-side |
| PhotoPrism | Self-hosted | Partial | GPS maps | No | Self-hosted |
| **Fotollection** | **Desktop (Rust)** | **Yes** | **Yes** | **Yes** | **100% local, offline** |

## 3. Users

| Persona | Description | Key need |
|---------|-------------|----------|
| **Enthusiast photographer** | Shoots a mirrorless camera (e.g. Lumix GX9) plus a phone. Folders of 24 MP JPEG and HEIC files. | Speed, no quality loss, colour fidelity |
| **Traveller / family archivist** | Comes home with 800 photos from a two-week trip and wants a printed album. | Automatic grouping, date and place on every print |
| **Small studio / event shooter** | Delivers prints of events in bulk. | Large-batch throughput, consistent crops, predictable output naming |

## 4. Core user journey

1. Open a folder, or drag and drop photos or folders into the app.
2. The library grid appears instantly. Photos are grouped by date (by trip in Phase 2).
3. Choose a print size (for example 10×15) and stamp style. A live preview updates as settings change.
4. Optionally adjust the crop frame for individual photos.
5. Export to a destination folder. A progress view shows the batch, and the user can cancel at any time.
6. Upload the exported folder to the print lab.

## 5. Print sizes (CEWE / EU standard)

| ID | Size (cm) | Size (inch) | Aspect ratio | Use case |
|----|-----------|-------------|--------------|----------|
| `9x13` | 9×13 | 3.5×5 | ~1:1.44 | Wallet/small |
| `10x15` | 10×15 | 4×6 | 2:3 | Standard |
| `11x17` | 11×17 | 4.5×6.5 | ~1:1.55 | Slightly larger |
| `13x18` | 13×18 | 5×7 | ~1:1.38 | Medium |
| `15x20` | 15×20 | 6×8 | 3:4 | Large |
| `20x30` | 20×30 | 8×12 | 2:3 | Extra large |
| `15x15` | 15×15 | 6×6 | 1:1 | Square |
| `5x15` | 5×15 | 2×6 | 1:3 | Photo strip |

Target labs: CEWE, Rossmann, dm, Boots Photo, Snapfish, FreePrints, Photobox.

## 6. Functional requirements

Priority: **P0** = MVP blocker · **P1** = MVP desirable · **P2** = later.
The phase assignment for each requirement lives in [ROADMAP.md](ROADMAP.md), and its progress in [STATUS.md](STATUS.md).

### FR-001 — Import photos

**P0.** Import photos from a folder picker (recursive), drag and drop of files and folders, or a file picker.

- Accepts JPEG, HEIC/HEIF and PNG, detected by magic bytes rather than file extension.
- Unsupported or invalid files are listed with a reason. They never abort the import.
- Importing does not modify source files. The app opens sources read-only.

### FR-002 — Read photo metadata

**P0.** Read the allowlisted EXIF fields: capture date and time (with offset), GPS position, camera make and model, orientation, dimensions and exposure fields.

- A photo without EXIF is still usable. The app falls back to the file modification time and shows a warning badge.
- Orientation is applied, so photos are never displayed or exported rotated incorrectly.

### FR-003 — Library grid

**P0.** Show a virtualised thumbnail grid of all imported photos, sorted by capture time and grouped by day.

- Shows a badge for missing date, missing GPS and processing errors.
- Supports multi-select (shift-click and cmd/ctrl-click) for per-photo overrides.

### FR-004 — Date stamp

**P0.** Render the capture date onto the photo.

- Configurable format (presets plus a custom pattern), font (bundled fonts), size relative to the short edge, colour, opacity, optional shadow or outline, and position (each of the 4 corners, with a margin).
- Rendered with anti-aliasing at output resolution and placed inside the final crop area, so the stamp is never cropped away.

### FR-005 — Location stamp (offline)

**P0.** Resolve GPS coordinates to a place name, for example "Prague, CZ", fully offline, and render it with the date stamp or on its own.

- Uses a bundled dataset. No network request is ever made.
- The user can edit the resolved name per day or per group, for example to change "Praha 1" to "Prague".

### FR-006 — Crop to print ratio

**P0.** Crop every photo to the selected print size ([§5](#5-print-sizes-cewe--eu-standard)).

- Portrait and landscape orientation is matched automatically.
- The default crop is centred. The user can drag or resize the crop frame per photo (with the ratio locked).
- Offers a "fit with border" (letterbox) alternative to cropping.

### FR-007 — Live preview

**P0.** Every change to the stamp or crop settings updates the preview immediately (see [NFR-002](#nfr-002--preview-latency)).

### FR-008 — Original-quality export

**P0.** Export at full resolution.

- JPEG quality is configurable, with a default of 95 and 4:4:4 chroma subsampling.
- The embedded ICC profile is preserved.
- Photos that are crop-only with no stamp use a lossless DCT-domain crop when possible.

### FR-009 — Export by date

**P0.** Write the export to a user-chosen folder using `YYYY-MM-DD/` subfolders and stable, sortable file names.

- Existing files are never overwritten silently. The user picks a rule: skip, rename, or overwrite after confirmation.

### FR-010 — Metadata privacy on export

**P0.** Control which metadata is written into exported files.

- GPS is stripped by default.
- Options: keep everything (minus GPS), keep the date only, or strip everything.
- Before export, a banner warns when photos with GPS are about to be exported with GPS kept.

### FR-011 — Batch progress, cancel and recovery

**P0.** Show export progress with a count and an estimated time remaining.

- The user can cancel instantly, and partially written files are removed.
- If one photo fails, it is skipped, the error is logged, and the batch continues. A summary report is shown at the end.

### FR-012 — Trip auto-grouping

**P1.** Cluster photos into trips by GPS and time proximity, and suggest a name for each trip, for example "2026-07 Lisbon".

- The user can merge, split and rename trips.

### FR-013 — Face-aware smart crop

**P1.** Detect faces on-device and position the default crop so that no face is cut. If faces cannot fit inside the crop, fall back to a centred crop.

### FR-014 — Export by trip

**P1.** Export into trip folders, for example `2026-07_Lisbon/`, as an alternative to [FR-009](#fr-009--export-by-date).

### FR-015 — Duplicate detection

**P1.** Flag near-duplicate photos, such as bursts or re-saves, using perceptual hashing, and let the user exclude them from the print set.

### FR-016 — RAW input

**P2.** Import camera RAW files (starting with Panasonic RW2), developing them with neutral defaults or using the embedded JPEG preview.

### FR-017 — Photo book layout preview

**P2.** Preview how the selection lays out on album pages.

### FR-018 — QR code on photo back

**P2.** Generate a printable QR sheet that links each print to its digital original.

### FR-019 — Print service upload

**P2.** Opt-in upload to print lab APIs. This is the **only** feature that may use the network, and it must be explicitly enabled by the user.

### FR-020 — Localization

**P2.** UI available in English, German and Vietnamese.

### FR-021 — Auto-update

**P2.** Signed in-app updates from GitHub Releases. Checking for updates is opt-in, consistent with [NFR-006](#nfr-006--offline-and-zero-telemetry).

## 7. Non-functional requirements

Performance budgets are measured as defined in [OPTIMIZATION.md](OPTIMIZATION.md). The reference machines are listed there.

### NFR-001 — Library load time

Opening a folder of 1,000 JPEGs shows the first screen of thumbnails in **under 1 s**. All thumbnails are ready in **under 10 s**.

### NFR-002 — Preview latency

A change to stamp or crop settings re-renders the visible preview in **under 50 ms** (p95).

### NFR-003 — Export throughput

Export (decode → crop → stamp → encode → write) runs at **8 or more 24 MP JPEGs per second** on an 8-core reference machine.

### NFR-004 — Memory ceiling

Peak resident memory during export stays **at or below 2 GB**, regardless of batch size.

### NFR-005 — UI responsiveness

Heavy work never runs on the UI thread. Input-to-feedback latency is **under 100 ms** during an export.

### NFR-006 — Offline and zero telemetry

The app works with no network connection. By default it makes **zero** network requests: no telemetry, no crash reporting and no update checks unless the user opts in.

### NFR-007 — Platforms

The app supports macOS 11 or later (Apple Silicon and Intel, shipped as a universal binary) and Windows 10/11 x64.

### NFR-008 — Install size

The installer is **at most 25 MB** per platform.

### NFR-009 — Colour fidelity

The output keeps the source ICC profile. A pixel outside the stamp area differs from a reference decode by at most 1 per channel, apart from JPEG re-encoding error at the configured quality.

### NFR-010 — Cold start

The app goes from launch to an interactive window in **under 1.5 s**.

### NFR-011 — Robustness

No input file, however malformed, can crash the app or hang the batch. A bad file only fails that one photo.

## 8. Out of scope

The following are out of scope (for now):

- Cloud sync, accounts, and any kind of server backend.
- General photo editing: exposure, colour grading, retouching.
- Mobile apps.
- Video.

## 9. Success metrics

- A 500-photo trip goes from folder to print-ready export in **under 5 minutes** of user time.
- Zero reports of photos with incorrect rotation or cut-off stamps in exported sets.
- All NFR budgets are green in CI benchmarks on every release.
