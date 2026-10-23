# Style Rules — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-10-23
>
> These rules apply to all code and docs. Tooling enforces them where possible. Where tooling can't, reviewers do.
> If a rule blocks you, propose changing it in a PR. Don't silently break it.

## 1. General

- **Clarity over cleverness.** Code is read far more often than it is written.
- **Measure before optimizing** ([OPTIMIZATION.md §1](OPTIMIZATION.md#1-rules)). A performance-motivated complication must cite a benchmark.
- **One concept per module.** If a file exceeds about 500 lines, consider splitting it.
- **No dead code** and no commented-out blocks. Git history is the archive.

## 2. Rust

### Formatting and linting

- `cargo fmt` uses the default `rustfmt` config. CI checks it.
- `cargo clippy --all-targets -- -D warnings` must pass. `clippy::pedantic` is enabled at the workspace level with these exceptions: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc` (until 1.0).
- `#![forbid(unsafe_code)]` is set in `crates/core`, except in the explicitly whitelisted FFI modules under `decode/`.

### Errors

- `crates/core`: each module exposes a typed error enum built with `thiserror`. Errors carry context such as the file path or photo ID.
- `src-tauri`: `anyhow` may be used internally. Errors that cross IPC are converted to a serializable `IpcError { code, message }`. `code` is a stable, machine-readable string, for example `decode.unsupported`.
- **No `unwrap()` or `expect()` in non-test code**, except on proven invariants. Those require a comment that starts with `// INVARIANT:`.
- One bad photo must never fail the batch ([NFR-011](PRD.md#nfr-011--robustness)). Return per-photo `Result`s and aggregate them.

### Unsafe and FFI

- Every `unsafe` block has a `// SAFETY:` comment that explains why the invariants hold.
- FFI wrappers live in a dedicated module, expose a safe API, and have fuzz targets ([TESTING.md §4](TESTING.md#4-fuzzing)).

### API and design

- Keep core functions **pure** where possible. In particular, `RenderPlan` construction does no I/O.
- Prefer newtypes for units and IDs (`PhotoId`, `Px`, `Millimetres`) over bare integers.
- Long-running functions take a `&CancellationToken`.
- Public items in `crates/core` have rustdoc comments. Doc examples must compile.

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Crates | `fotollection-<name>` | `fotollection-core` |
| Modules/files | `snake_case` | `render/stamp.rs` |
| Types/traits | `UpperCamelCase` | `RenderPlan` |
| Functions/vars | `snake_case` | `plan_crop` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_FILE_BYTES` |
| Tests tied to a requirement | `fr_NNN_<behaviour>` | `fr_006_portrait_crop_matches_ratio` |
| Benchmarks | `bench_<area>` | `bench_export_batch` |

## 3. TypeScript (UI)

The stack is React + TypeScript + Tailwind + shadcn/ui + Lucide ([ADR-0007](adr/0007-ui-react-shadcn-lucide.md)).

- `strict: true` and `noUncheckedIndexedAccess: true`. Never `any`. Use `unknown` and narrow it.
- **Never use `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write` or `eval`.** Render metadata only as JSX text, which React escapes. CI greps for these ([SECURITY.md §7](SECURITY.md#7-ci-security-gates)).
- IPC types come only from the generated `ui/src/ipc/bindings.ts`. Never hand-write IPC types. Wrap IPC calls in hooks under `features/*/`. Components never call `invoke` directly.
- The UI holds no business logic. Anything that decides *what* the output looks like belongs in `RenderPlan` in Rust.
- Write function components with hooks only. One component per file. Props types are named `<Component>Props`.
- **Performance:**
  - Lists over about 100 items must be virtualized.
  - Memoize (`memo`, `useMemo`) only when a profile shows it helps.
  - Never keep image bytes in React state. Use `fotothumb://` URLs.
- Formatting and linting: `prettier` (with `prettier-plugin-tailwindcss` for class order) + `eslint` (`typescript-eslint`, `eslint-plugin-react-hooks`).
- File names: `kebab-case.tsx`. Components: `UpperCamelCase`. Hooks: `useCamelCase`. Functions and variables: `camelCase`.

## 4. UI/UX

- **Components:** build from shadcn/ui first. Add components with `npx shadcn@latest add <name>`, which places them in `ui/src/components/ui/`. Customize them there. Don't write a parallel component library. Feature components compose these primitives.
- **Icons:** Lucide only (`lucide-react`). Import icons individually (`import { Crop } from "lucide-react"`). Default size is 16 px in dense UI and 20 px in toolbars, at the default stroke width. Icon-only buttons must have an `aria-label` and a tooltip.
- **Styling:** use Tailwind utility classes and the shadcn theme tokens (`bg-background`, `text-muted-foreground`…). Never hard-code colours. Merge classes with `cn()` from `lib/utils.ts`. Use a 4 px spacing scale (Tailwind default).
- Light and dark themes follow the OS setting, using shadcn's CSS-variable theming.
- Every action longer than 200 ms shows progress. Every destructive action can be undone or needs confirmation.
- Keyboard-first: every primary action has a shortcut, and focus is always visible.
- Error messages say what happened and what to do next. Never show raw error strings or stack traces.

## 5. Git

- **Conventional Commits:** `type(scope): summary [FR-xxx]`. Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`.
  - Example: `feat(stamp): render date with outline [FR-004]`
- One logical change per commit. Docs updates go **in the same commit** as the code they describe ([ADR-0006](adr/0006-docs-as-code.md)).
- Branch names: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>`.
- `main` is always releasable. Changes go through PRs once there is more than one contributor.

## 6. Documentation

- **Language:** English for all repository docs, code and commits. Vietnamese translations of user-facing docs can be added later under `docs/vi/`.
- **Single source of truth:** every fact has exactly one home (see [docs/README.md](README.md#where-does-each-fact-live)). Other files link to it rather than copying it.
- **Stable IDs:** `FR-NNN`, `NFR-NNN` and `ADR-NNNN` are never renumbered or reused. Retired items are marked *Withdrawn* or *Superseded*.
- **Write an ADR** when a decision is hard to reverse, affects more than one module, or chooses between real alternatives. Use [adr/0000-template.md](adr/0000-template.md).
- **Headers:** each doc starts with a status line (`Draft` / `Stable`) and a `Last reviewed` date.
- **Tone:** concise, active voice, present tense. Use tables for comparisons and Mermaid for diagrams.
- `markdownlint` ([.markdownlint.jsonc](../.markdownlint.jsonc)) and `scripts/check-docs.sh` must pass.
