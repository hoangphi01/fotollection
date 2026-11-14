// SPDX-License-Identifier: GPL-3.0-only
// The dry check of translations' lengths (FR-125, docs/LOCALIZATION.md §3): every string of
// every locale drawn in the interface's own face (Be Vietnam Pro, 500, 12 px, as a control's
// words) in a headless Chromium, compared with English. Lists the keys a translation makes
// more than 30 % and 20 px wider, widest first, among the strings short enough to be a control's
// words (English under 240 px; longer ones are sentences that wrap): the words a translator
// shortens before they shrink or cut a button. A report, never a failure; the app itself fits
// what is left (`FitText`). The pseudo-locale has its own pass (`MEASURE_LANG=en-XA`).
// `--all` lists every key of every locale.
//
//   npm run text-widths [-- --all]
import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'
import { pseudo } from '../ui/src/i18n/pseudo.ts'

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..')
const localeDir = path.join(root, 'ui/src/locales')
const fontDir = path.join(root, 'node_modules/@fontsource/be-vietnam-pro/files')
const all = process.argv.includes('--all')
const RATIO = 1.3
const MIN_PX = 20
const LABEL_PX = 240

const flatten = (tree, prefix = '', out = new Map()) => {
  for (const [k, v] of Object.entries(tree)) {
    const key = prefix ? `${prefix}.${k}` : k
    if (typeof v === 'string') out.set(key, v)
    else flatten(v, key, out)
  }
  return out
}

const english = JSON.parse(await readFile(path.join(localeDir, 'en.json'), 'utf8'))
const locales = { en: flatten(english), pseudo: flatten(pseudo(english)) }
for (const file of (await readdir(localeDir)).filter((f) => f.endsWith('.json') && f !== 'en.json').sort())
  locales[file.replace('.json', '')] = flatten(JSON.parse(await readFile(path.join(localeDir, file), 'utf8')))

const faces = await Promise.all(
  ['latin', 'latin-ext', 'vietnamese'].map(async (subset) => ({
    subset,
    data: (await readFile(path.join(fontDir, `be-vietnam-pro-${subset}-500-normal.woff2`))).toString('base64'),
  })),
)
// Japanese and Korean in the faces the app bundles for them (FR-128), as the interface's stack falls to them.
const cjk = await Promise.all(['jp', 'kr'].map(async (s) => (await readFile(path.join(root, `ui/src/assets/fonts/noto-sans-${s}-subset.woff2`))).toString('base64')))

const browser = await chromium.launch()
const page = await browser.newPage()
const widths = await page.evaluate(
  async ({ faces, cjk, locales }) => {
    for (const { data } of faces) {
      const face = new FontFace('BVP', `url(data:font/woff2;base64,${data})`, { weight: '500' })
      document.fonts.add(await face.load())
    }
    for (const [i, data] of cjk.entries())
      document.fonts.add(await new FontFace(`CJK${i}`, `url(data:font/woff2;base64,${data})`, { weight: '400 700' }).load())
    const ctx = document.createElement('canvas').getContext('2d')
    ctx.font = '500 12px BVP, CJK0, CJK1'
    const out = {}
    for (const [code, entries] of Object.entries(locales))
      out[code] = Object.fromEntries(entries.map(([k, v]) => [k, Math.round(ctx.measureText(v.replace(/\{\{[^}]+\}\}/g, '00')).width)]))
    return out
  },
  { faces, cjk, locales: Object.fromEntries(Object.entries(locales).map(([c, m]) => [c, [...m]])) },
)
await browser.close()

const codes = Object.keys(locales).filter((c) => c !== 'en' && (all || c !== 'pseudo'))
const rows = []
for (const key of locales.en.keys()) {
  const en = widths.en[key]
  for (const code of codes) {
    const w = widths[code][key.replace(/_one$/, '_other')] ?? widths[code][key]
    if (w === undefined || en === 0 || (!all && en > LABEL_PX)) continue
    if (all || (w > en * RATIO && w - en > MIN_PX)) rows.push({ key, code, en, w, ratio: w / en })
  }
}
rows.sort((a, b) => b.w - b.en - (a.w - a.en))

console.log(`text widths at 12 px, Be Vietnam Pro 500; listed: ${all ? 'every key' : `labels (English under ${LABEL_PX} px) over ${Math.round((RATIO - 1) * 100)} % and ${MIN_PX} px wider than English`}`)
for (const code of codes) {
  const mine = rows.filter((r) => r.code === code)
  console.log(`\n${code}: ${mine.length} key(s)`)
  for (const r of mine.slice(0, all ? undefined : 40))
    console.log(`  ${r.key.padEnd(48)} en ${String(r.en).padStart(4)} px  ${code} ${String(r.w).padStart(4)} px  ×${r.ratio.toFixed(2)}`)
  if (!all && mine.length > 40) console.log(`  … and ${mine.length - 40} more (--all lists every key)`)
}
