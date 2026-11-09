// SPDX-License-Identifier: GPL-3.0-only
// The UI test cases that a machine can judge (docs/UI_TESTS.md): each detector runs in the
// page and returns failures that start with their test ID, so a failure leads to its row and
// its rule. shoot.mjs runs them in every view it opens under MEASURE=1.
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

/**
 * The layout, text, state and target-size cases (UI-LAYOUT-001/002, UI-TEXT-001,
 * UI-STATE-001, UI-A11Y-001), in one pass over the page.
 */
export async function layoutChecks(page, where) {
  const found = await page.evaluate(() => {
    const out = []
    const vis = (el) => {
      const b = el.getBoundingClientRect()
      if (b.width < 0.5 || b.height < 0.5) return false
      for (let e = el; e; e = e.parentElement) {
        const cs = getComputedStyle(e)
        if (cs.visibility === 'hidden' || cs.display === 'none' || Number(cs.opacity) === 0) return false
      }
      return true
    }
    const name = (el) => {
      const words = (el.getAttribute('aria-label') || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 40)
      if (words) return words
      // No words: say what it is and where, so the failure can be found.
      const slot = el.dataset.slot ?? el.getAttribute('role') ?? el.className.toString().split(' ')[0]
      const near = el.closest('[aria-label], section')?.getAttribute('aria-label') ?? el.closest('section')?.querySelector('header')?.textContent?.trim()
      return `${el.tagName.toLowerCase()}[${slot}] in ${near ?? 'the page'}`.slice(0, 80)
    }
    // Inside a box that scrolls sideways on purpose (a toolbar row), a control may pass its card.
    const scrollsSideways = (el, stop) => {
      for (let e = el.parentElement; e && e !== stop; e = e.parentElement) {
        const x = getComputedStyle(e).overflowX
        if (x === 'auto' || x === 'scroll') return true
      }
      return false
    }
    const CONTROLS =
      'button, [role="button"], [role="combobox"], [role="slider"], [role="switch"], [role="checkbox"], input:not([type="hidden"]), textarea, a[href], [data-slot="badge"], label'

    // UI-LAYOUT-001: every control inside its card, panel or dialog, left and right.
    for (const el of document.querySelectorAll(CONTROLS)) {
      if (!vis(el) || el.closest('[data-overlap-ok]')) continue
      const box = el.closest('section, aside, [role="dialog"]')
      if (!box || scrollsSideways(el, box)) continue
      const b = el.getBoundingClientRect()
      const c = box.getBoundingClientRect()
      const past = Math.max(c.left - b.left, b.right - c.right)
      if (past > 0.5) out.push(`UI-LAYOUT-001 a control ${Math.round(past)} px outside its ${box.tagName.toLowerCase()}: "${name(el)}"`)
    }

    // UI-LAYOUT-002: no two controls on top of each other (nested ones aside).
    // A target counts where a click would reach it: on top at its centre (not under an open
    // dialog) and inside every box that clips it (not scrolled out of its panel).
    const reachable = (el) => {
      const r = el.getBoundingClientRect()
      const [x, y] = [r.left + r.width / 2, r.top + r.height / 2]
      if (x < 0 || y < 0 || x > innerWidth || y > innerHeight) return false
      for (let e = el.parentElement; e; e = e.parentElement) {
        const cs = getComputedStyle(e)
        if (cs.overflowX !== 'visible' || cs.overflowY !== 'visible') {
          const c = e.getBoundingClientRect()
          if (x < c.left || x > c.right || y < c.top || y > c.bottom) return false
        }
      }
      const hit = document.elementFromPoint(x, y)
      return !!hit && (hit === el || el.contains(hit) || hit.contains(el))
    }
    const targets = [...document.querySelectorAll('button, [role="button"], [role="combobox"], [role="slider"], [role="switch"], [role="checkbox"], input:not([type="hidden"]), textarea, a[href]')]
      .filter((el) => vis(el) && !el.closest('[data-overlap-ok]') && reachable(el))
    const rects = targets.map((el) => el.getBoundingClientRect())
    for (let i = 0; i < targets.length; i++) {
      for (let j = i + 1; j < targets.length; j++) {
        const [a, b] = [targets[i], targets[j]]
        if (a.contains(b) || b.contains(a)) continue
        // A popover or a dialog lies over the page by design: compare within one layer only.
        const layer = (el) => el.closest('[data-radix-popper-content-wrapper], [role="dialog"]')
        if (layer(a) !== layer(b)) continue
        const [p, q] = [rects[i], rects[j]]
        const w = Math.min(p.right, q.right) - Math.max(p.left, q.left)
        const h = Math.min(p.bottom, q.bottom) - Math.max(p.top, q.top)
        if (w > 1 && h > 1) out.push(`UI-LAYOUT-002 two controls overlap (${Math.round(w)}×${Math.round(h)} px): "${name(a)}" and "${name(b)}"`)
      }
    }

    // UI-TEXT-001: text cut short (ellipsis or line clamp) carries its whole in a hint.
    for (const el of document.querySelectorAll('*')) {
      if (el.children.length > 0 && !el.textContent.trim()) continue
      const cs = getComputedStyle(el)
      const clamps = cs.textOverflow === 'ellipsis' || (cs.webkitLineClamp && cs.webkitLineClamp !== 'none')
      if (!clamps || !vis(el)) continue
      const cut = el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1
      if (!cut || el.closest('[data-truncate-ok]')) continue
      let hinted = false
      for (let e = el, i = 0; e && i < 4; e = e.parentElement, i++) if (e.dataset.hint || e.title) hinted = true
      if (!hinted) out.push(`UI-TEXT-001 text cut with no hint giving it whole: "${name(el)}"`)
    }

    // UI-STATE-001: a disabled button's own hint never shows (no pointer events): on a wrapper.
    for (const el of document.querySelectorAll('button:disabled[data-hint]')) {
      if (vis(el)) out.push(`UI-STATE-001 a disabled button holds its own hint, which cannot show: "${name(el)}"`)
    }

    // UI-A11Y-001: a target at least 24 px, or its 24 px circle clear of every other target
    // (WCAG 2.2, 2.5.8).
    const centre = (r) => [r.left + r.width / 2, r.top + r.height / 2]
    targets.forEach((el, i) => {
      const r = rects[i]
      if (r.width >= 24 && r.height >= 24) return
      const [cx, cy] = centre(r)
      const clash = targets.some((other, j) => {
        if (j === i || other.contains(el) || el.contains(other)) return false
        const q = rects[j]
        const nx = Math.max(q.left, Math.min(cx, q.right))
        const ny = Math.max(q.top, Math.min(cy, q.bottom))
        return Math.hypot(cx - nx, cy - ny) < 12
      })
      if (clash) out.push(`UI-A11Y-001 a target under 24 px too close to another (${Math.round(r.width)}×${Math.round(r.height)}): "${name(el)}"`)
    })

    // UI-LAYOUT-016: what a control draws outside its box is not cut by a box that clips: its
    // ring (a crisp box-shadow, as `ring-*` and `ring-offset-*` draw) and its outline. Soft
    // shadows are left out (a stage clips a photo's on purpose); a control scrolled partly out
    // of its box is left out. (Owner, 2026-11-08: the filmstrip's chosen ring cut, again.)
    const reach = (cs) => {
      let r = 0
      for (const s of cs.boxShadow === 'none' ? [] : cs.boxShadow.split(/,(?![^(]*\))/)) {
        if (/inset/.test(s)) continue
        const n = (s.replace(/(rgba?|oklch|oklab|color|lab|lch|hsla?)\([^)]*\)/g, '').match(/-?[\d.]+px/g) ?? []).map(parseFloat)
        const [x = 0, y = 0, blur = 0, spread = 0] = n
        if (x === 0 && y === 0 && blur === 0 && spread > 0) r = Math.max(r, spread)
      }
      if (cs.outlineStyle !== 'none') r = Math.max(r, parseFloat(cs.outlineWidth) + Math.max(0, parseFloat(cs.outlineOffset)))
      return r
    }
    for (const el of document.querySelectorAll(`${CONTROLS}, [aria-current]`)) {
      if (!vis(el)) continue
      const r = reach(getComputedStyle(el))
      if (r < 1) continue
      const b = el.getBoundingClientRect()
      for (let e = el.parentElement; e && e !== document.body; e = e.parentElement) {
        const cs = getComputedStyle(e)
        if (cs.overflowX === 'visible' && cs.overflowY === 'visible') continue
        const c = e.getBoundingClientRect()
        const top = c.top + e.clientTop
        const left = c.left + e.clientLeft
        const bottom = top + e.clientHeight
        const right = left + e.clientWidth
        // Scrolled partly out of this box: not a cut ring, a scroll.
        if (b.left < left - 0.5 || b.right > right + 0.5 || b.top < top - 0.5 || b.bottom > bottom + 0.5) break
        const cut = Math.max(top - (b.top - r), b.bottom + r - bottom, cs.overflowX === 'visible' ? 0 : left - (b.left - r), cs.overflowX === 'visible' ? 0 : b.right + r - right)
        if (cut > 0.5) {
          out.push(`UI-LAYOUT-016 a ring cut by ${Math.round(cut)} px by the ${e.tagName.toLowerCase()} that clips it: "${name(el)}"`)
          break
        }
      }
    }

    // UI-LAYOUT-017: a scroll area scrolls inside its own box, never spills out of it over what
    // follows (owner, 2026-11-08: the look list under the hint; a `max-h` on the root did not
    // reach the viewport, which grew with its rows).
    for (const root of document.querySelectorAll('[data-slot="scroll-area"]')) {
      const view = root.querySelector(':scope > [data-slot="scroll-area-viewport"]')
      if (!view || !vis(root)) continue
      const r = root.getBoundingClientRect()
      const v = view.getBoundingClientRect()
      const spill = Math.max(v.bottom - r.bottom, v.right - r.right)
      if (spill > 0.5) out.push(`UI-LAYOUT-017 a scroll area spills ${Math.round(spill)} px out of its box: "${name(root)}"`)
    }

    // UI-LAYOUT-018: an overlay scroll bar never sits on words (owner, 2026-11-08: a group's
    // count against the bar). Radix draws the bar only under the pointer, so the room is
    // checked wherever a list does scroll: its words end at least the bar's 8 px and 4 px more
    // before the area's right edge.
    for (const area of document.querySelectorAll('[data-slot="scroll-area"]')) {
      const view = area.querySelector(':scope > [data-slot="scroll-area-viewport"]')
      if (!view || !vis(area) || view.scrollHeight <= view.clientHeight + 1) continue
      const edge = area.getBoundingClientRect().right - 12
      const walker = document.createTreeWalker(view, NodeFilter.SHOW_TEXT)
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent?.trim() || !n.parentElement || !vis(n.parentElement)) continue
        const range = document.createRange()
        range.selectNodeContents(n)
        const r = range.getBoundingClientRect()
        // Words cut with an ellipsis end where their clipping box does, not where the whole
        // string would.
        let right = r.right
        for (let e = n.parentElement; e && e !== view; e = e.parentElement) {
          if (getComputedStyle(e).overflowX !== 'visible') right = Math.min(right, e.getBoundingClientRect().right)
        }
        if (r.width > 0 && right > edge + 0.5) {
          out.push(`UI-LAYOUT-018 words ${Math.round(right - edge)} px into a scroll bar's room: "${n.textContent.trim().slice(0, 30)}"`)
          break
        }
      }
    }

    // UI-LAYOUT-019: a dialog's footer keeps clear of what is above it, at least 12 px from
    // its line (owner, 2026-11-08: a name field touched the footer's line).
    for (const foot of document.querySelectorAll('[data-slot="dialog-footer"], [data-slot="alert-dialog-footer"]')) {
      const above = foot.previousElementSibling
      if (!above || !vis(foot) || !vis(above)) continue
      const gap = foot.getBoundingClientRect().top - above.getBoundingClientRect().bottom
      if (gap < 11.5) out.push(`UI-LAYOUT-019 a dialog's footer ${Math.round(gap)} px under what is above it: "${name(above)}"`)
    }

    // UI-LAYOUT-020: the window's own bar takes the pointer while a dialog is open, so the
    // window can be moved, resized or closed (owner, 2026-11-08: Radix sets pointer-events
    // none on <body> for a modal, and the bar inherited it).
    const chrome = document.querySelector('[data-window-chrome]')
    if (chrome && document.querySelector('[role="dialog"], [role="alertdialog"]') && getComputedStyle(chrome).pointerEvents === 'none')
      out.push('UI-LAYOUT-020 the window bar takes no pointer while a dialog is open: the window cannot be moved')

    // UI-LAYOUT-021: an open dropdown sits whole inside the window, under its bar, and beside
    // its button, never over it (owner, 2026-11-08: the print size list lost its top when a
    // size low in the list was chosen; Radix's item-aligned lists open over their button).
    const bar = document.querySelector('[data-window-chrome]')?.getBoundingClientRect().bottom ?? 0
    for (const list of document.querySelectorAll('[data-slot="select-content"], [data-slot="popover-content"]')) {
      if (!vis(list)) continue
      const b = list.getBoundingClientRect()
      if (b.top < bar - 0.5 || b.bottom > window.innerHeight + 0.5 || b.left < -0.5 || b.right > window.innerWidth + 0.5)
        out.push(`UI-LAYOUT-021 an open dropdown runs out of the window (${Math.round(b.top)}–${Math.round(b.bottom)} of ${window.innerHeight}, bar ${Math.round(bar)})`)
      const button = document.querySelector('[data-slot="select-trigger"][data-state="open"], [data-slot="popover-trigger"][data-state="open"]')
      const t = button?.getBoundingClientRect()
      if (t && b.top < t.bottom - 0.5 && b.bottom > t.top + 0.5 && b.left < t.right && b.right > t.left)
        out.push(`UI-LAYOUT-021 an open dropdown covers its own button: "${name(button)}"`)
    }

    // UI-LAYOUT-022: a dialog's words keep 8 px clear of its close button (owner, 2026-11-08:
    // "the words nearly touch the ✕"; the header spanned the width under the button).
    for (const close of document.querySelectorAll('[data-slot="dialog-close"]')) {
      if (!vis(close)) continue
      const c = close.getBoundingClientRect()
      const dialog = close.closest('[role="dialog"], [role="alertdialog"]')
      if (!dialog) continue
      const walker = document.createTreeWalker(dialog, NodeFilter.SHOW_TEXT)
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent?.trim() || !n.parentElement || close.contains(n) || !vis(n.parentElement)) continue
        const range = document.createRange()
        range.selectNodeContents(n)
        // Words cut with an ellipsis end where their clipping box does (as UI-LAYOUT-018).
        let clip = Infinity
        for (let e = n.parentElement; e && e !== dialog; e = e.parentElement) {
          if (getComputedStyle(e).overflowX !== 'visible') clip = Math.min(clip, e.getBoundingClientRect().right)
        }
        for (const r of range.getClientRects()) {
          const right = Math.min(r.right, clip)
          if (right - r.left < 1) continue
          if (right > c.left - 8 && r.left < c.right + 8 && r.bottom > c.top - 8 && r.top < c.bottom + 8) {
            out.push(`UI-LAYOUT-022 words within 8 px of a dialog's close button: "${n.textContent.trim().slice(0, 30)}"`)
            break
          }
        }
      }
    }

    // UI-LAYOUT-024: a badge is seen whole, never cut by a box that clips it (owner,
    // 2026-11-09: the print size's verdict cut off inside its button on Windows).
    for (const badge of document.querySelectorAll('[data-slot="badge"]')) {
      if (!vis(badge)) continue
      const b = badge.getBoundingClientRect()
      for (let e = badge.parentElement; e; e = e.parentElement) {
        const cs = getComputedStyle(e)
        if (cs.overflowX === 'visible' && cs.overflowY === 'visible') continue
        const c = e.getBoundingClientRect()
        if (b.right > c.right + 0.5 || b.left < c.left - 0.5) {
          out.push(`UI-LAYOUT-024 a badge cut off by its box: "${name(badge)}"`)
        }
        break
      }
    }

    // UI-LAYOUT-011: the page never scrolls sideways.
    if (document.documentElement.scrollWidth > window.innerWidth + 1) out.push(`UI-LAYOUT-011 the page scrolls sideways (${document.documentElement.scrollWidth} > ${window.innerWidth})`)
    return out
  })
  return [...new Set(found)].map((f) => `${where}: ${f}`)
}

