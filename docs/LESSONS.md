# Lessons — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-25
>
> What each debugging session taught us, so the same mistake is not made twice. Every bug that took more than one look to find gets an entry here in the same commit as its fix ([DOCS_GUIDE.md](DOCS_GUIDE.md) §2). Newest first. Each entry says what we saw, why it happened, the fix, and the rule to follow from now on. Performance-only lessons stay in [OPTIMIZATION.md](OPTIMIZATION.md) (for example §4.3.1, interactive work off background queues).

## How we debug

1. **Measure or observe before guessing.** The v0.6.0 plan assumed a 32 MP preview might be built; measuring first showed the preview was already capped at 1600 px, and the real gap was zoom ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).
2. **Split the path and log both ends.** A temporary log on the UI side (what was asked) and one on the Rust side (what was answered, how long) tells which layer fails in one try. Name them `DEBUG-…` and remove them before the commit (`grep -rn DEBUG ui/src src-tauri/src` must be empty).
3. **Reproduce with the user's photo when the user sees it.** A synthetic test passed while the real app was soft: unit tests prove the pixels, only the real webview proves what is shown.
4. **Write the lesson here** before moving on.

## 2026-10-25

### A RAW can look like a plain TIFF (FR-016)

- **Seen:** measured before writing the feature, ImageIO returned a NEF as 160 × 120 and could not open an ARW at all, while `sips` handled both.
- **Why:** NEF, ARW, CR2 and DNG start with an ordinary TIFF header. From bytes alone ImageIO read the NEF as a TIFF (its first image is a thumbnail); `sips` passes the file name, so it knew.
- **Fix:** pass the format's type identifier as a hint, take the largest image in the file; on Windows ask WIC for the RAW decoder directly.
- **Rule:** measure every format on a real sample before building on it; a format family can hide a case that breaks the design.

### Some characters turn into emoji on iOS (website)

- **Seen:** the footnotes' back link `↩` and the `☕` in the support heading showed as colour emoji on an iPhone.
- **Why:** characters with an emoji presentation are drawn as emoji by iOS unless a text selector follows them.
- **Fix:** Lucide icons (`corner-left-up`, `coffee`) as inline SVG, like every other icon on the site.
- **Rule:** no pictographic characters in the UI or on the site; use Lucide (STYLE_RULE: Lucide icons only).

### A red check hid for three pushes (CI)

- **Seen:** the owner noticed red crosses on the last commits on GitHub; the `docs` workflow had failed since the v0.6.0 plan change, while only `pages` and `release` were being watched.
- **Why:** moving the releases added an item to the "Next, in order" list in STATUS.md without renumbering the last one (`7.` twice); markdownlint's MD029 rule fails on that. `check-docs.sh` does not run markdownlint, and the markdownlint line of DOCS_GUIDE §3 was skipped.
- **Fix:** renumbered; each workflow now runs only when its own files change (RELEASE.md §4), so every red cross means something.
- **Rule:** run every line of DOCS_GUIDE §3 before pushing docs.
- **It happened again** the same evening (`` `crx ` `` in ADR-0023 slipped through a `npx … | tail -1 && git commit` chain), so the fix became structural: `scripts/check-docs.sh` now runs markdownlint itself, with CI's version and rules, and its exit code is the only thing to check. After pushing, look at every run that started (`gh run list --limit 5`), not only the one expected.

### A CSS `scale()` throws image pixels away (FR-057)

- **Seen:** zoomed to 800 %, the full-resolution layer arrived (Rust log: 1,222 px wide, 45–137 ms) and sat in the right place, yet looked as soft as the 1600 px preview under it.
- **Why:** the layer was laid out at its small fit size (125 CSS px) and enlarged with `transform: scale(8)`. WebKit rasterises an element at its layout size first, then scales that bitmap, so the extra pixels never reach the screen.
- **Fix:** size the layer at its final on-screen size and only `translate` it (`use-sharp-region.ts`, `layerStyle`).
- **Rule:** to show detail, an image's layout box must have as many CSS pixels as it should display. CSS `scale()` is fine for motion, never for sharpness.

### Jekyll `include` parameters cannot be expressions (website)

- **Seen:** the live site still lacked the print-basket section; the GitHub Pages workflow had failed on every push since that commit, unnoticed.
- **Why:** `{% include shot.html alt=t[show_key] %}`: an `include` parameter must be a plain variable or a string. Liquid stops the whole build.
- **Fix:** `{% assign show_alt = t[show_key] %}` first, then `alt=show_alt`.
- **Rule:** after a website change, build it locally with the Pages image ([site/README.md](../site/README.md) "Preview locally") and check that the `pages.yml` run is green after the push.

### `command | tail` hides the exit code

- **Seen:** `scripts/check-docs.sh 2>&1 | tail -1 && git commit …` committed although the check had found two problems.
- **Why:** a pipeline returns the exit code of its last command, here `tail`, which succeeds.
- **Rule:** never chain on a piped check. Run the check on its own, or test its output (`grep -q "all checks passed"`), or use `set -o pipefail`.

### Editing code while a build runs produces a mixed build

- **Seen:** `tauri build` failed with "no `Region` in `preview`" although the code was complete.
- **Why:** Cargo had already compiled `fotollection-core` from the old `preview.rs` when the new code reached `src-tauri`.
- **Rule:** finish the edits, then build. While a long build runs, only edit files it does not compile (docs, the website).

### A hot reload that adds a hook to an open component blanks the app

- **Seen:** the dev app turned white after a temporary `useEffect` was added to the zoom hook while the preview was open.
- **Why:** React Fast Refresh keeps the mounted component's hook list; a different number of hooks is a hook-order error, and with no error boundary the tree unmounts.
- **Rule:** this is a dev-only artefact, not a bug: restart `npm run dev`. Prefer changes that keep the hook count when testing live.

### GPL-3.0 cannot forbid sharing or selling (website)

- **Seen:** the owner asked for "no redistribution or resale in any form" on the site.
- **Why:** GPL-3.0 grants both and §7/§10 void any added restriction; such a line would mislead.
- **Fix:** the footnotes protect what can be protected: the official name, logo and downloads, a request to tell the author, a request to credit fotollection.org. Real options are relicensing future versions ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)) or a registered trademark.
- **Rule:** never publish a licence term the licence itself does not allow; say what actually protects the author.

## Earlier

### Rounding a zoom region can step outside the photo (FR-057)

- **Why:** rounding fractions to 4 places can make `x + w` slightly more than 1, which Rust rightly refuses.
- **Rule:** round region starts and sizes **down** in the UI; Rust still validates every value ([SECURITY.md](SECURITY.md) §2).

### Mock data falls behind the real bridge (screenshots)

- **Seen:** website screenshots showed `NaN` counts and settings that did not stick.
- **Why:** the mock IPC bridge in `site/screenshots/` lacked fields added to `PhotoList` and did not store `set_settings`.
- **Rule:** when a command or a type gains a field, update the mock bridge in the same commit, and read the shots before publishing them.

### Ask before building a layout the user described in words (FR-040)

- **Seen:** *Fit* was built twice the wrong way before the owner explained it with real sizes.
- **Rule:** for a visual rule, restate it with numbers (10 × 15 cm keeps 10 × 15; *Fit* adds 5 mm on three sides plus the bar) and get a yes before coding.
