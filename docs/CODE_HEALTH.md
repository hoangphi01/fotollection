# Code Health — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-17
>
> How healthy the code is, in numbers, and the plan for the files that are not ([STYLE_RULE.md §1](STYLE_RULE.md#1-general), *Quality is a number*). `scripts/code-health.mjs` measures it on every `check-docs.sh` run (the pre-push hook and CI); `scripts/code-health.json` is the watch list it compares against. Owner, 2026-11-17: "code that runs is not good enough; how well it is built has to be a number".

## 1. The measures

| Measure | Green | Orange: watched | Red: must change | Tool |
|---|---|---|---|---|
| A file's production lines (Rust: before `#[cfg(test)]`; other code: all) | ≤ 800 | 801–1,500, or over 1,500 lines with its tests | > 1,500 | `scripts/code-health.mjs` |
| An orange file's growth | — | up to 10 % past its recorded size | any growth of a red file | `scripts/code-health.mjs` |
| A function's length | ≤ 100 lines | an `#[allow(clippy::too_many_lines)]`, counted against a budget that only goes down | a new allow past the budget | clippy (pedantic), `scripts/code-health.mjs` |
| Dead code | 0 | — | any | rustc `dead_code`, clippy, tsc `noUnusedLocals`/`noUnusedParameters` |
| A meaning used twice | one block | — | a second copy | STYLE_RULE §4, `check-lego`, review |
| Logic in the shell | `src-tauri` forwards to the core | — | pixel, file or rule logic in `src-tauri` or the webview | ADR-0044, review |
| Speed | within OPTIMIZATION §3's budgets | within 10 % of the 2026-10-23 baseline | more than 10 % slower than the baseline | the monthly benchmarks (ROADMAP's maintenance) |

Generated files (`ui/src/ipc/bindings.ts`) are not counted. Files under `tests/`, `examples/`, `benches/` and `fuzz/` count as tests.

## 2. When a file turns orange

Before it grows any further, read it against this checklist and write the answer into `scripts/code-health.json` (its `cause`) and §3:

1. **Dead code?** Items nothing calls, commented-out blocks, `pub` items used only in their own file.
2. **Missed in a review?** A doc comment on the wrong function, a helper written again beside the one that exists.
3. **A block not reused?** The same validation, conversion or drawing written twice instead of one function or component (STYLE_RULE §4's lego rule).
4. **Too many concerns?** More than one reason to change the file: split it by concern into a folder.
5. **Logic in the wrong layer?** Rules, maths or file work in `src-tauri` or the UI that belong in `crates/core` (Rust first).
6. **Test setup repeated?** The same struct literal or sampler in many tests: a fixtures helper.
7. **Data inside code?** Words, tables or profiles that belong in a data file.
8. **Inherent?** A layout read off a design board, a format's spec: say so, and keep each function under 100 lines.

## 3. The watch list (2026-11-17)

Summary: 508 files, 79,362 production lines, **73.7 % of them in green files**; 4 red, 13 orange; 17 `too_many_lines` allows. Dead code found: none (three misplaced doc comments, about 30 `pub` items used only in their own file).

| File | Production | Level | Main causes | Plan |
|---|---:|---|---|---|
| `src-tauri/src/commands.rs` | 3,246 | red | 17 concerns and the shell's helper hub; ~380 lines of logic for the core; ~330 lines repeated (lock errors 75 times, the edit frame 3 times) | FR-138 S1–S3 |
| `crates/core/src/preview.rs` | 1,704 | red | budget, standard previews, tiles, regions and the sheet front end in one file; the render plan 6 times | FR-138 S4 |
| `site/screenshots/shoot.mjs` | 1,550 | red | the IPC fake, profiles, checks, stress views and the shot loop in one file | FR-138 S5 |
| `bench/chart.mjs` | 1,545 | red | 55 % is the charts' words in 9 languages; RollSense's charts went to `bench/chart-sense.mjs` (2026-11-20) so it grew by only the import | FR-138 S5 |
| `src-tauri/src/camera.rs` | 1,469 | orange | the Sony driver, session files, state, IPC and the thread; ~500 lines for `oscamera` and the core | FR-139 S1 |
| `crates/core/src/sheet.rs` | 1,343 | orange | layout, render, card choice (4 places), credit (3 places); test setup repeated | FR-139 S3 |
| `crates/core/src/project.rs` | 636 (1,536 with tests) | orange | production green; each schema version's test repeats the load, the panic and the round trip (checklist 6) | FR-139: a `fixture(v)` and a round-trip helper |
| `crates/core/src/library.rs` | 1,270 | orange | edits, clocks, brand naming and the store | FR-139 S4 |
| `crates/core/src/postcard.rs` | 1,158 | orange | faces, the European card, the tape, the shared parts; mm to px 8 times | FR-139 S4 |
| `crates/core/src/settings.rs` | 1,158 | orange | every settings section and its sanitising | FR-139 S5 |
| `src-tauri/src/collage.rs` | 1,149 | orange | collage and postcard commands, grown with AddAssist 1.1 | FR-139 S5 |
| `crates/core/src/hagaki.rs` | 1,084 | orange | copies of the Korean card's message and code digits; draw functions over 200 lines | FR-139 S2 |
| `crates/core/src/looks.rs`, `export.rs`, `plan.rs`, `glyph.rs`, `raw.rs` | 804–865 | orange | just over the line; to read against §2 | FR-139 S5 |
| `crates/core/tests/pipeline.rs` | 1,555 (tests) | orange | `ExportOptions` written out 22 times | FR-139 S5 |

## 4. How a split is done

One stage at a time (owner, 2026-11-17: "nobody replaces both arms and both legs at once"): a stage is one concern, moved or shared in one commit or a few, with no change of behaviour; every test, clippy and the UI cases stay green, `scripts/code-health.mjs` reports fewer lines in the file and its new numbers go into `scripts/code-health.json` in the same commit, and a stage that touches a measured path runs its benchmark against the baseline. A new feature is never built on a half-moved file: the stage finishes first.

## 5. History

| Date | Production lines | In green files | Red | Orange | Allows |
|---|---:|---:|---:|---:|---:|
| 2026-11-17 | 79,362 | 73.7 % | 4 | 13 | 17 |
