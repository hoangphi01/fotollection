# Roadmap — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> This file defines **when**: it assigns requirements to phases. What each requirement means is defined in [PRD.md](PRD.md). Progress is tracked in [STATUS.md](STATUS.md).
> Every FR in the PRD appears in exactly one phase below. `scripts/check-docs.sh` enforces this.

## Phase 0 — Foundation

**Goal:** a running empty app on macOS and Windows, a benchmark harness, and crate choices confirmed by data.

- Cargo workspace (`crates/core`, `src-tauri`, `ui/`) and Tauri 2 scaffold
- UI scaffold: Vite + React + TS, Tailwind, shadcn/ui init (theme tokens, base components), `lucide-react`
- CI: fmt, clippy, tests and docs checks on the macOS and Windows matrix
- `tauri-specta` binding generation, with a staleness check in CI
- Benchmark harness (`criterion`) and the `bench` corpus ([TESTING.md §3](TESTING.md#3-sample-corpus))
- Spikes, each logged in [OPTIMIZATION.md §5](OPTIMIZATION.md#5-benchmark-log): turbojpeg vs `zune-jpeg` decode, `nom-exif` vs `kamadak-exif`, libheif build on Windows
- Release workflow producing unsigned installers ([RELEASE.md](RELEASE.md))

**Exit criteria:** the empty app builds and launches on both platforms from CI artifacts, and baseline benchmarks are recorded.

## Phase 1 — MVP

**Goal:** go from a folder to a culled, correctly dated, colour-correct, print-ready export, fast and private.

| ID | Requirement |
|----|-------------|
| FR-001 | Import photos |
| FR-002 | Read photo metadata |
| FR-003 | Library grid |
| FR-004 | Date stamp |
| FR-005 | Location stamp (offline) |
| FR-006 | Crop to print ratio |
| FR-007 | Live preview |
| FR-008 | Original-quality export |
| FR-009 | Export by date |
| FR-010 | Metadata privacy on export |
| FR-011 | Batch progress, cancel and recovery |
| FR-022 | Culling (rate, flag, filter) |
| FR-023 | Projects, autosave and undo |
| FR-025 | Mixed print sizes per order |
| FR-026 | Resolution warning |
| FR-027 | sRGB conversion for labs |
| FR-028 | Safe area and borders |
| FR-030 | Camera clock offset |
| FR-032 | Manual date and place |
| FR-033 | Retro film date stamp and stamp presets |
| FR-040 | EXIF info frame |
| FR-041 | Best-shot suggestions |
| FR-042 | Custom fonts |
| FR-043 | Custom print sizes |
| FR-045 | Original aspect ratio |
| FR-020 | Localization (English and Vietnamese; German later) |

**Exit criteria:** all P0 FRs are Done; NFR-001 through NFR-013 are within budget on `REF-MAC` and `REF-WIN`; v0.1.0 is released.

### Phase 1 work order (2026-10-23)

The open Phase 1 work is split into three stages, from easiest to hardest. Each stage ships on its own: tests and docs in the same commits, and CI green on macOS and Windows. The stages order work *inside* Phase 1; they are not new phases.

**Stage 1 — Small, low-risk**

| Work | Requirement | Why it is easy |
|------|-------------|----------------|
| `docs/EXTENSION.md`: registry of every Fotollection file format (`.fotollection`, `.fotopreset`, `settings.json`, fonts, logos, cache), with MIME/UTI, versioning rules and limits | FR-023, FR-038 | Documents formats that already exist |
| PNG input: decode with the `png` crate (already a dependency for logos), 8- and 16-bit to RGB8, alpha flattened on white | FR-001 | Pure Rust, no new dependency |
| Basic culling: stars `0`–`5`, flags `P`/`X`/`U`, filter Picks / Rejects / Unflagged; stored in the existing `PhotoEdits` | FR-022 | Data model and project file already hold rating and flag |
| Profile the preview path (resize and encode), and try 8 exports in flight instead of 6 | NFR-002, NFR-003 | Measurement only ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)) |

**Stage 2 — Medium, a new file format**

| Work | Requirement | Main risk |
|------|-------------|-----------|
| `.fotollection` as a self-contained ZIP bundle, like `.docx`: `manifest.json` with checksums, `project.json`, `settings.json`, embedded `fonts/`, `logos/` and `presets/`. Photos are not embedded | FR-023 | New schema version, migration from v1 and a fixture; an ADR superseding [ADR-0010](adr/0010-project-file-format.md) |
| Save as / Open, installing embedded fonts and logos after validation, plus a **Relink folder** dialog when photos moved | FR-023 | Untrusted input: zip-bomb limits, path-traversal checks, typed parsing of every part ([SECURITY.md](SECURITY.md)) |
| App localization: English (default) and Vietnamese through `i18next` + `react-i18next` (MIT), bundled locale files and typed keys; language stored in `settings.json` v3; Rust errors translated by `code`. An ADR records the choice | FR-020 | About 110 strings in 18 components; `settings.json` v3 needs a migration and a fixture |
| Faster previews: Stage 1 profiling shows JPEG encode is 62 % of the warm preview ([OPTIMIZATION.md §2.1](OPTIMIZATION.md#21-measured-the-same-job-in-chromium)). First plain Huffman tables and 4:2:0 for previews, then a `turbojpeg` codec behind `JpegCodec` if still needed; target ≤ 17 ms warm | NFR-002 | Native dependency on both platforms if `turbojpeg` is needed ([ADR-0012](adr/0012-pure-rust-codecs-first.md)) |

**Stage 3 — Hard, research and native code**

| Work | Requirement | Main risk |
|------|-------------|-----------|
| Best-shot suggestions: grouping, sharpness/shake/exposure/noise metrics, face and eye models, explained "Best of N", Keep best | FR-041, NFR-014 | Model licences must be permissive (checked first, recorded in an ADR); ML runtime (`tract-onnx`); within-group scoring needs tuning on real bursts |
| HEIC/HEIF input (iPhone) | FR-001 | Codec licence: libheif and libde265 are LGPL, so [ADR-0005](adr/0005-input-formats-jpeg-heic-png.md) must be re-checked against the permissive-only rule ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)). Alternatives are the OS codecs (ImageIO on macOS, WIC on Windows). Static Windows build |

## Phase 2 — Smart features

**Goal:** the app organizes photos and avoids bad crops automatically.

| ID | Requirement |
|----|-------------|
| FR-012 | Trip auto-grouping |
| FR-013 | Face-aware smart crop |
| FR-014 | Export by trip |
| FR-015 | Duplicate detection |
| FR-024 | Copies and order summary |
| FR-031 | Borrow GPS from a phone or GPX track |
| FR-034 | Lab presets |
| FR-036 | Custom captions |
| FR-038 | Print presets |
| FR-039 | Photo Style looks |
| FR-044 | Per-photo overrides |

## Phase 3 — Pro & growth

| ID | Requirement |
|----|-------------|
| FR-016 | RAW input |
| FR-017 | Photo book layout preview |
| FR-018 | QR code on photo back |
| FR-019 | Print service upload |
| FR-021 | Auto-update |
| FR-029 | Output sharpening |
| FR-035 | Multi-photo layouts |
| FR-037 | Contact sheet |

## Ideas backlog (not yet requirements)

These ideas have no ID yet. Promote one to a PRD requirement before scheduling it.

- XMP sidecar export of ratings and flags (sources stay read-only)
- Lossless rotation fix-up tool
- Linux build
