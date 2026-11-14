# UI Tests — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-09
>
> Every UI and UX test case of the app, with the rule each one guards and how it is run. Most are judged by a machine (`scripts/ui-check.sh`), so a person checks only what a machine cannot. The rules come from [STYLE_RULE.md](STYLE_RULE.md) §4, the single source of truth for the interface, and from public standards (WCAG 2.2, Nielsen's heuristics, Apple's and Google's guidelines). Test strategy as a whole: [TESTING.md](TESTING.md).

## 1. How to run

```sh
python3 -I site/screenshots/fetch_photos.py <folder>   # the CC0 demo photos, once
scripts/ui-check.sh <folder> [owner-folder]            # the whole matrix
QUICK=1 scripts/ui-check.sh <folder>                   # English, light, Chromium only
QUICK=2 scripts/ui-check.sh                            # English and Hungarian, light: the pre-push hook
UPDATE_BASELINES=1 scripts/ui-check.sh <folder>        # approve pictures after an intended change
```

Only the recording needs `tauri dev` quit (the app runs once at a time, and the recording would go to it); once it exists the folder may be left out and the checks run beside the dev app. The recording (`target/ui-check/dump`) is made once and kept; delete it after a change to what Rust answers. Each failure is one line that starts with its test ID; the script exits non-zero when there is any.

**The matrix, declared once.** Rows say which axes they run on, instead of repeating:

| Axis | Values |
|---|---|
| Language | English, Vietnamese, Hungarian (the longest words), Japanese (its own face), the pseudo-locale `en-XA` (English 40 % longer, a `VITE_PSEUDO=1` build) |
| Theme | light, dark |
| Engine | Chromium; WebKit, the engine of the app on macOS and Linux |
| Window width | 1180 px (the app's minimum), 1400, 1600 |
| Data | the demo recording; the stress fixtures (a long custom print size chosen, every sheet shape, every Settings section, the welcome); each view checked again with every folded card open (`allChecksOpen`, the process review of 2026-11-12) |

**When it runs:** on every push that changes the interface, English and Hungarian in light, Chromium (`QUICK=2`, the pre-push hook, CONTRIBUTING §4); the full matrix before a release: English, light, Chromium; Vietnamese, dark, WebKit; Hungarian, light; Japanese, light; `en-XA`, dark; each at every width. A case marked "release" below is in both; "commit" runs with `npm run lint` too. `MEASURE=1 node site/screenshots/shoot.mjs` is the engine under the script ([site/screenshots/README.md](../site/screenshots/README.md)); the detectors are in [`checks.mjs`](../site/screenshots/checks.mjs).

## 2. Rules for this document

- **IDs are permanent:** `UI-<CATEGORY>-NNN`, never renumbered or reused; a new case is added at the end of its category.
- **One row, one check, an objective pass rule.** "Looks fine" is never a pass rule.
- **How:** **A** a machine judges it; **P** a machine flags, a person decides; **M** a person, with the steps given.
- **When:** *commit* (`npm run lint` in `ui/`), *before a release* (`scripts/ui-check.sh`), *per release, by hand*.
- **Source:** the STYLE_RULE §4 row, the requirement (`FR-`, `NFR-`) or the public rule the case guards.
- **A fault the owner finds by eye is a missing row:** it gets its row and, where it can, its detector, in the fix's commit (LESSONS).
- **Exceptions are markers in the code** (`data-truncate-ok`, `data-overlap-ok`) with a row in §4 giving the reason; a threshold is never loosened to make a failure pass.

## 3. The cases

### Layout

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-LAYOUT-001 | Every control stays inside its card, panel or dialog | No button, field, select, switch, badge or label passes its `section`, `aside` or dialog's left or right edge by more than 0.5 px (a row that scrolls sideways aside) | A `layoutChecks` | release | STYLE_RULE §4 lego; the owner's trash button, 2026-11-07 |
| UI-LAYOUT-002 | No two controls on top of each other | No two interactive elements intersect by more than 1 × 1 px, unless one holds the other | A `layoutChecks` | release | Nielsen 5; WCAG 2.5.8 |
| UI-LAYOUT-003 | The lego measures | Between cards 12 px, a card's blocks 12, a field's parts 6, a list's rows 4, the mode switch's rows 12 | A `measure` | release | STYLE_RULE §4 table |
| UI-LAYOUT-004 | One height for every control | Select, segmented toggle, input and icon button 32 px | A `measure` | release | STYLE_RULE §4 |
| UI-LAYOUT-005 | One width for the side columns | The folder list and every panel 18.75 rem (300 px) | A `measurePanels` | release | STYLE_RULE §4 |
| UI-LAYOUT-006 | A folded card is its header alone | No band under a folded card's header (> 0.5 px) | A `foldedBands` | release | STYLE_RULE §4 |
| UI-LAYOUT-007 | An icon-only button centres its icon | Icon centre within 0.5 px of the button's, at 1180 and 1600 px | A `iconOffsets` | release | STYLE_RULE §4 toolbar button |
| UI-LAYOUT-008 | A control's words fit it | A button's, toggle's, badge's, select's or label's text no wider than its room, unless it ends in an ellipsis | A `overflows` | release | STYLE_RULE §4 "a label is never cut" |
| UI-LAYOUT-009 | A control's words fit its height | No text taller than its box | A `overflows` | release | STYLE_RULE §4 |
| UI-LAYOUT-010 | A row never pushes its last child out | In a column head, toolbar, button group or dialog footer, no child passes the right edge | A `overflows` | release | STYLE_RULE §4 "the toolbar stays on one line" |
| UI-LAYOUT-011 | The page never scrolls sideways | `scrollWidth` ≤ the window's width | A `layoutChecks` | release | WCAG 1.4.10 |
| UI-LAYOUT-012 | The sheet preview shows the whole sheet | The sheet inside its stage, landscape, square and tall | A `stressViews` | release | LESSONS: a row added above a stage |
| UI-LAYOUT-013 | A segment's words are centred | Within 0.5 px | A `overflows` | release | STYLE_RULE §4 |
| UI-LAYOUT-014 | Spacing only on the scale | Gaps 4, 8 or 12 px; no vertical margin; no `space-y` in the features | A `check-lego` | commit | STYLE_RULE §4 three spacings |
| UI-LAYOUT-015 | Edges line up in a card | A card's labels and controls start on the same left edge | M, the measure pictures | release | Apple HIG, Layout |
| UI-LAYOUT-016 | A ring is never cut | A control's ring or outline (a crisp box-shadow, as `ring-*` draws, or `outline`) lies inside every box that clips it, scroll bars shown as in the app | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-017 | A scroll area scrolls inside its box | Its viewport never reaches past its root (> 0.5 px), so a list with a height of its own never spills over what follows | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-018 | A scroll bar keeps clear of words | In every area that scrolls, words end at least 12 px (the 8 px bar and 4 px) before its right edge, measured at their clipping box | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-019 | A dialog's footer keeps clear | A footer's line is 12 px or more under the block above it | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-020 | The window can be moved with a dialog open | The title bar (`data-window-chrome`) keeps `pointer-events` other than `none` while a dialog is open | A `layoutChecks` | release | FR-052; LESSONS |
| UI-LAYOUT-021 | An open dropdown sits whole in the window, beside its button | An open `select-content` or `popover-content` lies inside the window and under its bar, and never covers its own open button; the print size list is opened with the last size chosen, from the main panel (downwards) and from the photo preview's Print tab under the macOS title bar (upwards, 2026-11-11) | A `layoutChecks` | release | FR-052; LESSONS |
| UI-LAYOUT-022 | A dialog's words clear its close button | No word of a dialog, cut where its ellipsis cuts it, comes within 8 px of its close button; *Copy settings* opened in every run | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-023 | The previews' toolbars agree | The groups and *Before* the sheet preview's toolbar shares with the photo preview's come in the same order, and it has none the photo's lacks | A `shoot.mjs` | release | STYLE_RULE §4; LESSONS |
| UI-LAYOUT-024 | A badge is seen whole | No badge passes the box that clips it; the photo preview's Print tab is opened with every size *acceptable* (the bridge's `verdict`), the longest verdict in both languages | A `layoutChecks` | release | STYLE_RULE §4; LESSONS |

### Text

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-TEXT-001 | Text cut short says itself whole | Text ended by an ellipsis or a line clamp carries a hint (`data-hint`) with all of it | A `layoutChecks` | release | Apple HIG, truncation |
| UI-TEXT-002 | Every word is translated | Every visible string comes from every locale, with the same keys | A `check-i18n` | commit | FR-020 |
| UI-TEXT-003 | Long names stay inside | A long custom size, file name or address keeps every control in its card | A, stress fixtures + UI-LAYOUT-001 | release | the owner, 2026-11-07 |
| UI-TEXT-004 | Vietnamese marks are never clipped | No stacked mark (Ằ, Ễ, Ộ) cut at the top of a line in a 32 px control | M, the VI measure pictures | release | W3C i18n |
| UI-TEXT-005 | Hungarian fits | Every panel, card and control holds the Hungarian words (long compounds such as *nyomatméret*) without overflow, at 1180, 1400 and 1600 px, both themes | A `MEASURE=1 MEASURE_LANG=hu`, run by `scripts/ui-check.sh` | release | FR-123; LOCALIZATION §3 |
| UI-TEXT-006 | Placeholders survive translation | Every locale has English's placeholders (`{{n}}`, `{{name}}`) for each key, its plural forms together | A `check-i18n` | commit | FR-123; LOCALIZATION §3 |
| UI-TEXT-007 | English fits whole | No control shrinks or cuts the interface's English words (`data-fit` other than `full` on a string of `en.json`); the three steps are for translations and for a user's own words (an address, a file name) | A `layoutChecks` | release | FR-125 |
| UI-TEXT-008 | Fitted words say themselves whole | A translation's words cut by `FitText` carry the whole in their hint; MEASURE lists every word fitted smaller or cut, in hu and the pseudo-locale `en-XA` | A `layoutChecks`, `fitReport` | release | FR-125; LOCALIZATION §3 |
| UI-TEXT-009 | Japanese fits, in its own face | Every panel holds the Japanese words drawn in the bundled Noto Sans JP, without overflow, at 1180, 1400 and 1600 px; every character of `ja.json` and its manual is in that face | A `MEASURE=1 MEASURE_LANG=ja`, run by `scripts/ui-check.sh`; `check-i18n` for the characters | release; commit (`check-i18n`) | FR-128; LOCALIZATION §5 |

### States

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-STATE-001 | A disabled button still explains itself | A disabled button's hint sits on a wrapper, never on the button (which gets no pointer events) | A `layoutChecks` | release | STYLE_RULE §4 |
| UI-STATE-002 | One look for the chosen row | Chosen rows come from `ListRow` or `CHOSEN_ROW` | A `check-lego` | commit | STYLE_RULE §4 |
| UI-STATE-003 | Red only for removing and resetting | A ✕, bin, cleared filter or ↺ is `RemoveButton` or `ResetButton` | A `check-lego` | commit | STYLE_RULE §4 |
| UI-STATE-004 | Hover deepens, pressed is pressed in, focus is visible | Under the pointer a filled control gets darker; a pressed toggle is the theme's colour, pressed in; Tab shows a ring | M, every control in both themes | release | STYLE_RULE §4; WCAG 2.4.7 |
| UI-STATE-005 | Every wait shows progress | Work over 1 s shows progress, over 10 s a count and Cancel | M | release | Nielsen 1; FR-099 |
| UI-STATE-006 | Every error says what to do | Its words, its `ERR-` code and what to do next | M + `ERROR_CONTROL.md` | release | Nielsen 9 |
| UI-STATE-007 | Empty views explain | No photos, no groups, a filter with no match: a sentence and the next step | M | release | Nielsen 10 |
| UI-STATE-008 | An app, not a page | A drag across a card's title in a dialog selects nothing; a photo's facts and an error message select and copy; a right click on bare ground shows no menu, on a field shows Cut, Copy and Paste; F5, ⌘R / Ctrl+R, ⌘P / Ctrl+P and ⌘F / Ctrl+F do nothing, ⇧⌘R and ⇧⌘C still work | M (until a detector) + `check-lego` | release | Owner, 2026-11-08 |
| UI-STATE-009 | A press is seen | A button held down has the theme's colour as its background, in both themes (the photo preview's *Rotate left*) | A `shoot.mjs` | release | STYLE_RULE §4; TRANS_ANIMATION |
| UI-STATE-010 | A part that fails fails alone | With every card broken on purpose, then the main view (`lib/fault.ts`, a build with `VITE_FAULTS=1`), each shows `PartFailed` in its place, the gallery or the side panel stays, and the window never fails as a whole | A `shoot.mjs` | release | ADR-0048; LESSONS |

