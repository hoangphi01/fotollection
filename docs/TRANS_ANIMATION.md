# Transitions and Animation — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-09
>
> How the interface moves: the motion tokens, which block uses which, the rules, reduced motion, the transitions between views and how motion is measured. The single source of truth for motion ([FR-100](PRD.md#fr-100--motion-that-feels-like-one-app), [ADR-0046](adr/0046-one-motion-system.md)); [STYLE_RULE.md §4](STYLE_RULE.md#4-uiux) points here. The values live once in `ui/src/index.css`; `check-docs` checks that this page, the splash (`ui/public/splash.css`) and the website (`site/assets/css/site.css`) give the same ones, and `check-lego` refuses motion written by hand.

## 1. Why the app moves

Motion is information, never decoration (owner, 2026-11-08: "each place its own kind; one standard"). A movement earns its place only when it does one of these jobs:

- **Cause and effect:** a button presses in, a toggle slides, so the click registered before any result.
- **Where things come from and go:** a popover grows from its button, a photo grows from its tile, a card folds under its title.
- **Shorter waits:** a skeleton in the shape of what comes; the last photo stays until the next is ready, then crosses to it.
- **Attention:** a notice slides in where the eye already is.

Everything else stays still. Nothing moves on load, nothing loops but a spinner, and **nothing waits for an animation**: a key or a click during a transition acts at once, and a new transition replaces the running one.

**Rust first ([ADR-0044](adr/0044-rust-first-our-edge.md)).** Motion is the webview's work, and its rule is that it never waits for Rust: a transition starts from what is already in memory (the tile's thumbnail, the preview Rust has prefetched into its cache, ADR-0041), and Rust's answer lands during it, not after it. The *Motion* setting is a Rust enum, checked when read from disk and over IPC.

## 2. The tokens

Three paces, an exit one step shorter than its entry, three curves. Durations follow Fluent 2's scale (`@fluentui/tokens`: 100, 150, 200, 250, 300 ms), the curves Material 3's emphasized ones; below about 100 ms a movement reads as a glitch, above 500 ms as a wait.

| Token | Value | For |
|---|---|---|
| `--motion-fast` | `120ms` | Hover, press, a toggle, a switch, a checkbox, a focus ring |
| `--motion-base` | `200ms` | A popover, a menu, a select, a hint; a fold (card, strip, notice) and its chevron; a photo arriving; a row moving; a zoom by key |
| `--motion-slow` | `300ms` | A dialog; a view to another (a photo growing from its tile, a mode); a thumbnail arriving |
| `--motion-base-out` | `120ms` | What leaves at the base pace (exits are quicker than entries) |
| `--motion-slow-out` | `200ms` | What leaves at the slow pace (a dialog, the start-up screen) |
| `--ease-enter` | `cubic-bezier(0.05, 0.7, 0.1, 1)` | Coming onto the screen: fast, then settling |
| `--ease-exit` | `cubic-bezier(0.3, 0, 0.8, 0.15)` | Leaving the screen: slow, then quick |
| `--ease-move` | `cubic-bezier(0.2, 0, 0, 1)` | Moving or changing size on screen (folds, rows, zoom, colours) |
| `--delay-hint` | `450ms` | Before a hint shows (`HintLayer`) |
| `--delay-stale` | `700ms` | Before a value being recounted dims (the export's estimate) |

Loops are `linear` and only two: the `Spinner` (1 s a turn) and the start-up bar.

## 3. Using them

Classes, never numbers. Each utility sets what moves, its pace and its curve:

| Utility | Moves | Pace, curve | Used by |
|---|---|---|---|
| `motion-control` | colours, border, shadow, opacity, transform (never `all`) | fast, move | `Button`, `Toggle`, `Badge`, `Switch`, `Checkbox`, `Input`, `Select`'s trigger and items, `Slider`'s thumb, `ListRow`, the filmstrip's cells, the title bar's buttons |
| `motion-move` | transform, opacity | base, move | every chevron, a dragged folder's neighbours, the progress bar |
| `motion-fade` | opacity | base, move | hover reveals (the stage's arrows, a tile's date), a dimmed stale value, a thumbnail arriving (with `duration-(--motion-slow)`) |
| `motion-fold` | grid rows, margin, opacity | base, move | `Collapse`, the strips under the stage, `NoticeStack` and each notice |
| `motion-in` / `motion-out` | with tw-animate's `animate-in` / `animate-out` | base / base-out, enter / exit | `Popover`, `Select`, `HintLayer`, the quick preview, a photo on the stage, a notice |
| `motion-in-slow` / `motion-out-slow` | the same | slow / slow-out, enter / exit | `Dialog`, `AlertDialog` |

- **Another pace** for a utility is a token too: `duration-(--motion-slow)`, `delay-(--delay-stale)`.
- **Scripts** read the same values: `motionMs('motion-base')`, `motionEase('move')`, `reducedMotion()`, `useReducedMotion()` and `usePresence()` in `ui/src/lib/motion.ts`, which reads them from the CSS; no millisecond is copied into a script.
- **A wait** shows `Spinner` (`components/forms/spinner.tsx`), sized by its block in a button or a line, `tile` over a tile, `stage` on the stage; a wait with a shape (a thumbnail) shows a skeleton block instead, never both.

## 4. Patterns

| Pattern | How | Where |
|---|---|---|
| A surface comes and goes | fade and scale from 95 %, popovers also slide 4 px from their side, in and out alike; dialogs at slow | `Dialog`, `Popover`, `Select`, `HintLayer` |
| Something is released | it stays drawn for its exit, lets clicks through, then goes (`usePresence`) | the quick preview, a hint, a notice |
| A fold | the height in grid rows, the gap given back, the content fading; the chevron (down when open, −90° when folded) and the head's line move with it | `Collapse`, `SectionCard`, `GroupRows`, the strips, *Version history* |
| A list grows or shrinks | the stack folds to its new height; a leaving item keeps its place until it has folded | `NoticeStack` |
| A row is dragged | the dragged row follows the pointer with no transition (`transition-none`), the others slide by one row | the folder list |
| Hover and press | colours and depth change fast; a hover deepens, never fades (STYLE_RULE §4); a button held down is the theme's colour pressed in, coming on `ease-enter` (most of it in the first 40 ms, so a quick click shows it) and rising again on `ease-move` (owner, 2026-11-08) | every control |

**Planned in v0.14.1** (moved by the owner, 2026-11-09; FR-100, the rest of [ADR-0046](adr/0046-one-motion-system.md)), each added to the table above when built:

1. **A photo grows from its tile** into the preview and back, through the View Transitions API (`view-transition-name` on the tile and the stage's photo), the thumbnail as the first frame; a web view without it fades. Also Simple mode and a collage sheet to its preview.
2. **The next photo crosses over:** the last one stays until the next is ready, then crosses in base; a spinner only after `--motion-slow` of waiting. The GPU canvas, the compare view and the quick preview's render too.
3. **A zoom by key or button** (`Z`, *Fit*, 1:1, + / −) slides in base; the wheel, a pinch and a drag follow the hand directly.
4. **Rows that move** (History, presets after a rename or a move, the folder list after a drop) slide to their place (`useFlip`, about 40 lines, no library); the virtual gallery after a filter fades.
5. **A mode, a Settings section, the dialog and the full-window layout** cross-fade at slow; the filmstrip scrolls smoothly to the photo shown.

## 5. Reduced motion

- **When:** the system's *Reduce motion* (macOS *Accessibility → Display*, Windows *Animation effects* off, GNOME *Reduce animation*), or *Settings → General → Motion: Reduce* whatever the system says. `lib/motion.ts` sets `data-motion="reduce"` on `<html>`; `useMotion` applies the setting in `App`.
- **What changes:** every pace becomes `--motion-fast`; slides, zooms and turns of tw-animate are set to none (their `--tw-enter-*` / `--tw-exit-*` variables), so what is left is a quick fade (Apple HIG: make motion optional); smooth scrolls jump; view transitions only fade; scripts check `reducedMotion()` before sliding, zooming or scrolling.
- **What stays:** the `Spinner` and the start-up bar keep turning (slower), as they say work runs.
- **Never the only signal:** an error, a confirmation or a state always has words or an icon too; motion only helps.

## 6. Rules (checked)

`ui/scripts/check-lego.mjs` refuses, in every `.ts` and `.tsx` of `ui/src` but comments:

- a pace or delay by number (`duration-150`, `delay-700`), a curve by keyword (`ease-out`);
- `transition`, `transition-all`, `transition-colors` and the other `transition-*` (each the default 150 ms), but `transition-none` for what follows the pointer;
- `motion-reduce:` and `motion-safe:` (the tokens do it);
- `behavior: 'smooth'` without `reducedMotion()`;
- `animate-spin` outside `Spinner`, `animate-pulse` outside the thumbnail's skeleton.

`scripts/check-docs.sh` checks that `index.css`, `splash.css`, `site.css` and the table of §2 give each token the same value. Each rule was proven to fail on a bait file before it was trusted.

Also, by review: animate `transform` and `opacity`; a fold may animate grid rows (`motion-fold`), nothing else animates a size or a place.

## 7. Measuring

Motion that drops frames is worse than none (OPTIMIZATION §3).

- **Budget:** 60 fps while a transition runs: on average at least 58 frames a second, no frame over 33 ms, on the reference machines, in both themes.
- **Planned (v0.14.1):** `MOTION=1 node site/screenshots/shoot.mjs` plays each transition (a dialog, a fold, the strips, a photo from its tile, the next photo, `Z`, History) and logs the frames from `requestAnimationFrame`; once in WKWebView with Web Inspector's timeline, and on Windows by hand (WebView2 at 125 % and 150 %).
- **First to measure:** the strips' fold, where the stage beside them changes size every frame; if it misses the budget, the strip slides over the stage and the layout settles on the last frame.

## 8. References

- Fluent 2 motion tokens (`@fluentui/tokens`, `durations.ts`, `curves.ts`).
- Material 3 motion: durations and emphasized easing.
- Apple Human Interface Guidelines, *Motion*: make motion optional; with *Reduce motion* on, minimise or remove animation.
- The owner's guide *Animation in UX Design: A Practical Guide* (2026-11-08): motion for feedback, orientation, perceived speed and attention; 200–500 ms, shorter for small moves; ease, never linear; restraint; `prefers-reduced-motion`; never motion as the only signal; one small set of reusable values.
