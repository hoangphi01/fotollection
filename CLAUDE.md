# CLAUDE.md — Rules for AI assistants

Fotollection is an offline desktop app (Tauri 2 + Rust core + React/shadcn UI) that prepares photos for printing. **Read [docs/README.md](docs/README.md) first**, because it maps every fact to its single source of truth.

## Non-negotiable rules

1. **Docs move with code.** Any change to behaviour, architecture, performance or security updates the matching doc **in the same commit** ([ADR-0006](docs/adr/0006-docs-as-code.md)):
   - Behaviour or scope → [docs/PRD.md](docs/PRD.md), and also [docs/ROADMAP.md](docs/ROADMAP.md) if the phase changes
   - Structure, stack or IPC → [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
   - Performance → a new row in [docs/OPTIMIZATION.md §5](docs/OPTIMIZATION.md#5-benchmark-log)
   - Progress → [docs/STATUS.md](docs/STATUS.md)
   - Significant decision → a new ADR in [docs/adr/](docs/adr/README.md)
   - User-visible change → [CHANGELOG.md](CHANGELOG.md) under `Unreleased`
2. **Requirement IDs are permanent.** Never renumber or reuse `FR-`, `NFR-` or `ADR-` IDs. Add new ones at the end.
3. **Performance first, measured.** No optimization without a benchmark. Respect the budgets in [docs/OPTIMIZATION.md §3](docs/OPTIMIZATION.md#3-budgets).
4. **Privacy.** No network calls in the core, no telemetry, GPS stripped on export by default.
5. **Security.** Never render metadata as HTML (`dangerouslySetInnerHTML` and `innerHTML` are banned). Validate all IPC input in Rust. Follow [docs/SECURITY.md](docs/SECURITY.md).
6. **Style.** Follow [docs/STYLE_RULE.md](docs/STYLE_RULE.md): no `unwrap` in non-test code, `thiserror` in the core, shadcn/ui components, Lucide icons only.

## Before finishing a task

- Run `scripts/check-docs.sh`. It must pass.
- Once code exists, also run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and the UI lint and tests.
- Commit messages follow Conventional Commits, with the requirement ID: `feat(stamp): … [FR-004]`.

## Repository facts

- Remote: `github.com/hoangphi01/fotollection` (public), default branch `main`.
- Docs are written in English. The project owner often discusses in Vietnamese.
