---
name: style-audit
description: Audit a Fotollection UI change against STYLE_RULE §4 (the lego rule) before it is committed. Use for every new or changed component, card, panel, dialog, toolbar or hint in ui/src, and when the owner reports a spacing, size, alignment, tooltip or "looks different" problem.
---

# Style audit (Fotollection UI)

A UI change is done only when it is built from the shared blocks, measured, and seen in
both themes and both languages. This skill is the procedure. The rules themselves live in
`docs/STYLE_RULE.md` §4; read the lego table there first, and never restate its numbers
from memory: if the code and the table disagree, the table wins or is changed in the same
commit.

## 1. Read before touching

- `docs/STYLE_RULE.md` §4: the lego table (widths, gaps, control heights, labels, hints).
- `docs/LESSONS.md`: the UI entries (search "UI", "spacing", "tooltip", "Collapse").
- The blocks: `ui/src/components/forms/` (`field.tsx`: `Field`, `FieldLabel`, `Hint`,
  `Inset`, `ValueRow`, `RowList`, `FactLines`, `Stack`; `switch-field.tsx`;
  `segmented-toggle.tsx`; `swatch-toggle.tsx`; `commit-input.tsx`) and
  `ui/src/components/layout/` (`SectionCard`, `InspectorSection`, `PanelNote`, `Collapse`,
  `SidePanel`, `ViewerFrame`, `NoticeStack`, `HintLayer`).
- If the same card exists elsewhere (panel and preview, Pro and Simple, Collage and the
  sheet preview), open both: the change goes into the shared card, never into one copy.

## 2. Check the code (each item is a yes/no)

1. Built only from the blocks: no `space-y-*`, vertical margin, `ToggleGroup`, sized
   `SelectTrigger` or `Switch`, `text-[Npx]` or hard-coded `id` in `src/features`. A new
   pattern that two places need becomes a block in `components/`, with its measure added to
   the §4 table.
2. A switch's options sit in `SwitchField` (or `Collapse` with `stack`), so they fold away
   with no gap left.
3. Labels are `FieldLabel`, hints `Hint`, notes with an icon `PanelNote`, facts `ValueRow`
   or `FactLines`. A card of controls is a `SectionCard`, a card of facts an
   `InspectorSection`.
4. Icons are Lucide, sized by the block, not by hand. Icon-only buttons have `aria-label`
   and `data-hint`, are `icon-sm` in a row.
5. Hints use `data-hint` (never `title`, never Radix `Tooltip`); a disabled button gets it
   on a wrapper.
6. Colours are theme tokens (`primary`, `muted-foreground`…), never hex, except swatches of
   what prints; photos stay on neutral ground.
7. Every visible string is in `ui/src/locales/en.json` and `vi.json`.
8. Nothing blinks or jumps: the last answer stays while the next loads (`useLiveQuery`), a
   note that comes and goes keeps its room.
9. No `dangerouslySetInnerHTML` or `innerHTML`; metadata is shown as text.

## 3. Run the gates

```sh
cd ui && npx tsc -b && npm run lint      # lint = oxlint, check-i18n, check-lego
```

`check-lego` must pass. Adding a file to its exception list needs a reason a reviewer
would accept (not a panel: grid, dialog, popover, title bar).

## 4. Measure, don't eyeball

The measures are checked on the real interface with the recorded demo
(`site/screenshots/README.md`):

```sh
# Stop `tauri dev` first: the single-instance app would take the launch (LESSONS).
python3 -I site/screenshots/fetch_photos.py <empty-folder>        # CC0 demo photos
FOTOLLECTION_DEMO_FOLDER=<folder> FOTOLLECTION_DEMO_DUMP=<dump> target/debug/fotollection
npm run ui:build
MEASURE=1 node site/screenshots/shoot.mjs <dump> <out>                      # en, light
MEASURE=1 MEASURE_LANG=vi MEASURE_THEME=dark node site/screenshots/shoot.mjs <dump> <out>
```

It prints one row per view (folder list, Pro, Simple, Collage, Postcard panels, the photo,
sheet and postcard previews) and fails when a gap or a control height differs from the §4
table, or the two side panels differ in width. It also opens the gallery and Collage mode at
1180 and 1600 px and fails when an icon-only button's icon is more than 0.5 px off its centre
(a toolbar button is a `ToolbarButton`, never a hand-hidden label). A new kind of panel or card gets a row in
`measurePanels` (`site/screenshots/shoot.mjs`).

## 5. Look at it

Open the `measure-*.png` shots it writes (and the running app when the change is
interactive) and check, side by side:

- the same card in each place it appears: same blocks, same order, same spacing;
- every switch both on and off;
- the longest text: Vietnamese, the longest file name, a three-line hint near each window
  edge;
- light and dark;
- Windows: hints and popovers clear of the title bar (`DEMO_OS=windows`).

## 6. Report and record

- Report what was checked, the measure table, and anything left open; never "looks fine"
  without the numbers.
- A fault found here that slipped past the rules becomes a `docs/LESSONS.md` entry and, if
  it can be caught by code, a rule in `ui/scripts/check-lego.mjs` or a column in
  `measurePanels`, in the same commit (ADR-0006, docs move with code).
- Screenshots on the website change when panels do: `site/screenshots/build.sh`.