/**
 * Contrasts the owner kept on purpose (docs/UI_TESTS.md §4): the logo's teal under the cream
 * text of a filled button in the light theme, 4.41:1 (owner, 2026-11-07: "keep #2d836d").
 */
const ALLOWED_CONTRAST = [['#fffaee', '#2d836d']]

/**
 * WCAG 2.2 A and AA by axe-core (UI-A11Y-002; ADR-0042: a test-only tool). Each place that
 * breaks a rule is one line.
 */
export async function axeChecks(page, where) {
  const { AxeBuilder } = require('@axe-core/playwright')
  const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze()
  // Each place on its own line, with axe's reason (a contrast's colours and ratio), less the
  // exceptions the owner decided (docs/UI_TESTS.md §4).
  return result.violations.flatMap((v) =>
    v.nodes
      .map((n) => ({ n, why: (n.any[0] ?? n.all[0] ?? n.none[0])?.message ?? v.help }))
      .filter(({ why }) => !(v.id === 'color-contrast' && ALLOWED_CONTRAST.some(([fg, bg]) => why.includes(`foreground color: ${fg}`) && why.includes(`background color: ${bg}`))))
      .map(({ n, why }) => `${where}: UI-A11Y-002 axe ${v.id} (${v.impact}): ${n.target.join(' ')} — ${why}`),
  )
}

