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
- The workspace forbids `unsafe` (`unsafe_code = "forbid"`). The one exception is the `fotollection-oscodec` crate, the OS codec FFI for HEIC ([ADR-0017](adr/0017-heic-through-os-codecs.md)): it uses `deny` plus a crate-level allow, so any new `unsafe` elsewhere still fails to compile.

### API and design

- Keep core functions **pure** where possible. In particular, `RenderPlan` construction does no I/O.
- Prefer newtypes for units and IDs (`PhotoId`, `Px`, `Millimetres`) over bare integers.
- Long-running functions take a `&CancellationToken`.
- **One feature, one module.** Each processing step lives in its own module with a narrow input and output, for example `geocode` (GPS → place), `culling::similar` (photo → signature), and the render stages crop, stamp and frame. A new feature is a new module plus a field in the plan that feeds it (`RenderPlan`, `PlanInput`), never edits scattered through other modules. Modules talk through these plain data types, not each other's internals, so each can be tested, measured and replaced alone.
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
- Linting: **oxlint** (`ui/.oxlintrc.json`), run with `--deny-warnings`. It includes `rules-of-hooks`, and bans `dangerouslySetInnerHTML` and `eval`. We chose oxlint over ESLint for speed, and it is the Vite template default. Vendored shadcn components are exempt from the fast-refresh rule only.
- Formatting: `prettier` with `prettier-plugin-tailwindcss`, planned. Until it is added, follow the existing style: 2 spaces, single quotes, no semicolons. Generated files (`ui/src/ipc/bindings.ts`, `ui/src/components/ui/*`) keep their generator's formatting.
- **Every user-visible string goes through `t()`** from `react-i18next`, with its key in every `ui/src/locales/*.json` ([ADR-0015](adr/0015-ui-localization-i18next.md)). `npm run lint` fails on a missing key or hard-coded text. Show IPC errors with `errorText(err)`, which translates by `code`. Format numbers with `formatNumber` (`lib/format.ts`), never `toFixed` in visible text.
- File names: `kebab-case.tsx`. Components: `UpperCamelCase`. Hooks: `useCamelCase`. Functions and variables: `camelCase`.

## 4. UI/UX