### Stability

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-STABLE-001 | Nothing jumps when W or J is pressed, nor on a slider's first step | The preview's cards keep their place (≤ 1 px) after W, J, and the first step of a Faves and a Color mixer slider on a photo with the original look (owner, 2026-11-08) | A `stableChecks` | release | STYLE_RULE §4 "nothing jumps" |
| UI-STABLE-002 | Nothing jumps on the first slider step or a note | The card's height is the same before and after | M (A planned) | release | STYLE_RULE §4 |

### Theme

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-THEME-001 | Text can be read in both themes | Contrast ≥ 4.5:1 (3:1 for large text) | A `axeChecks` (`color-contrast`) | release | WCAG 1.4.3 |
| UI-THEME-002 | Colours only from tokens | No palette colour, no `rgb()` or `oklch()` written in a component | A `check-lego` | commit | STYLE_RULE §4 |
| UI-THEME-003 | Controls can be told apart | Borders, tracks and focus rings ≥ 3:1 against their card | M | release | WCAG 1.4.11 |

### Accessibility

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-A11Y-001 | Targets big enough | Every target ≥ 24 × 24 px, or its 24 px circle clear of the others | A `layoutChecks` | release | WCAG 2.5.8 |
| UI-A11Y-002 | WCAG 2.2 A and AA | No axe violation (names, roles, `aria-*`, contrast, labels) | A `axeChecks` | release | WCAG 2.2; Deque axe-core |
| UI-A11Y-003 | Keys work where they should | Every main action has its key; single-letter keys stay quiet in a text field | P `isTyping` in each handler + M | release | WCAG 2.1.1, 2.1.4 |
| UI-A11Y-004 | Dialogs behave | Esc closes (except the welcome), focus moves in and back | M | release | WAI-ARIA dialog pattern |
| UI-A11Y-005 | Screen reader | VoiceOver reads the three main flows | M | per release | WCAG 4.1.2 |