/**
 * Nothing jumps (UI-STABLE-001): the boxes named by `selector` keep their place while
 * `act` runs, as STYLE_RULE §4 asks of a first slider step, W, J or a note that comes.
 */
export async function stableChecks(page, where, selector, act, label) {
  const boxes = () =>
    page.evaluate((sel) => [...document.querySelectorAll(sel)].map((e) => {
      const b = e.getBoundingClientRect()
      return [Math.round(b.top), Math.round(b.height)]
    }), selector)
  const before = await boxes()
  await act()
  await page.waitForTimeout(400)
  const after = await boxes()
  const moved = before.findIndex((b, i) => !after[i] || Math.abs(after[i][0] - b[0]) > 1)
  return moved >= 0 ? [`${where}: UI-STABLE-001 "${label}" moved box ${moved + 1} of ${selector} by ${after[moved] ? after[moved][0] - before[moved][0] : '?'} px`] : []
}

/**
 * Pixel baselines (UI-VISUAL-001): an element's picture against the one approved before, per
 * engine, language, theme and width. `UPDATE_BASELINES=1` approves the current pictures; a
 * difference writes `<name>-diff.png` beside the output for review.
 */
export async function baseline(page, locator, name, { dir, outDir, tolerance = 0.002 }) {
  const { PNG } = require('pngjs')
  const pixelmatch = (await import('pixelmatch')).default
  const file = path.join(dir, `${name}.png`)
  // The same picture each run: every scrolling box inside at its top, the pointer away.
  await locator.evaluate((root) => {
    for (const el of [root, ...root.querySelectorAll('*')]) if (el.scrollTop) el.scrollTop = 0
  })
  await page.mouse.move(0, 0)
  await page.waitForTimeout(200)
  const shot = await locator.screenshot({ animations: 'disabled', caret: 'hide', scale: 'css' })
  if (process.env.UPDATE_BASELINES || !existsSync(file)) {
    await mkdir(dir, { recursive: true })
    await writeFile(file, shot)
    return []
  }
  const a = PNG.sync.read(await readFile(file))
  const b = PNG.sync.read(shot)
  if (a.width !== b.width || a.height !== b.height) {
    return [`UI-VISUAL-001 ${name}: size ${b.width}×${b.height}, approved ${a.width}×${a.height}`]
  }
  const diff = new PNG({ width: a.width, height: a.height })
  const changed = pixelmatch(a.data, b.data, diff.data, a.width, a.height, { threshold: 0.1 })
  if (changed / (a.width * a.height) <= tolerance) return []
  await writeFile(path.join(outDir, `${name}-diff.png`), PNG.sync.write(diff))
  return [`UI-VISUAL-001 ${name}: ${changed} px changed (${((changed / (a.width * a.height)) * 100).toFixed(2)} %), see ${name}-diff.png`]
}
