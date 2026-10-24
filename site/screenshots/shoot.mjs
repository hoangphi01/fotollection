// SPDX-License-Identifier: GPL-3.0-only
// Screenshots of the real interface for the website (site/screenshots/README.md).
//
//   node site/screenshots/shoot.mjs <demo-dump> [out-dir]
//
// Serves ui/dist, answers IPC with the answers the real app recorded for the demo photos
// (src-tauri/src/demo.rs), and serves the recorded thumbnails and previews. No Tauri, no
// network: every request outside the two local hosts is refused.

import { readFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const dump = path.resolve(process.argv[2] ?? '')
const outDir = path.resolve(process.argv[3] ?? path.join(root, 'site/assets/img'))
const dist = path.join(root, 'ui/dist')
if (!existsSync(path.join(dump, 'fixtures.json')) || !existsSync(path.join(dist, 'index.html'))) {
  console.error('usage: node site/screenshots/shoot.mjs <demo-dump> [out-dir]  (build ui/dist first)')
  process.exit(2)
}

// The demo folder lived in a scratch directory; show a plain one instead.
let text = await readFile(path.join(dump, 'fixtures.json'), 'utf8')
const folder = JSON.parse(text).common.photo_details['0'].folderPath
text = text.replaceAll(folder, '/Users/demo/Pictures/Trip 2018').replaceAll('"demo-src"', '"Trip 2018"')
const fixtures = JSON.parse(text)

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.json': 'application/json' }

/** Runs in the page before the app: a stand-in for Tauri's IPC bridge. */
function bridge({ fixtures, scenario, language }) {
  const { common, scenarios } = fixtures
  const look = scenarios[scenario]
  const byId = (table, args) => table[String(args?.id)]
  const answers = {
    app_info: () => common.app_info,
    list_print_sizes: () => common.list_print_sizes,
    list_fonts: () => common.list_fonts,
    get_ui_settings: () => ({ ...common.get_ui_settings, language }),
    get_document: () => ({ ...common.get_document, edited: false, name: 'Trip 2018' }),
    list_groups: () => common.list_groups,
    list_photos: () => common.list_photos,
    photo_details: (a) => byId(common.photo_details, a),
    get_crop_geometry: (a) => byId(common.get_crop_geometry, a),
    get_settings: () => look.get_settings,
    get_frame_config: () => look.get_frame_config,
    get_photo_stamp: (a) => byId(look.get_photo_stamp, a),
    history_state: () => ({ canUndo: true, canRedo: false }),
  }
  let next = 1
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} }
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
    transformCallback: () => next++,
    unregisterCallback() {},
    convertFileSrc: (p) => p,
    async invoke(cmd, args) {
      if (cmd.startsWith('plugin:event|')) return next++
      const answer = answers[cmd]
      if (answer) return answer(args)
      return null // writes and dialogs: nothing happens in a screenshot
    },
  }
}

const browser = await chromium.launch()
const shots = []

async function open({ scenario, language, theme, width = 1440, height = 900 }) {
  const context = await browser.newContext({
    viewport: { width, height },
    deviceScaleFactor: 2,
    colorScheme: theme,
    // WebView2-style image URLs (http://fotothumb.localhost), which a test browser can serve.
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36',
    locale: language === 'vi' ? 'vi-VN' : 'en-GB',
  })
  await context.route('**/*', async (route) => {
    const url = new URL(route.request().url())
    if (url.host === 'app.localhost') {
      const file = path.join(dist, url.pathname === '/' ? 'index.html' : decodeURIComponent(url.pathname))
      if (!file.startsWith(dist) || !existsSync(file)) return route.fulfill({ status: 404 })
      return route.fulfill({ body: await readFile(file), contentType: TYPES[path.extname(file)] ?? 'application/octet-stream' })
    }
    if (url.host === 'fotothumb.localhost') {
      const [, kind, id] = url.pathname.split('/')
      const file = kind === 'preview' ? path.join(dump, 'img/preview', scenario, `${id}.jpg`) : path.join(dump, 'img', kind, `${id}.jpg`)
      if (!existsSync(file)) return route.fulfill({ status: 404 })
      return route.fulfill({ body: await readFile(file), contentType: 'image/jpeg' })
    }
    return route.abort()
  })
  await context.addInitScript(bridge, { fixtures, scenario, language })
  const page = await context.newPage()
  page.on('pageerror', (e) => console.error('page error:', e.message))
  await page.goto('http://app.localhost/')
  await page.waitForSelector('img[src*="/thumb/"]')
  await page.waitForFunction(() => [...document.images].every((i) => i.complete))
  return { context, page }
}

