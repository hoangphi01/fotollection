// SPDX-License-Identifier: GPL-3.0-only
// Screenshots of the real interface for the website (site/screenshots/README.md).
//
//   node site/screenshots/shoot.mjs <demo-dump> [out-dir] [best-shot-dump]
//   MEASURE=1 node site/screenshots/shoot.mjs <demo-dump> <out-dir>   (the panels' measures)
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
const outDir = path.resolve(process.argv[3] ?? path.join(root, 'site/assets/img'))
const dist = path.join(root, 'ui/dist')
const usage = 'usage: node site/screenshots/shoot.mjs <demo-dump> [out-dir] [best-shot-dump]  (build ui/dist first)'
if (!existsSync(path.join(path.resolve(process.argv[2] ?? ''), 'fixtures.json')) || !existsSync(path.join(dist, 'index.html'))) {
  console.error(usage)
  process.exit(2)
}

/** A recording of the real app (demo.rs), its scratch folder shown as a plain one. */
async function load(dir, name, folderName) {
  const dump = path.resolve(dir)
  let text = await readFile(path.join(dump, 'fixtures.json'), 'utf8')
  const folder = JSON.parse(text).common.photo_details['0'].folderPath
  text = text.replaceAll(folder, `/Users/demo/Pictures/${name}`).replaceAll(`"${folderName ?? path.basename(folder)}"`, `"${name}"`)
  return { dump, fixtures: JSON.parse(text) }
}
// The CC0 demo photos for every shot but best shot, which shows the owner's cat: a burst
// made by best_shot_photos.py, recorded on its own (site/screenshots/README.md).
const trip = await load(process.argv[2], 'Trip 2018')
const cat = process.argv[4] ? await load(process.argv[4], 'Cat') : trip
const fixtures = trip.fixtures

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.json': 'application/json' }

