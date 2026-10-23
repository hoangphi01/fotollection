# Status — Fotollection

> **Last updated:** 2026-10-23
>
> Living progress tracker. Update it **in the same commit** as the work ([ADR-0006](adr/0006-docs-as-code.md)).
> Statuses: `Not Started` · `In Progress` · `Blocked` · `Done` · `Withdrawn`. An FR is `Done` only when its `fr_NNN_*` test passes ([TESTING.md §2](TESTING.md#2-requirement-traceability)).

## Current phase: Phase 1 — MVP, first vertical slice done

The app works end to end for **JPEG**: open or drop a folder → thumbnail grid → preview with crop and date stamp → export to `YYYY-MM-DD/` folders with progress and cancel. Next slices: HEIC/PNG, offline place names (FR-005), per-photo crop, culling and projects (FR-022/023), sRGB (FR-027).

## Phase 0 — Foundation

| Task | Status | Notes |
|------|--------|-------|
| Git repository | Done | `hoangphi01/fotollection` (public), branch `main` |
| Documentation framework + docs CI | Done | This commit set |
| Cargo workspace + Tauri 2 scaffold | Done | Tauri 2.12, Rust 1.99.1 pinned |
| UI scaffold: React + Tailwind + shadcn/ui + Lucide | Done | [ADR-0007](adr/0007-ui-react-shadcn-lucide.md) |
| Build-size profiles in `Cargo.toml` + `scripts/clean.sh` | Done | First dev build: `target/` ≈ 1.0 GB ([DEVELOPMENT.md §4](DEVELOPMENT.md#4-build-disk-hygiene)) |
| CI build/test matrix (macOS, Windows) | Done | `.github/workflows/ci.yml` |
| Reproducible setup (`scripts/bootstrap.sh` / `.ps1`, lockfiles) | Done | [DEVELOPMENT.md §1](DEVELOPMENT.md#1-setting-up-a-machine) |
| `tauri-specta` bindings + staleness check | Done | `cargo test` regenerates; CI runs `git diff --exit-code` |
| Benchmark harness + `bench` corpus | In Progress | `examples/measure_folder.rs` on a real folder, plus `bench/run.sh`, the Rust vs Chromium comparison (OPTIMIZATION §2.1). `criterion` benches pending |
| Codec / EXIF crate spikes | In Progress | Pure-Rust codecs adopted for now ([ADR-0012](adr/0012-pure-rust-codecs-first.md)). turbojpeg comparison pending |
| libheif static build on Windows | Not Started | Highest build risk |
| Release workflow (unsigned) | Not Started | |
| Benchmark budget gate (`bench/budgets.toml`) | Not Started | Needs harness |
| Enforce "Done FR has test" in `check-docs.sh` | Not Started | Needs code |

## Functional requirements

| ID | Requirement | Phase | Status | Notes |
|----|-------------|-------|--------|-------|
| FR-001 | Import photos | 1 | In Progress | JPEG import (folder dialog + drag-and-drop), magic bytes, size + polyglot checks. HEIC/PNG pending |
| FR-002 | Read photo metadata | 1 | In Progress | Allowlisted EXIF (date, offset, GPS, orientation, camera) + validation. `fr_002_*` tests pass |
| FR-003 | Library grid | 1 | In Progress | Virtualized grid, T0/T1 thumbnails, skeleton + fade-in, task progress bar with %. Day grouping and multi-select pending |
| FR-004 | Date stamp | 1 | In Progress | 3 formats, 4 corners, white/retro-orange, inside the safe area. More styles with FR-033 |
| FR-005 | Location stamp (offline) | 1 | Not Started | GeoNames packer needed |
| FR-006 | Crop to print ratio | 1 | In Progress | Crop editor with edge labels (cm/in + px), ratio pill and live DPI; centre + zoom storage |
| FR-007 | Live preview | 1 | In Progress | Preview dialog with inspector (capture, camera, image, GPS, print DPI, stamp controls). Rust-rendered, p95 26 ms warm |
| FR-008 | Original-quality export | 1 | In Progress | Full resolution, q95 4:4:4, ICC copied. Lossless crop-only path pending |
| FR-009 | Export by date | 1 | Done | `YYYY-MM-DD/` folders, never overwrites (`_1`, `_2`…). `fr_009_*` passes |
| FR-010 | Metadata privacy on export | 1 | In Progress | No EXIF written, so GPS is always stripped. Keep-metadata options pending |
| FR-011 | Batch progress, cancel and recovery | 1 | Done | Progress events, cancel removes partials, per-photo failures. `fr_011_*` pass |
| FR-012 | Trip auto-grouping | 2 | Not Started | |
| FR-013 | Face-aware smart crop | 2 | Not Started | |
| FR-014 | Export by trip | 2 | Not Started | |
| FR-015 | Duplicate detection | 2 | Not Started | |
| FR-016 | RAW input | 3 | Not Started | |
| FR-017 | Photo book layout preview | 3 | Not Started | |
| FR-018 | QR code on photo back | 3 | Not Started | |
| FR-019 | Print service upload | 3 | Not Started | |
| FR-020 | Localization | 3 | Not Started | |
| FR-021 | Auto-update | 3 | Not Started | |
| FR-022 | Culling (rate, flag, filter) | 1 | Not Started | |
| FR-023 | Projects, autosave and undo | 1 | In Progress | Session project autosaves atomically and is restored at launch (re-scan + re-apply); undo/redo for crops. Save as/Open pending |
| FR-024 | Copies and order summary | 2 | Not Started | |
| FR-025 | Mixed print sizes per order | 1 | Not Started | Builds on `PrintSize` |
| FR-026 | Resolution warning | 1 | In Progress | Effective DPI + band shown in the inspector. Grid badge and pre-export summary pending |
| FR-027 | sRGB conversion for labs | 1 | Not Started | `lcms2` |
| FR-028 | Safe area and borders | 1 | Not Started | |
| FR-029 | Output sharpening | 3 | Not Started | Needs real print tests |
| FR-030 | Camera clock offset | 1 | Not Started | |
| FR-031 | Borrow GPS from a phone or GPX track | 2 | Not Started | Depends on FR-030 |
| FR-032 | Manual date and place | 1 | Not Started | |
| FR-033 | Retro film date stamp and stamp presets | 1 | Not Started | Font must be OFL/MIT |
| FR-034 | Lab presets | 2 | Not Started | |
| FR-035 | Multi-photo layouts | 3 | Not Started | Shares layout engine with FR-017 |
| FR-036 | Custom captions | 2 | Not Started | `{trip}` needs FR-012 |
| FR-037 | Contact sheet | 3 | Not Started | |
| FR-038 | Print presets | 2 | Not Started | Storage model: [DATABASE.md §7](DATABASE.md#7-presets) |
| FR-040 | EXIF info frame | 1 | In Progress | Crop/Fit at print ratio, field switches, per-camera overrides (persisted), user PNG logo. Place needs FR-005 |
| FR-041 | Best-shot suggestions | 1 | Not Started | Specified. Next slice: license check of face/eye models (ADR) first, then `culling/` module |
| FR-042 | Custom fonts | 1 | Done | OTF/TTF import (Rust dialog, validated, copied), per stamp and per frame, remove → Inter. `fr_042_*` pass |
| FR-043 | Custom print sizes | 1 | Done | cm/in entry, stored in mm, used by crop/frame/DPI/export. `fr_043_*` pass |
| FR-044 | Per-photo overrides | 2 | Not Started | Future: e.g. untick the stamp for photos that already have one; batch first |
| FR-039 | Photo Style looks | 2 | Not Started | [ADR-0011](adr/0011-photo-style-looks.md); "C200 Natural" built-in |

## Non-functional requirements

| ID | Requirement | Budget | Last measured | Status |
|----|-------------|--------|---------------|--------|
| NFR-001 | Library load time | < 1 s first screen | 95 photos: import+EXIF 28 ms, T0 7 ms | In Progress |
| NFR-002 | Preview latency | < 50 ms p95 | 26 ms p95 (warm) | In Progress |
| NFR-003 | Export throughput | ≥ 8 photos/s | 19.7/s (16 MP) | In Progress |
| NFR-004 | Memory ceiling | ≤ 2 GB | 1.41 GB | In Progress |
| NFR-005 | UI responsiveness | < 100 ms | — | Not Started |
| NFR-006 | Offline and zero telemetry | 0 requests | — | Not Started |
| NFR-007 | Platforms | macOS 11+, Win 10/11 | — | Not Started |
| NFR-008 | Install size | ≤ 25 MB | — | Not Started |
| NFR-009 | Colour fidelity | ICC kept, Δ ≤ 1 | — | Not Started |
| NFR-010 | Cold start | < 1.5 s | — | Not Started |
| NFR-011 | Robustness | 0 crashes | — | Not Started |
| NFR-012 | Culling navigation | < 100 ms p95 | — | Not Started |
| NFR-013 | Project safety | ≤ 5 s, atomic | autosave ≤ ~1 s, atomic + .bak | In Progress |
| NFR-014 | Best-shot analysis speed | ≤ 20 ms/photo (≤ 40 ms with faces) | — | Not Started |

## Security tasks

Defined in [SECURITY.md](SECURITY.md).

| # | Task | Priority | Status |
|---|------|----------|--------|
| S1 | 4-layer input/metadata pipeline (`crates/core/src/import.rs`, `metadata.rs`) | P0 | In Progress |
| S2 | Tauri capabilities + CSP (`src-tauri/capabilities/`, `tauri.conf.json`) | P0 | Done |
| S3 | Export path sanitizer + root check (`crates/core/src/export.rs`) | P0 | In Progress |
| S4 | `cargo deny` / `cargo audit` / `npm audit` in CI | P1 | Done |
| S5 | Fuzz targets (`crates/core/fuzz/`) | P1 | Not Started |
| S6 | DOM-API grep gate + network-free core check | P1 | Done |
| S7 | Panic hook + user-facing error mapping | P2 | Not Started |

## Open decisions

| Decision | Options | Owner | Notes |
|----------|---------|-------|-------|
| `REF-WIN` reference machine | — | @hoangphi01 | Needed for NFR measurement |

Decided:

- Project license: **GPL-3.0-only**, with an option to relicense later ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)).
- Code signing: **unsigned** for now ([ADR-0009](adr/0009-unsigned-distribution.md)).

## Licensing tasks

| Task | Status | Notes |
|------|--------|-------|
| `LICENSE` (GPL-3.0) | Done | |
| CLA set up (e.g. CLA Assistant) | Not Started | **Required before merging the first outside PR** ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)) |
| `cargo deny` license allowlist (permissive only, GPL banned) | Done | `deny.toml` |
| SPDX headers in source files | Done | All hand-written `.rs` / `.ts(x)` files |
| Apply to SignPath Foundation (Windows signing) | Not Started | After the first public releases |

## Dependencies to watch

Native libraries pinned for security tracking (versions to be filled in at scaffold time):

| Library | Pinned version | Notes |
|---------|----------------|-------|
| libjpeg-turbo | — | via `turbojpeg` crate |
| libheif | — | via `libheif-rs` |
| libde265 | — | HEVC decoder used by libheif |
| lcms2 | — | |

## Blockers

None.
