# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Phase 0 scaffold: a Cargo workspace (`fotollection-core` + the Tauri 2 shell), and a React + Tailwind + shadcn/ui + Lucide UI shell (library sidebar, empty state, print-settings panel) with typed IPC through `tauri-specta`.
- Retro app logo and generated platform icons.
- CI: build, lint and test on macOS and Windows; stale-bindings check; `cargo deny`, `npm audit` and DOM-API security gates.
- Reproducible machine setup: `scripts/bootstrap.sh` / `bootstrap.ps1`, pinned Rust toolchain and Node version, committed lockfiles.
- Documentation framework: PRD, architecture, optimization, style rules, security, testing, release, roadmap, status, glossary, and ADRs 0001–0007.
- `LICENSE`: GPL-3.0-only ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)). Distribution policy: unsigned builds with checksums ([ADR-0009](docs/adr/0009-unsigned-distribution.md)).
- `docs/DEVELOPMENT.md` and `scripts/clean.sh`, a tiered cleanup of build artifacts (sweep, release, full, deep).
- Docs CI: markdownlint, link check, and the requirement-ID consistency check (`scripts/check-docs.sh`).

### Changed

- Project renamed from PhotoOrder to **Fotollection**.
- Direction changed from a browser web app to an offline **Tauri 2 desktop app** with a Rust core ([ADR-0001](docs/adr/0001-desktop-tauri-over-web.md)).
- UI stack changed to React, shadcn/ui and Lucide ([ADR-0007](docs/adr/0007-ui-react-shadcn-lucide.md)).

### Removed

- `PLAN.md`. Its content was split into `docs/PRD.md`, `docs/ARCHITECTURE.md` and `docs/ROADMAP.md`.
