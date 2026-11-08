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
  'features/library/thumb-tile.tsx': 'a grid tile, its badges sized to the thumbnail',
  'features/viewer/crop-editor.tsx': 'the crop overlay on the photo, not a panel',
  'features/places/borrow-dialog.tsx': 'a dialog, its own layout',
  'features/best-shot/compare-dialog.tsx': 'a dialog, its own layout',
  'features/best-shot/group-strip.tsx': 'a strip of thumbnails, its badges sized to them',
  'features/library/raw-badge.tsx': 'a badge, sized to sit on a thumbnail',
  'features/best-shot/groups-view.tsx': 'the stage listing the groups, not a panel',
}

const RULES = [
  {
    test: /<ToggleGroup\b|import \{[^}]*\bToggleGroup\b/,
    except: {
      'components/forms/segmented-toggle.tsx': 'the block itself',
      'features/modes/mode-switch.tsx': "two rows, Simple and Collage then Pro the whole width (owner, 2026-11-06), which SegmentedToggle's one row cannot draw",
    },
    say: 'a hand-made toggle group: use SegmentedToggle or SwatchToggle (components/forms)',
  },
  {
    test: /<InputGroup\b/,
    say: 'a field with its own frame: every text field is an Input (the manual\'s search, FR-110: InputGroup drew a second box)',
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
    test: /^(?!.*swatch).*\b(text|bg|border|ring|fill|stroke|from|via|to|outline|accent)-(red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone|white|black)(-\d+)?(\/\d+)?\b/,
    everywhere: true,
    say: 'a raw palette colour: use a theme token (primary, muted-foreground, warning, success, destructive) or a stage token (bg-stage-chip, text-stage-ink…)',
    except: {
      'features/window/title-bar.tsx': 'white on red under the close button, as the system draws it',
      'components/layout/panel-note.tsx': "a danger note's white on its deep red, as the red badges'",
    },
  },
  {
    test: /\b(min-|max-)?[hw]-\[\d+(\.\d+)?px\]|\btext-\[\d+(\.\d+)?(px|rem)\]/,
    everywhere: true,
    say: 'an arbitrary size: use the spacing scale or a token (text-2xs, w-panel)',
    except: {
      'features/window/title-bar.tsx': "the Windows caption buttons' own 46 px width",
    },
  },
  {
    // One look per meaning (owner, 2026-11-07: "no button in a style of its own"): a link
    // styled as a button is no block of ours; an action is a Button or a ToolbarButton.
    test: /variant="link"/,
    say: 'a link-styled button: use a Button (outline, sm) or a ToolbarButton',
    except: {
    },
  },
  {
    // One chosen style for every list, menu and dropdown (owner, 2026-11-06).
    test: /\brow-chosen\b/,
    everywhere: true,
    say: 'a chosen row comes from ListRow or CHOSEN_ROW (components/forms/list-row.tsx), never by hand',
    except: {
      'components/forms/list-row.tsx': 'where CHOSEN_ROW is defined',
    },
  },
  {
    // Audit 2026-11-07: colours written by hand slipped past the palette rule.
    test: /(rgb|rgba|hsl|oklch)\(/,
    everywhere: true,
    say: 'a colour written by hand: add a token in index.css (as --stage-dim, --hist-r) and use it',
  },
  {
    // One height for every control (STYLE_RULE §4): a 24 px icon button only inside a line,
    // which RemoveButton small is; a button with words in a row's value (*Choose…*) may be xs.
    test: /size="icon-xs"/,
    everywhere: true,
    say: 'an icon button under 32 px: every control is 32 px (only RemoveButton small sits inside a line)',
    except: {
      'components/forms/danger-button.tsx': 'RemoveButton small, the ✕ inside a line or a tab',
    },
  },
  {
    test: /\bhtmlFor="[^"{]+"/,
    everywhere: true,
    say: 'a hard-coded htmlFor: two panels can show the same card, use useId()',
  },
  {
    // Audit 2026-11-07: `variant={on ? 'ghost' : …}` escaped the literal checks below.
    test: /variant=\{[^}]*'(ghost|link)'/,
    everywhere: true,
    say: 'a variant chosen by an expression that can be ghost or link: a standalone control is raised (ToolbarButton)',
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
// Audit 2026-11-07: a check covers only what it reads. The app's shell is a panel like any
// other; the blocks themselves set the spacing, so only the rules that hold everywhere apply.
files.push(join(root, 'src/app.tsx'))
const blocks = []
for (const dir of ['components/forms', 'components/layout']) {
  for (const name of readdirSync(join(root, 'src', dir))) if (name.endsWith('.tsx')) blocks.push(join(root, 'src', dir, name))
}
files.push(...blocks)

const problems = []
for (const path of files) {
  // Forward slashes on every system, as the exception list writes them.
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  // An exempt file may space itself, but colours and sizes hold everywhere (`everywhere`).
  const exempt = rel in EXEMPT || rel.startsWith('components/')
  readFileSync(path, 'utf8')
    .split('\n')
    .forEach((line, i) => {
      for (const rule of RULES) {
        if (exempt && !rule.everywhere) continue
        if (rule.test.test(line) && !(rule.except && rel in rule.except)) problems.push(`${rel}:${i + 1}: ${rule.say}`)
      }
    })
}

// Red for removing, one block (owner, 2026-11-06: "every button reused, one class"; STYLE_RULE
// §4 "Red"): a button that removes, deletes, clears, stops or resets is `RemoveButton` or
// `ResetButton` (components/forms/danger-button.tsx), never a hand-made <Button>: no
// `variant="destructive"` and no ✕, bin, cleared filter or ↺ icon inside a <Button>. A ↺ that
// is a choice (`aria-pressed`) is not a removal; a dismiss that hides a note is one (owner,
// 2026-11-06: "this ✕ forgot its red"); the title bar's close is a plain <button> the system styles.
const REMOVE_ICON = /<(X|XIcon|Trash2|FilterX|Eraser|CircleX|RotateCcw)\b/
const OWN_LOOK = new Set(['components/forms/danger-button.tsx'])
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
  if (rel.startsWith('components/ui/') || OWN_LOOK.has(rel)) continue
  const text = readFileSync(path, 'utf8')
  for (const m of text.matchAll(/<Button\b[\s\S]*?(<\/Button>|\/>)/g)) {
    const block = m[0]
    const removal = REMOVE_ICON.test(block) && !block.includes('aria-pressed')
    if (removal || block.includes('variant="destructive"')) {
      const line = text.slice(0, m.index).split('\n').length
      problems.push(`${rel}:${line}: a button that removes or resets is RemoveButton or ResetButton (components/forms/danger-button.tsx), never a hand-made one`)
    }
  }
}

// SECURITY.md §7, as CI's security job greps it (LESSONS: v0.13.2's release stopped on a comment
// that named one): no HTML-writing DOM API, not even in a comment, in any .ts or .tsx but ipc/.
const DANGEROUS = /dangerouslySetInnerHTML|innerHTML|outerHTML|insertAdjacentHTML|document\.write|eval\(|new Function/
const walkCode = (dir, out = []) => {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) { if (name !== 'ipc') walkCode(p, out) } else if (/\.tsx?$/.test(name)) out.push(p)
  }
  return out
}
// An app, not a page (owner, 2026-11-08): text is selectable only through `Selectable`
// (components/forms/field.tsx) and fields; the web view's menu is refused in lib/no-web.ts only.
const SELECT = /\bselect-(text|all)\b|user-select\s*:/
const CONTEXT = /addEventListener\(\s*['"]contextmenu/
// One motion system (FR-100, docs/TRANS_ANIMATION.md): every pace and curve is a token of
// index.css, taken through its utilities (`motion-control`, `motion-move`, `motion-fade`,
// `motion-fold`, `motion-in`/`-out`) or as `duration-(--motion-*)`, `delay-(--delay-*)`. Refused:
// a pace, delay or curve by number or keyword, `transition-*` (each sets the default 150 ms),
// `motion-reduce:`/`motion-safe:` (reduced motion is the tokens' job), a smooth scroll that
// ignores it, and the pulse outside the one skeleton.
const MOTION = [
  [/\b(duration|delay)-\d+\b/, 'a pace or delay by number: use a motion token (duration-(--motion-*), delay-(--delay-*))'],
  [/\bease-(in|out|in-out|linear)\b/, 'a curve by keyword: the motion utilities carry ease-enter, ease-exit or ease-move'],
  // `transition-none` stays: what follows the pointer (a dragged row) must not lag.
  [/(^|[\s'"`])transition(-(?!none\b)[a-z[][^\s'"`]*)?(?=[\s'"`]|$)/, 'a transition by hand: use motion-control, motion-move, motion-fade or motion-fold'],
  [/\bmotion-(reduce|safe):/, 'reduced motion by hand: the tokens shorten every pace and drop slides (index.css, lib/motion.ts)'],
  [/behavior:\s*['"]smooth['"]/, "a smooth scroll that ignores reduced motion: reducedMotion() ? 'auto' : 'smooth' (lib/motion.ts)"],
]
const PULSE_OK = new Set(['features/library/thumb-tile.tsx'])
const isComment = (line) => /^\s*(\/\/|\/\*|\*|\{\/\*)/.test(line)
for (const path of walkCode(join(root, 'src'))) {
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  readFileSync(path, 'utf8').split('\n').forEach((line, i) => {
    if (!isComment(line)) {
      for (const [test, say] of MOTION) if (test.test(line)) problems.push(`${rel}:${i + 1}: ${say} (FR-100)`)
      if (/\banimate-spin\b/.test(line) && rel !== 'components/forms/spinner.tsx')
        problems.push(`${rel}:${i + 1}: a spinner by hand: use Spinner (components/forms/spinner.tsx) (FR-100)`)
      if (/\banimate-pulse\b/.test(line) && !PULSE_OK.has(rel))
        problems.push(`${rel}:${i + 1}: a pulse outside the thumbnail's skeleton: a wait shows a Spinner or a skeleton block (FR-100)`)
    }
    if (DANGEROUS.test(line)) problems.push(`${rel}:${i + 1}: a dangerous DOM API named (SECURITY.md §7; CI refuses it, even in a comment)`)
    if (SELECT.test(line) && rel !== 'components/forms/field.tsx')
      problems.push(`${rel}:${i + 1}: selectable text by hand: use Selectable (an app selects only fields, facts and error codes)`)
    if (CONTEXT.test(line) && rel !== 'lib/no-web.ts')
      problems.push(`${rel}:${i + 1}: a context menu listener outside lib/no-web.ts (an element's own menu is onContextMenu)`)
  })
}

// Cancel is red, the stop block (owner, 2026-11-08: "Cancel should be red"): `RemoveButton
// kind="stop"`, as the crop's and the export's. A dialog whose own action is red (*Don't save*,
// *Discard*) keeps a neutral `AlertDialogCancel`, so one red stays the danger.
for (const path of allTsx) {
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  if (rel.startsWith('components/ui/')) continue
  const text = readFileSync(path, 'utf8')
  for (const m of text.matchAll(/<Button\b[^<]*?>\s*\{t\('common\.cancel'\)\}/g)) {
    const line = text.slice(0, m.index).split('\n').length
    problems.push(`${rel}:${line}: a Cancel by hand: RemoveButton kind="stop" (components/forms/danger-button.tsx)`)
  }
}

// One size for every pop-up dialog (owner, 2026-11-08: "every pop-up one size, roomy"): 512 px,
// set in components/ui/dialog.tsx and alert-dialog.tsx. Only a dialog that is a screen of
// its own (photos side by side) sizes itself, here with its reason.
const OWN_SIZE = {
  'features/places/borrow-dialog.tsx': 'two photos side by side per place',
  'features/cameras/clock-match-dialog.tsx': 'a grid of photos to match a clock',
  'features/best-shot/compare-dialog.tsx': 'photos compared at full size',
}
for (const path of allTsx) {
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  if (rel.startsWith('components/') || rel in OWN_SIZE) continue
  const text = readFileSync(path, 'utf8')
  for (const m of text.matchAll(/<(Alert)?DialogContent\b[^>]*\bmax-w-/g)) {
    const line = text.slice(0, m.index).split('\n').length
    problems.push(`${rel}:${line}: a pop-up of its own size: every pop-up dialog is the one size of components/ui (512 px)`)
  }
}

// Raised unless it belongs to something (owner, 2026-11-06: "Select all is still the old
// style"): a standalone button is never `ghost`. Flat stays only for a row of a list
// (ListRow), a button inside a raised ButtonGroup, a dismiss inside a note and an on/off
// choice (`aria-pressed`).
const GHOST_OK = {
  'components/forms/list-row.tsx': 'the rows of lists and menus',
  'features/library/library-sidebar.tsx': 'its icon buttons sit inside a raised ButtonGroup',
  'features/capture/connect-camera-button.tsx': "it sits inside the Library row's ButtonGroup",
}
for (const path of allTsx) {
  const rel = relative(join(root, 'src'), path).split(sep).join('/')
  if (rel.startsWith('components/ui/') || rel in GHOST_OK) continue
  const text = readFileSync(path, 'utf8')
  for (const m of text.matchAll(/<Button\b[\s\S]*?(<\/Button>|\/>)/g)) {
    const block = m[0]
    if (block.includes('variant="ghost"') && !block.includes('aria-pressed')) {
      const line = text.slice(0, m.index).split('\n').length
      problems.push(`${rel}:${line}: a standalone button is raised (outline, secondary or default), never ghost`)
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
