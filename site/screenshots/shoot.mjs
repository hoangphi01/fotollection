// SPDX-License-Identifier: GPL-3.0-only
// Screenshots of the real interface for the website (site/screenshots/README.md).
//
//   node site/screenshots/shoot.mjs <demo-dump> [out-dir] [best-shot-dump]
//   MEASURE=1 node site/screenshots/shoot.mjs <demo-dump> <out-dir>   (the panels' measures)
//
// Serves ui/dist, answers IPC with the answers the real app recorded for the demo photos
// (src-tauri/src/demo.rs), and serves the recorded thumbnails and previews. No Tauri, no
// network: every request outside the two local hosts is refused.

import { axeChecks, baseline, fitReport, layoutChecks, stableChecks } from './checks.mjs'
import { measureSidePanels } from './side-panels.mjs'
import { readFile } from 'node:fs/promises'
import { existsSync, writeFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium, webkit } from 'playwright'
import { pseudo } from '../../ui/src/i18n/pseudo.ts'

// Every locale the checks may run in (FR-123, FR-125), so a button is found by its words in any
// of them: `say('export.button')` instead of a pattern written in two languages.
const localeDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '../../ui/src/locales')
const english = JSON.parse(await readFile(path.join(localeDir, 'en.json'), 'utf8'))
const LOCALES = [english, pseudo(english)]
const ENGLISH_WORDS = (function flat(tree) {
  return Object.values(tree).flatMap((v) => (typeof v === 'string' ? [v] : flat(v)))
})(english)
for (const code of ['de', 'hu', 'ja', 'ko', 'vi']) LOCALES.push(JSON.parse(await readFile(path.join(localeDir, `${code}.json`), 'utf8')))

/**
 * A pattern for a key's words in every locale, its plural forms too; a placeholder matches
 * anything. Whole by default; `{ whole: false }` matches the words anywhere in a name, and
 * `{ whole: false, start: true }` at its start.
 */
function say(key, { whole = true, start = false } = {}) {
  const words = new Set()
  for (const locale of LOCALES) {
    const parts = key.split('.')
    const leaf = parts.pop()
    const node = parts.reduce((o, k) => o?.[k], locale) ?? {}
    for (const [k, v] of Object.entries(node)) if (k === leaf || k.startsWith(`${leaf}_`)) words.add(v)
  }
  if (words.size === 0) throw new Error(`say: no words for ${key}`)
  const alts = [...words].map((w) => w.split(/\{\{[^}]+\}\}/).map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('.+?'))
  const body = `(?:${alts.join('|')})`
  return new RegExp(whole ? `^${body}$` : start ? `^${body}` : body)
}

