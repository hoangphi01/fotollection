# Contributing to Fotollection

> **Last reviewed:** 2026-11-04
>
> How to work on this repository without making a mess of it, for people and for AI agents alike. It explains the rules; the facts themselves live in the files it links to. Read it once before your first issue or pull request, and come back to §4 before every commit.

Fotollection is an offline desktop app (Tauri 2, a Rust core, a React and shadcn/ui interface) that prepares photos for printing. It has one maintainer, a lot of documentation and a few strict rules. The rules exist so that anyone, a new contributor, the owner in a year or an AI agent with no memory of last week, can find out how anything works by reading, not by asking.

## 1. The rules in one screen

1. **Every fact has one home.** Find it through [docs/README.md](docs/README.md); link to it, never copy it (§2).
2. **Docs move with code, in the same commit.** [docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) is the checklist (§4).
3. **IDs are permanent.** `FR-`, `NFR-`, `ADR-` and `ERR-` numbers are never renumbered or reused; new ones go at the end (§3).
4. **Every change traces to a requirement**: its ID is in the commit, the test name and the docs.
5. **Every error the app can show has a code** and a row in [docs/ERROR_CONTROL.md](docs/ERROR_CONTROL.md) (§5).
6. **Performance is measured, never guessed.** No optimization without a benchmark; the budgets are in [docs/OPTIMIZATION.md §3](docs/OPTIMIZATION.md#3-budgets).
7. **Privacy.** No network calls in the core, no telemetry, GPS removed on export. Photos never leave the user's computer.
8. **Security.** Metadata is never rendered as HTML (`innerHTML` and `dangerouslySetInnerHTML` are banned); every IPC input is validated in Rust ([docs/SECURITY.md](docs/SECURITY.md)).
9. **Licences.** GPL-3.0-only code; new dependencies must be permissive (MIT, Apache-2.0, BSD, ISC, Zlib…), never GPL or AGPL. New source files start with `// SPDX-License-Identifier: GPL-3.0-only`.
10. **`scripts/check-docs.sh` passes and CI is green** before anything is merged.

## 2. Single source of truth

Each fact is written in exactly one file, its *single source of truth* (SSOT). Every other place links to it. Two copies of a fact drift apart, and then nobody knows which one is right.

- **Where a fact lives:** the table "Where does each fact live?" in [docs/README.md](docs/README.md). Some examples:

  | Fact | Its home |
  |------|----------|
  | What the app does, and the `FR-`/`NFR-` IDs | [docs/PRD.md](docs/PRD.md) |
  | Whether something is done, and what comes next | [docs/STATUS.md](docs/STATUS.md) |
  | Which phase a requirement belongs to | [docs/ROADMAP.md](docs/ROADMAP.md) |
  | Why a decision was made | [docs/adr/](docs/adr/README.md) |
  | What is saved to disk, and its schema versions | [docs/DATABASE.md](docs/DATABASE.md) |
  | Every error code | [docs/ERROR_CONTROL.md](docs/ERROR_CONTROL.md) |
  | How to use the app (also the website's manual) | [docs/USER_GUIDE.md](docs/USER_GUIDE.md), mirrored in [USER_GUIDE.vi.md](docs/USER_GUIDE.vi.md) |
  | What changed for users in each version | [CHANGELOG.md](CHANGELOG.md) |

- **Reading order** by task (new to the project, writing code, debugging, performance, security, releases) is the first table of [docs/README.md](docs/README.md).
- **Generated files are not sources.** `ui/src/ipc/bindings.ts` is generated from Rust by `cargo test`; the website's manual is generated from the user guide; the charts are generated from `bench/results/`. Change the source, then regenerate.
- **When you find a fact in two places**, keep it in its home, replace the other copy with a link, and mention it in your pull request.
- **Language.** Docs and code are in English. The Vietnamese files ([README.vi.md](README.vi.md), [docs/USER_GUIDE.vi.md](docs/USER_GUIDE.vi.md), `ui/src/locales/vi.json`, the website's `vi` strings) mirror their English file and change in the same commit. The website's other languages (Czech, German, Hungarian, Polish) follow English too: their words in `site/_data/i18n/<code>.yml`, the Performance texts in `site/_includes/perf/<part>-<code>.md` and a short guide in `site/_includes/manual-short-<code>.md`; a correction from a native speaker is welcome as a pull request, and `node scripts/check-site-i18n.mjs --build` tells what a language still shows in English. A new language starts with one block in `site/_data/languages.yml` ([site/README.md](site/README.md)).

## 3. IDs: requirements, decisions, errors

| ID | Lives in | Example | Used in |
|----|----------|---------|---------|
| `FR-NNN`, `NFR-NNN` | [docs/PRD.md](docs/PRD.md), with a row in ROADMAP and STATUS | `FR-073` postcards | Commit messages, test names (`fr_073_…`), docs |
| `ADR-NNNN` | [docs/adr/](docs/adr/README.md), from [the template](docs/adr/0000-template.md) | `ADR-0031` the manual on the website | Docs, code comments on the decision |
| `ERR-<KIND>-<NNNN>` | [docs/ERROR_CONTROL.md](docs/ERROR_CONTROL.md) | `ERR-IMG-0006` a cut-short JPEG | What the user sees, the error log, issues |

- **Never renumber, never reuse.** A requirement dropped from scope, a superseded decision or an error no longer raised keeps its ID and is marked as such.
- **New IDs go at the end** of their list. `scripts/check-docs.sh` fails when an FR is missing from ROADMAP or STATUS.
- **A change with no ID** (a typo, a build script) says `none (chore)` in its pull request.

## 4. Docs move with code: how to use the Docs Guide

[docs/DOCS_GUIDE.md](docs/DOCS_GUIDE.md) §2 is a table: each row is a *kind of change* and the files that kind of change must update. Before every commit:

1. **Name every kind your change is.** Most changes are several. A new feature is usually "anything a user can see", "behaviour or scope", "UI text", "a workflow" and "the website"; a new saved setting adds "anything saved to disk".
2. **Do every file in those rows**, in the same commit as the code. "Also check" lists files that change only sometimes: open them and decide.
3. **Run the checks** of its §3 (below). When a change needs an update the guide does not list, add the row to the guide in the same commit.

The rows people miss most:

| You changed | Don't forget |
|-------------|--------------|
| Something a user can see | [CHANGELOG.md](CHANGELOG.md) under `Unreleased`, in user terms, and its **Highlights** (3–6 one-line points that become the release page) |
| How the user does something | Both user guides, with the app's own labels from `ui/src/locales/{en,vi}.json`, and the screenshots if the screen changed |
| Interface text | Both `en.json` and `vi.json`, the same keys (`npm run lint` checks) |
| Anything saved to disk | [DATABASE.md](docs/DATABASE.md): the version table, a `schema_version` bump, a migration, a fixture and a test |
| A new error | Its `ERR-` row, the Rust code, and the text in both locales under `errors.<code>` |
| A bug that took more than one look | An entry at the top of [docs/LESSONS.md](docs/LESSONS.md) |
| `README.md` | `README.vi.md` in the same commit: same sections, same charts |

**The checks enforce it.** `scripts/check-docs.sh` (and the `docs-with-code` workflow on every pushed commit) fails when a commit changes `crates/`, `src-tauri/src/` or `ui/src/` without touching a doc, the CHANGELOG or a README. A commit that truly needs no doc says `[no-docs]` and why in its message.

```sh
scripts/resources.sh                       # free disk, memory, swap: before, during and after builds
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                     # also regenerates ui/src/ipc/bindings.ts: commit it
(cd ui && npm run typecheck && npm run lint)
scripts/check-docs.sh                      # run on its own; it must pass
```

The build files of a Rust project grow fast. Free disk with `scripts/clean.sh`, never by deleting parts of `target/` by hand ([docs/DEVELOPMENT.md §4](docs/DEVELOPMENT.md#4-build-disk-hygiene)).

## 5. Error codes: reading one, adding one

The app is offline and sends nothing, so there is no crash reporter. **The code at the end of an error message, for example `ERR-IMG-0006`, is the only clue a bug report brings.**

**Reading a code.** `ERR-<KIND>-<NNNN>`: the kind of fault, then its number within that kind.

| Kind | What failed |
|------|-------------|
| `IMG` | The photo file itself |
| `RUS` | The Rust core (a caught panic, a broken invariant) |
| `HDW` | Hardware: a camera, a card or drive removed |
| `DRV` | Drivers: camera USB/PTP, WebKitGTK, WIC |
| `EXT` | Extensions and outside codecs (HEIF/HEVC, Raw Image Extension, libheif) |
| `FS` | The file system |
| `PRJ` | Project files (`.fotollection`) |
| `CFG` | Settings, fonts and logos |
| `EXP` | Export |
| `NET` | The network (*Check for updates* only) |
| `IPC` | Invalid input from the interface |

To debug a code:

1. Find its row in [ERROR_CONTROL.md §4](docs/ERROR_CONTROL.md#4-dictionary). It says the stable Rust code (such as `decode.truncated`), where it is raised (`file:line`), what the user saw, the cause and what to do.
2. Read the user's **Copy diagnostics** text (version, system, the last 20 error lines). The log is `errors.log` in the app's Logs folder ([DATABASE.md](docs/DATABASE.md), "Directories per platform"). It never holds photos, places, paths or file names; a photo's facts come in brackets, such as `[4:2:0 progressive 8-bit displayP3 zune]`.
3. Search [docs/LESSONS.md](docs/LESSONS.md) for the code or the symptom: it may have happened before.

**Adding an error.** Take the next free number of its kind (never reuse one); one code means one cause with one fix. Add the row in ERROR_CONTROL, the Rust code, and the English and Vietnamese text under `errors.<code>` in `ui/src/locales`. The tests in `crates/core/src/errors.rs` fail when a code and its row disagree, or when an error raised in the source has no row.

## 6. Debugging

Read [docs/LESSONS.md](docs/LESSONS.md) first: its "How we debug" section and the entries closest to your symptom.

- **Measure or observe before guessing.**
- **Split the path and log both ends**: one temporary log in the interface, one in Rust. Name them `DEBUG-…` and remove them before the commit.
- **Reproduce with the reporter's file** when you can; a synthetic test can pass while the real photo fails.
- **Write the lesson down** in the fix's commit: what was seen, why, the fix, the rule from now on.

## 7. Issues

**Issues are the inbox; the docs are the record** (owner, 2026-11-08). What users report or ask arrives as a GitHub issue; what the project decides lives in [docs/PRD.md](docs/PRD.md), [docs/STATUS.md](docs/STATUS.md) and the [CHANGELOG](CHANGELOG.md), checked by `scripts/check-docs.sh`. An issue never replaces a requirement or the plan: two lists of the same work drift apart.

Every new issue is triaged, labelled `triaged`, and ends one of four ways:

1. **A fault in what the app promises:** fixed in a commit that says `Fixes #n` (GitHub closes the issue), with its CHANGELOG line, and an `ERR-` row or a LESSONS entry when the rules ask for one.
2. **A new or changed behaviour that is accepted:** a new `FR-` at the end of the PRD naming the issue (`… (#n)`), its rows in ROADMAP and STATUS, the issue put in that version's milestone and linked from the FR; it closes when the FR ships.
3. **Not enough to act on:** labelled `needs info`, with the question asked; closed after 30 days without an answer.
4. **Not taken:** closed with the reason (`duplicate`, `wontfix`, `invalid`) and, for a duplicate, the link.

The order of work stays the "Next, in order" list of STATUS; milestones only tell people which version an accepted issue is planned for.

Open one from the [issue templates](https://github.com/hoangphi01/fotollection/issues/new/choose). Users reach the same page from the app: the version number at the foot of any panel, then **Report a problem or an idea**.

- **Bug report:** the version, the system, the photo's source and format, what you did, what you expected, what happened, and the **error code** with the **Copy diagnostics** text. Name the requirement if you know it (`FR-006`).
- **Feature request:** the problem first, then the behaviour you propose, the acceptance criteria and any effect on performance or privacy. Say whether it extends an existing `FR-` or is new; a new one gets its ID at the end of the PRD when it is accepted.
- **Photos:** attach a sample only if you are happy for it to be public, and strip GPS first. A file that fails is the best report, but never send one you would not post.
- **Security problems** go to a private [security advisory](https://github.com/hoangphi01/fotollection/security/advisories/new), never a public issue ([SECURITY.md](docs/SECURITY.md)).
- Labels: `bug`, `enhancement`, `documentation`, `question`, `accessibility`, `good first issue`, `help wanted`; `triaged` once decided, `needs info` while waiting for the reporter.

## 8. Branches, commits and pull requests

- **Branch:** `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>`. `main` is always releasable.
- **Commit:** [Conventional Commits](https://www.conventionalcommits.org/) with the requirement ID: `type(scope): summary [FR-xxx]`. Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`. Example: `fix(crop): keep portrait ratio on rotated input [FR-006]`. One logical change per commit, with its docs. No co-author lines.
- **Pull request:** one concern, from [the template](.github/pull_request_template.md). Tick its Definition of Done honestly; a performance change includes its benchmark, before and after, with the machine and the command.
- **Before you ask for a review**, the checks of §4 pass on your machine. CI then runs only the workflows your files touch (the table in [docs/RELEASE.md §4](docs/RELEASE.md#4-release-pipeline)); check that each one that started is green.

**Done means all of this:**

- [ ] Code follows [docs/STYLE_RULE.md](docs/STYLE_RULE.md); formatters and linters pass.
- [ ] Tests added or updated; a test tied to a requirement is named `fr_NNN_<behaviour>` ([docs/TESTING.md](docs/TESTING.md)).
- [ ] Every row of [DOCS_GUIDE](docs/DOCS_GUIDE.md) that matches the change is done, in the same commit.
- [ ] `scripts/check-docs.sh` passes and CI is green.

## 9. Code rules worth knowing before you start

The full rules are in [docs/STYLE_RULE.md](docs/STYLE_RULE.md); these catch most first pull requests:

- **Rust:** no `unwrap()` or `expect()` outside tests, except on a proven invariant with a `// INVARIANT:` comment; errors with `thiserror` in the core; `clippy -D warnings` clean.
- **Locks:** never hold a lock while rayon work runs, and read what you need from one lock before taking the next ([STYLE_RULE.md §2](docs/STYLE_RULE.md#errors), [LESSONS.md](docs/LESSONS.md)).
- **Interface:** shadcn/ui components and Lucide icons only; panel cards built only from the lego blocks of [STYLE_RULE §4](docs/STYLE_RULE.md#4-uiux) (`npm run lint` runs `check-lego`); the same control in the same order everywhere; every string in both locales. The steps to check a UI change are in [`.claude/skills/style-audit/SKILL.md`](.claude/skills/style-audit/SKILL.md), for people and agents alike.
- **Untrusted input:** photos, metadata and IPC arguments are untrusted. Validate in Rust, never render metadata as HTML.
- **Performance:** respect the budgets; an optimization comes with its benchmark and a row in [OPTIMIZATION.md §5](docs/OPTIMIZATION.md#5-benchmark-log).

## 10. For AI agents

Agents follow everything above. In addition:

- **Start with [CLAUDE.md](CLAUDE.md)** (the rules for AI assistants), then [docs/README.md](docs/README.md) and the "Next, in order" list of [docs/STATUS.md](docs/STATUS.md), and list the issues not triaged yet (`gh issue list --search "-label:triaged"`) to bring them to the owner; never triage, label or close one without the owner's word.
- **Look facts up, never invent them.** Versions, numbers, file names, labels and commands come from the repository. A number on a public page is measured or sourced.
- **Use the app's own labels**, from `ui/src/locales`, when you write about the interface.
- **Run `scripts/check-docs.sh` on its own**, not in a chain, and read its output.
- **Never push, release or publish without the owner's explicit go-ahead**, and add no AI attribution to commits or pull requests.
- **Check resources** with `scripts/resources.sh` around builds and tests; the owner's machine has a small disk.

## 11. Licence and the CLA

Fotollection is licensed under [GPL-3.0-only](LICENSE). To keep the option of changing the licence of future versions ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)), **an outside contribution can be merged only after its author signs a Contributor License Agreement**. The CLA lets the owner relicense the contribution; you keep your copyright. It is not set up yet ([STATUS.md](docs/STATUS.md)): you are welcome to open a pull request, and the owner will ask you to sign before it is merged.
