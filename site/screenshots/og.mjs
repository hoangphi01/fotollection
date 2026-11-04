// SPDX-License-Identifier: GPL-3.0-only
// The link previews (Open Graph, 1200 × 630) of every page, in English and Vietnamese, from
// og.html and the screenshots already in site/assets/img (site/screenshots/README.md).
//
//   node site/screenshots/og.mjs
//
// Writes site/assets/img/og-{lang}.jpg for the home page and og-{page}-{lang}.jpg for the
// others. No network: every request outside the local host is refused.

import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const img = path.join(root, 'site/assets/img')
const PAGES = ['home', 'download', 'performance', 'manual', 'code-signing', 'story']
const TYPES = { '.html': 'text/html', '.woff2': 'font/woff2', '.png': 'image/png', '.webp': 'image/webp', '.svg': 'image/svg+xml' }

const browser = await chromium.launch()
const written = []
for (const language of ['en', 'vi']) {
  for (const name of PAGES) {
    const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 })
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
    await page.goto(`http://og.localhost/?lang=${language}&page=${name}`)
    await page.waitForFunction(() => document.fonts.ready.then(() => [...document.images].every((i) => i.complete && i.naturalWidth > 0)))
    const file = path.join(img, name === 'home' ? `og-${language}.jpg` : `og-${name}-${language}.jpg`)
    await page.screenshot({ path: file, type: 'jpeg', quality: 86 })
    written.push(path.relative(root, file))
    await page.close()
  }
}
await browser.close()
console.log(written.join('\n'))