/** A Settings section (FR-086): its pattern in every locale, and a short English tag for names. */
function section(key) {
  return { re: say(`settingsWindow.sections.${key}`), tag: english.settingsWindow.sections[key].slice(0, 6).replace(/\W/g, '') }
}

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
function bridge({ fixtures, scenario, language, os, tabs, queues, mode, ui, update, settingsPatch, verdict, profile }) {
  const { common, scenarios } = fixtures
  // Collage mode (FR-035) has its own recording; older dumps have none.
  // Postcards (FR-073) are recorded after the collage, as its own sheets; older dumps have none.
  const empty = { list_sheets: [], sheet_info: {}, get_crop_geometry: {}, get_postcard_text: {} }
  const collage = (scenario === 'postcard' ? fixtures.postcard : scenario === 'postcard-full' ? fixtures.postcard_full : fixtures.collage) ?? empty
  const look = scenarios[scenario] ?? scenarios.stamp
  const byId = (table, args) => table[String(args?.id)]
  let current = ['collage', 'postcard', 'postcard-full'].includes(scenario) ? collage.get_settings : look.get_settings
  // A state the recording lacks (docs/UI_TESTS.md, the stress fixtures): a long custom size…
  if (settingsPatch) current = { ...current, settings: { ...current.settings, ...settingsPatch } }
  // The print basket and set-aside photos (FR-050), by position (TRIP_QUEUES).
  const withQueues = (photos) =>
    photos.map((p, i) => ({ ...p, queue: queues.print.includes(i) ? 'print' : queues.aside.includes(i) ? 'aside' : null }))
  // The folder-tabs shot (FR-056): the demo photos as three folders, by file name.
  // The set's folder tabs (PROFILES `tabs`, passed as data: this runs in the page).
  const tabOf = (name) => {
    const rule = profile.tabRules.find(([pattern]) => new RegExp(pattern).test(name))
    return rule ? [rule[1], rule[2]] : profile.tabElse
  }
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
    // Nothing to tell at start-up (ADR-0048: a damaged settings file would be).
    take_startup_notices: () => [],
    list_print_sizes: () => common.list_print_sizes,
    list_fonts: () => common.list_fonts,
    // The welcome (FR-109) seen, so it covers no shot; a shot of it sets `ui.welcomeSeen: false`.
    get_ui_settings: () => ({ ...common.get_ui_settings, welcomeSeen: true, language, mode, ...ui }),
    // The update window at start (FR-103) and Settings' copy of it, when a shot asks for it.
    check_for_update_at_start: () => update, last_update_check: () => update,
    // The preview's colour cards (FR-079): the scenario's look, shared by every photo.
    get_photo_look: () => ({ look: current.settings.look, custom: false, raw: null }),
    // The Looks card and Simple mode's strip (FR-079): the built-in looks, as recorded.
    list_looks: () => common.list_looks ?? [],
    // FR-114: what a batch of colour covers; no photo of the demo has a colour of its own, so
    // the Colour tab shows no reset.
    look_batch_plan: () => ({ total: common.photo_count ?? 9, edited: 0 }),
    // The histogram (FR-098): counted here from the preview the shot shows, as Rust counts it.
    photo_histogram: async (args) => {
      for (let i = 0; i < 60; i++) {
        const shown = document.querySelector(`img[src*="/preview/${args?.id}"]`)
        if (shown?.complete && shown.naturalWidth) {
          const img = new Image()
          img.crossOrigin = 'anonymous'
          img.src = shown.src
          try {
            await img.decode()
          } catch {
            return null
          }
          const w = Math.max(1, Math.round(img.naturalWidth / 4))
          const h = Math.max(1, Math.round(img.naturalHeight / 4))
          const canvas = Object.assign(document.createElement('canvas'), { width: w, height: h })
          const g = canvas.getContext('2d')
          g.drawImage(img, 0, 0, w, h)
          let data
          try {
            data = g.getImageData(0, 0, w, h).data
          } catch {
            return null // a picture from another origin cannot be read
          }
          const bins = [0, 1, 2, 3].map(() => new Array(256).fill(0))
          let high = 0
          let low = 0
          for (let k = 0; k < data.length; k += 4) {
            const [r, gr, b] = [data[k], data[k + 1], data[k + 2]]
            bins[0][r]++, bins[1][gr]++, bins[2][b]++, bins[3][(54 * r + 183 * gr + 19 * b) >> 8]++
            if (r >= 254 || gr >= 254 || b >= 254) high++
            if (r <= 1 || gr <= 1 || b <= 1) low++
          }
          const peak = Math.max(1, ...bins.flat())
          const scale = (c) => c.map((n) => Math.floor((n * 1000) / peak))
          const n = data.length / 4
          return { r: scale(bins[0]), g: scale(bins[1]), b: scale(bins[2]), l: scale(bins[3]), highPermille: Math.floor((high * 1000) / n), lowPermille: Math.floor((low * 1000) / n) }
        }
        await new Promise((r) => setTimeout(r, 100))
      }
      return null
    },
    // The export dialog's example name (FR-104): the first photo's own name.
    export_name_example: () => profile.exportName,
    // The Settings window's memory and cache (FR-086): a 16 GB Mac at 25 %, a trip's cache.
    memory_info: () => ({ total: 16 * 2 ** 30, budget: 4 * 2 ** 30, held: 1.3 * 2 ** 30 }),
    machine_info: () => ({ cpu: 'Apple M3', cores: 8, memory: 16 * 2 ** 30, freeDisk: 31e9 }),
    // Settings → Data (FR-116): each store with a plausible size, two unused LUTs.
    data_usage: () =>
      [
        ['settings', 18_400, 1, 0], ['looks', 0, 6, 0], ['luts', 2_310_000, 9, 2], ['fonts', 412_000, 2, 0],
        ['logos', 38_000, 1, 0], ['addresses', 0, 3, 0], ['cameraSessions', 0, 0, 0], ['cache', 1_840_000_000, 5210, 0],
        ['logs', 96_000, 2, 0],
      ].map(([kind, bytes, count, unused]) => ({ kind, bytes, count, unused })),
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
    // AddAssist (FR-133): its countries; a lookup or search answers nothing in the demo, and an
    // address set on a card changes nothing (writes do nothing here).
    addassist_countries: () => [
      { country: 'jp', codeDigits: 7, dataDate: '2026-10-23', version: '1.1' },
      { country: 'kr', codeDigits: 5, dataDate: '2025-11-30', version: '1.1' },
    ],
    // The address book (FR-133): the profile's saved recipients; nothing recent in the demo,
    // and no other card without the recipient (writes do nothing here).
    address_book: () => ({ saved: profile.book ?? [], recent: [] }),
    postcard_recipient_gaps: () => ({ blank: 0, differ: 0 }),
    addassist_lookup: () => [],
    addassist_search: () => [],
    recent_addresses: () => profile.recent,
    sheet_info: (a) => collage.sheet_info[a.sheet.second == null ? `${a.sheet.first}` : `${a.sheet.first}-${a.sheet.second}`],
    get_document: () => ({ ...common.get_document, edited: false, name: 'Trip 2018' }),
    list_groups: () => common.list_groups,
    sort_facets: () => common.sort_facets, // RollSense (FR-141): recorded; its pass is done
    analyze_sorting: () => 1,
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
        queueCounts: Object.fromEntries(['print', 'aside', 'delete'].map((q) => [q, all.filter((p) => p.queue === q).length])),
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
    // A stress state (UI-LAYOUT-024): every size at one verdict, as a small photo gets.
    print_advice: (a) => {
      const advice = byId(common.print_advice ?? {}, a)
      return verdict && advice ? { ...advice, sizes: advice.sizes.map((s) => ({ ...s, verdict, dpi: 271 })) } : advice
    },
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

// MEASURE_ENGINE=webkit measures in WebKit, the engine of the app's window on macOS and Linux.
// Scroll bars shown, as in the app (headless Chromium hides them by default): an 8 px bar takes
// room inside a strip, and a ring cut by it only shows with it (LESSONS, UI-LAYOUT-016).
const browser = await (process.env.MEASURE_ENGINE === 'webkit' ? webkit : chromium).launch(
  process.env.MEASURE_ENGINE === 'webkit' ? {} : { ignoreDefaultArgs: ['--hide-scrollbars'] },
)
const shots = []

// Demo culling state for the print basket (FR-050), by position: the trip's, or none.
// The languages shot from this recording (SHOT_LANGS, comma-separated; en,vi by default):
// build.sh records each language's own photo set in that language (demo.rs), so a はがき
// shows on the Japanese pages and a 우편엽서 on the Korean ones.
const LANGUAGES = (process.env.SHOT_LANGS ?? 'en,vi').split(',').map((l) => l.trim()).filter(Boolean)
// Each photo set's picks by file name (PRD FR-130, "Its pictures"): the grid's photo, the one
// with GPS for the date and place, the info frame, the postcard, the straightening, the before
// and after, the full-bleed card; the folder tabs, the export's example name and the recent
// addresses (famous places' public addresses, owner 2026-11-15).
const PROFILES = {
  prague: {
    grid: ['P1068073', 'erdmaennchen-in-der-wilhelma-1'],
    stamp: ['bennetts-beach'],
    frame: ['P1067865', 'erdmaennchen-in-der-wilhelma-1'],
    card: ['P1067865', 'erdmaennchen-in-der-wilhelma-1'],
    crop: ['bir-hakeim-bridge'],
    compare: ['P1067950', 'charles-bridge-at-sunset'],
    night: ['P1068073'],
    pair: ['P1068256'],
    tabRules: [['wilhelma', 0, 'Wilhelma Zoo'], ['tuebingen|ammerschlag|anlagensee|neckar', 1, 'Tübingen'], ['^P10|charles-bridge', 2, 'Praha']],
    tabElse: [3, 'Travels'],
    tabs: ['Wilhelma Zoo', 'Praha', 'Travels'],
    exportName: 'P1067823.jpg',
    recent: ['Národní muzeum\nVáclavské náměstí 68\n110 00 Praha 1, Czechia', 'Magyar Nemzeti Múzeum\nMúzeum krt. 14–16\n1088 Budapest, Hungary'],
    // The address book (FR-133, AddAssist 1.1): famous places' public addresses, demo names.
    book: [{ id: 1, label: 'Magyar Nemzeti Múzeum', recipient: { kind: 'free', value: 'Magyar Nemzeti Múzeum\nMúzeum krt. 14–16\n1088 Budapest, Hungary' } }],
  },
  // The owner's photos of Japan (JAPAN DEMO by hoangphi01), their city read from the GPS.
  ja: {
    grid: ['IMG_5773'],
    stamp: ['IMG_5745'],
    frame: ['IMG_5745'],
    card: ['Gokurakuji', 'IMG_5710'],
    crop: ['IMG_5758'],
    compare: ['IMG_4453'],
    night: ['IMG_5773'],
    pair: [],
    tabRules: [['IMG_57(10|45)|Gokurakuji', 0, '鎌倉'], ['IMG_57(58|73)', 1, '東京']],
    tabElse: [2, '旅行'],
    tabs: ['鎌倉', '東京', '旅行'],
    exportName: 'IMG_5773.heic',
    queues: { print: ['IMG_4329'], aside: ['IMG_5710'] },
    recent: ['東京都港区芝公園4-2-8 東京タワー', '京都府京都市東山区清水1-294 清水寺'],
    book: [{ id: 1, label: '清水寺', recipient: { kind: 'assisted', value: { country: 'jp', filledBy: '1.1', code: '6050862', region: '京都府', city: '京都市東山区', town: '清水', road: '', line: '1-294', name: '鈴木花子', honorific: true, latin: false } } }],
  },
  // Seoul by @9adox (Fujifilm XF10) and @imyourctus (Sony ZV-1).
  ko: {
    grid: ['DSCF1990'],
    stamp: ['DSCF1983'],
    frame: ['DSCF1909'],
    card: ['DSC06332'],
    crop: ['DSCF1905'],
    compare: ['DSCF1938'],
    night: ['DSCF2041'],
    // Two upright photos side by side, as the keynote's collage scene shows them (2026-11-15).
    pair: ['IMG_1471'],
    tabRules: [['^(IMG_14|DSC0)', 1, 'ZV-1'], ['DSCF19[0-6]', 0, '광화문']],
    tabElse: [2, '서울'],
    tabs: ['광화문', '서울', 'ZV-1'],
    exportName: 'DSCF1990.JPG',
    queues: { print: ['DSCF1986', 'DSCF2022', 'DSCF1915'], aside: ['DSCF2068'] },
    recent: ['서울특별시 종로구 세종대로 175 세종문화회관', '서울특별시 종로구 사직로 161 경복궁'],
    book: [{ id: 1, label: '경복궁', recipient: { kind: 'assisted', value: { country: 'kr', filledBy: '1.1', code: '03045', region: '서울특별시', city: '종로구', town: '', road: '사직로', line: '161', name: '이서연', honorific: true, latin: false } } }],
  },
}
const PROFILE = PROFILES[process.env.SHOT_SET ?? LANGUAGES[0]] ?? PROFILES.prague

// The print basket and set-aside photos by position, never the shown ones (SHOW): the Prague
// set's, or a set's own by file name (PROFILES `queues`).
const TRIP_QUEUES = (() => {
  const named = PROFILE.queues
  if (!named) return { print: [1, 4, 6], aside: [7, 13] }
  const at = (stem) => fixtures.common.list_photos.photos.findIndex((p) => p.fileName.startsWith(stem))
  return { print: named.print.map(at).filter((i) => i >= 0), aside: named.aside.map(at).filter((i) => i >= 0) }
})()
const NO_QUEUES = { print: [], aside: [] }

async function open({ scenario, language, theme, width = Number(process.env.DEMO_WIDTH) || 1440, height = 900, scale = 2, tabs = false, source = trip, queues, mode = 'professional', ui = {}, update = null, settingsPatch = null, fault = null, verdict = null, os = process.env.DEMO_OS }) {
  const { dump } = source
  // DEMO_WIDTH=1000 checks narrow windows (the screenshots for the site stay at 1440).
  const context = await browser.newContext({
    viewport: { width, height },
    deviceScaleFactor: scale,
    colorScheme: theme,
    // WebView2-style image URLs (http://fotothumb.localhost), which a test browser can serve.
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36',
    locale: { vi: 'vi-VN', hu: 'hu-HU', de: 'de-DE', ja: 'ja-JP', ko: 'ko-KR' }[language] ?? 'en-GB',
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
      if (kind === 'looktile') {
        // A look's tile (FR-079): `{photo}-{look id, ':' as '-'}`, recorded by demo.rs.
        const tile = path.join(dump, 'img/looktile', `${id}-${(url.searchParams.get('look') ?? '').replace(':', '-')}.jpg`)
        if (!existsSync(tile)) return route.fulfill({ status: 404 })
        return route.fulfill({ body: await readFile(tile), contentType: 'image/jpeg' })
      }
      // The look scenario's *Before* is its plain preview, recorded beside it (demo.rs).
      const shown = kind === 'preview' && scenario === 'look' && url.searchParams.get('plain') ? 'look-plain' : scenario
      const file = kind === 'preview' ? path.join(dump, 'img/preview', shown, `${id}.jpg`) : path.join(dump, 'img', kind, `${id}.jpg`)
      if (!existsSync(file)) return route.fulfill({ status: 404 })
      // Readable from the page (the histogram's canvas), as Tauri's protocol is to the app.
      return route.fulfill({ body: await readFile(file), contentType: 'image/jpeg', headers: { 'Access-Control-Allow-Origin': '*' } })
    }
    return route.abort()
  })
  // DEMO_OS=windows shows the Windows title bar, for checking it on a Mac.
  // UI-STATE-010: a part told to fail (`lib/fault.ts`, in builds made with VITE_FAULTS=1).
  if (fault) await context.addInitScript((part) => { window.__fotollectionFault = part }, fault)
  // The interface's English words (en.json), so UI-TEXT-007 judges only them, never a user's
  // address or file name (FR-125).
  await context.addInitScript((words) => {
    globalThis.__englishWords = words
  }, ENGLISH_WORDS)
  await context.addInitScript(bridge, { fixtures: source.fixtures, scenario, language, os, tabs, queues: queues ?? (source === trip ? TRIP_QUEUES : NO_QUEUES), mode, ui, update, settingsPatch, verdict, profile: { tabRules: PROFILE.tabRules, tabElse: PROFILE.tabElse, exportName: PROFILE.exportName, recent: PROFILE.recent, book: PROFILE.book } })
  const page = await context.newPage()
  page.on('pageerror', (e) => console.error('page error:', e.message))
  await page.goto('http://app.localhost/')
  // The gallery shows thumbnails; Simple mode a preview, Collage mode sheets (FR-048).
  // A part broken on purpose may be the one with the photos (UI-STATE-010).
  if (!fault) await page.waitForSelector('img[src*="/thumb/"], img[src*="/preview/"], img[src*="/sheet/"]')
  // Every picture on screen loaded; a lazy one out of view never loads in WebKit (LESSONS
  // 2026-11-13: 35 photos in a 700 px window), so it is not waited for.
  await page.waitForFunction(() =>
    [...document.images].every((i) => {
      if (i.complete) return true
      const r = i.getBoundingClientRect()
      return i.loading === 'lazy' && (r.bottom < 0 || r.top > innerHeight || r.right < 0 || r.left > innerWidth)
    }),
  )
  // The start-up screen (FR-060) fades out once start-up is over: never shoot it.
  if (!fault) await page.waitForSelector('#splash', { state: 'detached' })
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
  // A lazy picture out of view (the manual's) loads only when scrolled to: WebKit never starts it.
  await page.waitForFunction(() => [...document.images].every((i) => (i.complete && i.naturalWidth > 0) || (i.loading === 'lazy' && !i.complete)))
  await page.waitForTimeout(300)
}

