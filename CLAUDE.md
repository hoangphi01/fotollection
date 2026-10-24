# CLAUDE.md — Rules for AI assistants

Fotollection is an offline desktop app (Tauri 2 + Rust core + React/shadcn UI) that prepares photos for printing. **Read [docs/README.md](docs/README.md) first**, because it maps every fact to its single source of truth.

## Non-negotiable rules

1. **Docs move with code.** Any change to behaviour, architecture, performance, security or the website updates the matching docs **in the same commit** ([ADR-0006](docs/adr/0006-docs-as-code.md)). **[docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) is the checklist:** find the rows for your change and do all of them (CHANGELOG, PRD/ROADMAP/STATUS, ARCHITECTURE, DATABASE + schema bump, README.vi.md, the website and its screenshots…).
2. **Requirement IDs are permanent.** Never renumber or reuse `FR-`, `NFR-` or `ADR-` IDs. Add new ones at the end.
3. **Performance first, measured.** No optimization without a benchmark. Respect the budgets in [docs/OPTIMIZATION.md §3](docs/OPTIMIZATION.md#3-budgets).
4. **Privacy.** No network calls in the core, no telemetry, GPS stripped on export by default.
5. **Security.** Never render metadata as HTML (`dangerouslySetInnerHTML` and `innerHTML` are banned). Validate all IPC input in Rust. Follow [docs/SECURITY.md](docs/SECURITY.md).
6. **License (GPL-3.0-only, relicensable).** Add only permissive-licensed dependencies (MIT, Apache-2.0, BSD…). **Never add GPL or AGPL dependencies.** New source files start with `// SPDX-License-Identifier: GPL-3.0-only` ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)).
7. **Style.** Follow [docs/STYLE_RULE.md](docs/STYLE_RULE.md): no `unwrap` in non-test code, `thiserror` in the core, shadcn/ui components, Lucide icons only.

## Before finishing a task

- Walk through [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) §2 for every kind of change you made, then run its §3 checks.
- Run `scripts/check-docs.sh`. It must pass.
- Once code exists, also run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and the UI lint and tests.
- Commit messages follow Conventional Commits, with the requirement ID: `feat(stamp): … [FR-004]`.

## Repository facts

- Remote: `github.com/hoangphi01/fotollection` (public), default branch `main`.
- Docs are written in English, except `README.vi.md`. The project owner often discusses in Vietnamese.
