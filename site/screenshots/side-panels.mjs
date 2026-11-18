// SPDX-License-Identifier: GPL-3.0-only
// The right columns beyond Pro's gallery panel, each measured on its own page (STYLE_RULE §4):
// Simple mode's panel, and RollSense's (FR-141), Pro's third tab, its facets as cards. Split out
// of shoot.mjs (red in CODE_HEALTH, FR-139): shoot.mjs hands over the helpers it owns.
import path from 'node:path'

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
  await context.close()
}