/** Runs in the page before the app: a stand-in for Tauri's IPC bridge. */
function bridge({ fixtures, scenario, language, os, tabs, queues, mode }) {
  const { common, scenarios } = fixtures
  // Collage mode (FR-035) has its own recording; older dumps have none.
  // Postcards (FR-073) are recorded after the collage, as its own sheets; older dumps have none.
  const empty = { list_sheets: [], sheet_info: {}, get_crop_geometry: {}, get_postcard_text: {} }
  const collage = (scenario === 'postcard' ? fixtures.postcard : scenario === 'postcard-full' ? fixtures.postcard_full : fixtures.collage) ?? empty
  const look = scenarios[scenario] ?? scenarios.stamp
  const byId = (table, args) => table[String(args?.id)]
  let current = ['collage', 'postcard', 'postcard-full'].includes(scenario) ? collage.get_settings : look.get_settings
  // The print basket and set-aside photos (FR-050), by position (TRIP_QUEUES).
  const withQueues = (photos) =>
    photos.map((p, i) => ({ ...p, queue: queues.print.includes(i) ? 'print' : queues.aside.includes(i) ? 'aside' : null }))
  // The folder-tabs shot (FR-056): the demo photos as three folders, by file name.
  const tabOf = (name) =>
    /wilhelma/.test(name) ? [0, 'Wilhelma Zoo'] : /tuebingen|ammerschlag|anlagensee|neckar/.test(name) ? [1, 'Tübingen'] : /^P10|charles-bridge/.test(name) ? [2, 'Praha'] : [3, 'Travels']
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
    // Start-up is over at once, so the start-up screen fades out as in the app (FR-060).
    startup_finished: () => true,
    list_print_sizes: () => common.list_print_sizes,
    list_fonts: () => common.list_fonts,
    get_ui_settings: () => ({ ...common.get_ui_settings, language, mode }),
    // The Settings window's memory and cache (FR-086): a 16 GB Mac at 25 %, a trip's cache.
    memory_info: () => ({ total: 16 * 2 ** 30, budget: 4 * 2 ** 30, held: 1.3 * 2 ** 30 }),
    machine_info: () => ({ cpu: 'Apple M3', cores: 8, memory: 16 * 2 ** 30, freeDisk: 31e9 }),
    cache_usage: () => ({
      bytes: 0.44 * 2 ** 30,
      files: 1290,
      limitBytes: 5e9,
      thumbnailsBytes: 0.012 * 2 ** 30,
      previewsBytes: 0.42 * 2 ** 30,
      analysesBytes: 0.004 * 2 ** 30,
    }),
    suggest_memory_settings: (a) => ({
      ...common.get_ui_settings,
      language,
      mode,
      ...(a?.auto ? { previewMemoryPercent: 25, cacheLimitGb: 10 } : { previewMemoryPercent: 25, cacheLimitGb: 5 }),
      preparePreviews: true,
    }),
    list_sheets: () => collage.list_sheets,
    get_postcard_text: (a) => collage.get_postcard_text?.[String(a?.id)] ?? null,
    // The demo's texts all fit: no card is marked as cut.
    postcard_cuts: () => [],
    // "Use for N postcards" (FR-077): the other cards without this card's address, counted
    // from the recorded texts; and two public addresses as this machine's recent ones.
    postcard_text_gaps: (a) => {
      const texts = collage.get_postcard_text ?? {}
      const own = texts[String(a?.source)]?.[a?.part] ?? ''
      if (!own) return { blank: 0, differ: 0 }
      let blank = 0
      let differ = 0
      for (const sheet of collage.list_sheets) {
        if (sheet.first === a.source) continue
        const other = texts[String(sheet.first)]?.[a.part] ?? ''
        if (!other) blank += 1
        else if (other !== own) differ += 1
      }
      return { blank, differ }
    },
    recent_addresses: () => [
      'Národní muzeum\nVáclavské náměstí 68\n110 00 Praha 1, Czechia',
      'Magyar Nemzeti Múzeum\nMúzeum krt. 14–16\n1088 Budapest, Hungary',
    ],
    sheet_info: (a) => collage.sheet_info[a.sheet.second == null ? `${a.sheet.first}` : `${a.sheet.first}-${a.sheet.second}`],
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
    // Recorded once, for "pra" (FR-032); older dumps have no places.
    list_places: () => common.list_places ?? [],
    // Lists the demo has none of: camera clocks (FR-030) and camera sessions (FR-063).
    list_cameras: () => [],
    camera_sessions: () => [],
    search_places: () => common.search_places ?? [],
    get_crop_geometry: (a) => byId(a?.sheet ? collage.get_crop_geometry : common.get_crop_geometry, a),
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
      // Collage sheets (FR-035) export at 600 DPI: about 2.8 MB a 10 × 15 sheet (RESULT §18).
      if (a?.sheets) {
        const count = a.only != null ? 1 : collage.list_sheets.length
        return { photos: count, sourceBytes: source, estimatedBytes: count * 2.8e6, sampled: Math.min(4, count) }
      }
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

// Demo culling state for the print basket (FR-050), by position: the trip's, or none.
const TRIP_QUEUES = { print: [1, 4, 6], aside: [7, 13] } // not the shown photos (SHOW)
const NO_QUEUES = { print: [], aside: [] }

async function open({ scenario, language, theme, width = Number(process.env.DEMO_WIDTH) || 1440, height = 900, scale = 2, tabs = false, source = trip, queues, mode = 'professional' }) {
  const { dump } = source
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
      const [, kind, id, second] = url.pathname.split('/')
      if (kind === 'sheet') {
        // A collage sheet (FR-035): `{first}-{second}`, `-small` for the sheet grid.
        const name = `${id}${second ? `-${second}` : ''}${url.searchParams.get('small') ? '-small' : ''}`
        const sheet = path.join(dump, scenario === 'postcard' ? 'img/sheet-postcard' : scenario === 'postcard-full' ? 'img/sheet-postcard-full' : 'img/sheet', `${name}.jpg`)
        if (!existsSync(sheet)) return route.fulfill({ status: 404 })
        return route.fulfill({ body: await readFile(sheet), contentType: 'image/jpeg' })
      }
      const file = kind === 'preview' ? path.join(dump, 'img/preview', scenario, `${id}.jpg`) : path.join(dump, 'img', kind, `${id}.jpg`)
      if (!existsSync(file)) return route.fulfill({ status: 404 })
      return route.fulfill({ body: await readFile(file), contentType: 'image/jpeg' })
    }
    return route.abort()
  })
  // DEMO_OS=windows shows the Windows title bar, for checking it on a Mac.
  await context.addInitScript(bridge, { fixtures: source.fixtures, scenario, language, os: process.env.DEMO_OS, tabs, queues: queues ?? (source === trip ? TRIP_QUEUES : NO_QUEUES), mode })
  const page = await context.newPage()
  page.on('pageerror', (e) => console.error('page error:', e.message))
  await page.goto('http://app.localhost/')
  // The gallery shows thumbnails; Simple mode a preview, Collage mode sheets (FR-048).
  await page.waitForSelector('img[src*="/thumb/"], img[src*="/preview/"], img[src*="/sheet/"]')
  await page.waitForFunction(() => [...document.images].every((i) => i.complete))
  // The start-up screen (FR-060) fades out once start-up is over: never shoot it.
  await page.waitForSelector('#splash', { state: 'detached' })
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
/** Opens the Settings window by its gear (FR-086) at the section named `section`. */
async function openSettings(page, section) {
  if (!(await page.locator('[data-slot="settings-body"]').count())) {
    await page.getByRole('button', { name: /^(Settings|Cài đặt)$/ }).click()
  }
  await page.getByRole('button', { name: section }).click()
  await settle(page)
}

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
// The owner's Prague photos when the set has them (demo_set.txt), the CC0 ones otherwise.
const has = (name) => fixtures.common.list_photos.photos.some((p) => p.fileName.startsWith(name))
const SHOW = {
  grid: idOf(has('P1068073') ? 'P1068073' : 'erdmaennchen-in-der-wilhelma-1'),
  stamp: idOf('bennetts-beach'), // has GPS: the place joins the date
  // The Lumix frame on the owner's tram, the Fujifilm one otherwise.
  frame: idOf(has('P1067865') ? 'P1067865' : 'erdmaennchen-in-der-wilhelma-1'),
  crop: idOf('bir-hakeim-bridge'),
  // A tram by night on a postcard filled to the edge (demo.rs FULL), when the set has it.
  night: has('P1068073') ? idOf('P1068073') : null,
  // A collage sheet by night: a tram and the National Museum, two photos on one print.
  pair: has('P1068256') ? idOf('P1068256') : null,
}
const tile = (page, id) => page.locator(`img[src*="/thumb/${id}?"]`).first()
// The grid is virtualized: scroll it (up, then down) until the photo's tile is drawn.
const reach = async (page, id) => scrollTo(page, tile(page, id))
// A collage sheet's cell holding the photo, scrolled to in the same way.
const cell = (page, id) => scrollTo(page, page.locator(`[data-cell][data-photo="${id}"]`).first())
const scrollTo = async (page, t) => {
  for (const dy of [...Array(12).fill(-900), ...Array(24).fill(900)]) {
    if (await t.count()) break
    await page.mouse.move(700, 500)
    await page.mouse.wheel(0, dy)
    await page.waitForTimeout(150)
  }
  // Drawn in the overscan is not yet on screen: bring it in and let the grid settle.
  await t.scrollIntoViewIfNeeded()
  await page.waitForTimeout(300)
  return t
}