/** A zoomed-in part of the window, 16:10 like the full screenshots. */
/** Opens the Settings window by its gear (FR-086) at the section named `section`. */
async function openSettings(page, section) {
  if (!(await page.locator('[data-slot="settings-body"]').count())) {
    await page.getByRole('button', { name: say('settingsWindow.title') }).click()
  }
  await page.getByRole('button', { name: section }).click()
  await settle(page)
}

async function zoomed(page, name, clip) {
  const file = path.join(outDir, `${name}.png`)
  await page.screenshot({ path: file, clip })
  shots.push(file)
  await phone(page, name, clip)
}


async function save(page, name) {
  const file = path.join(outDir, `${name}.png`)
  await page.screenshot({ path: file })
  shots.push(file)
  await phone(page, name)
}

// FR-135, legible on a phone: each picture of the landing and Features pages also comes as a
// crop of the part worth reading, 4:5, served below 600 px (shot.html). Its smallest text, as
// shown on a 390 px phone, is at least PHONE_MIN px: the DOM's text inside the crop, each run
// measured where it is topmost (not behind a dialog's veil), its size scaled from the crop's
// width to PHONE_SHOWN. Text inside a picture (a photo, a rendered sheet) is the print itself
// and is not measured. A crop under the minimum fails the run; its facts go to phone.json.
const PHONE_SHOWN = 328 // site.css .carousel-track, min(84vw, 760px) on a 390 px phone: the narrowest frame
const PHONE_MIN = 11
const PHONE_W = 320
// Top-left corner of each crop in the 1440 × 900 window, on the part the picture's caption is
// about (owner, 2026-11-15: a crop that misses its caption means nothing), or a function giving
// the box to centre it on.
const PHONE = {
  grid: [310, 190], // starred tiles, their badges
  compare: [560, 140], // After, beside Before
  window: [0, 340], // Navigator, Presets & LUTs on the left
  'export-dialog': [170, 60], // the export's presets
  groups: [40, 270], // the best shot and its bars
  select: [650, 100], // ticked photos
  swipe: [310, 60], // In the basket, on the preview
  estimate: 'zoom', // the size estimate on its button: the middle of the zoomed part
  tabs: [0, 320], // the folders as tabs, a tab's hint
  simple: [560, 480], // the photo as it prints, its looks strip, Drop and Keep
  settings: [180, 90], // one window for every setting: its sections, Data among them
  collage: (page) => page.evaluate(sheetBox, SHOW.pair), // the chosen sheet, two photos on one print
  // The sheet's first photo whole (a crop on the cut line showed two halves): its left half.
  'collage-preview': async (page) => {
    const b = await page.locator('[role="dialog"] img[src*="/sheet/"]').first().boundingBox()
    return b && { x: b.x, y: b.y, width: b.width / 2, height: b.height }
  },
  postcard: [290, 300], // a postcard sheet
  'postcard-night': [566, 150], // the card's typed words
  addassist: [1100, 120], // the address AddAssist filled
  stamp: 'photo-corner', // the date stamp: the photo's bottom right corner
  frame: 'photo-foot', // the info frame: the middle of the photo's foot
  crop: [400, 160], // the crop's size and sharpness
}
const phoneFacts = {}
// The sheet holding a photo in Collage mode (its cell's sheet image), else the third cell's.
const sheetBox = (id) => {
  const cell = (id !== null && document.querySelector(`[data-cell][data-photo="${id}"]`)) || document.querySelectorAll('[data-cell]')[2]
  // Up from the cell (itself `relative`) to the tile that holds the sheet's image.
  let tile = cell?.parentElement ?? null
  while (tile && !tile.querySelector('img')) tile = tile.parentElement
  const r = tile?.querySelector('img')?.getBoundingClientRect()
  return r ? { x: r.x, y: r.y, width: r.width, height: r.height } : null
}
const smallestText = (clip) => {
  let min = null
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT)
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const el = n.parentElement
    if (!el || !n.textContent.trim()) continue
    const style = getComputedStyle(el)
    if (style.visibility === 'hidden') continue
    const range = document.createRange()
    range.selectNodeContents(n)
    for (const r of range.getClientRects()) {
      if (r.width < 2 || r.height < 2) continue
      if (r.right <= clip.x || r.left >= clip.x + clip.width || r.bottom <= clip.y || r.top >= clip.y + clip.height) continue
      const top = document.elementFromPoint(Math.min(Math.max(r.left + r.width / 2, 0), innerWidth - 1), Math.min(Math.max(r.top + r.height / 2, 0), innerHeight - 1))
      if (!top || !(top === el || el.contains(top) || top.contains(el))) continue
      const box = el.getBoundingClientRect()
      const scale = el.offsetWidth > 0 ? box.width / el.offsetWidth : 1
      const px = parseFloat(style.fontSize) * scale
      if (min === null || px < min.px) min = { px, text: n.textContent.trim().slice(0, 40) }
    }
  }
  return min
}
async function phone(page, name, zoom) {
  const m = name.match(/^(.+)-([a-z]{2})-(dark|light)$/)
  const at = m && PHONE[m[1]]
  if (!at) return
  const height = PHONE_W * 1.25
  let [x, y] = Array.isArray(at) ? at : [0, 0]
  if (typeof at === 'function') {
    const b = await at(page)
    if (!b) throw new Error(`${name}: nothing to centre its phone crop on`)
    ;[x, y] = [b.x + b.width / 2 - PHONE_W / 2, b.y + b.height / 2 - height / 2]
  }
  if (at === 'zoom') [x, y] = [zoom.x + zoom.width / 2 - PHONE_W / 2, zoom.y + zoom.height / 2 - height / 2]
  if (at === 'photo-corner' || at === 'photo-foot') {
    // The photo in the preview, wherever its shape puts it in each language's set.
    const boxes = await page.locator('img[src*="/preview/"]').evaluateAll((imgs) => imgs.map((i) => i.getBoundingClientRect()).map((r) => ({ x: r.x, y: r.y, w: r.width, h: r.height })))
    const photo = boxes.sort((a, b) => b.w * b.h - a.w * a.h)[0]
    if (!photo) throw new Error(`${name}: no photo in the preview for its phone crop`)
    // A wide photo's frame is too wide for the crop: its right end, where "Shot on" is.
    x = at === 'photo-corner' || photo.w > PHONE_W * 1.5 ? photo.x + photo.w - PHONE_W + 8 : photo.x + photo.w / 2 - PHONE_W / 2
    y = photo.y + photo.h - height + 8
  }
  const clip = { x: Math.round(Math.min(Math.max(x, 0), 1440 - PHONE_W)), y: Math.round(Math.min(Math.max(y, 0), 900 - height)), width: PHONE_W, height }
  const min = await page.evaluate(smallestText, clip)
  const shown = min === null ? null : Math.round((min.px * PHONE_SHOWN * 100) / clip.width) / 100
  const crop = `${m[1]}-phone-${m[2]}-${m[3]}`
  const file = path.join(outDir, `${crop}.png`)
  await page.screenshot({ path: file, clip })
  shots.push(file)
  phoneFacts[crop] = { width: clip.width, height: clip.height, minPx: shown, text: min?.text ?? null }
  if (process.env.PHONE_WHOLE) {
    // The check shown failing on the pictures as they were: the whole window in the same frame.
    const whole = await page.evaluate(smallestText, { x: 0, y: 0, width: 1440, height: 900 })
    console.log(`${name}: the whole window shows its smallest text at ${whole ? ((whole.px * PHONE_SHOWN) / 1440).toFixed(2) : '-'} px on a phone`)
  }
  if (shown !== null && shown < PHONE_MIN) {
    console.error(`${crop}: its smallest text shows at ${shown} px on a phone ("${min.text}"), under ${PHONE_MIN} px (FR-135)`)
    process.exitCode = 1
  }
}