- **Components:** build from shadcn/ui first. Add components with `npx shadcn@latest add <name>`, which places them in `ui/src/components/ui/`. Customize them there. Don't write a parallel component library. Feature components compose these primitives.
- **Icons:** Lucide only (`lucide-react`). Import icons individually (`import { Crop } from "lucide-react"`). Default size is 16 px in dense UI and 20 px in toolbars, at the default stroke width. Icon-only buttons must have an `aria-label` and a tooltip.
- **Styling:** use Tailwind utility classes and the shadcn theme tokens (`bg-background`, `text-muted-foreground`…). Never hard-code colours. Merge classes with `cn()` from `lib/utils.ts`. Use a 4 px spacing scale (Tailwind default).
- Light and dark themes follow the OS setting, using shadcn's CSS-variable theming.
- **Brand theme from the logo** (`ui/src/index.css`): **one accent per mode, never mixed:** `primary` is the logo's teal tile in light mode and its wooden frame in dark mode. Buttons, switches, checkboxes, the selected photo's ring, badges and focus rings all use `primary`, so they change together. Lines (`border`, `input`, `sidebar-border`) are a pale tint of the same accent; section cards draw theirs at 1.5 px, everything else at 1 px so the column headers stay aligned. The panels carry a faint cream (light) or warm brown (dark). Use the tokens (`bg-primary`, `ring-primary`), never the hex values. **Photos are judged on neutral ground:** surfaces right around photos keep a chroma of 0.01 or less, and the preview stage stays neutral black, so the theme never shifts how a photo's colours read. The website (`site/`) follows the same rule.
- **Logo:** at 64 px and below, use the **simplified tile** (`assets/logo-small.png`: frame + flower, no wordmark) through `ui/public/brand/mark-{28,56,84}.png`, with `srcSet` for 1x/2x/3x. The same tile supplies the favicon and every app-icon layer up to 64 px (`.ico`, `.icns`, `32x32`, `64x64`, `Square30x30`, `Square44x44`). Never let the webview downscale the full logo: WebKit's resampling turns its fine line art and text into noise. Regenerate all of these with `uv run scripts/small-icons.py` (Lanczos resampling).
- **Selected options stand out:** a pressed toggle (segmented choices such as Crop to fill / Fit whole photo, date formats, colours) is filled with `primary`, set once in `ui/src/components/ui/toggle.tsx`. Never style a selected option grey.
- **The layout has a floor.** The toolbar stays on one line; when controls are added, measure the narrowest window that still fits them (`DEMO_WIDTH` in `site/screenshots/shoot.mjs`) and raise `minWidth` in `src-tauri/tauri.conf.json` (now 1,180 px) rather than letting rows wrap.
- **Sections are cards.** Every settings section, in the settings panel and the preview inspector, is a `SectionCard` (`ui/src/components/layout/section-card.tsx`): a tinted header with the icon in the accent colour, the title and the master switch on the right; the body folds away when the switch is off, and a footer holds actions that must stay visible.
- **Same tools, same order.** The settings panel and the preview inspector show the shared cards in one order: *Print / export size*, *Date stamp*, *Info frame*, *Export*. The preview then lists the photo's own facts (rating and flag, capture date, camera, image, location). A new shared card takes the same place in both.
- **Select mode, not a second click model.** Ticking several photos happens only in select mode (the *Select* button, like iOS). Outside it, a click selects one photo and Space or Enter opens the preview; inside it, a click ticks and Space is held for Quick-preview. New multi-photo actions go into the select-mode bar (`selection-bar.tsx`).
- **Arrow keys belong to the focused control.** Global shortcuts (← → to change photo) check `ownsArrowKeys` (`ui/src/lib/keyboard.ts`) and stay quiet while a field, slider or select has focus; clicking the photo gives the keyboard back.
- **No native tooltips.** Never use the HTML `title` attribute for hints: put the text in `data-hint` and the app-wide `HintLayer` (`ui/src/components/layout/hint-layer.tsx`) shows it in the app's style. For rich content use shadcn's `Tooltip`.
- **Scroll bars are thin and in the theme colour** (`primary`, 8 px, clear track), set once in `ui/src/index.css` for native scroll areas and in `ScrollArea`'s thumb, like the website's. Never the platform's default grey bar.
- **Hover deepens, never fades.** A filled button gets darker under the pointer, so the user sees it react: the red close square of dialogs (`ui/src/components/ui/dialog.tsx`) turns burgundy (`color-mix` of `--destructive` with 30 % black, in both themes). Never lighten a filled control on hover.
- **Switched-off features fold away.** When a section has an on/off switch (date stamp, info frame, place…), its options sit inside `Collapse` (`ui/src/components/layout/collapse.tsx`) so they disappear while it is off, instead of staying on screen dimmed. Folded options are `inert`, so the keyboard skips them.
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

- **Language:** English for all repository docs, code and commits. The one exception is `README.vi.md`, the Vietnamese README that GitHub readers reach through the **VI** link at the top of `README.md`. Every change to `README.md` updates `README.vi.md` in the same commit, and `scripts/check-docs.sh` checks that both have the same sections and charts. Any other translation goes under `docs/vi/`.
- **Single source of truth:** every fact has exactly one home (see [docs/README.md](README.md#where-does-each-fact-live)). Other files link to it rather than copying it.
- **Stable IDs:** `FR-NNN`, `NFR-NNN` and `ADR-NNNN` are never renumbered or reused. Retired items are marked *Withdrawn* or *Superseded*.
- **Write an ADR** when a decision is hard to reverse, affects more than one module, or chooses between real alternatives. Use [adr/0000-template.md](adr/0000-template.md).
- **Headers:** each doc starts with a status line (`Draft` / `Stable`) and a `Last reviewed` date.
- **Tone:** concise, active voice, present tense. Use tables for comparisons and Mermaid for diagrams.
- `markdownlint` ([.markdownlint.jsonc](../.markdownlint.jsonc)) and `scripts/check-docs.sh` must pass.