// MEASURE=1: instead of the screenshots, a table of every panel's spacing and control sizes,
// failing when one differs from STYLE_RULE §4's measures (the lego rule; the style-audit skill).
/** Spacing and control sizes inside one panel, as sets of distinct values. */
async function measure(page, rootSel) {
  return page.evaluate((rootSel) => {
    const roots = [...document.querySelectorAll(rootSel)]
    const root = roots[roots.length - 1]
    const r = (n) => Math.round(n * 10) / 10
    const vis = (el) => { const b = el.getBoundingClientRect(); return b.height > 0.5 && b.width > 0.5 }
    const gaps = (parent) => {
      const kids = [...parent.children].filter(vis)
      const out = []
      for (let i = 1; i < kids.length; i++) out.push(r(kids[i].getBoundingClientRect().top - kids[i - 1].getBoundingClientRect().bottom))
      return out
    }
    const set = (xs) => [...new Set(xs)].sort((a, b) => a - b).join(' ')
    const sections = [...root.querySelectorAll('section')].filter(vis)
    const cardGap = set([...new Set(sections.map((s) => s.parentElement))].flatMap((p) => gaps(p).filter((g) => g >= 0)))
    const bodies = sections.map((s) => s.querySelector(':scope > div > div > div')).filter(Boolean).filter(vis)
    const blockGap = set(bodies.flatMap(gaps))
    const fields = [...root.querySelectorAll('section .space-y-1\\.5')].filter(vis)
    const fieldGap = set(fields.flatMap(gaps))
    const odd = fields.filter((f) => gaps(f).some((g) => g > 7)).map((f) => f.textContent.slice(0, 40))
    const ctl = (sel) => set([...root.querySelectorAll(`section ${sel}`)].filter(vis).map((e) => r(e.getBoundingClientRect().height)))
    return {
      width: r(root.getBoundingClientRect().width),
      cardWidth: set(sections.map((s) => r(s.getBoundingClientRect().width))),
      cardGap, blockGap, fieldGap, odd: odd.join(' / '),
      select: ctl('[role="combobox"]'),
      // The whole control, the set-in track (FR-099), not one segment inside it.
      toggle: ctl('[data-slot="toggle-group"]'),
      input: ctl('input:not([type="checkbox"]):not([role="switch"])'),
      iconBtn: ctl('button[data-size="icon-sm"]'),
    }
  }, rootSel)
}