// Photos by file name, so adding demo photos never changes which one is shown.
const idOf = (name) => {
  const photo = fixtures.common.list_photos.photos.find((p) => p.fileName.startsWith(name))
  if (!photo) throw new Error(`demo photo ${name} is missing`)
  return photo.id
}
// The owner's Prague photos when the set has them (demo_set.txt), the CC0 ones otherwise.
const has = (name) => fixtures.common.list_photos.photos.some((p) => p.fileName.startsWith(name))
// The first of a pick's names the set has; none, null.
const pick = (names) => {
  const name = names.find(has)
  return name === undefined ? null : idOf(name)
}
const SHOW = {
  grid: pick(PROFILE.grid),
  stamp: pick(PROFILE.stamp), // has GPS (in the Prague set): the place joins the date
  frame: pick(PROFILE.frame),
  // The card carrying the first typed text (demo.rs).
  card: pick(PROFILE.card),
  crop: pick(PROFILE.crop),
  compare: pick(PROFILE.compare),
  // A postcard filled to the edge (demo.rs FULL), when the set has it.
  night: pick(PROFILE.night),
  // A collage sheet of two photos, when the set has one named.
  pair: pick(PROFILE.pair),
}
for (const [what, id] of Object.entries(SHOW)) {
  if (id === null && !['night', 'pair'].includes(what)) throw new Error(`demo photo for ${what} is missing`)
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
/** Folded cards taller than their header (owner, 2026-11-07: a band under a folded card's
 * title): every `SectionCard` whose body is folded and that has no footer must be its header. */
async function foldedBands(page, where) {
  const bands = await page.evaluate(() =>
    [...document.querySelectorAll('section.rounded-xl > header')].flatMap((header) => {
      const card = header.parentElement
      const body = card.querySelector(':scope > [aria-hidden="true"]')
      if (!body || card.children.length !== 2) return []
      const extra = card.clientHeight - header.offsetHeight
      return extra > 0.5 ? [`"${header.textContent.trim()}" ${Math.round(extra * 10) / 10} px under its header`] : []
    }),
  )
  return bands.map((b) => `${where}: UI-LAYOUT-006 folded card ${b}`)
}

async function measure(page, rootSel) {
  return page.evaluate((rootSel) => {
    const roots = [...document.querySelectorAll(rootSel)]
    const root = roots[roots.length - 1]
    // Whole pixels: WebKit lays out in fractions (11.9 for a 12 px gap).
    const r = (n) => Math.round(n)
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
      // The three spacings (STYLE_RULE §4): a list's rows 4 px, the mode switch's rows 12 px.
      listGap: set([...root.querySelectorAll('ul.flex-col.gap-1, nav.flex-col.gap-1, div.flex-col.gap-1')].filter(vis).flatMap(gaps)),
      modeGap: set([...root.querySelectorAll('.grid.gap-y-3')].filter(vis).map((g) => {
        const items = [...g.children].filter(vis)
        return items.length > 2 ? r(items[2].getBoundingClientRect().top - items[0].getBoundingClientRect().bottom) : null
      }).filter((v) => v !== null)),
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

/**
 * What overflows on screen (FR-099, step 5; LESSONS: the owner found them by eye): a
 * control's text wider or taller than its box, unless it is meant to end in an ellipsis; a
 * head, toolbar or button group whose last child passes its right edge; a segment's text off
 * its box's centre by more than 0.5 px. Each as a line naming the place.
 */
async function overflows(page, where) {
  const found = await page.evaluate(() => {
    const out = []
    const vis = (el) => {
      const b = el.getBoundingClientRect()
      return b.width > 0.5 && b.height > 0.5 && getComputedStyle(el).visibility !== 'hidden'
    }
    const name = (el) => (el.getAttribute('aria-label') || el.textContent || el.tagName).trim().slice(0, 40)
    const controls = '[data-slot="button"], [data-slot="toggle-group-item"], [data-slot="badge"], [data-slot="select-trigger"], label'
    for (const el of document.querySelectorAll(controls)) {
      if (!vis(el)) continue
      const cs = getComputedStyle(el)
      // Meant to end in an ellipsis: its own, a `.truncate` inside, or a select's value (clamped).
      const cut = cs.textOverflow === 'ellipsis' || el.querySelector('.truncate, [data-slot="select-value"]')
      // The content's own width against the room inside the padding: centred text spills on
      // both sides, which scrollWidth counts only on one.
      const range = document.createRange()
      range.selectNodeContents(el)
      const used = range.getBoundingClientRect().width
      const room = el.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)
      if (!cut && used > room + 1) out.push(`UI-LAYOUT-008 text wider than its room (${Math.round(used)} > ${Math.round(room)}): ${el.dataset.slot ?? el.tagName.toLowerCase()} "${name(el)}"`)
      if (el.tagName !== 'LABEL' && el.scrollHeight > el.clientHeight + 1) out.push(`UI-LAYOUT-009 text taller than its box (${el.scrollHeight} > ${el.clientHeight}): "${name(el)}"`)
    }
    const rows = '.h-header, [data-slot="button-group"], [role="toolbar"], [data-slot="dialog-footer"]'
    for (const row of document.querySelectorAll(rows)) {
      if (!vis(row)) continue
      const right = row.getBoundingClientRect().right
      for (const kid of row.children) {
        if (vis(kid) && kid.getBoundingClientRect().right > right + 0.5) {
          out.push(`UI-LAYOUT-010 a row's child past its edge by ${Math.round(kid.getBoundingClientRect().right - right)} px: "${name(kid)}" in "${name(row)}"`)
        }
      }
    }
    for (const seg of document.querySelectorAll('[data-slot="toggle-group-item"]')) {
      if (!vis(seg)) continue
      const walker = document.createTreeWalker(seg, NodeFilter.SHOW_TEXT)
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent.trim()) continue
        const range = document.createRange()
        range.selectNodeContents(n)
        const t = range.getBoundingClientRect()
        const b = seg.getBoundingClientRect()
        const off = Math.abs(t.left + t.width / 2 - (b.left + b.width / 2))
        // A segment with an icon or a colour swatch before its words is not centred on them.
        const marked = seg.children.length > 0
        if (t.width > 0 && off > 0.5 && !marked) out.push(`UI-LAYOUT-013 segment text ${Math.round(off * 10) / 10} px off centre: "${name(seg)}"`)
        break
      }
    }
    return out
  })
  return [...new Set(found)].map((f) => `${where}: ${f}`)
}

/** Every machine-judged UI test case on the page as it is (docs/UI_TESTS.md). */
/** Words fitted smaller or cut in the views checked (FR-125), printed after the measures. */
const fitted = new Set()
/** The last fitting pass on a resize, "count:ms" (FR-125). */
let fitCost = 'none'

async function allChecks(page, where) {
  for (const row of await fitReport(page, where)) fitted.add(row)
  return [...(await overflows(page, where)), ...(await layoutChecks(page, where)), ...(await axeChecks(page, where))]
}

/** Every view the overflow checks walk, at one window width (step 5). */
/**
 * Opens every folded card on the page (the process review, 2026-11-12: a fault inside a folded
 * card met no check), so the checks after it see what each card holds.
 */
async function openFolds(page) {
  const folded = page.locator('section > header button[aria-expanded="false"]:visible')
  for (let i = 0; i < 24 && (await folded.count()); i++) {
    await folded.first().click()
    await page.waitForTimeout(80)
  }
  await settle(page)
}

/** The checks of a view, then the same view with every folded card open. */
async function allChecksOpen(page, where) {
  const out = await allChecks(page, where)
  await openFolds(page)
  return [...out, ...(await allChecks(page, `${where}, every card open`))]
}

async function overflowViews({ language, theme, width }) {
  const out = []
  let { context, page } = await open({ scenario: 'stamp', language, theme, width, queues: NO_QUEUES })
  await settle(page)
  out.push(...(await allChecksOpen(page, `gallery at ${width}`)))
  await page.getByRole('button', { name: new RegExp(`${say('culling.filters').source}|${say('culling.filtersOn').source}`) }).first().click()
  await page.waitForTimeout(400)
  out.push(...(await allChecks(page, `filters at ${width}`)))
  await page.keyboard.press('Escape')
  await page.getByRole('button', { name: say('export.buttonCount') }).first().click()
  await page.waitForSelector('[data-slot="settings-body"]')
  await settle(page)
  out.push(...(await allChecks(page, `export dialog at ${width}`)))
  await page.screenshot({ path: path.join(outDir, `measure-export-${width}.png`) })
  await page.keyboard.press('Escape')
  await page.waitForTimeout(300)
  for (const { re, tag } of ['general', 'export', 'updates'].map(section)) {
    await openSettings(page, re)
    out.push(...(await allChecks(page, `settings ${tag} at ${width}`)))
  }
  await context.close()
  for (const mode of ['simple', 'collage']) {
    ;({ context, page } = await open({ scenario: mode === 'collage' ? 'collage' : 'stamp', language, theme, width, mode, queues: NO_QUEUES }))
    await settle(page)
    out.push(...(await allChecksOpen(page, `${mode} at ${width}`)))
    await context.close()
  }
  for (const layout of ['dialog', 'window']) {
    ;({ context, page } = await open({ scenario: 'stamp', language, theme, width, ui: { previewLayout: layout } }))
    await (await reach(page, SHOW.crop)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
    await settle(page)
    out.push(...(await allChecksOpen(page, `preview (${layout}) at ${width}`)))
    if (width === 1180) await page.screenshot({ path: path.join(outDir, `measure-preview-${layout}-1180.png`) })
    await context.close()
  }
  const update = { current: '0.12.0', latest: '0.13.0', newer: true, pageUrl: '', minSafe: '0.13.0', belowMinSafe: true, reasonEn: 'This version has a fault that can lose a project’s edits.', reasonVi: 'Phiên bản này có lỗi có thể làm mất chỉnh sửa của dự án.' }
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, width, update }))
  await page.waitForSelector('[role="dialog"]')
  await settle(page)
  out.push(...(await allChecks(page, `update window at ${width}`)))
  await context.close()
  return out
}

