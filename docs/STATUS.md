# Status — Fotollection

> **Last updated:** 2026-10-23
>
> Living progress tracker. Update it **in the same commit** as the work ([ADR-0006](adr/0006-docs-as-code.md)).
> Statuses: `Not Started` · `In Progress` · `Blocked` · `Done` · `Withdrawn`. An FR is `Done` only when its `fr_NNN_*` test passes ([TESTING.md §2](TESTING.md#2-requirement-traceability)).

## Current phase: Phase 0 — Foundation (in progress)

The app scaffold runs on macOS: a Tauri 2 window, the React/shadcn UI shell, and typed IPC. Image processing has not started yet.

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
| Benchmark harness + `bench` corpus | Not Started | |
| Codec / EXIF crate spikes | Not Started | Log results in OPTIMIZATION.md §5 |
| libheif static build on Windows | Not Started | Highest build risk |
| Release workflow (unsigned) | Not Started | |
| Benchmark budget gate (`bench/budgets.toml`) | Not Started | Needs harness |
| Enforce "Done FR has test" in `check-docs.sh` | Not Started | Needs code |

## Functional requirements

| ID | Requirement | Phase | Status | Notes |
|----|-------------|-------|--------|-------|
| FR-001 | Import photos | 1 | Not Started | |
| FR-002 | Read photo metadata | 1 | Not Started | |
| FR-003 | Library grid | 1 | Not Started | |
| FR-004 | Date stamp | 1 | Not Started | |
| FR-005 | Location stamp (offline) | 1 | Not Started | GeoNames packer needed |
| FR-006 | Crop to print ratio | 1 | Not Started | |
| FR-007 | Live preview | 1 | Not Started | |
| FR-008 | Original-quality export | 1 | Not Started | |
| FR-009 | Export by date | 1 | Not Started | |
| FR-010 | Metadata privacy on export | 1 | Not Started | |
| FR-011 | Batch progress, cancel and recovery | 1 | Not Started | |
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
| FR-023 | Projects, autosave and undo | 1 | Not Started | Format: [ADR-0010](adr/0010-project-file-format.md) |
| FR-024 | Copies and order summary | 2 | Not Started | |
| FR-025 | Mixed print sizes per order | 1 | Not Started | Builds on `PrintSize` |
| FR-026 | Resolution warning | 1 | Not Started | |
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
| FR-039 | Photo Style looks | 2 | Not Started | [ADR-0011](adr/0011-photo-style-looks.md); "C200 Natural" built-in |

## Non-functional requirements

| ID | Requirement | Budget | Last measured | Status |
|----|-------------|--------|---------------|--------|
| NFR-001 | Library load time | < 1 s first screen | — | Not Started |
| NFR-002 | Preview latency | < 50 ms p95 | — | Not Started |
| NFR-003 | Export throughput | ≥ 8 photos/s | — | Not Started |
| NFR-004 | Memory ceiling | ≤ 2 GB | — | Not Started |
| NFR-005 | UI responsiveness | < 100 ms | — | Not Started |
| NFR-006 | Offline and zero telemetry | 0 requests | — | Not Started |
| NFR-007 | Platforms | macOS 11+, Win 10/11 | — | Not Started |
| NFR-008 | Install size | ≤ 25 MB | — | Not Started |
| NFR-009 | Colour fidelity | ICC kept, Δ ≤ 1 | — | Not Started |
| NFR-010 | Cold start | < 1.5 s | — | Not Started |
| NFR-011 | Robustness | 0 crashes | — | Not Started |
| NFR-012 | Culling navigation | < 100 ms p95 | — | Not Started |
| NFR-013 | Project safety | ≤ 5 s, atomic | — | Not Started |

## Security tasks

Defined in [SECURITY.md](SECURITY.md).

| # | Task | Priority | Status |
|---|------|----------|--------|
| S1 | 4-layer input/metadata pipeline (`crates/core/src/import.rs`, `metadata.rs`) | P0 | Not Started |
| S2 | Tauri capabilities + CSP (`src-tauri/capabilities/`, `tauri.conf.json`) | P0 | In Progress |
| S3 | Export path sanitizer + root check (`crates/core/src/export.rs`) | P0 | Not Started |
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