/**
 * Every visible button that shows an icon and no text: how far the icon's centre is from the
 * button's, in CSS pixels (STYLE_RULE §4: an icon is centred, the owner saw toolbar icons off
 * by 2 px when their labels hid).
 */
async function iconOffsets(page) {
  return page.evaluate(() => {
    const out = []
    for (const b of document.querySelectorAll('button')) {
      const box = b.getBoundingClientRect()
      if (box.width < 1 || box.height < 1) continue
      const svg = b.querySelector('svg')
      // Text that takes room on screen, wherever it sits (a hidden label takes none).
      const walker = document.createTreeWalker(b, NodeFilter.SHOW_TEXT)
      let text = false
      for (let n = walker.nextNode(); n && !text; n = walker.nextNode()) {
        if (!n.textContent.trim()) continue
        const range = document.createRange()
        range.selectNodeContents(n)
        text = range.getBoundingClientRect().width > 0
      }
      if (!svg || text) continue
      const i = svg.getBoundingClientRect()
      const dx = Math.abs(i.left + i.width / 2 - (box.left + box.width / 2))
      const dy = Math.abs(i.top + i.height / 2 - (box.top + box.height / 2))
      out.push({ label: b.getAttribute('aria-label') ?? '?', off: Math.round(Math.max(dx, dy) * 10) / 10 })
    }
    return out
  })
}

/** STYLE_RULE §4: each column holds this one value, in every view that has it. */
const MEASURES = { cardGap: '12', blockGap: '12', fieldGap: '6', select: '32', toggle: '32', input: '32', iconBtn: '28' }