/** STYLE_RULE §4: each column holds this one value, in every view that has it. */
const MEASURES = { cardGap: '12', blockGap: '12', fieldGap: '6', select: '32', toggle: '32', input: '32', iconBtn: '32', listGap: '4', modeGap: '12' }

/**
 * The stress fixtures and the dynamic cases of docs/UI_TESTS.md: states the demo recording
 * never shows (a long custom size chosen, every Settings section, the welcome), the sheet
 * preview in every shape, nothing jumping when W or J is pressed, and the pixel baselines.
 */
/** The passes that have approved pictures: English in light in Chromium, Vietnamese in dark in WebKit. */
const BASELINE_PASSES = ['chromium-en-light', 'webkit-vi-dark']
/** The approved pictures this run compared, to find the ones no pass compares any more. */
const compared = new Set()

/**
 * UI-VISUAL-002: every approved picture in the repo is one a pass still compares. A picture of
 * another pass, or of this pass's view that no longer exists, fails until it is removed.
 */
function orphanBaselines({ language, theme }) {
  const engine = process.env.MEASURE_ENGINE === 'webkit' ? 'webkit' : 'chromium'
  const pass = `${engine}-${language}-${theme}`
  const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), 'baselines')
  const tracked = execFileSync('git', ['ls-files', dir], { encoding: 'utf8' })
    .split('\n')
    .filter(Boolean)
    .map((f) => path.basename(f))
  return tracked
    .filter((f) => !BASELINE_PASSES.some((p) => f.startsWith(`${p}-`)) || (f.startsWith(`${pass}-`) && !compared.has(f)))
    .map((f) => `UI-VISUAL-002 an approved picture no pass compares: site/screenshots/baselines/${f} (remove it)`)
}

