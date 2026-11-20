// SPDX-License-Identifier: GPL-3.0-only
// The right columns beyond Pro's gallery panel, each measured on its own page (STYLE_RULE §4):
// Simple mode's panel, and RollSense's (FR-141), Pro's third tab, its facets as cards. Split out
// of shoot.mjs (red in CODE_HEALTH, FR-139): shoot.mjs hands over the helpers it owns.
import path from 'node:path'

/** IPC answers for RollSense's calendar and the home time, run in the page before the bridge,
 * which takes them in (`window.__senseAnswers`). */
export function senseAnswers() {
  const cell = (d, month) => ({ day: d, date: new Date(d * 864e5).getUTCDate(), inMonth: new Date(d * 864e5).getUTCMonth() === month - 1 })
  window.__senseAnswers = {
    // The calendar's month (FR-141 C1), laid out as the core's `month_grid` does.
    month_grid: ({ year, month, firstWeekday }) => {
      const first = Date.UTC(year, month - 1, 1) / 864e5
      const start = first - ((new Date(first * 864e5).getUTCDay() + 6 - firstWeekday) % 7)
      const weeks = Math.ceil((Date.UTC(year, month, 1) / 864e5 - start) / 7)
      return Array.from({ length: weeks }, (_, w) => Array.from({ length: 7 }, (_, i) => cell(start + w * 7 + i, month)))
    },
    // The grid's headers (FR-141 C2): the demo's photos as one event, the list as the bridge
    // answers it, so the run covers the grid.
    gallery_sections: async ({ filter }) => {
      const list = await window.__TAURI_INTERNALS__.invoke('list_photos', { filter })
      const count = list?.photos?.length ?? 0
      return count ? [{ kind: 'event', start: 0, count, day: 18802, startMin: 600, endMin: 642, place: 'Praha' }] : []
    },
    // The home time (FR-145): a fixed system zone, and one search hit.
    home_zone_guess: () => 'Europe/Prague',
    zone_search: () => [{ zone: 'Asia/Seoul', place: 'Seoul', country: 'South Korea' }],
  }
}

/** Measures Simple's panel and RollSense's into `rows`, their checks into `extra`. */
export async function measureSidePanels({ open, settle, measure, allChecks, say, outDir }, { language, theme }, rows, extra) {
  let { context, page } = await open({ scenario: 'stamp', language, theme, mode: 'simple' })
  await settle(page)
  rows.push(['Simple panel', await measure(page, 'aside.border-l')])
  await page.screenshot({ path: path.join(outDir, 'measure-simple.png') })
  await context.close()
  // UI-STATE-013 (FR-146, owner 2026-11-20): after an update, a user who saw the welcome meets
  // only the settings still unchosen (the home time), never the whole welcome again.
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, ui: { homeZone: null } }))
  await settle(page)
  const steps = await page.locator('[role="dialog"] nav button').count()
  if ((await page.locator('[role="dialog"]').count()) !== 1 || steps !== 1)
    extra.push(`UI-STATE-013 an update without a home time shows ${steps} welcome pages, not the home time's alone`)
  extra.push(...(await allChecks(page, 'welcome after an update')))
  await page.screenshot({ path: path.join(outDir, 'measure-welcome-new.png') })
  await context.close()
  // Pro's gallery panel, its Print tab: the date stamp, camera clocks and places cards, every
  // row checked (UI-LAYOUT-025, owner 2026-11-19: no scene opened this tab, so the Places card's
  // reset hung below its field unseen).
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await page.locator('aside.border-l [data-slot="inspector-tabs"] button').nth(1).click()
  await settle(page)
  // Every folded card opened, so its rows are checked too.
  for (const fold of await page.locator('aside.border-l section > header button[aria-expanded="false"]').all()) await fold.click()
  await settle(page)
  extra.push(...(await allChecks(page, 'Pro panel: Print')))
  await page.locator('aside.border-l').screenshot({ path: path.join(outDir, 'measure-pro-print.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await page.getByRole('button', { name: say('rollsense.view') }).click()
  await settle(page)
  rows.push(['RollSense panel', await measure(page, 'aside.border-l')])
  extra.push(...(await allChecks(page, 'RollSense panel')))
  await page.screenshot({ path: path.join(outDir, 'measure-rollsense.png') })
  // UI-STABLE-003 (FR-141, owner 2026-11-19: "the actions jumped over the calendar, then it
  // scrolled by itself"): ticking a day moves neither the day, nor the column's foot, nor the
  // grid's scroll, and the grid then shows its event headers.
  const day = page.locator('[data-day]:not(:disabled)').last()
  await day.scrollIntoViewIfNeeded()
  await settle(page)
  const foot = page.locator('aside.border-l > div').last()
  const grid = page.getByLabel(say('library.gridLabel'))
  const before = { day: await day.boundingBox(), foot: await foot.boundingBox() }
  await day.click()
  await settle(page)
  const after = { day: await day.boundingBox(), foot: await foot.boundingBox(), top: await grid.evaluate((e) => e.scrollTop) }
  await page.waitForTimeout(2000)
  const later = await grid.evaluate((e) => e.scrollTop)
  const moved = (a, b) => Math.abs((a?.y ?? 0) - (b?.y ?? 0)) > 1 || Math.abs((a?.height ?? 0) - (b?.height ?? 0)) > 1
  if (moved(before.day, after.day)) extra.push(`UI-STABLE-003 ticking a day moved it from ${before.day?.y} to ${after.day?.y}`)
  if (moved(before.foot, after.foot)) extra.push('UI-STABLE-003 ticking a day resized the column\'s foot')
  if (after.top !== later) extra.push(`UI-STABLE-003 the grid scrolled by itself, ${after.top} to ${later} px`)
  if ((await page.locator('[data-slot="grid-section-header"]').count()) === 0) extra.push('UI-STABLE-003 a ticked day shows no event header in the grid')
  extra.push(...(await allChecks(page, 'RollSense, a day ticked')))
  await page.screenshot({ path: path.join(outDir, 'measure-rollsense-when.png') })
  await context.close()
}
