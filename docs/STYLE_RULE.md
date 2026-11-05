# Style Rules — Fotollection

> **Status:** Draft v0.1 · **Last reviewed:** 2026-11-04
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
- `cargo clippy --all-targets -- -D warnings` must pass. `clippy::pedantic` is enabled at the workspace level with these exceptions: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc` (until 1.0), `many_single_char_names`; `unwrap_used` and `expect_used` warn, so `-D warnings` keeps them out of non-test code.
- `unsafe_code = "forbid"` is set for the workspace; the only exceptions are the OS bindings described below.

### Errors

- `crates/core`: each module exposes a typed error enum built with `thiserror`. Errors carry context such as the file path or photo ID.
- `src-tauri`: `anyhow` may be used internally. Errors that cross IPC are converted to a serializable `IpcError { code, err, message }`. `code` is a stable, machine-readable string, for example `decode.unsupported`.
- **No `unwrap()` or `expect()` in non-test code**, except on proven invariants. Those require a comment that starts with `// INVARIANT:`.
- **Never hold a lock while rayon work runs** (2026-11-03). A rayon thread that waits for its parallel work runs other jobs of its pool meanwhile; if one needs the same lock, the thread waits for itself and the pool stops for good. Work that must run one at a time behind a lock (the RAW develop, `raw::one_at_a_time`) runs on a plain thread of its own, which never takes up other jobs.
- One bad photo must never fail the batch ([NFR-011](PRD.md#nfr-011--robustness)). Return per-photo `Result`s and aggregate them.

### Unsafe and FFI

- Every `unsafe` block has a `// SAFETY:` comment that explains why the invariants hold.
- FFI wrappers live in a dedicated module, expose a safe API, and have fuzz targets ([TESTING.md §4](TESTING.md#4-fuzzing)).
- The workspace forbids `unsafe` (`unsafe_code = "forbid"`). The two exceptions are the OS FFI crates `fotollection-oscodec` (codecs, [ADR-0017](adr/0017-heic-through-os-codecs.md)) and `fotollection-oscamera` (cameras over USB, [ADR-0026](adr/0026-cameras-through-the-os-usb-stack.md)): each uses `deny` plus an allow on its FFI module only, so any new `unsafe` elsewhere still fails to compile.

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
| Benchmarks | `measure_<area>` examples in `crates/core/examples/` | `measure_folder` |

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
- Light and dark themes follow the OS setting by default, or the System / Light / Dark choice in the version popover (`ui.theme`, FR-052), using shadcn's CSS-variable theming.
- **Brand theme from the logo** (`ui/src/index.css`): **one accent per mode, never mixed:** `primary` is the logo's teal tile in light mode and its wooden frame in dark mode. Buttons, switches, checkboxes, the selected photo's ring, badges and focus rings all use `primary`, so they change together. Lines (`border`, `input`, `sidebar-border`) are a pale tint of the same accent; section cards draw theirs at 1.5 px, everything else at 1 px so the column headers stay aligned. The panels carry a faint cream (light) or warm brown (dark). Use the tokens (`bg-primary`, `ring-primary`), never the hex values. **Photos are judged on neutral ground:** surfaces right around photos keep a chroma of 0.01 or less, and the preview stage stays neutral black, so the theme never shifts how a photo's colours read (kept on 2026-11-01 when the owner weighed a teal or wood stage). A photo or sheet on the stage carries the `stage-photo` utility, a faint neutral outline, so its edge shows even when its border is black. The website (`site/`) follows the same rule.
- **Logo:** at 64 px and below, use the **simplified tile** (`assets/logo-small.png`: frame + flower, no wordmark) through `ui/public/brand/mark-{28,56,84}.png`, with `srcSet` for 1x/2x/3x. The same tile supplies the favicon and every app-icon layer up to 64 px (`.ico`, `.icns`, `32x32`, `64x64`, `Square30x30`, `Square44x44`). Never let the webview downscale the full logo: WebKit's resampling turns its fine line art and text into noise. Regenerate all of these with `uv run scripts/small-icons.py` (Lanczos resampling).
- **Selected options stand out:** a pressed toggle (segmented choices such as Crop to fill / Fit whole photo, date formats, colours) is filled with `primary`, set once in `ui/src/components/ui/toggle.tsx`. Never style a selected option grey.
- **The layout has a floor.** The toolbar stays on one line; when controls are added, measure the narrowest window that still fits them (`DEMO_WIDTH` in `site/screenshots/shoot.mjs`) and raise `minWidth` in `src-tauri/tauri.conf.json` (now 1,180 px) rather than letting rows wrap.
- **Sections are cards.** Every settings section, in the settings panel and the preview inspector, is a `SectionCard` (`ui/src/components/layout/section-card.tsx`): a tinted header with the icon in the accent colour, the title and the master switch on the right; the body folds away when the switch is off, and a footer holds actions that must stay visible.
- **The lego rule: one set of blocks, one set of measures (2026-11-04).** Every card in a panel (Pro, Simple, Collage and both previews) is built only from these blocks, so spacing and control sizes are decided once:

  | Level | Measure | Block |
  |---|---|---|
  | Width of the folder list and the tools panel | 18 rem (`w-panel`), overlay scroll bars on both | `LibrarySidebar`, `SidePanel` |
  | Between cards | 12 px, 12 px inset | `SidePanel`, `ViewerFrame` |
  | Between a card's blocks | 12 px | `SectionCard`, `Stack` (a panel footer), `Collapse stack="card"` |
  | Within one field: label, control, options, hint | 6 px | `Field`, `SwitchField`, `Collapse stack="field"` |
  | Lines of facts | 4 px | `FactLines`, `InspectorSection` |
  | A list of rows (camera bodies, places, addresses) | 8 px | `RowList` |
  | Select, input, segmented choice, swatch choice | 32 px tall, choices in `text-xs` | default `Select`, `CommitInput`, `SegmentedToggle`, `SwatchToggle` |
  | Icon button in a row | 28 px | `size="icon-sm"`, `Toggle size="sm"` |
  | A field's label / the line under a control | `text-xs`, muted, sentence case | `FieldLabel` / `Hint` |
  | A fact and its value / a note with an icon | `text-xs` | `ValueRow` / `PanelNote` |

  The blocks live in `ui/src/components/forms/` (`field.tsx`, `switch-field.tsx`, `segmented-toggle.tsx`, `swatch-toggle.tsx`). A folded `Collapse` gives back its stack's gap (`--stack-gap`), so a switched-off option leaves no gap. `ui/scripts/check-lego.mjs`, part of `npm run lint`, fails on what bypasses them in `src/features`: a hand-made `ToggleGroup`, a `SelectTrigger` or `Switch` of its own size, `space-y-*`, a vertical margin, an odd gap, `text-[10/11/12px]`, a hard-coded `id`. Its short exception list names each file outside the panels (grid, dialogs, popovers, the title bar) with its reason. A change to the panels is checked by measuring them side by side, not by eye alone: `MEASURE=1 node site/screenshots/shoot.mjs` prints every panel's measures and fails on one off the table. The `style-audit` skill (`.claude/skills/style-audit/SKILL.md`) is the whole procedure for any UI change: the blocks, the gates, the measures, both themes and languages.
- **Notices are blocks in one stack (2026-11-02).** Whatever appears over the main view (an error, a camera session to save, a task's progress) is a block of `NoticeStack` (`ui/src/components/layout/notice-stack.tsx`): the stack owns every space, 12 px around and 8 px between; a block owns no margin and wears the `Alert` shell or `NoticeCard`, so a new one only joins the list and nothing ever touches. Draw a block only when it has something to say (a hook gives `null` otherwise, as `useTaskProgress`), so an empty stack takes no room. An error notice has a dismiss button and ends with its `ERR-` code; a dismissed error comes back only from a new attempt. The same rule holds for panels: `SidePanel` and `ViewerFrame` space their cards, the cards never do.
- **Same tools, same order.** The settings panel and the preview inspector show the shared cards in one order: *Print / export size*, *Date stamp*, *Info frame*, *Export*. The preview opens with **This photo**, the photo at a glance (FR-078), above the shared cards, and then lists the photo's own facts to change them (rating and flag, capture date, location); the camera and the image are in **This photo**, which folds from its title. A new shared card takes the same place in both.
- **Nothing blinks, nothing jumps (2026-11-01).** A change of setting or a keystroke never blanks what is on screen: the last answer stays until the next has loaded. Ask Rust about a photo or a sheet with `useLiveQuery` (`features/settings/use-ipc-query.ts`), which keeps the last answer and says when it is stale; show a render with `useShownSrc`; a spinner only before the first image, never over one already shown. Settings sent to Rust apply on screen at once, and only the answer to the latest change replaces them (`useRenderSettings`, `useFrameConfig`). A text field never takes a value from outside while it has focus (`CommitInput`). Reserve the room a note takes when it can come and go under the pointer.
- **Shared pieces, not copies.** Before writing a control a second time, use the shared one: `SidePanel` (every mode's right-hand panel), `InspectorSection` (a preview's cards of facts, with tighter rows; a card of controls, such as *Print / export size*, is a `SectionCard` in the preview too, spaced as in the panel), `PrintSizeCard` (print size in Pro, Simple and the preview), `DateStampCard` (the date stamp in every mode), `KeyHints` with shadcn's `Kbd` (every keyboard hint line), `StageSpinner` (the stage's loading state), `TickCircle` and `BasketBadge` (gallery tiles and sheets), `useSelectMode` with `SelectModeButton` (select mode wherever it appears). A card used in two places lives in its feature folder and is mounted by both.
- **Select mode, not a second click model.** Ticking several photos happens only in select mode (the *Select* button, like iOS). Outside it, a click selects one photo and Space or Enter opens the preview; inside it, a click ticks and Space is held for Quick-preview. New multi-photo actions go into the select-mode bar (`selection-bar.tsx`).
- **Arrow keys belong to the focused control.** Global shortcuts (← → to change photo) check `ownsArrowKeys` (`ui/src/lib/keyboard.ts`) and stay quiet while a field, slider or select has focus; clicking the photo gives the keyboard back.
- **One kind of tooltip.** Never use the HTML `title` attribute for hints: put the text in `data-hint` and the app-wide `HintLayer` (`ui/src/components/layout/hint-layer.tsx`) shows it in the app's style, under its element, or above it when the hint as measured would pass the window's bottom, and clear of the Windows title bar; a line break (`\n`) starts a second line. On a button that can be disabled, put `data-hint` on a wrapper, as a disabled button gets no pointer events. shadcn's `Tooltip` is kept only for content a hint cannot hold, and none uses it today (2026-11-04).
- **Scroll bars are thin and in the theme colour** (`primary`, 8 px, clear track), set once in `ui/src/index.css` for native scroll areas and in `ScrollArea`'s thumb, like the website's. Never the platform's default grey bar.
- **A scroll area over the app keeps its scroll** (a popover, a combobox list, a dialog): `overscroll-contain`, so reaching its end never scrolls the panel behind it.
- **Hover deepens, never fades.** A filled button gets darker under the pointer, so the user sees it react: the red close square of dialogs (`ui/src/components/ui/dialog.tsx`) turns burgundy (`color-mix` of `--destructive` with 30 % black, in both themes). Never lighten a filled control on hover.
- **Switched-off features fold away.** When a section has an on/off switch (date stamp, info frame, place…), its options sit inside `Collapse` (`ui/src/components/layout/collapse.tsx`) so they disappear while it is off, instead of staying on screen dimmed. Folded options are `inert`, so the keyboard skips them.
- Every action longer than 200 ms shows progress. Every destructive action can be undone or needs confirmation.
- Keyboard-first: every primary action has a shortcut, and focus is always visible.
- Error messages say what happened and what to do next. Never show raw error strings or stack traces.

### Website (`site/`)

- **A new page copies an existing one.** Start from the Performance page: the `perf-head` header (kicker, title, a `.narrow` lead), then `.section`s, alternating `tint`. Reuse its classes (`.card`, `.cards`, `.showcase-row`, `.pill`, `.button`, `shot.html`) before adding CSS ([site/README.md](../site/README.md)).
- **Content spans the `.wrap`, like the landing page's sections.** Never give a block its own `max-width` while it stays left-aligned: the page looks lopsided. Only a lead paragraph is `.narrow`. Check by measuring: the content's left and right margins are equal, and match the header's (162 px at 1440 px, 16 px on a phone).
- **The story page is the one book on the site** (owner, 2026-11-04): its title block is centred, and its text is one centred serif column at a book's measure (`.story`, 40 rem), paragraphs indented rather than spaced, chapters opening on a centred title and a two-line initial and parted by an ornament. Centred, so both margins stay equal; no other page takes a column.
- **The header melts into the page at the top** and turns into a frosted bar with a line under it only once the page scrolls (`.scrolled`, set by `default.html`; without script it is always the bar). Behind the top of every page sits one soft glow in the logo's colours (wood, teal, flower) at `--glow` strength, fading out downward; never a second gradient elsewhere (2026-11-03).
- **Round floating buttons sit in the bottom corners**, 44 px, 16 px from the edges, `pill pill-primary` with a Lucide icon: "back to top" on the right, a page's own menu (the manual's chapters) on the left, mirrored.
- **A list that opens over the page is a native `popover`**, opened by `popovertarget`, with no library. Picking an item closes it, and it marks where the reader is (`aria-current`).
- **Tables never break their first column.** A table sits in a sideways scroller (`.table-scroll`) on a phone; its first column does not wrap.
- **Icons are Lucide SVGs** in `site/_includes/icons/`, `currentColor`, one file each. Every string is in `site/_data/i18n.yml` in both languages.
- **Check before pushing,** at 1280–1440 px and 390 px, light and dark, in both languages, with no sideways scroll on the page.

## 5. Git

- **Conventional Commits:** `type(scope): summary [FR-xxx]`. Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`.
  - Example: `feat(stamp): render date with outline [FR-004]`
- One logical change per commit. Docs updates go **in the same commit** as the code they describe ([ADR-0006](adr/0006-docs-as-code.md)).
- Branch names: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>`.
- `main` is always releasable. Changes go through PRs once there is more than one contributor.

## 6. Documentation

- **Language:** English for all repository docs, code and commits. Two exceptions: `README.vi.md`, the Vietnamese README that GitHub readers reach through the **VI** link at the top of `README.md`, and `docs/USER_GUIDE.vi.md`, the Vietnamese manual on the website ([ADR-0031](adr/0031-manual-on-the-website-from-the-user-guide.md)). Each changes in the same commit as its English original, and `scripts/check-docs.sh` checks that both have the same sections (and charts, or chapters and screenshots). Any other translation goes under `docs/vi/`.
- **Single source of truth:** every fact has exactly one home (see [docs/README.md](README.md#where-does-each-fact-live)). Other files link to it rather than copying it.
- **Stable IDs:** `FR-NNN`, `NFR-NNN` and `ADR-NNNN` are never renumbered or reused. Retired items are marked *Withdrawn* or *Superseded*.
- **Write an ADR** when a decision is hard to reverse, affects more than one module, or chooses between real alternatives. Use [adr/0000-template.md](adr/0000-template.md).
- **Headers:** each doc starts with a status line (`Draft` / `Stable`) and a `Last reviewed` date.
- **Tone:** concise, active voice, present tense. Use tables for comparisons and Mermaid for diagrams.
- `markdownlint` ([.markdownlint.jsonc](../.markdownlint.jsonc)) and `scripts/check-docs.sh` must pass.
