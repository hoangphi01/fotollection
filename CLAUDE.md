# CLAUDE.md — Rules for AI assistants

Fotollection is an offline desktop app (Tauri 2 + Rust core + React/shadcn UI) that prepares photos for printing. **Read [docs/README.md](docs/README.md) first**, because it maps every fact to its single source of truth.

## Non-negotiable rules

1. **Docs move with code.** Any change to behaviour, architecture, performance, security or the website updates the matching docs **in the same commit** ([ADR-0006](docs/adr/0006-docs-as-code.md)). **[docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) is the checklist:** find the rows for your change and do all of them (CHANGELOG, PRD/ROADMAP/STATUS, ARCHITECTURE, DATABASE + schema bump, README.vi.md, the website and its screenshots…).
2. **IDs are permanent.** Never renumber or reuse `FR-`, `NFR-`, `ADR-` or `ERR-` IDs. Add new ones at the end. Every error the app can show has its `ERR-<KIND>-<NNNN>` row in [docs/ERROR_CONTROL.md](docs/ERROR_CONTROL.md), the only debugging clue in an offline app.
3. **Performance first, measured.** No optimization without a benchmark. Respect the budgets in [docs/OPTIMIZATION.md §3](docs/OPTIMIZATION.md#3-budgets).
4. **Privacy (owner, 2026-11-11).** Privacy means never collecting a person's data: no name, address, nationality, photos, contacts or anything that identifies someone. Essential, anonymous services are not a breach and are allowed where they make something work: on the website, reading the visitor's country from the IP to serve the right language, anonymous visit counts. In the app: GPS stripped on export by default, the photos never leave the computer, and the core makes no network call beyond what a feature needs (the update check).
5. **Security.** Never render metadata as HTML (`dangerouslySetInnerHTML` and `innerHTML` are banned). Validate all IPC input in Rust. Follow [docs/SECURITY.md](docs/SECURITY.md).
6. **License (GPL-3.0-only, relicensable).** Add only permissive-licensed dependencies (MIT, Apache-2.0, BSD…). **Never add GPL or AGPL dependencies.** New source files start with `// SPDX-License-Identifier: GPL-3.0-only` ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)).
7. **Style.** Follow [docs/STYLE_RULE.md](docs/STYLE_RULE.md): no `unwrap` in non-test code, `thiserror` in the core, shadcn/ui components, Lucide icons only.
8. **Rust first ([ADR-0044](docs/adr/0044-rust-first-our-edge.md)), the compass for every design and doc.** Our edge over web-runtime apps and over RawTherapee and darktable is the Rust core: pixel and file work in Rust, never in the webview; rayon by default on pools no waiting caller holds; no `unsafe`, bounded and fuzzed parsers; small and lean; typed IPC; every claim measured.
9. **Stay on the engineering goal (owner, 2026-11-10, hard rule).** We are engineers: performance and speed, UI/UX, SEO and running the project. Advise and build as a user and a developer would, never as a lawyer: no legal opinions, disclaimers or digressions that pull away from the main goal, and no notices or links that clutter Fotollection.

## How to work (owner, 2026-11-15: the project grows, so the work must be more scientific)

1. **Think before coding.** State your assumptions; if a request reads two ways, name both and ask, never pick silently; if a simpler way exists, say so; if something is unclear, stop and ask.
2. **Simplicity first.** The least code that does what was asked: no feature, option or abstraction nobody asked for, no handling of cases that cannot happen. If 200 lines could be 50, write 50.
3. **Surgical changes.** Every changed line traces to the request. Do not restyle, reformat or refactor what is not broken; a rule for one block names that block, never a shared class (the card icons that moved site-wide, 2026-11-14). Remove only what your change left unused; mention other dead code, do not delete it.
4. **Goal-driven.** Turn the task into checks that prove it: a test that fails first, a measure, a screenshot. For several steps, say the plan as `step → verify: check` before starting, and loop until every check passes.

## Before starting a task

- Run the `pre-work` skill (`.claude/skills/pre-work/`): read [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) in full and [docs/LESSONS.md](docs/LESSONS.md)'s Rules table in full, then every lesson of the task's areas, then STATUS "Next, in order" (owner, 2026-11-05; narrowed in the process review of 2026-11-12).

## Before finishing a task

- Walk through [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) §2 for every kind of change you made, then run its §3 checks.
- Check resources before, during and after every build and test (`scripts/resources.sh`, [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) §3): free disk, memory pressure, swap.
- After a debugging session, add what it taught to [docs/LESSONS.md](docs/LESSONS.md) in the fix's commit; read it before debugging.
- A UI change (component, card, panel, dialog, hint) follows the `style-audit` skill (`.claude/skills/style-audit/SKILL.md`): lego blocks, `check-lego`, the measure table, both themes and languages.
- Run the checks of [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) §3 (`scripts/check-docs.sh` on its own, never piped). The pre-push hook (`scripts/hooks/pre-push`, [CONTRIBUTING.md](CONTRIBUTING.md) §4) runs them again, and the UI cases when the interface changed.
- One thing at a time, and a feature that leaves the app is tried for real once before it is done ([CONTRIBUTING.md](CONTRIBUTING.md) §10).
- Commit messages follow Conventional Commits, with the requirement ID: `feat(stamp): … [FR-004]`.

## Repository facts

- Remote: `github.com/hoangphi01/fotollection` (public), default branch `main`.
- Docs are written in English, except `README.vi.md` and `docs/USER_GUIDE.vi.md` (the Vietnamese manual, [ADR-0031](docs/adr/0031-manual-on-the-website-from-the-user-guide.md)). The project owner often discusses in Vietnamese.
