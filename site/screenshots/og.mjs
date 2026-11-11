// SPDX-License-Identifier: GPL-3.0-only
// The link previews (Open Graph, 1200 × 630) of every page in every published language
// (site/_data/languages.yml), from og.html, each language's og_card_<page> words and the
// screenshots already in site/assets/img (site/screenshots/README.md).
//
//   node site/screenshots/og.mjs
//
// Writes site/assets/img/og-{lang}.jpg for the home page and og-{page}-{lang}.jpg for the
// others. No network: every request outside the local host is refused.

import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
import yaml from 'js-yaml'
import { chromium } from 'playwright'
import { published } from '../../eleventy.registry.mjs'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const img = path.join(root, 'site/assets/img')
const words = (code) => yaml.load(readFileSync(path.join(root, `site/_data/i18n/${code}.yml`), 'utf8'))
const TYPES = { '.html': 'text/html', '.woff2': 'font/woff2', '.png': 'image/png', '.webp': 'image/webp', '.svg': 'image/svg+xml' }

const browser = await chromium.launch()
const written = []
for (const { code: language, pages, assets } of published) {
  const t = words(language)
  for (const name of pages) {
    const card = t[`og_card_${name.replace(/-/g, '_')}`]
    if (!card) throw new Error(`${language}: no og_card_${name.replace(/-/g, '_')} in site/_data/i18n/${language}.yml`)
    const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 })
    await page.addInitScript((c) => { window.CARD = c }, { ...card, vnlf: t.og_vnlf })
    await page.route('**/*', async (route) => {
      const url = new URL(route.request().url())
      if (url.host !== 'og.localhost') return route.abort()
      const file =
        url.pathname === '/'
          ? path.join(root, 'site/screenshots/og.html')
          : url.pathname === '/mark.png'
            ? path.join(root, 'ui/public/brand/mark-84.png')
            : url.pathname.startsWith('/fonts/')
              ? path.join(root, 'site/assets/fonts', path.basename(url.pathname))
              : path.join(img, path.normalize(url.pathname.replace(/^\/img\//, '')))
      if (!file.startsWith(root)) return route.fulfill({ status: 404 })
      return route.fulfill({ body: await readFile(file), contentType: TYPES[path.extname(file)] ?? 'application/octet-stream' })
    })
    await page.goto(`http://og.localhost/?lang=${language}&page=${name}&assets=${assets}`)
    await page.waitForFunction(() => document.fonts.ready.then(() => [...document.images].every((i) => i.complete && i.naturalWidth > 0)))
    const file = path.join(img, name === 'home' ? `og-${language}.jpg` : `og-${name}-${language}.jpg`)
    await page.screenshot({ path: file, type: 'jpeg', quality: 86 })
    written.push(path.relative(root, file))
    await page.close()
  }
}
await browser.close()
console.log(written.join('\n'))
