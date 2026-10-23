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
| FR-042 | Custom fonts |
| FR-043 | Custom print sizes |

**Exit criteria:** all P0 FRs are Done; NFR-001 through NFR-013 are within budget on `REF-MAC` and `REF-WIN`; v0.1.0 is released.

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

## Phase 3 — Pro & growth

| ID | Requirement |
|----|-------------|
| FR-016 | RAW input |
| FR-017 | Photo book layout preview |
| FR-018 | QR code on photo back |
| FR-019 | Print service upload |
| FR-020 | Localization |
| FR-021 | Auto-update |
| FR-029 | Output sharpening |
| FR-035 | Multi-photo layouts |
| FR-037 | Contact sheet |

## Ideas backlog (not yet requirements)

These ideas have no ID yet. Promote one to a PRD requirement before scheduling it.

- XMP sidecar export of ratings and flags (sources stay read-only)
- Lossless rotation fix-up tool
- Linux build
