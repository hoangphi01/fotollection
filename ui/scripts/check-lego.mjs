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
  'features/about/version-popover.tsx': 'a popover, not a panel',
  'features/capture/camera-controls.tsx': 'the capture dialog, its own layout',
  'features/places/date-place-dialog.tsx': 'a dialog, its own layout',
  'features/project/project-dialogs.tsx': 'dialogs, their own layout',
  'features/simple/simple-view.tsx': 'the stage and its big Keep and Drop buttons',
  'features/library/thumb-tile.tsx': 'a grid tile, its badges sized to the thumbnail',
  'features/print-advice/verdict-badge.tsx': 'a badge, sized to sit inside a line of text',
  'features/viewer/crop-editor.tsx': 'the crop overlay on the photo, not a panel',
  'features/places/borrow-dialog.tsx': 'a dialog, its own layout',
  'features/modes/mode-switch.tsx': 'the window title bar, not a panel',
  'features/appearance/theme-switch.tsx': 'icon buttons at the foot of the folder list',
  'features/best-shot/compare-dialog.tsx': 'a dialog, its own layout',
  'features/best-shot/group-strip.tsx': 'a strip of thumbnails, its badges sized to them',
  'features/library/raw-badge.tsx': 'a badge, sized to sit on a thumbnail',
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
  {
    test: /\bhidden @[0-9a-z]+:inline\b/,
    say: 'a label that hides by hand: use ToolbarButton (components/forms), which keeps the icon centred when it hides',
  },
  { test: /\bid="[^"{]+"/, say: 'a hard-coded id: two panels can show the same card, use useId()' },
  {
    // FR-099: colours come from the theme and stage tokens (STYLE_RULE §4, "The stage's own
    // colours"); a print's own colour, named in a `swatch`, is the exception.
    test: /^(?!.*swatch).*\b(text|bg|border|ring|fill|stroke|from|via|to|outline)-(red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone|white|black)(-\d+)?(\/\d+)?\b/,
    everywhere: true,
    say: 'a raw palette colour: use a theme token (primary, muted-foreground, warning, success, destructive) or a stage token (bg-stage-chip, text-stage-ink…)',
    except: {
      'features/about/report-issue.tsx': 'white on the destructive red of its button',
      'features/window/title-bar.tsx': 'white on red under the close button, as the system draws it',
    },
  },
  {
    test: /\b(min-|max-)?[hw]-\[\d+(\.\d+)?px\]|\btext-\[\d+(\.\d+)?(px|rem)\]/,
    everywhere: true,
    say: 'an arbitrary size: use the spacing scale or a token (text-2xs, w-panel)',
    except: {
      'features/window/title-bar.tsx': "the Windows caption buttons' own 46 px width",
      'features/simple/simple-view.tsx': "its bar matches the gallery toolbar's 49 px",
      'features/collage/collage-view.tsx': "its bar matches the gallery toolbar's 49 px",
      'features/library/selection-bar.tsx': 'its bar sits in the gallery toolbar, 45 px inside its border',
    },
  },
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
  // An exempt file may space itself, but colours and sizes hold everywhere (`everywhere`).
  const exempt = rel in EXEMPT
  readFileSync(path, 'utf8')
    .split('\n')
    .forEach((line, i) => {
      for (const rule of RULES) {
        if (exempt && !rule.everywhere) continue
        if (rule.test.test(line) && !(rule.except && rel in rule.except)) problems.push(`${rel}:${i + 1}: ${rule.say}`)
      }
    })
}

// Red for removing (owner, 2026-11-06; STYLE_RULE §4 "Red"): a Button whose icon deletes,
// clears, removes or stops (✕, a bin, a cleared filter) is `destructive`: deep red, white
// ink, icon-only ones too. A dismiss (`common.dismiss`) only hides
// a note, so it stays neutral; the title bar's close is a plain <button> the system styles.
const REMOVE_ICON = /<(X|XIcon|Trash2|FilterX|Eraser|CircleX)\b/
const RED = /variant="destructive"/
const allTsx = []
const walkAll = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) walkAll(path)
    else if (path.endsWith('.tsx')) allTsx.push(path)
  }
}
walkAll(join(root, 'src'))
for (const path of allTsx) {
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  const text = readFileSync(path, 'utf8')
  for (const m of text.matchAll(/<Button\b[\s\S]*?<\/Button>/g)) {
    const block = m[0]
    if (REMOVE_ICON.test(block) && !RED.test(block) && !block.includes("'common.dismiss'")) {
      const line = text.slice(0, m.index).split('\n').length
      problems.push(`${rel}:${line}: a button that removes is red: variant="destructive"`)
    }
  }
}

// Where each card shows (STYLE_RULE §4): its file says `@shown both`, `gallery`, `preview` or
// `settings` (the Settings window, FR-086), and the places below may only hold the cards their
// flag allows.
const PLACES = {
  'features/settings/settings-panel.tsx': 'gallery',
  'features/simple/simple-panel.tsx': 'gallery',
  'features/collage/collage-panel.tsx': 'gallery',
  'features/viewer/preview-dialog.tsx': 'preview',
  'features/collage/sheet-preview-dialog.tsx': 'preview',
  'features/settings-window/settings-window.tsx': 'settings',
}
const shownIn = new Map()
for (const [panel, place] of Object.entries(PLACES)) {
  const text = readFileSync(join(root, 'src', panel), 'utf8')
  const imports = new Map()
  for (const m of text.matchAll(/import \{([^}]*)\} from ['"](?:@\/|\.\/)([^'"]+)['"]/g)) {
    const from = m[2].startsWith('features/') || m[2].startsWith('components/') ? m[2] : `${panel.replace(/[^/]+$/, '')}${m[2]}`
    for (const name of m[1].split(',').map((n) => n.trim()))
      if (name) imports.set(name, from)
  }
  for (const m of new Set([...text.matchAll(/<([A-Z]\w*(?:Card|Cards|Section))\b/g)].map((m) => m[1]))) {
    const from = imports.get(m)
    // The shared pieces (SectionCard…) live in components and show anywhere.
    if (!from || !from.startsWith('features/')) continue
    const source = readFileSync(join(root, 'src', `${from}.tsx`), 'utf8')
    const flag = /@shown (both|gallery|preview|settings)\b/.exec(source)?.[1]
    if (!flag) problems.push(`${from}.tsx: no @shown flag (both, gallery, preview or settings), and ${panel} shows it`)
    else if (flag !== 'both' && flag !== place)
      problems.push(`${panel}: shows ${m}, flagged @shown ${flag}, in the ${place}`)
    shownIn.set(m, flag)
  }
}

if (problems.length) {
  console.error(problems.join('\n'))
  console.error(`check-lego: ${problems.length} problem(s); see STYLE_RULE §4 (the lego rule)`)
  process.exit(1)
}
console.log(`check-lego: ${files.length} files, every card built from the shared pieces; ${shownIn.size} cards in their place`)
