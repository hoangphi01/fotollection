// SPDX-License-Identifier: GPL-3.0-only
// The lego rule (STYLE_RULE §4): a panel's cards are built only from the shared pieces in
// src/components, so spacing and control sizes are decided once. This check fails on what
// bypasses them in src/features: hand-made toggle groups, a select of another height, vertical
// spacing or margins written by hand, odd text sizes, hard-coded ids. `npm run lint` runs it.
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

// fileURLToPath, not `.pathname`: on Windows the latter gives `/C:/…` (LESSONS.md).
const root = fileURLToPath(new URL('..', import.meta.url))
const features = join(root, 'src/features')

/** Files outside the panels, with the reason they may space or size themselves. */
const EXEMPT = {
  'features/library/library-grid.tsx': 'the photo grid, laid out by its own virtual rows',
  'features/library/library-sidebar.tsx': 'the folder list, a list of rows, not cards',
  'features/library/library-empty-state.tsx': 'the empty window, centred text',
  'features/about/version-history.tsx': 'a popover list of releases',
  'features/about/version-popover.tsx': 'a popover, not a panel',
  'features/about/report-issue.tsx': 'a popover section',
  'features/about/cache-usage.tsx': 'a popover section',
  'features/about/update-check.tsx': 'a popover section',
  'features/capture/camera-controls.tsx': 'the capture dialog, its own layout',
  'features/places/date-place-dialog.tsx': 'a dialog, its own layout',
  'features/project/project-dialogs.tsx': 'dialogs, their own layout',
  'features/simple/simple-view.tsx': 'the stage and its big Keep and Drop buttons',
  'features/library/thumb-tile.tsx': 'a grid tile, its badges sized to the thumbnail',
  'features/print-advice/verdict-badge.tsx': 'a badge, sized to sit inside a line of text',
  'features/viewer/crop-editor.tsx': 'the crop overlay on the photo, not a panel',
  'features/places/borrow-dialog.tsx': 'a dialog, its own layout',
  'features/modes/mode-switch.tsx': 'the window title bar, not a panel',
  'features/about/language-setting.tsx': 'a compact row in the version popover',
  'features/appearance/theme-switch.tsx': 'icon buttons in the version popover',
  'features/best-shot/compare-dialog.tsx': 'a dialog, its own layout',
  'features/best-shot/group-strip.tsx': 'a strip of thumbnails, its badges sized to them',
  'features/best-shot/groups-view.tsx': 'the stage listing the groups, not a panel',
}

const RULES = [
  {
    test: /<ToggleGroup\b|import \{[^}]*\bToggleGroup\b/,
    say: 'a hand-made toggle group: use SegmentedToggle or SwatchToggle (components/forms)',
  },
  { test: /<SelectTrigger[^>]*\bsize=/, say: 'a select of its own size: every select in a panel is 32 px, the default' },
  {
    test: /\bspace-y-[0-9.]+/,
    say: 'vertical spacing by hand: a card spaces its blocks (SectionCard), a field its parts (Field, SwitchField)',
  },
  {
    test: /\bgap-(x-|y-)?(0\.5|2\.5|[4-9]|1[0-9])\b/,
    say: 'an odd gap: rows use gap-1, gap-1.5 or gap-2, a grid at most gap-3',
  },
  { test: /className="[^"]*\b-?m[ty]-[0-9.]+/, say: 'a margin: a block owns none, its container spaces it' },
  { test: /\btext-\[(10|11|12)px\]/, say: 'an odd text size: text-xs or text-sm' },
  { test: /\bid="[^"{]+"/, say: 'a hard-coded id: two panels can show the same card, use useId()' },
  {
    test: /<Switch\b[^>]*size=/,
    say: 'a switch of its own size: switches in a panel are the default size (SwitchField)',
  },
]

const files = []
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) walk(path)
    else if (path.endsWith('.tsx')) files.push(path)
  }
}
walk(features)

const problems = []
for (const path of files) {
  // Forward slashes on every system, as the exception list writes them.
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  if (rel in EXEMPT) continue
  readFileSync(path, 'utf8')
    .split('\n')
    .forEach((line, i) => {
      for (const rule of RULES) {
        if (rule.test.test(line)) problems.push(`${rel}:${i + 1}: ${rule.say}`)
      }
    })
}

if (problems.length) {
  console.error(problems.join('\n'))
  console.error(`check-lego: ${problems.length} problem(s); see STYLE_RULE §4 (the lego rule)`)
  process.exit(1)
}
console.log(`check-lego: ${files.length} files, every card built from the shared pieces`)