async function stressViews({ language, theme }) {
  const out = []
  const engine = process.env.MEASURE_ENGINE === 'webkit' ? 'webkit' : 'chromium'
  const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), 'baselines')
  // Approved pictures exist only for the matrix's two corners (UI-VISUAL-001, UI_TESTS §1);
  // any other pass (another language, theme or engine) is measured and checked, never written
  // as a baseline into the repo (LESSONS 2026-11-15).
  const pass = `${engine}-${language}-${theme}`
  const shot = (page, locator, name) => {
    if (!BASELINE_PASSES.includes(pass)) return []
    compared.add(`${pass}-${name}.png`)
    return baseline(page, locator, `${pass}-${name}`, { dir, outDir })
  }
  // UI-FIX-001: a long custom print size chosen, its delete button beside the size's name.
  const longSize = { customSizes: [{ id: 'custom-1', name: 'Demo very long custom print size', shortMm: 1000, longMm: 1000, unit: 'cm' }], printSize: 'custom-1' }
  let { context, page } = await open({ scenario: 'stamp', language, theme, settingsPatch: longSize })
  await settle(page)
  out.push(...(await allChecks(page, 'Pro panel, a long custom size')))
  out.push(...(await shot(page, page.locator('aside.border-l'), 'pro-panel')))
  out.push(...(await shot(page, page.locator('aside.border-r'), 'folder-list')))
  await context.close()
  ;({ context, page } = await open({ scenario: 'collage', language, theme, mode: 'collage', queues: NO_QUEUES, settingsPatch: longSize }))
  await settle(page)
  out.push(...(await allChecks(page, 'Collage panel, a long custom size')))
  out.push(...(await shot(page, page.locator('aside.border-l'), 'collage-panel')))
  await context.close()
  // UI-LAYOUT-024: the chosen size's verdict whole, as a small photo gets "acceptable"
  // (owner, 2026-11-09: "Tạm được" cut off in the button on Windows).
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, verdict: 'acceptable' }))
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await page.locator('[role="dialog"]').getByRole('radio', { name: say('inspector.print') }).click()
  await settle(page)
  out.push(...(await allChecks(page, 'photo preview, Print, every size acceptable')))
  await context.close()
  // UI-LAYOUT-021: the print size list open with the last size chosen, where an item-aligned
  // list ran over the window's top (owner, 2026-11-08).
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, settingsPatch: { printSize: '5x15' } }))
  await settle(page)
  await page.getByRole('combobox', { name: say('settings.printSize') }).first().click()
  await page.waitForSelector('[role="option"]')
  await page.waitForTimeout(400)
  // Layout only: an open list is modal (Radix hides the rest from readers and keeps the focus
  // in the list), which axe's aria-hidden-focus takes for a fault; the closed views run axe.
  const where = 'the print size list open, the last size chosen'
  out.push(...(await overflows(page, where)), ...(await layoutChecks(page, where)))
  await context.close()
  // The same list in the photo preview's Print tab, its button low in the column, under the
  // macOS title bar: it opens upwards and lost its first sizes under the bar (owner,
  // 2026-11-11), which the list above, opening downwards with no bar, never showed.
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, os: 'macos', height: 700, settingsPatch: { printSize: '5x15' } }))
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await page.locator('[role="dialog"]').getByRole('radio', { name: say('inspector.print') }).click()
  await settle(page)
  {
    const sizes = page.locator('[role="dialog"]').getByRole('combobox', { name: say('settings.printSize') }).first()
    await sizes.scrollIntoViewIfNeeded()
    await sizes.click()
    await page.waitForSelector('[role="option"]')
    await page.waitForTimeout(400)
    const where = 'the photo preview\'s print size list, opening upwards under the title bar'
    out.push(...(await overflows(page, where)), ...(await layoutChecks(page, where)))
  }
  await context.close()
  // UI-LAYOUT-023: the buttons both previews share sit in the same order (owner, 2026-11-08:
  // "the same buttons in the same places"): each toolbar's groups and its *Before*, by name.
  const barOrder = (page) =>
    page.evaluate(() =>
      [...document.querySelectorAll('[role="dialog"] [data-slot="button-group"], [role="dialog"] button[aria-label="Before"], [role="dialog"] button[aria-label="Trước"]')]
        .filter((el) => !el.closest('aside'))
        .map((el) => el.getAttribute('aria-label')),
    )
  let sheetBar = null
  // The sheet preview in every shape (LESSONS: a row above a stage): landscape, square, tall.
  for (const [size, label] of [['10x15', 'landscape'], ['10x10', 'square'], ['13x18', 'tall']]) {
    ;({ context, page } = await open({ scenario: 'collage', language, theme, mode: 'collage', queues: NO_QUEUES, settingsPatch: { printSize: size } }))
    const sheet = page.locator('[data-cell]').nth(2)
    await sheet.dblclick()
    await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]')
    await settle(page)
    out.push(...(await allChecks(page, `sheet preview, ${label}`)))
    sheetBar ??= await barOrder(page)
    const fits = await page.evaluate(() => {
      const img = document.querySelector('[role="dialog"] img[src*="/sheet/"]')
      const stage = img?.closest('.group\\/stage')
      if (!img || !stage) return null
      const a = img.getBoundingClientRect()
      const b = stage.getBoundingClientRect()
      return a.top >= b.top - 0.5 && a.bottom <= b.bottom + 0.5 && a.left >= b.left - 0.5 && a.right <= b.right + 0.5
    })
    if (fits === false) out.push(`sheet preview, ${label}: UI-LAYOUT-012 the sheet passes its stage`)
    await context.close()
  }
  // Nothing jumps (UI-STABLE-001): W and J in the photo preview leave its cards in place.
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await settle(page)
  out.push(...(await shot(page, page.locator('[role="dialog"] aside'), 'preview-inspector')))
  for (const key of ['w', 'j']) {
    out.push(...(await stableChecks(page, 'photo preview', '[role="dialog"] aside section', () => page.keyboard.press(key), key.toUpperCase())))
    await page.keyboard.press(key === 'w' ? 'Escape' : key)
  }
  await context.close()
  // Nor does the first step of a slider on a photo with the original look (owner, 2026-11-08:
  // the Looks card unfolded its amount and every card below dropped under the pointer). One
  // fresh photo per card: only the first step leaves the original.
  for (const [card, title] of [['Faves', say('look.faves', { whole: false, start: true })], ['Color mixer', say('look.mixer.title', { whole: false, start: true })]]) {
    ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
    await (await reach(page, SHOW.crop)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
    await settle(page)
    const thumb = page.locator('[role="dialog"] aside section', { hasText: title }).locator('[role="slider"]').first()
    await thumb.focus()
    out.push(...(await stableChecks(page, 'photo preview', '[role="dialog"] aside section', () => page.keyboard.press('ArrowRight'), `${card}: a first slider step`)))
    await context.close()
  }
  // UI-STATE-009: a button held down is `bg-pressed`, the accent sunk halfway, in both themes
  // (owner 2026-11-08: "a press shows nothing"; 2026-11-18: one pressed face for every control).
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await settle(page)
  {
    const held = page.getByRole('button', { name: say('viewer.rotateLeftShort') })
    const box = await held.boundingBox()
    if (box) {
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
      await page.mouse.down()
      // Past the press's fast pace (120 ms), so the colour has fully come.
      await page.waitForTimeout(250)
      const [bg, primary] = await held.evaluate((el) => {
        const probe = document.createElement('i')
        probe.className = 'bg-pressed'
        el.append(probe)
        const want = getComputedStyle(probe).backgroundColor
        probe.remove()
        return [getComputedStyle(el).backgroundColor, want]
      })
      if (bg !== primary) out.push(`photo preview: UI-STATE-009 a held button is ${bg}, not the pressed face ${primary}`)
      // UI-LAYOUT-023: the sheet's toolbar is the photo's, less what a sheet has not.
      const photoBar = await barOrder(page)
      const common = (sheetBar ?? []).filter((n) => photoBar.includes(n))
      const inPhoto = photoBar.filter((n) => common.includes(n))
      if (!sheetBar || common.length !== sheetBar.length || common.join('|') !== inPhoto.join('|'))
        out.push(`sheet preview: UI-LAYOUT-023 its toolbar [${(sheetBar ?? []).join(', ')}] is not the photo preview's order [${photoBar.join(', ')}]`)
      await page.mouse.up()
    } else out.push('photo preview: UI-STATE-009 no Rotate left button to press')
  }
  await context.close()
  // UI-LAYOUT-022: a dialog with a long header (Copy settings, two lines in Vietnamese) keeps
  // its words clear of the close button.
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await settle(page)
  await page.getByRole('button', { name: say('develop.copy') }).first().click()
  await page.waitForSelector('[data-slot="dialog-form"]')
  await page.waitForTimeout(400)
  out.push(...(await allChecks(page, 'Copy settings dialog')))
  await context.close()
  // UI-STATE-010: a part that fails fails alone (ADR-0048): every card broken, then the main
  // view; each shows "This part hit an error" in its place, the rest goes on, never a blank.
  for (const [fault, rest, restName] of [['card', 'main img[src*="/thumb/"]', 'the gallery'], ['main', 'aside.border-l', 'the side panel']]) {
    ;({ context, page } = await open({ scenario: 'stamp', language, theme, fault }))
    await page.waitForTimeout(1500)
    const seen = await page.evaluate((restSel) => ({
      failed: document.querySelectorAll('[data-slot="part-failed"]').length,
      blank: document.querySelector('[data-slot="app-failed"]') !== null,
      rest: document.querySelector(restSel) !== null,
    }), rest)
    if (seen.blank || seen.failed === 0 || !seen.rest)
      out.push(`UI-STATE-010 a broken ${fault} did not fail alone: ${seen.failed} part(s) failed, ${seen.blank ? 'the whole window failed' : 'the window stayed'}, ${restName} ${seen.rest ? 'stayed' : 'went'}`)
    await context.close()
  }
  // Every Settings section, the manual among them, and the welcome.
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  const sections = ['general', 'export', 'memory', 'data', 'updates', 'manual', 'report', 'whatsNew'].map(section)
  for (const { re, tag } of sections) {
    await openSettings(page, re)
    await page.waitForTimeout(300)
    out.push(...(await allChecks(page, `settings ${tag}`)))
    out.push(...(await shot(page, page.locator('[data-slot="settings-body"]'), `settings-${tag}`)))
  }
  await context.close()
  ;({ context, page } = await open({ scenario: 'stamp', language, theme, ui: { welcomeSeen: false } }))
  await page.waitForSelector('[role="dialog"]')
  await settle(page)
  // The first page by eye: its photo behind the greeting and the flags (FR-124, 2026-11-15).
  await page.screenshot({ path: path.join(outDir, 'measure-welcome.png') })
  out.push(...(await allChecks(page, 'welcome')))
  await context.close()
  return out
}