async function measurePanels() {
  const rows = []
  const theme = process.env.MEASURE_THEME ?? 'light'
    const language = process.env.MEASURE_LANG ?? 'en'
  let { context, page } = await open({ scenario: 'stamp', language, theme })
  rows.push(['folder list', await measure(page, 'aside.border-r')])
  rows.push(['Pro panel', await measure(page, 'aside.border-l')])
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await settle(page)
  rows.push(['photo preview', await measure(page, '[role="dialog"] aside')])
  await page.screenshot({ path: path.join(outDir, 'measure-preview.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, mode: 'simple' }))
  await settle(page)
  rows.push(['Simple panel', await measure(page, 'aside.border-l')])
  await page.screenshot({ path: path.join(outDir, 'measure-simple.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'collage', language, theme, mode: 'collage', queues: NO_QUEUES }))
  let sheet = page.locator('[data-cell]').nth(2)
  await sheet.click(); await settle(page)
  rows.push(['Collage panel', await measure(page, 'aside.border-l')])
  await page.screenshot({ path: path.join(outDir, 'measure-collage.png') })
  await sheet.dblclick(); await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]'); await settle(page)
  rows.push(['sheet preview', await measure(page, '[role="dialog"] aside')])
  await page.screenshot({ path: path.join(outDir, 'measure-sheet.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'postcard', language, theme, mode: 'collage', queues: NO_QUEUES }))
  sheet = await cell(page, SHOW.frame)
  await sheet.click(); await settle(page)
  rows.push(['Postcard panel', await measure(page, 'aside.border-l')])
  await page.screenshot({ path: path.join(outDir, 'measure-postcard.png') })
  await sheet.dblclick(); await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]'); await settle(page)
  rows.push(['postcard preview', await measure(page, '[role="dialog"] aside')])
  await page.screenshot({ path: path.join(outDir, 'measure-postcard-preview.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  for (const name of [/^(General|Chung)$/, /^(RAM & cache|Bộ nhớ RAM & cache)$/]) {
    await openSettings(page, name)
    rows.push([`settings: ${name.source.slice(2, 9).toLowerCase()}`, await measure(page, '[data-slot="settings-body"]')])
  }
  await page.screenshot({ path: path.join(outDir, 'measure-settings.png') })
  await context.close()
  // Icon-only buttons at the narrowest window and a wide one, in the gallery and Collage mode.
  const offCentre = []
  for (const width of [1180, 1600]) {
    for (const mode of ['professional', 'collage']) {
      ;({ context, page } = await open({ scenario: mode === 'collage' ? 'collage' : 'stamp', language, theme, mode, width, queues: NO_QUEUES }))
      await settle(page)
      for (const b of await iconOffsets(page)) if (b.off > 0.5) offCentre.push(`${mode} at ${width} px: "${b.label}" icon ${b.off} px off centre`)
      await context.close()
    }
  }
  const keys = Object.keys(rows[0][1])
  console.log(['view', ...keys].join(' | '))
  for (const [name, m] of rows) console.log([name, ...keys.map((k) => m[k] || '–')].join(' | '))
  const off = rows.flatMap(([name, m]) =>
    Object.entries(MEASURES).filter(([k, v]) => m[k] && m[k] !== v).map(([k, v]) => `${name}: ${k} is ${m[k]}, not ${v}`),
  )
  const widths = new Set(rows.filter(([name]) => /panel|folder/.test(name)).map(([, m]) => m.width))
  if (widths.size > 1) off.push(`the folder list and the panels differ in width: ${[...widths].join(', ')}`)
  off.push(...offCentre)
  console.log(offCentre.length ? '' : 'icons: every icon-only button has its icon centred (≤ 0.5 px)')
  if (off.length) {
    console.error(off.join('\n'))
    process.exitCode = 1
  }
}

if (process.env.MEASURE) {
  await measurePanels()
  await browser.close()
  process.exit()
}

for (const language of ['en', 'vi']) {
  for (const theme of ['dark', 'light']) {
    const suffix = `${language}-${theme}`

    // 1. The library grid, culled.
    let { context, page } = await open({ scenario: 'stamp', language, theme })
    await (await reach(page, SHOW.grid)).click()
    await settle(page)
    await save(page, `grid-${suffix}`)

    // 1a. For the manual: the filters open (FR-022), and the Project block (FR-023),
    // zoomed in like the print-basket shots.
    await page.getByRole('button', { name: /^(Filters|Bộ lọc)$/ }).click()
    await page.waitForSelector('[role="dialog"], [data-slot="popover-content"]')
    await page.waitForTimeout(300)
    await save(page, `manual-filters-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(200)
    await zoomed(page, `manual-project-${suffix}`, { x: 0, y: 0, width: 576, height: 360 })

    // 1b. Print-size advice for the manual: the picker open, a badge per size (FR-026).
    // By its name: the panel's ids come from useId (STYLE_RULE §4, no hard-coded id).
    await page.getByRole('combobox', { name: /^(Print \/ export size|Khổ in \/ xuất ảnh)$/ }).first().click()
    await page.waitForSelector('[role="option"]')
    await page.waitForTimeout(300)
    await save(page, `manual-size-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(200)
    await page.getByRole('button', { name: /Versions|Các phiên bản/ }).click()
    await page.waitForTimeout(300)
    await save(page, `check-versions-${suffix}`)
    await page.keyboard.press('Escape')

    // 2. Preview with the date stamp (the default style) and the place (GPS photo).
    await (await reach(page, SHOW.stamp)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.stamp}"]`)
    await settle(page)
    await save(page, `stamp-${suffix}`)
    // 2a. For the manual: the photo at a glance, at the top of the inspector (FR-078).
    const summary = await page
      .locator('[role="dialog"] section', { hasText: /^(This photo|Ảnh này)/ })
      .first()
      .boundingBox()
    if (summary) {
      await zoomed(page, `manual-summary-${suffix}`, { x: summary.x - 16, y: summary.y - 16, width: summary.width + 32, height: summary.height + 32 })
    }
    // 2b. The place in the Location card (for checking only), and the place picker for the
    // manual (FR-032).
    const inspector = page.locator('[role="dialog"]')
    await inspector.getByRole('button', { name: /Set the place|Đặt nơi chụp/ }).scrollIntoViewIfNeeded()
    await page.waitForTimeout(300)
    await save(page, `check-place-${suffix}`)
    await inspector.getByRole('button', { name: /Set the place|Đặt nơi chụp/ }).click()
    await page.getByRole('combobox').last().click()
    await page.keyboard.type('pra')
    await page.waitForSelector('[cmdk-item]')
    await page.waitForTimeout(300)
    await save(page, `manual-place-picker-${suffix}`)
    await context.close()

    // 3. Preview with the "Shot on" info frame, and the panel's frame card for the manual.
    ;({ context, page } = await open({ scenario: 'frame', language, theme }))
    // Scroll the panel alone (scrollIntoView would move the page too, and show the splash).
    await page.locator('aside').last().getByText(/^(Info frame|Khung thông tin)$/).first().evaluate((e) => {
      let box = e.parentElement
      while (box && !(box.scrollHeight > box.clientHeight && /auto|scroll/.test(getComputedStyle(box).overflowY))) box = box.parentElement
      if (box) box.scrollTop += e.getBoundingClientRect().top - box.getBoundingClientRect().top - 24
    })
    await page.waitForTimeout(300)
    await save(page, `manual-frame-${suffix}`)
    await page.locator('aside').last().hover()
    // The Places card at the foot of the settings, for the manual (FR-005).
    await page.mouse.wheel(0, 4000)
    await page.waitForTimeout(300)
    await save(page, `manual-places-${suffix}`)
    await (await reach(page, SHOW.frame)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.frame}"]`)
    await settle(page)
    await save(page, `frame-${suffix}`)
    // The export card at the bottom of the inspector: this photo only (FR-055).
    await page.locator('[role="dialog"]').getByRole('button', { name: /^(Export|Xuất ảnh)$/ }).click()
    await page.waitForTimeout(600)
    await save(page, `check-export-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
    await (await reach(page, SHOW.crop)).dblclick()
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

    // 5. Best-shot groups, on the cat burst.
    ;({ context, page } = await open({ scenario: 'stamp', language, theme, source: cat }))
    await page.getByRole('radio', { name: /Groups|Nhóm/ }).click()
    await settle(page)
    await save(page, `manual-groups-${suffix}`)
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
    await context.close()
    // 6b. The preview of a photo in the basket: ↑ keeps it, ↓ sets it aside. The owner's cat,
    // its sharp shot in the basket.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 3, source: cat, queues: { print: [0], aside: [] } }))
    await page.locator('img[src*="/thumb/"]').first().dblclick()
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
    // 6d. For the manual: each shared photo whole in a square frame, a fill round it (FR-075).
    await page.getByRole('switch', { name: /^(Whole photo in a frame|Giữ trọn ảnh trong khung)$/ }).click()
    await page.waitForTimeout(500)
    const fillCard = await page.locator('section', { has: page.getByRole('switch', { name: /^(Whole photo in a frame|Giữ trọn ảnh trong khung)$/ }) }).first().boundingBox()
    if (fillCard) {
      const c = fillCard
      await zoomed(page, `manual-share-fill-${suffix}`, { x: c.x - 16, y: c.y - 16, width: c.width + 32, height: c.height + 32 })
      // For checking by eye: a colour of one's own, with its picker and code.
      await page.getByRole('radio', { name: /^(Custom|Tùy chọn)$/ }).click()
      await page.waitForTimeout(400)
      const open = await page.locator('section', { has: page.getByRole('switch', { name: /^(Whole photo in a frame|Giữ trọn ảnh trong khung)$/ }) }).first().boundingBox()
      if (open) await zoomed(page, `check-share-fill-${suffix}`, { x: open.x - 16, y: open.y - 16, width: open.width + 32, height: open.height + 32 })
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
    await tab('Praha').click()
    await page.waitForTimeout(300)
    await page.locator('img[src*="/thumb/"]').nth(2).click()
    await tab('Travels').hover()
    await settle(page)
    await zoomed(page, `tabs-${suffix}`, { x: 0, y: 0, width: 1040, height: 650 })
    await context.close()

    // 9. Simple mode (FR-048): one photo as it will print, ↑ keep, ↓ drop.
    ;({ context, page } = await open({ scenario: 'stamp', language, theme, mode: 'simple' }))
    await settle(page)
    await save(page, `simple-${suffix}`)
    await context.close()

    // 9b. The Settings window (FR-086): the gear beside the byline, at *RAM & cache*.
    ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
    await openSettings(page, /^(RAM & cache|Bộ nhớ RAM & cache)$/)
    await save(page, `settings-${suffix}`)
    await context.close()

    // 10. Collage mode (FR-035): two photos on one print, the panel with its size and cut line.
    ;({ context, page } = await open({ scenario: 'collage', language, theme, mode: 'collage', queues: NO_QUEUES }))
    const sheet = SHOW.pair !== null ? await cell(page, SHOW.pair) : page.locator('[data-cell]').nth(2)
    await sheet.click()
    await settle(page)
    await save(page, `collage-${suffix}`)
    // 10b. A sheet's preview (FR-035), opened like a photo's.
    await sheet.dblclick()
    await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]')
    await settle(page)
    await save(page, `manual-collage-preview-${suffix}`)
    await save(page, `collage-preview-${suffix}`) // the same preview, for the landing page
    await context.close()

    // 11. Postcards (FR-073): one photo and a postcard back on each print, the panel's
    // postcard card open, for the landing page.
    ;({ context, page } = await open({ scenario: 'postcard', language, theme, mode: 'collage', queues: NO_QUEUES }))
    // The card on the owner's red tram, which carries the first typed text (demo.rs).
    const card = await cell(page, SHOW.frame)
    await card.click()
    await settle(page)
    await save(page, `postcard-${suffix}`)
    await context.close()

    // 11b. The other way to set a postcard: trams by night, each photo filling its half with
    // no border; the sheets for the manual, then a card's preview with its typed text, for the
    // manual and the landing page.
    if (SHOW.night !== null) {
      ;({ context, page } = await open({ scenario: 'postcard-full', language, theme, mode: 'collage', queues: NO_QUEUES }))
      const night = await cell(page, SHOW.night)
      await night.click()
      await settle(page)
      await save(page, `manual-postcard-${suffix}`)
      await night.dblclick()
      await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]')
      await settle(page)
      await save(page, `postcard-night-${suffix}`)
      // 11c. For the manual (FR-077): the typed address, "Use for N postcards" under it; then
      // a card without an address, and the recent addresses offered on it.
      const textCard = (pattern) =>
        page.locator('[role="dialog"] section', { has: page.getByText(pattern) }).first()
      const apply = textCard(/^(Use for|Dùng cho) \d+/)
      if (await apply.count()) {
        await apply.scrollIntoViewIfNeeded()
        await settle(page)
        const box = await apply.boundingBox()
        if (box) await zoomed(page, `manual-postcard-apply-${suffix}`, { x: box.x - 16, y: box.y - 16, width: box.width + 32, height: box.height + 32 })
      }
      for (let i = 0; i < 8 && !(await page.getByText(/^(Recent addresses|Địa chỉ gần đây)$/).count()); i++) {
        await page.keyboard.press('ArrowRight')
        await settle(page)
      }
      const recent = textCard(/^(Recent addresses|Địa chỉ gần đây)$/)
      if (await recent.count()) {
        await recent.scrollIntoViewIfNeeded()
        await settle(page)
        const box = await recent.boundingBox()
        if (box) await zoomed(page, `manual-postcard-recent-${suffix}`, { x: box.x - 16, y: box.y - 16, width: box.width + 32, height: box.height + 32 })
      }
      await context.close()
    }
  }
}

await browser.close()
console.log(shots.map((s) => path.relative(root, s)).join('\n'))
