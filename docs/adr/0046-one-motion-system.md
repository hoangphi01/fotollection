# ADR-0046: One motion system

- **Status:** Accepted (owner, 2026-11-08); amended 2026-11-09: M1–M2 shipped in v0.14.0, M3 (a photo from its tile, crossings, sliding rows, the 60 fps log) moved to v0.14.1 by the owner
- **Date:** 2026-11-08
- **Related:** FR-100, FR-099, ADR-0013, ADR-0041, ADR-0044; [TRANS_ANIMATION.md](../TRANS_ANIMATION.md)

## Context

The owner asked on 2026-11-06 for motion "that feels like one app" (FR-100), and on 2026-11-08 for a standard set released with v0.14.0: "each place animates its own way". An audit that day found no motion token at all:

- 11 durations (100, 150, 200, 220, 300, 450, 700, 800, 1000, 1100, 2000 ms) and at least 7 curves;
- dialogs at 100 ms, hints at 150, folds at 200, thumbnails at 300;
- hints and the quick preview with no exit;
- strips and notices that jumped;
- five `transition-all`, and timings copied by hand into scripts;
- reduced motion honoured in four places.

The owner chose the whole of FR-100 for v0.14.0 (including a photo growing from its tile), the system's *Reduce motion* plus a setting, and the website on the same tokens.

## Options considered

1. **A motion library** (Framer Motion / Motion, MIT). Pros: springs, layout animation, presence. Cons: about 40 kB to the bundle for what CSS, tw-animate and the Web Animations API already do; a second way to animate beside Radix's `data-state` and tw-animate.
2. **Tailwind's defaults, used consistently.** Pros: nothing to add. Cons: one pace (150 ms) and one curve for everything; nothing to stop the next hand-written `duration-300`.
3. **Tokens of our own in `index.css`, semantic utilities, a script mirror that reads the CSS, a lint rule and a docs check.**

## Decision

Option 3:

- **Tokens and values.**
  - Three paces (fast 120, base 200, slow 300 ms), exits one step shorter, three curves (enter, exit, move) and two delays.
  - The values follow Fluent 2's scale and Material 3's emphasized curves, written once in `index.css`.
  - The splash and the website declare the same values; `check-docs` compares them.
- **Utilities, never numbers.**
  - Blocks use `motion-control`, `motion-move`, `motion-fade`, `motion-fold`, `motion-in`/`-out` and `motion-in-slow`/`-out-slow`.
  - Scripts use `lib/motion.ts` (`motionMs`, `usePresence`, `reducedMotion`).
  - `check-lego` refuses paces, curves and transitions written by hand.
- **Reduced motion is the tokens' job.**
  - `data-motion="reduce"` (system or setting) shortens every pace and zeroes tw-animate's slides, scales and turns, so only quick fades remain; spinners keep turning.
  - *Motion: Follow the system / Reduce* is a Rust enum in `UiSettings` (settings v18).
- **No library.**
  - Exits through a small presence hook.
  - Rows through a FLIP hook of our own.
  - Views through the View Transitions API, with a fade where the web view lacks it (older WebKitGTK).
- **Rust first (ADR-0044).** Motion never waits for Rust: a transition starts from what is in memory and Rust's answer lands during it.
- **Measured.** 60 fps while a transition runs (OPTIMIZATION §3), by a frame log in the screenshot harness. A fold that resizes the stage is measured first, and slides over it instead if it misses.

## Consequences

- One place to change the app's pace; the website moves at the same one.
- A fold still animates grid rows, a layout property, to keep content its natural height; it is the one exception, and measured.
- The start-up screen's styles moved from an inline `<style>` to `public/splash.css`, which also removes the inline style the release CSP would have needed (SECURITY.md §2).
- New blocks must pick a utility; a pattern the table lacks is added to TRANS_ANIMATION.md first.
