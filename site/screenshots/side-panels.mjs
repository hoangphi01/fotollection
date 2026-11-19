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
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await page.getByRole('button', { name: say('rollsense.view') }).click()
  await settle(page)
  rows.push(['RollSense panel', await measure(page, 'aside.border-l')])
  extra.push(...(await allChecks(page, 'RollSense panel')))
  await page.screenshot({ path: path.join(outDir, 'measure-rollsense.png') })
  // The When calendar with its last day chosen, the day's rows under it (FR-141 C1–C2).
  await page.locator('[data-day]:not(:disabled)').last().click()
  await page.locator('aside.border-l').getByText(/\d:\d\d/).last().scrollIntoViewIfNeeded()
  await settle(page)
  extra.push(...(await allChecks(page, 'RollSense, a day chosen')))
  await page.locator('aside.border-l').screenshot({ path: path.join(outDir, 'measure-rollsense-when.png') })
  await context.close()
}