async function settle(page) {
  // A dialog focuses its first button, which shows that button's tooltip; give focus to the
  // dialog itself instead, as a mouse user would see it.
  await page.evaluate(() => {
    const active = document.activeElement
    if (active instanceof HTMLButtonElement) active.blur()
    document.querySelector('[role="dialog"]')?.focus()
  })
  await page.waitForTimeout(400)
  await page.waitForFunction(() => [...document.images].every((i) => i.complete && i.naturalWidth > 0))
  await page.waitForTimeout(300)
}

async function save(page, name) {
  const file = path.join(outDir, `${name}.png`)
  await page.screenshot({ path: file })
  shots.push(file)
}

// Photos by file name, so adding demo photos never changes which one is shown.
const idOf = (name) => {
  const photo = fixtures.common.list_photos.photos.find((p) => p.fileName.startsWith(name))
  if (!photo) throw new Error(`demo photo ${name} is missing`)
  return photo.id
}
const SHOW = {
  grid: idOf('erdmaennchen-in-der-wilhelma-1'),
  stamp: idOf('bennetts-beach'), // has GPS: the place joins the date
  frame: idOf('erdmaennchen-in-der-wilhelma-1'),
  crop: idOf('bir-hakeim-bridge'),
}
const tile = (page, id) => page.locator(`img[src*="/thumb/${id}?"]`).first()

for (const language of ['en', 'vi']) {
  for (const theme of ['dark', 'light']) {
    const suffix = `${language}-${theme}`

    // 1. The library grid, culled.
    let { context, page } = await open({ scenario: 'stamp', language, theme })
    await tile(page, SHOW.grid).click()
    await settle(page)
    await save(page, `grid-${suffix}`)

    // 2. Preview with the retro date stamp and the place (GPS photo).
    await tile(page, SHOW.stamp).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.stamp}"]`)
    await settle(page)
    await save(page, `stamp-${suffix}`)
    await context.close()

    // 3. Preview with the "Shot on" info frame.
    ;({ context, page } = await open({ scenario: 'frame', language, theme }))
    await tile(page, SHOW.frame).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.frame}"]`)
    await settle(page)
    await save(page, `frame-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
    await tile(page, SHOW.crop).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
    await settle(page)

    // 4. Crop editor, straightening.
    await page.keyboard.press('c')
    await page.waitForTimeout(300)
    await page.waitForSelector(`img[src*="/source/${SHOW.crop}"]`)
    await page.keyboard.press('ArrowRight')
    await page.keyboard.press('ArrowRight')
    await settle(page)
    await save(page, `crop-${suffix}`)
    await context.close()

    // 5. Best-shot groups.
    ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
    await page.getByRole('radio', { name: /Groups|Nhóm/ }).click()
    await settle(page)
    await save(page, `groups-${suffix}`)
    await context.close()
  }
}

// Link previews (Open Graph), 1200 × 630: the info-frame preview under the headline.
for (const language of ['en', 'vi']) {
  const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 })
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url())
    if (url.host !== 'og.localhost') return route.abort()
    const name = path.basename(url.pathname)
    const file =
      url.pathname === '/'
        ? path.join(root, 'site/screenshots/og.html')
        : name === 'mark.png'
          ? path.join(root, 'ui/public/brand/mark-84.png')
          : path.join(outDir, name)
    return route.fulfill({ body: await readFile(file), contentType: file.endsWith('.html') ? 'text/html' : 'image/png' })
  })
  await page.goto(`http://og.localhost/?lang=${language}`)
  await page.waitForFunction(() => document.fonts.ready.then(() => [...document.images].every((i) => i.complete)))
  const file = path.join(outDir, `og-${language}.png`)
  await page.screenshot({ path: file })
  shots.push(file)
  await page.close()
}

await browser.close()
console.log(shots.map((s) => path.relative(root, s)).join('\n'))
