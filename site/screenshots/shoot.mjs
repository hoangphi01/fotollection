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
function bridge({ fixtures, scenario, language, os, tabs }) {
  const { common, scenarios } = fixtures
  const look = scenarios[scenario]
  const byId = (table, args) => table[String(args?.id)]
  let current = look.get_settings
  // Demo culling state for the print basket (FR-050), by position in the trip.
  const withQueues = (photos) => photos.map((p, i) => ({ ...p, queue: [1, 4, 6].includes(i) ? 'print' : [9, 13].includes(i) ? 'aside' : null }))
  // The folder-tabs shot (FR-056): the demo photos as three folders, by file name.
  const tabOf = (name) =>
    /wilhelma/.test(name) ? [0, 'Wilhelma Zoo'] : /tuebingen|ammerschlag|anlagensee|neckar/.test(name) ? [1, 'Tübingen'] : [2, 'Travels']
  const inTabs = (photos) =>
    tabs ? photos.map((p) => { const [folderId, folderName] = tabOf(p.fileName); return { ...p, folderId, folderName } }) : photos
  const inView = (photos, filter) => {
    const view = filter?.queue ?? 'gallery'
    const scope = filter?.folders ?? []
    return photos
      .filter((p) => (view === 'gallery' ? p.queue !== 'aside' : p.queue === view))
      .filter((p) => !scope.length || scope.includes(p.folderId))
  }
  const answers = {
    app_info: () => ({ ...common.app_info, ...(os ? { os } : {}) }),
    list_print_sizes: () => common.list_print_sizes,
    list_fonts: () => common.list_fonts,
    get_ui_settings: () => ({ ...common.get_ui_settings, language }),
    get_document: () => ({ ...common.get_document, edited: false, name: 'Trip 2018' }),
    list_groups: () => common.list_groups,
    // Dumps recorded before FR-056 have no folder list: count it from the photos. A few
    // photos sit in the print basket and set aside (FR-050), as after a culling session.
    list_photos: (a) => {
      const list = { ...common.list_photos, photos: inTabs(common.list_photos.photos) }
      const folders = (!tabs && list.folders) || Object.values(
        list.photos.reduce((acc, p) => {
          acc[p.folderId] ??= { id: p.folderId, name: p.folderName, count: 0 }
          acc[p.folderId].count += 1
          return acc
        }, {}),
      )
      const all = withQueues(list.photos)
      return {
        ...list,
        photos: inView(all, a?.filter),
        folders,
        printCount: all.filter((p) => p.queue === 'print').length,
        asideCount: all.filter((p) => p.queue === 'aside').length,
      }
    },
    photo_details: (a) => byId(common.photo_details, a),
    get_crop_geometry: (a) => byId(common.get_crop_geometry, a),
    print_advice: (a) => byId(common.print_advice ?? {}, a),
    get_settings: () => current,
    // Keep a change, as Rust does, so a click in a screenshot shows its result.
    set_settings: (a) => (current = { settings: a.settings, revision: (current.revision ?? 0) + 1 }),
    get_frame_config: () => look.get_frame_config,
    get_photo_stamp: (a) => byId(look.get_photo_stamp, a),
    history_state: () => ({ canUndo: true, canRedo: false }),
    // The export card's size estimate (FR-053): the demo photos' real sizes and the ratios
    // measured in docs/RESULT.md §10 for the selected target.
    estimate_export: (a) => {
      const scope = a?.only != null ? withQueues(common.list_photos.photos).filter((p) => p.id === a.only) : inView(withQueues(common.list_photos.photos), a?.filter)
      const source = scope.reduce((n, p) => n + (common.photo_details[String(p.id)]?.fileBytes ?? 3e6), 0)
      const target = current.settings?.exportTarget
      const ratio = target?.kind === 'share' ? { original: 0.46, facebook: 0.17, instagram: 0.08, custom: 0.2 }[target.preset?.kind] ?? 0.17 : 1
      return { photos: scope.length, sourceBytes: source, estimatedBytes: Math.round(source * ratio), sampled: Math.min(4, scope.length) }
    },
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

async function open({ scenario, language, theme, width = Number(process.env.DEMO_WIDTH) || 1440, height = 900, scale = 2, tabs = false }) {
  // DEMO_WIDTH=1000 checks narrow windows (the screenshots for the site stay at 1440).
  const context = await browser.newContext({
    viewport: { width, height },
    deviceScaleFactor: scale,
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
  // DEMO_OS=windows shows the Windows title bar, for checking it on a Mac.
  await context.addInitScript(bridge, { fixtures, scenario, language, os: process.env.DEMO_OS, tabs })
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

/** A zoomed-in part of the window, 16:10 like the full screenshots. */
async function zoomed(page, name, clip) {
  const file = path.join(outDir, `${name}.png`)
  await page.screenshot({ path: file, clip })
  shots.push(file)
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

    // 1b. Print-size advice: the picker open, a badge per size (FR-026).
    await page.locator('#print-size').click()
    await page.waitForSelector('[role="option"]')
    await page.waitForTimeout(300)
    await save(page, `check-advice-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(200)
    await page.getByRole('button', { name: /Versions|Các phiên bản/ }).click()
    await page.waitForTimeout(300)
    await save(page, `check-versions-${suffix}`)
    await page.keyboard.press('Escape')

    // 2. Preview with the retro date stamp and the place (GPS photo).
    await tile(page, SHOW.stamp).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.stamp}"]`)
    await settle(page)
    await save(page, `stamp-${suffix}`)
    await context.close()

    // 3. Preview with the "Shot on" info frame (and, for checking only, the panel's frame controls).
    ;({ context, page } = await open({ scenario: 'frame', language, theme }))
    await page.locator('aside').last().hover()
    await page.mouse.wheel(0, 2000)
    await page.waitForTimeout(300)
    await save(page, `check-frame-${suffix}`)
    await tile(page, SHOW.frame).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.frame}"]`)
    await settle(page)
    await save(page, `frame-${suffix}`)
    // The export card at the bottom of the inspector: this photo only (FR-055).
    await page.locator('[role="dialog"]').getByRole('button', { name: /^(Export|Xuất ảnh)$/ }).click()
    await page.waitForTimeout(600)
    await save(page, `check-export-${suffix}`)
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
    // The compare view shows the shots large, side by side, with a bar per measure.
    await page.getByRole('button', { name: /^(Compare|So sánh)$/ }).first().click()
    await page.waitForSelector('[role="dialog"]')
    await settle(page)
    await save(page, `groups-${suffix}`)
    await context.close()

    // 6. The print basket (FR-050, FR-059), shown zoomed in at three times the resolution.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 3 }))
    const tiles = page.locator('img[src*="/thumb/"]')
    // 6a. Select mode: ticks, the action bar, the basket and set-aside rows.
    await page.getByRole('button', { name: /^(Select|Chọn)$/ }).click()
    await tiles.nth(2).click()
    await tiles.nth(3).click()
    await tiles.nth(5).click()
    await page.mouse.move(5, 890)
    await settle(page)
    await zoomed(page, `select-${suffix}`, { x: 0, y: 0, width: 1040, height: 650 })
    // 6b. The preview of a photo in the basket: ↑ keeps it, ↓ sets it aside.
    await page.getByRole('button', { name: /^(Done|Xong)$/ }).click()
    await tiles.nth(1).dblclick()
    await page.waitForSelector('[role="dialog"]')
    await settle(page)
    await zoomed(page, `swipe-${suffix}`, { x: 40, y: 45, width: 960, height: 600 })
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
    await context.close()
    // 6c. The size estimate, sharing for Facebook: framed on the estimate and the button.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 5 }))
    await page.getByRole('radio', { name: /^(Share online|Mạng xã hội)$/ }).first().click()
    await page.waitForTimeout(900)
    const note = await page.getByText(/^(Estimated|Ước tính):/).boundingBox()
    const button = await page.getByRole('button', { name: /^(Export|Xuất) \d+/ }).boundingBox()
    if (note && button) {
      const top = note.y - 56
      const height = button.y + button.height + 40 - top
      const width = height * 1.6
      const x = Math.min(1440 - width, note.x + note.width / 2 - width / 2)
      await zoomed(page, `estimate-${suffix}`, { x, y: top, width, height })
    }
    await context.close()
    // 7. Folders as tabs (FR-056): tick photos in one folder, switch tabs, tick more; the bar
    // counts them all, and hovering a tab shows its close button.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 3, tabs: true }))
    const tab = (name) => page.getByRole('button', { name: new RegExp(`^${name}`) })
    await tab('Wilhelma Zoo').click()
    await page.getByRole('button', { name: /^(Select|Chọn)$/ }).click()
    await page.locator('img[src*="/thumb/"]').nth(1).click()
    await page.locator('img[src*="/thumb/"]').nth(4).click()
    await tab('Tübingen').click()
    await page.waitForTimeout(300)
    await page.locator('img[src*="/thumb/"]').nth(2).click()
    await tab('Travels').hover()
    await settle(page)
    await zoomed(page, `tabs-${suffix}`, { x: 0, y: 0, width: 1040, height: 650 })
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
          : url.pathname.startsWith('/fonts/')
            ? path.join(root, 'site/assets/fonts', name)
            : path.join(outDir, name)
    return route.fulfill({ body: await readFile(file), contentType: file.endsWith('.html') ? 'text/html' : file.endsWith('.woff2') ? 'font/woff2' : 'image/png' })
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
