// SPDX-License-Identifier: GPL-3.0-only
// The showcase tiles of the landing page, its wide image, and the catalog images (the VNLF
// App Explorer, Flathub / AppStream, the README), from showcase.html and the screenshots
// already in site/assets/img (site/screenshots/README.md). Never a mock-up: every window is
// a screenshot of the real app on the demo set (CC0 photos and the owner's Prague trip).
//
//   node site/screenshots/showcase.mjs <demo-dump>
//
// <demo-dump> is the recording build.sh made; one of its Prague previews is the wide image's
// blurred backdrop. Writes site/assets/img/showcase/<tile>-<lang>.webp and
// site/assets/img/catalog/<name>-<lang>.jpg (captioned) and <name>.jpg (plain, AppStream). No
// network: every request outside the local host is refused.
import { readFile, mkdir } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const img = path.join(root, 'site/assets/img')
const dump = process.argv[2]
if (!dump || !existsSync(path.join(dump, 'fixtures.json'))) {
  console.error('usage: node site/screenshots/showcase.mjs <demo-dump>')
  process.exit(2)
}
// The view over Prague (P1067950), the owner's photo already shown in the screenshots.
const fixtures = JSON.parse(await readFile(path.join(dump, 'fixtures.json'), 'utf8'))
const photos = fixtures.common.list_photos.photos
const view = photos.find((p) => p.fileName.startsWith('P1067950')) ?? photos.find((p) => p.fileName.startsWith('charles-bridge'))
const backdrop = path.join(dump, 'img/preview/look', `${view.id}.jpg`)

const TYPES = { '.html': 'text/html', '.woff2': 'font/woff2', '.png': 'image/png', '.webp': 'image/webp', '.jpg': 'image/jpeg' }
const TILES = ['looks', 'simple', 'compare', 'prints', 'yours', 'desktops']
const CATALOG = ['gallery', 'window', 'compare', 'export', 'collage']
await mkdir(path.join(img, 'showcase'), { recursive: true })
await mkdir(path.join(img, 'catalog'), { recursive: true })

const browser = await chromium.launch()
const written = []
async function render(query, width, height, out, type) {
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 })
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url())
    if (url.host !== 'showcase.localhost') return route.abort()
    const file =
      url.pathname === '/'
        ? path.join(root, 'site/screenshots/showcase.html')
        : url.pathname === '/mark.png'
          ? path.join(root, 'ui/public/brand/mark-84.png')
          : url.pathname === '/backdrop.jpg'
            ? backdrop
            : url.pathname.startsWith('/fonts/')
              ? path.join(root, 'site/assets/fonts', path.basename(url.pathname))
              : path.join(img, path.normalize(url.pathname.replace(/^\/img\//, '')))
    if (!existsSync(file)) return route.fulfill({ status: 404 })
    return route.fulfill({ body: await readFile(file), contentType: TYPES[path.extname(file)] ?? 'application/octet-stream' })
  })
  await page.goto(`http://showcase.localhost/?${query}&w=${width}&h=${height}`)
  await page.waitForFunction(() => document.fonts.ready.then(() => [...document.images].every((i) => i.complete && i.naturalWidth > 0)))
  // Words that do not fit their room fail the run, as MEASURE does in the app.
  const spilled = await page.evaluate(() =>
    [...document.querySelectorAll('h1, p.sub, .chip, .caption')]
      .filter((e) => e.scrollWidth > e.clientWidth + 1 || e.getBoundingClientRect().right > window.innerWidth)
      .map((e) => e.textContent.trim()),
  )
  if (spilled.length) throw new Error(`${out}: words past their room: ${spilled.join(' | ')}`)
  if (type === 'webp') {
    const png = `${out}.tmp.png`
    await page.screenshot({ path: png })
    execFileSync('cwebp', ['-quiet', '-q', '84', png, '-o', out])
    execFileSync('rm', [png])
  } else {
    await page.screenshot({ path: out, type: 'jpeg', quality: 88 })
  }
  written.push(path.relative(root, out))
  await page.close()
}

for (const lang of ['en', 'vi']) {
  for (const tile of TILES) await render(`lang=${lang}&tile=${tile}`, 1200, 900, path.join(img, 'showcase', `${tile}-${lang}.webp`), 'webp')
  await render(`lang=${lang}&tile=wide`, 2000, 860, path.join(img, 'showcase', `wide-${lang}.webp`), 'webp')
  for (const name of CATALOG) await render(`lang=${lang}&tile=catalog-${name}-caption`, 1600, 1120, path.join(img, 'catalog', `${name}-${lang}.jpg`), 'jpg')
}
// AppStream's screenshots carry no words of ours: one set, the English interface.
for (const name of CATALOG) await render(`lang=en&tile=catalog-${name}-plain`, 1600, 1000, path.join(img, 'catalog', `${name}.jpg`), 'jpg')
await browser.close()
console.log(written.join('\n'))
