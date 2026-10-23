# Status — Fotollection

> **Last updated:** 2026-10-23
>
> Living progress tracker. Update it **in the same commit** as the work ([ADR-0006](adr/0006-docs-as-code.md)).
> Statuses: `Not Started` · `In Progress` · `Blocked` · `Done` · `Withdrawn`. An FR is `Done` only when its `fr_NNN_*` test passes ([TESTING.md §2](TESTING.md#2-requirement-traceability)).

## Current phase: Phase 0 — Foundation (not started)

The documentation framework is complete. No code has been written yet.

## Phase 0 — Foundation

| Task | Status | Notes |
|------|--------|-------|
| Git repository | Done | `hoangphi01/fotollection` (public), branch `main` |
| Documentation framework + docs CI | Done | This commit set |
| Cargo workspace + Tauri 2 scaffold | Not Started | |
| UI scaffold: React + Tailwind + shadcn/ui + Lucide | Not Started | [ADR-0007](adr/0007-ui-react-shadcn-lucide.md) |
| Build-size profiles in `Cargo.toml` + `scripts/clean.sh` | In Progress | Script done; profiles set at scaffold ([DEVELOPMENT.md §4](DEVELOPMENT.md#4-build-disk-hygiene)) |
| CI build/test matrix (macOS, Windows) | Not Started | |
| `tauri-specta` bindings + staleness check | Not Started | |
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

## Security tasks

Defined in [SECURITY.md](SECURITY.md).

| # | Task | Priority | Status |
|---|------|----------|--------|
| S1 | 4-layer input/metadata pipeline (`crates/core/src/import.rs`, `metadata.rs`) | P0 | Not Started |
| S2 | Tauri capabilities + CSP (`src-tauri/capabilities/`, `tauri.conf.json`) | P0 | Not Started |
| S3 | Export path sanitizer + root check (`crates/core/src/export.rs`) | P0 | Not Started |
| S4 | `cargo deny` / `cargo audit` / `npm audit` in CI | P1 | Not Started |
| S5 | Fuzz targets (`crates/core/fuzz/`) | P1 | Not Started |
| S6 | DOM-API grep gate + network-free core check | P1 | Not Started |
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
| `cargo deny` license allowlist (permissive only, GPL banned) | Not Started | At scaffold time |
| SPDX headers in source files | Not Started | At scaffold time |
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