### Heuristics

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-HEUR-001 | The same thing looks the same everywhere | One block per meaning; the same card in the same order in panel and preview | A `check-lego` `@shown` + M | commit | Nielsen 4; STYLE_RULE §4 |
| UI-HEUR-002 | Undo and Esc are always there | Every edit is one undo step; Esc leaves the current mode | M | release | Nielsen 3 |
| UI-HEUR-003 | Hints teach the keys | Every toolbar button's hint names its key | M | release | Nielsen 6, 7 |
| UI-HEUR-004 | One motion system | Every transition on the motion tokens: no pace, curve or `transition-*` by hand, no hand-made spinner (FR-100, TRANS_ANIMATION.md) | A `check-lego`, `check-docs` (tokens) | commit | TRANS_ANIMATION.md §6 |

### Views

Every view below runs the A cases of §3 at each width (`overflowViews`, `measurePanels`, `stressViews`):

| ID | View | States opened |
|---|---|---|
| UI-MODE-001 | Gallery, Pro panel, filters | the demo; a long custom size |
| UI-MODE-002 | Simple mode | the demo; the strip of looks under the photo (FR-079) |
| UI-MODE-003 | Collage panel and sheet preview | a sheet chosen; its Edit and Print tabs, a photo picked and *Whole sheet*; the strip of looks under landscape, square and tall sheets (the sheet's four edges in sight); the colour cards still while the tabs switch and a photo is picked or let go; landscape, square, tall; a long custom size |
| UI-MODE-004 | Postcards | panel and preview |
| UI-MODE-005 | Photo preview | both layouts; W and J; the Looks card in the Edit tab; *Presets & LUTs* in the left column, empty, with looks in groups, a long list with its filter, the ⋯ menu open, an import's report |
| UI-MODE-006 | Settings | every section, the manual among them |
| UI-MODE-007 | Export dialog, update window, welcome | as recorded |
| UI-MODE-008 | Crop, compare, best-shot groups, select mode, capture, places and project dialogs | M until each has its fixture |
| UI-MODE-009 | Pro mode's gallery, Colour tab (FR-114) | a look picked, its tile, amount and button; measured as *Pro panel: Colour*; by hand: ticked photos, all photos shown, the question for photos with their own colour, ⌘Z |
| UI-MODE-010 | The grid shows each photo's colour (FR-114, ADR-0047) | by hand: a look given to the ticked photos, the bar *Applying colour* over the grid, their tiles in the new colour; ⌘Z brings the old colour back; the filmstrip and the quick preview alike |
| UI-MODE-011 | The Color mixer (FR-079) | by hand: a dot picked, its three sliders move only that colour, greys stay; ↺ resets; a preset with a mixer shows its values; in a collage sheet's preview, not in Simple mode |

### Pictures

| ID | Check | Pass rule | How | When | Source |
|---|---|---|---|---|---|
| UI-VISUAL-001 | Nothing changed that was not meant to | Each approved picture (Pro panel, folder list, Collage panel, the preview's inspector, every Settings section) differs by ≤ 0.2 % of its pixels, per engine, language and theme | A `baseline` (pixelmatch) | release | Playwright visual comparisons |

The approved pictures live in `site/screenshots/baselines/`. After an intended change, `UPDATE_BASELINES=1` approves the new ones, and the commit shows them; a failure writes `<name>-diff.png` into the run's output.

### By hand, once per release

| ID | Check | Steps |
|---|---|---|
| UI-MANUAL-001 | The real app on macOS | Open a folder; cull with keys; one photo through crop, look and export; a collage sheet and a postcard; Settings and the manual; both languages |
| UI-MANUAL-002 | The real app on Windows | The same, with the title bar and its hints |
| UI-MANUAL-003 | The smallest window | 1180 px wide: every toolbar on one line, the left column folding |
| UI-MANUAL-004 | Reading the words | A read of new strings in both languages against the [GLOSSARY](GLOSSARY.md) |
| UI-MANUAL-005 | Motion (FR-100) | Open and close a dialog, a popover and a hint (each fades out too); fold a card and a strip under the stage; a notice comes and goes; hold Space for the quick preview and let go; then *Settings → Motion: Reduce* and the system's *Reduce motion*: only quick fades left, spinners turning; nothing waits for an animation |
| UI-MANUAL-006 | Hold for the original (FR-096) | Press and hold the photo in the preview: its original while held, its colour back on release, a click still zooms; on a collage sheet, hold *Before* in the toolbar and the picked photo shows its original, no photo picked by the press |
| UI-MANUAL-007 | Settings → Data (FR-116) | Each store's size and *Open folder*; *Export settings* with and without addresses; *Import settings* showing what the file holds, then Merge and Replace; *Remove unused LUTs*, *Forget recent addresses*, *Reset settings* and *Erase data* each asked first, red, and the section's sizes updated after |

## 4. Allowed exceptions

| Where | Case | Reason |
|---|---|---|
| `features/modes/mode-switch.tsx` | `ToggleGroup` by hand | two rows, Simple and Collage then Pro the whole width (owner, 2026-11-06) |
| `components/forms/danger-button.tsx` | 24 px icon button | `RemoveButton small`, the ✕ inside a line or a tab |
| `components/layout/panel-note.tsx` | white text | a danger note's white on its deep red |
| A row that scrolls sideways | passes its card | the toolbars scroll when narrow (STYLE_RULE §4) |
| `components/layout/stage-nav.tsx` (`data-overlap-ok`) | lies over the sheet's halves | a viewer's arrows sit over the stage on purpose, as the photo preview's |
| A popover or dialog | lies over the page | compared within its own layer only |
| `#fffaee` on `#2d836d` (light theme) | contrast 4.41:1, under 4.5 | the logo's own teal under a filled button's cream words (owner, 2026-11-07: "keep #2d836d"); `ALLOWED_CONTRAST` in `checks.mjs` |

## 5. Last run

| Date | Matrix | Result |
|---|---|---|
| 2026-11-07 | first run, English, light, Chromium, before the fixes | 1,545 lines: the trash button 27 px out of its card (UI-LAYOUT-001) and its name cut (UI-LAYOUT-008); 50 contrast, 16 name/label, 13 unnamed sliders, 6 tab roles (UI-A11Y-002); 20 cut names with no hint (UI-TEXT-001); 3 disabled buttons holding their hint (UI-STATE-001); the ink names cut (UI-LAYOUT-008); overlap counted controls under a dialog (detector fixed) |
| 2026-11-07 | the whole matrix, after the fixes | every case passes, English/light/Chromium and Vietnamese/dark/WebKit, 1180 to 1600 px; 213 s on the M3; 22 approved pictures, 1.9 MB; a second run against them passes too |
| 2026-11-08 | the whole matrix, after the motion system, the 300 px columns and UI-LAYOUT-016 | every case passes, Chromium with scroll bars shown; UI-LAYOUT-016 found the filmstrip's chosen ring cut by 4 px before its fix; 8 pictures approved (the wider columns, *Motion* in Settings, *What's new*) |
| 2026-11-09 | the whole matrix, after the Data section, Dehaze and the joined bars | every case passes (Chromium and WebKit, both languages and themes); the Data section's pictures approved; the screenshot build found the place picker's lost `combobox` role, fixed in `PickerTrigger` (LESSONS) |