async function measurePanels() {
  const rows = []
  const extra = []
  const theme = process.env.MEASURE_THEME ?? 'light'
    const language = process.env.MEASURE_LANG ?? 'en'
  let { context, page } = await open({ scenario: 'stamp', language, theme })
  // FR-125: what fitting the words costs when the window is resized (one pass of the shared
  // observer over the controls that changed width), for OPTIMIZATION §5.
  await page.setViewportSize({ width: 1180, height: 900 })
  await page.waitForTimeout(300)
  fitCost = await page.evaluate(() => document.documentElement.dataset.fitPass ?? 'none')
  await page.setViewportSize({ width: 1440, height: 900 })
  await settle(page)
  rows.push(['folder list', await measure(page, 'aside.border-r')])
  rows.push(['Pro panel', await measure(page, 'aside.border-l')])
  const panelBands = await foldedBands(page, 'Pro panel')
  // The gallery's Colour tab (FR-114): a look picked, its tile, amount and button shown.
  const galleryTabs = page.locator('aside.border-l [data-slot="inspector-tabs"] button')
  await galleryTabs.nth(0).click()
  await page.locator('aside.border-l div[data-hint] > button').first().click()
  await settle(page)
  rows.push(['Pro panel: Colour', await measure(page, 'aside.border-l')])
  extra.push(...(await allChecks(page, 'Pro panel: Colour')))
  await page.locator('aside.border-l').screenshot({ path: path.join(outDir, 'measure-gallery-colour.png') })
  await galleryTabs.nth(1).click()
  await settle(page)
  await (await reach(page, SHOW.crop)).dblclick()
  await page.waitForSelector(`img[src*="/preview/${SHOW.crop}"]`)
  await settle(page)
  rows.push(['photo preview', await measure(page, '[role="dialog"] aside')])
  const bands = await foldedBands(page, 'photo preview')
  // The inspector's tabs: Edit, Crop | Print, Info; the Print and Info tabs hold folded cards.
  for (const [index, tab] of [[2, 'Print'], [3, 'Info']]) {
    await page.locator('[data-slot="inspector-tabs"] button').nth(index).click()
    await settle(page)
    bands.push(...(await foldedBands(page, `photo preview, ${tab}`)))
  }
  // Every folded card of the Print and Info tabs opened, so what it holds is checked too (owner,
  // 2026-11-12: the date card's clock button ran out of its card in Hungarian, folded away from
  // every check). The tabs are found by their words, the cards by their headers.
  for (const [key, tab] of [['inspector.print', 'Print'], ['inspector.info', 'Info']]) {
    await page.locator('[role="dialog"]').getByRole('radio', { name: say(key) }).click()
    await settle(page)
    const folded = page.locator('[role="dialog"] aside section > header button[aria-expanded="false"]')
    for (let i = 0; i < 12 && (await folded.count()); i++) {
      await folded.first().click()
      await page.waitForTimeout(100)
    }
    await settle(page)
    extra.push(...(await allChecks(page, `photo preview, ${tab}, every card open`)))
    await page.screenshot({ path: path.join(outDir, `measure-preview-${tab.toLowerCase()}-open.png`) })
  }
  await page.screenshot({ path: path.join(outDir, 'measure-preview.png') })
  await context.close()
  await measureSidePanels({ open, settle, measure, allChecks, say, outDir }, { language, theme }, rows, extra)
  ;({ context, page } = await open({ scenario: 'collage', language, theme, mode: 'collage', queues: NO_QUEUES }))
  let sheet = page.locator('[data-cell]').nth(2)
  await sheet.click(); await settle(page)
  rows.push(['Collage panel', await measure(page, 'aside.border-l')])
  extra.push(...(await allChecks(page, 'Collage panel, a sheet chosen')))
  await page.screenshot({ path: path.join(outDir, 'measure-collage.png') })
  await sheet.dblclick(); await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]'); await settle(page)
  rows.push(['sheet preview', await measure(page, '[role="dialog"] aside')])
  extra.push(...(await allChecks(page, 'sheet preview')))
  await page.screenshot({ path: path.join(outDir, 'measure-sheet.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'postcard', language, theme, mode: 'collage', queues: NO_QUEUES }))
  sheet = await cell(page, SHOW.frame)
  await sheet.click(); await settle(page)
  rows.push(['Postcard panel', await measure(page, 'aside.border-l')])
  extra.push(...(await allChecks(page, 'Postcard panel')))
  await page.screenshot({ path: path.join(outDir, 'measure-postcard.png') })
  await sheet.dblclick(); await page.waitForSelector('[role="dialog"] img[src*="/sheet/"]'); await settle(page)
  rows.push(['postcard preview', await measure(page, '[role="dialog"] aside')])
  extra.push(...(await allChecks(page, 'postcard preview')))
  await page.screenshot({ path: path.join(outDir, 'measure-postcard-preview.png') })
  await context.close()
  ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
  for (const { re, tag } of ['general', 'memory', 'data'].map(section)) {
    await openSettings(page, re)
    rows.push([`settings: ${tag.toLowerCase()}`, await measure(page, '[data-slot="settings-body"]')])
  }
  await page.screenshot({ path: path.join(outDir, 'measure-settings.png') })
  await context.close()
  // Icon-only buttons at the narrowest window and a wide one, in the gallery and Collage mode.
  const offCentre = []
  for (const width of [1180, 1600]) {
    for (const mode of ['professional', 'collage']) {
      ;({ context, page } = await open({ scenario: mode === 'collage' ? 'collage' : 'stamp', language, theme, mode, width, queues: NO_QUEUES }))
      await settle(page)
      for (const b of await iconOffsets(page)) if (b.off > 0.5) offCentre.push(`${mode} at ${width} px: UI-LAYOUT-007 "${b.label}" icon ${b.off} px off centre`)
      await context.close()
    }
  }
  const keys = Object.keys(rows[0][1])
  console.log(['view', ...keys].join(' | '))
  for (const [name, m] of rows) console.log([name, ...keys.map((k) => m[k] || '–')].join(' | '))
  const off = rows.flatMap(([name, m]) =>
    Object.entries(MEASURES).filter(([k, v]) => m[k] && m[k] !== v).map(([k, v]) => `${name}: ${/Gap$/.test(k) ? 'UI-LAYOUT-003' : 'UI-LAYOUT-004'} ${k} is ${m[k]}, not ${v}`),
  )
  const widths = new Set(rows.filter(([name]) => /panel|folder/.test(name)).map(([, m]) => m.width))
  if (widths.size > 1) off.push(`UI-LAYOUT-005 the folder list and the panels differ in width: ${[...widths].join(', ')}`)
  off.push(...offCentre, ...panelBands, ...bands)
  console.log(panelBands.length + bands.length ? '' : 'folded cards: every folded card is its header alone')
  console.log(offCentre.length ? '' : 'icons: every icon-only button has its icon centred (≤ 0.5 px)')
  // Text and rows that overflow, at the narrowest window, a middle one and a wide one (step 5).
  const spilled = []
  for (const width of [1180, 1400, 1600]) spilled.push(...(await overflowViews({ language, theme, width })))
  off.push(...spilled)
  console.log(spilled.length ? '' : 'overflow: no text, row or segment overflows at 1180, 1400 or 1600 px')
  // The states the recording lacks, nothing jumping, and the approved pictures (docs/UI_TESTS.md).
  extra.push(...(await stressViews({ language, theme })))
  extra.push(...orphanBaselines({ language, theme }))
  off.push(...extra)
  console.log(extra.length ? '' : 'ui tests: every view and state passes layout, text, target, axe, stability and baselines')
  if (off.length) {
    console.error(off.join('\n'))
    process.exitCode = 1
  }
}

if (process.env.MEASURE) {
  await measurePanels()
  // FR-125: what a translation made smaller or cut, for the translator (never a failure).
  if (fitted.size) console.log(`\nfitted words (view | step | width | words):\n${[...fitted].join('\n')}`)
  else console.log('\nfitted words: none, every word whole')
  console.log(`fitting on a resize to 1180 px: ${fitCost.replace(':', ' controls, ')} ms`)
  await browser.close()
  process.exit()
}

for (const language of LANGUAGES) {
  for (const theme of ['dark', 'light']) {
    const suffix = `${language}-${theme}`

    // 1. The library grid, culled.
    let { context, page } = await open({ scenario: 'stamp', language, theme })
    await (await reach(page, SHOW.grid)).click()
    await settle(page)
    await save(page, `grid-${suffix}`)

    // 1a. For the manual: the filters open (FR-022), and the Project block (FR-023),
    // zoomed in like the print-basket shots.
    await page.getByRole('button', { name: say('culling.filters') }).click()
    await page.waitForSelector('[role="dialog"], [data-slot="popover-content"]')
    await page.waitForTimeout(300)
    await save(page, `manual-filters-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(200)
    await zoomed(page, `manual-project-${suffix}`, { x: 0, y: 0, width: 576, height: 360 })

    // 1b. Print-size advice for the manual: the picker open, a badge per size (FR-026).
    // By its name: the panel's ids come from useId (STYLE_RULE §4, no hard-coded id).
    await page.getByRole('combobox', { name: say('settings.printSize') }).first().click()
    await page.waitForSelector('[role="option"]')
    await page.waitForTimeout(300)
    await save(page, `manual-size-${suffix}`)
    await page.keyboard.press('Escape')
    await page.waitForTimeout(200)
    await page.getByRole('button', { name: say('about.versions', { whole: false }) }).click()
    await page.waitForTimeout(300)
    await save(page, `check-versions-${suffix}`)
    await page.keyboard.press('Escape')

    // 2. Preview with the date stamp (the default style) and the place (GPS photo).
    await (await reach(page, SHOW.stamp)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.stamp}"]`)
    await settle(page)
    // 2a. For the manual: the photo at a glance, at the top of the inspector (FR-078).
    const summary = await page
      .locator('[role="dialog"] section', { hasText: say('viewer.summary', { whole: false, start: true }) })
      .first()
      .boundingBox()
    if (summary) {
      await zoomed(page, `manual-summary-${suffix}`, { x: summary.x - 16, y: summary.y - 16, width: summary.width + 32, height: summary.height + 32 })
    }
    // The stamp's card is in the inspector's Print tab (FR-095's tabs): the shot shows it.
    await page.locator('[role="dialog"]').getByRole('radio', { name: say('inspector.print') }).click()
    await settle(page)
    await save(page, `stamp-${suffix}`)
    // 2b. The place in the Location card (for checking only), and the place picker for the
    // manual (FR-032).
    const inspector = page.locator('[role="dialog"]')
    // The place lives in the inspector's Info tab (FR-095's tabs).
    await inspector.getByRole('radio', { name: say('inspector.info') }).click()
    await page.waitForTimeout(300)
    await inspector.getByRole('button', { name: say('places.setPlace', { whole: false }) }).scrollIntoViewIfNeeded()
    await page.waitForTimeout(300)
    await save(page, `check-place-${suffix}`)
    await inspector.getByRole('button', { name: say('places.setPlace', { whole: false }) }).click()
    await page.getByRole('combobox').last().click()
    await page.keyboard.type('pra')
    await page.waitForSelector('[cmdk-item]')
    await page.waitForTimeout(300)
    await save(page, `manual-place-picker-${suffix}`)
    await context.close()

    // 3. Preview with the "Shot on" info frame, and the panel's frame card for the manual.
    ;({ context, page } = await open({ scenario: 'frame', language, theme }))
    // Scroll the panel alone (scrollIntoView would move the page too, and show the splash).
    await page.locator('aside').last().getByText(say('frame.title')).first().evaluate((e) => {
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
    await page.locator('[role="dialog"]').getByRole('radio', { name: say('inspector.print') }).click()
    await settle(page)
    await save(page, `frame-${suffix}`)
    // The export card at the bottom of the inspector: this photo only (FR-055).
    await page.locator('[role="dialog"]').getByRole('button', { name: say('export.button') }).click()
    await page.waitForTimeout(600)
    await save(page, `check-export-${suffix}`)
    // Export opens the export dialog over the preview (FR-093): Esc closes it, then the preview.
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)
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

    // 4b. For the showcase (showcase.mjs): before and after side by side (FR-096), the
    // full-window develop view (FR-095), and the export dialog (FR-093).
    ;({ context, page } = await open({ scenario: 'look', language, theme }))
    await (await reach(page, SHOW.compare)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.compare}"]`)
    await page.keyboard.press('y')
    await page.waitForTimeout(400)
    await settle(page)
    await save(page, `compare-${suffix}`)
    await context.close()
    ;({ context, page } = await open({ scenario: 'look', language, theme, ui: { previewLayout: 'window' } }))
    await (await reach(page, SHOW.compare)).dblclick()
    await page.waitForSelector(`img[src*="/preview/${SHOW.compare}"]`)
    await settle(page)
    await save(page, `window-${suffix}`)
    await context.close()
    ;({ context, page } = await open({ scenario: 'stamp', language, theme }))
    await page.getByRole('button', { name: say('export.buttonCount') }).first().click()
    await page.waitForSelector('[data-slot="settings-body"]')
    await settle(page)
    await save(page, `export-dialog-${suffix}`)
    await context.close()

    // 5. Best-shot groups, on the cat burst.
    ;({ context, page } = await open({ scenario: 'stamp', language, theme, source: cat }))
    await page.getByRole('radio', { name: say('bestShot.viewGroups', { whole: false }) }).click()
    await settle(page)
    await save(page, `manual-groups-${suffix}`)
    // The compare view shows the shots large, side by side, with a bar per measure.
    await page.getByRole('button', { name: say('bestShot.compare') }).first().click()
    await page.waitForSelector('[role="dialog"]')
    await settle(page)
    await save(page, `groups-${suffix}`)
    await context.close()

    // 6. The print basket (FR-050, FR-059), shown zoomed in at three times the resolution.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 3 }))
    const tiles = page.locator('img[src*="/thumb/"]')
    // 6a. Select mode: ticks, the action bar, the basket and set-aside rows.
    await page.getByRole('button', { name: say('select.start') }).click()
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
    await page.getByRole('radio', { name: say('export.target.share') }).first().click()
    await page.waitForTimeout(900)
    // The estimate is a badge of figures beside the button (FR-053, FR-099).
    const note = await page.locator('[data-slot="badge"]', { hasText: /→/ }).last().boundingBox()
    const button = await page.getByRole('button', { name: say('export.buttonCount') }).boundingBox()
    if (note && button) {
      const top = note.y - 56
      // A multiple of 5 px, so 16:10 is exact and the picture comes out 1600 × 1000.
      const height = Math.ceil((button.y + button.height + 40 - top) / 5) * 5
      const width = height * 1.6
      const x = Math.min(1440 - width, note.x + note.width / 2 - width / 2)
      await zoomed(page, `estimate-${suffix}`, { x, y: top, width, height })
    }
    // 6d. For the manual: each shared photo whole in a square frame, a fill round it (FR-075).
    await page.getByRole('switch', { name: say('export.fill.label') }).click()
    await page.waitForTimeout(500)
    const fillCard = await page.locator('section', { has: page.getByRole('switch', { name: say('export.fill.label') }) }).first().boundingBox()
    if (fillCard) {
      const c = fillCard
      await zoomed(page, `manual-share-fill-${suffix}`, { x: c.x - 16, y: c.y - 16, width: c.width + 32, height: c.height + 32 })
      // For checking by eye: a colour of one's own, with its picker and code.
      await page.getByRole('radio', { name: say('export.fill.colors.custom') }).click()
      await page.waitForTimeout(400)
      const open = await page.locator('section', { has: page.getByRole('switch', { name: say('export.fill.label') }) }).first().boundingBox()
      if (open) await zoomed(page, `check-share-fill-${suffix}`, { x: open.x - 16, y: open.y - 16, width: open.width + 32, height: open.height + 32 })
    }
    await context.close()
    // 7. Folders as tabs (FR-056): tick photos in one folder, switch tabs, tick more; the bar
    // counts them all, and hovering a tab shows its close button.
    ;({ context, page } = await open({ scenario: 'frame', language, theme, scale: 3, tabs: true }))
    const tab = (name) => page.getByRole('button', { name: new RegExp(`^${name}`) })
    await tab(PROFILE.tabs[0]).click()
    await page.getByRole('button', { name: say('select.start') }).click()
    // The second and fifth photos, or the last where a folder has fewer.
    const thumbs = page.locator('img[src*="/thumb/"]')
    const tick = async (i) => thumbs.nth(Math.min(i, (await thumbs.count()) - 1)).click()
    await tick(1)
    await tick(4)
    await tab(PROFILE.tabs[1]).click()
    await page.waitForTimeout(300)
    await tick(2)
    await tab(PROFILE.tabs[2]).hover()
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
    await openSettings(page, section('memory').re)
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
    // The card carrying the first typed text (demo.rs): the owner's red tram, Gokurakuji, Gwanghwamun.
    const card = await cell(page, SHOW.card)
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
      // The card's text has its own tab in the preview (Mail icon, 2026-11-15).
      const postcardTab = page.locator('[role="dialog"] [data-slot="inspector-tabs"]').getByLabel(say('postcard.title'))
      if (await postcardTab.count()) {
        await postcardTab.first().click()
        await settle(page)
      }
      // 11c. For the manual (FR-077): the typed address, the note with "Use for them" under it;
      // then the recent addresses, listed on every card (v0.13.2).
      const textCard = (pattern) =>
        page.locator('[role="dialog"] section', { has: page.getByText(pattern) }).first()
      // AddAssist (FR-133) on a Japanese or Korean card, for those pages' carousel: the whole
      // window, the card beside its filled address, at the carousel's 16:10 like every slide
      // (a zoomed crop of the card was 1600 × 3909 in a 1600 × 1000 frame, LESSONS 2026-11-15).
      const assist = textCard(say('addassist.recipient'))
      if (await assist.count()) {
        await assist.scrollIntoViewIfNeeded()
        await settle(page)
        await save(page, `addassist-${suffix}`)
      }
      const apply = textCard(say('postcard.reuse.fill'))
      if (await apply.count()) {
        await apply.scrollIntoViewIfNeeded()
        await settle(page)
        const box = await apply.boundingBox()
        if (box) await zoomed(page, `manual-postcard-apply-${suffix}`, { x: box.x - 16, y: box.y - 16, width: box.width + 32, height: box.height + 32 })
      }
      for (let i = 0; i < 8 && !(await page.getByText(say('postcard.reuse.recent')).count()); i++) {
        await page.keyboard.press('ArrowRight')
        await settle(page)
      }
      const recent = textCard(say('postcard.reuse.recent'))
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
if (Object.keys(phoneFacts).length) writeFileSync(path.join(outDir, 'phone.json'), `${JSON.stringify(phoneFacts, null, 2)}\n`)
console.log(shots.map((s) => path.relative(root, s)).join('\n'))
