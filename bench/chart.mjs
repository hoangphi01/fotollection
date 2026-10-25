// SPDX-License-Identifier: GPL-3.0-only
// Turns bench/results/*.json into bench/results/summary.json and the README/RESULT charts
// (plain SVG, no dependencies), in English and Vietnamese. Each metric is the median of runs.
//
//   node bench/chart.mjs
//
// Charts (docs/images/, `-vi` suffix for Vietnamese):
//   bench-budgets.svg  measured vs the NFR budgets (reference: budget line)
//   bench-memory.svg   peak memory vs batch size (reference: 2 GB ceiling)
//   bench.svg          Fotollection vs Chromium (reference: the browser)
//   bench-preview.svg  where the warm preview time goes (reference: budget and Chromium)
//   bench-share.svg    share export after Facebook's recompression, vs the usual way (FR-053)

import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const root = new URL('..', import.meta.url).pathname
const results = join(root, 'bench/results')
const images = join(root, 'docs/images')
const read = (f) => JSON.parse(readFileSync(join(results, f), 'utf8'))

function runs(prefix) {
  return readdirSync(results)
    .filter((f) => new RegExp(`^${prefix}-\\d+\\.json$`).test(f))
    .map(read)
}
function median(list, key) {
  const v = list
    .map((r) => r[key])
    .filter((x) => typeof x === 'number')
    .sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : undefined
}
function summarize(list) {
  if (list.length === 0) return undefined
  const keys = Object.keys(list[0]).filter((k) => typeof list[0][k] === 'number')
  return {
    engine: list[0].engine,
    runs: list.length,
    ...Object.fromEntries(keys.map((k) => [k, median(list, k)])),
  }
}

const rust = summarize(runs('rust'))
const web = summarize(runs('web'))
const inflight8 = summarize(runs('inflight8'))
// Largest batch first; batch-<n>.json comes from bench/batch.sh.
const batches = readdirSync(results)
  .filter((f) => /^batch-\d+\.json$/.test(f))
  .map(read)
  .sort((a, b) => b.photos - a.photos)
const batch = batches[0]
const budgets = read('budgets.json')
const bestshot = read('bestshot.json')
const quality = read('quality.json')
const share = read('share.json')
const meta = read('machine.json')
if (!rust || !web) throw new Error('run bench/run.sh first')

const openSource = batch ?? rust
const measured = {
  // Import + EXIF + embedded thumbnails, scaled to the NFR-001 unit (1,000 photos).
  openPer1000S: (openSource.importMs + openSource.t0Ms) / openSource.photos,
  openPhotos: openSource.photos,
  previewP95Ms: rust.previewWarmP95Ms,
  exportPerSecond: rust.exportPerSecond,
  peakGb: Math.max(rust.peakRssBytes, batch?.peakRssBytes ?? 0) / 1e9,
}
writeFileSync(
  join(results, 'summary.json'),
  `${JSON.stringify({ ...meta, measured, rust, web, inflight8, batch }, null, 2)}\n`,
)

// ---------------------------------------------------------------------------------------
// Words. Numbers use the locale's decimal separator (18.6 / 18,6).

const T = {
  en: {
    lower: 'lower is better',
    higher: 'higher is better',
    budget: 'budget',
    ceiling: 'ceiling',
    photos: 'photos',
    rust: 'Fotollection (Rust)',
    web: 'Browser (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'photos/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `${meta.megapixels} MP JPEGs · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} photos: median of ${rust.runs} runs${withBatch && batch ? ` · ${f(batch.photos, 0)} photos: 1 run` : ''}`,
    budgetsTitle: 'Measured against the performance budgets',
    open: 'Open a folder (import + EXIF + first thumbnails)',
    openUnit: (n) => `s per 1,000 photos (measured on ${n})`,
    preview: 'Preview update after a settings change',
    previewUnit: 'ms, p95',
    export: 'Export throughput',
    exportUnit: (mp) => `photos/s at ${mp} MP (budget is for 24 MP)`,
    memory: 'Peak memory, largest batch',
    memoryUnit: 'GB',
    memTitle: 'Peak memory stays flat as the batch grows',
    memUnit: 'GB peak RSS: import + thumbnails + export',
    compareTitle: (n) => `Same ${n} photos, same job: decode → crop 2:3 → date stamp → JPEG q95`,
    cExport: 'Batch export',
    cExportUnit: 'photos/s',
    cPreview: 'Warm preview (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Peak memory',
    cMemoryUnit: 'GB',
    previewTitle: 'Where the 1600 px preview time goes',
    resize: 'crop + resize',
    stamp: 'date stamp',
    encode: 'JPEG encode',
    ms: 'ms, p50',
    qTitle: 'Print quality: what an export keeps of the photo',
    qWeb: 'Typical web editor',
    qDetail: 'Detail kept (camera JPEGs)',
    qChroma: 'Colour saturation kept (Display P3, like iPhone photos)',
    qDe: 'Colour error, ΔE2000 mean (camera JPEGs)',
    qPercent: '% of the original · higher is better',
    qDeUnit: 'lower is better',
    qVisible: 'visible at 1',
    qFooter: (n, m) =>
      `${n} camera JPEGs and ${m} Display P3 versions, every pixel compared with the original · web editor: profile dropped, 4:2:0, q80, 2000 px`,
    shTitle: 'Photos for Facebook: what viewers see after Facebook recompresses them',
    shUsual: 'Usual way',
    shChroma: 'Colour saturation kept (Display P3, like iPhone photos)',
    shDiff: 'Difference from the original, 1 − SSIM (camera JPEGs)',
    shSize: 'File size per photo (camera JPEGs)',
    shSource: 'Camera file',
    shOriginal: 'Original size',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · lower is lighter',
    shFooter: (n, m) =>
      `${n} camera JPEGs and ${m} Display P3 versions · usual way: 2000 px, q80, profile dropped · Facebook: q71, 4:2:0, ≤ 2048 px`,
    bsTitle: 'Best-shot suggestions, step by step',
    bsGroups: (n) => `Groups found, of ${n} labelled by hand`,
    bsPicks: 'Suggested photo acceptable, in labelled groups with a clear best',
    bsFooter: (n, g) => [
      `${n} real photos (people, food, streets, night, sunsets) · ${g} groups labelled by hand`,
      `No group was ever wrong: pair precision 100 % at every step`,
    ],
  },
  vi: {
    lower: 'càng thấp càng tốt',
    higher: 'càng cao càng tốt',
    budget: 'ngân sách',
    ceiling: 'trần',
    photos: 'ảnh',
    rust: 'Fotollection (Rust)',
    web: 'Trình duyệt (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'ảnh/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `JPEG ${meta.megapixels} MP · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} ảnh: trung vị của ${rust.runs} lần chạy${withBatch && batch ? ` · ${f(batch.photos, 0)} ảnh: 1 lần` : ''}`,
    budgetsTitle: 'Số đo so với ngân sách hiệu năng',
    open: 'Mở thư mục (import + EXIF + thumbnail đầu tiên)',
    openUnit: (n) => `giây cho 1.000 ảnh (đo trên ${n} ảnh)`,
    preview: 'Cập nhật preview khi đổi thiết lập',
    previewUnit: 'ms, p95',
    export: 'Tốc độ export',
    exportUnit: (mp) => `ảnh/s với ảnh ${mp} MP (ngân sách tính cho 24 MP)`,
    memory: 'RAM đỉnh, batch lớn nhất',
    memoryUnit: 'GB',
    memTitle: 'RAM đỉnh giữ nguyên khi batch tăng',
    memUnit: 'GB RSS đỉnh: import + thumbnail + export',
    compareTitle: (n) => `Cùng ${n} ảnh, cùng việc: giải mã → crop 2:3 → dấu ngày → JPEG q95`,
    cExport: 'Export cả lô',
    cExportUnit: 'ảnh/s',
    cPreview: 'Preview đã cache (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'RAM đỉnh',
    cMemoryUnit: 'GB',
    previewTitle: 'Thời gian preview 1600 px đi đâu',
    resize: 'crop + resize',
    stamp: 'dấu ngày',
    encode: 'mã hóa JPEG',
    ms: 'ms, p50',
    qTitle: 'Chất lượng in: bản xuất giữ lại được bao nhiêu',
    qWeb: 'Web editor thông thường',
    qDetail: 'Chi tiết giữ lại (ảnh JPEG máy ảnh)',
    qChroma: 'Độ bão hoà màu giữ lại (Display P3, như ảnh iPhone)',
    qDe: 'Sai lệch màu, ΔE2000 trung bình (ảnh JPEG máy ảnh)',
    qPercent: '% so với ảnh gốc · càng cao càng tốt',
    qDeUnit: 'càng thấp càng tốt',
    qVisible: 'thấy được từ 1',
    qFooter: (n, m) =>
      `${n} ảnh JPEG máy ảnh và ${m} bản Display P3, so từng điểm ảnh với ảnh gốc · web editor: bỏ profile, 4:2:0, q80, 2000 px`,
    shTitle: 'Ảnh đăng Facebook: người xem thấy gì sau khi Facebook nén lại',
    shUsual: 'Cách thông thường',
    shChroma: 'Độ bão hoà màu giữ lại (Display P3, như ảnh iPhone)',
    shDiff: 'Khác biệt so với ảnh gốc, 1 − SSIM (ảnh JPEG máy ảnh)',
    shSize: 'Dung lượng mỗi ảnh (ảnh JPEG máy ảnh)',
    shSource: 'File máy ảnh',
    shOriginal: 'Giữ cỡ gốc',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · càng thấp càng nhẹ',
    shFooter: (n, m) =>
      `${n} ảnh JPEG máy ảnh và ${m} bản Display P3 · cách thông thường: 2000 px, q80, bỏ profile · Facebook: q71, 4:2:0, ≤ 2048 px`,
    bsTitle: 'Gợi ý ảnh đẹp nhất, từng bước cải tiến',
    bsGroups: (n) => `Số nhóm tìm được, trên ${n} nhóm gán nhãn tay`,
    bsPicks: 'Ảnh được gợi ý chấp nhận được, ở các nhóm có ảnh tốt nhất rõ ràng',
    bsFooter: (n, g) => [
      `${n} ảnh thật (người, đồ ăn, phố, cảnh đêm, hoàng hôn) · ${g} nhóm gán nhãn tay`,
      `Không lần nào gom sai: độ chính xác theo cặp 100 % ở mọi bước`,
    ],
  },
}

// ---------------------------------------------------------------------------------------
// SVG building blocks. Validated palette (dataviz reference): slot 1 blue = Fotollection,
// slot 2 orange = browser, slots 3–5 = preview segments. Light and dark steps are chosen.

const W = 720
const LEFT = 190
const RIGHT = 110
const PLOT = W - LEFT - RIGHT
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;')
// Two themes: the docs' neutral one (README, RESULT), and the website's, taken from the logo
// (site/assets/css/site.css). Both pairs pass the dataviz palette validator on their surface.
const THEMES = {
  docs: {
    light: { bg: '#fcfcfb', t1: '#1f1f1e', t2: '#52514c', t3: '#6f6e68', axis: '#c9c8c1', s1: '#2a78d6', s2: '#eb6834', s3: '#1baf7a', s4: '#eda100', s5: '#e87ba4' },
    dark: { bg: '#1a1a19', t1: '#f0efec', t2: '#c3c2b7', t3: '#9d9c93', axis: '#45443f', s1: '#3987e5', s2: '#d95926', s3: '#199e70', s4: '#c98500', s5: '#d55181' },
  },
  site: {
    light: { bg: '#fffaee', t1: '#2b2219', t2: '#514435', t3: '#7a6a55', axis: '#e2d4b3', s1: '#008a74', s2: '#b5652a', s3: '#1baf7a', s4: '#eda100', s5: '#e87ba4' },
    dark: { bg: '#262019', t1: '#f3ead6', t2: '#d8cab0', t3: '#a8977c', axis: '#3b3127', s1: '#21a593', s2: '#cc7f3e', s3: '#199e70', s4: '#c98500', s5: '#d55181' },
  },
}
const styleFor = ({ light: l, dark: d }) => `<style>
  .bg{fill:${l.bg}} .t1{fill:${l.t1};font:600 13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t2{fill:${l.t2};font:13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t3{fill:${l.t3};font:12px system-ui,-apple-system,Segoe UI,sans-serif}
  .h{font-size:15px} .num{font-variant-numeric:tabular-nums} .axis{stroke:${l.axis};stroke-width:1}
  .ref{stroke:${l.t2};stroke-width:1.5;stroke-dasharray:4 3} .refw{stroke:${l.s2};stroke-width:1.5;stroke-dasharray:4 3}
  .s1{fill:${l.s1}} .old{fill-opacity:.45} .s2{fill:${l.s2}} .s3{fill:${l.s3}} .s4{fill:${l.s4}} .s5{fill:${l.s5}} .gap{stroke:${l.bg};stroke-width:2}
  .in{fill:#fff}
  @media (prefers-color-scheme: dark){
    .bg{fill:${d.bg}} .t1{fill:${d.t1}} .t2{fill:${d.t2}} .t3{fill:${d.t3}} .axis{stroke:${d.axis}}
    .ref{stroke:${d.t2}} .refw{stroke:${d.s2}}
    .s1{fill:${d.s1}} .s2{fill:${d.s2}} .s3{fill:${d.s3}} .s4{fill:${d.s4}} .s5{fill:${d.s5}} .gap{stroke:${d.bg}}
  }
</style>`
let STYLE = styleFor(THEMES.docs)

function doc(title, desc, height, body) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-16 -16 ${W + 32} ${height + 32}" width="${W + 32}" height="${height + 32}" role="img" aria-labelledby="t d">
<title id="t">${esc(title)}</title>
<desc id="d">${esc(desc)}</desc>
${STYLE}
<rect class="bg" x="-16" y="-16" width="${W + 32}" height="${height + 32}" rx="8"/>
<text class="t1 h" x="0" y="8">${esc(title)}</text>
${body}
</svg>
`
}

/** A bar with a 4 px rounded data end, square at the baseline. */
function bar(cls, x, y, w, h, tip) {
  const r = Math.min(4, w / 2)
  return `<path class="${cls}" d="M${x} ${y}h${w - r}a${r} ${r} 0 0 1 ${r} ${r}v${h - 2 * r}a${r} ${r} 0 0 1 ${-r} ${r}h${r - w}z"><title>${esc(tip)}</title></path>`
}

/**
 * A dashed vertical reference line with its label above. `gap` = [top, bottom] leaves the
 * line out where it would cross a bar and its value label.
 */
function refLine(cls, x, y0, y1, label, anchor = 'middle', gap = null) {
  const line = (a, b) => `<line class="${cls}" x1="${x}" x2="${x}" y1="${a}" y2="${b}"/>`
  const lines = gap ? line(y0, gap[0]) + line(gap[1], y1) : line(y0, y1)
  return `${lines}<text class="t3" x="${x}" y="${y0 - 4}" text-anchor="${anchor}">${esc(label)}</text>`
}

function charts(lang) {
  const t = T[lang]
  const f = (v, d) =>
    new Intl.NumberFormat(lang, { minimumFractionDigits: d, maximumFractionDigits: d }).format(v)
  const out = {}
  // "< 50 ms" from budgets.json, with the unit and number in this language.
  const budgetText = (b) => `${b.text.split(' ')[0]} ${f(b.value, 0)} ${t.units[b.unit] ?? b.unit}`

  // 1. Budgets: one panel per NFR, the measured bar against a dashed budget line.
  {
    const panels = [
      { title: t.open, unit: t.openUnit(f(measured.openPhotos, 0)), v: measured.openPer1000S, b: budgets['NFR-001'], d: 2 },
      { title: t.preview, unit: t.previewUnit, v: measured.previewP95Ms, b: budgets['NFR-002'], d: 0 },
      { title: t.export, unit: t.exportUnit(meta.megapixels), v: measured.exportPerSecond, b: budgets['NFR-003'], d: 1 },
      { title: t.memory, unit: t.memoryUnit, v: measured.peakGb, b: budgets['NFR-004'], d: 2 },
    ]
    const ph = 84
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 30 + i * ph
      const max = Math.max(p.v, p.b.value) * 1.2
      const x = (v) => LEFT + (PLOT * v) / max
      const ok = p.b.better === 'lower' ? p.v <= p.b.value : p.v >= p.b.value
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="0" y="${y0 + 30}">${esc(p.unit)} · ${esc(p.b.better === 'lower' ? t.lower : t.higher)}</text>`
      const by = y0 + 50
      body += `<text class="t2" x="${LEFT - 10}" y="${by + 16}" text-anchor="end">${esc(t.rust)}</text>`
      body += bar('s1', LEFT, by, Math.max(4, x(p.v) - LEFT), 22, `${p.title}: ${f(p.v, p.d)}`)
      body += `<text class="t1 num" x="${x(p.v) + 8}" y="${by + 16}">${f(p.v, p.d)}${ok ? ' ✓' : ''}</text>`
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${by - 4}" y2="${by + 26}"/>`
      body += refLine('ref', x(p.b.value), by - 4, by + 26, `${t.budget} ${budgetText(p.b)}`)
    })
    const h = 30 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, true))}</text>`
    const desc = panels.map((p) => `${p.title}: ${f(p.v, p.d)} (${t.budget} ${budgetText(p.b)})`).join('. ')
    out['bench-budgets'] = doc(t.budgetsTitle, desc, h, body)
  }

  // 2. Memory vs batch size, against the 2 GB ceiling.
  {
    const rows = [{ n: rust.photos, gb: rust.peakRssBytes / 1e9 }]
    for (const b of [...batches].reverse()) rows.push({ n: b.photos, gb: b.peakRssBytes / 1e9 })
    const ceiling = budgets['NFR-004'].value
    const max = ceiling * 1.2
    const x = (v) => LEFT + (PLOT * v) / max
    let body = `<text class="t3" x="0" y="30">${esc(t.memUnit)} · ${esc(t.lower)}</text>`
    const y0 = 62
    rows.forEach((r, i) => {
      const y = y0 + i * 32
      const label = `${f(r.n, 0)} ${t.photos}`
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(label)}</text>`
      body += bar('s1', LEFT, y, x(r.gb) - LEFT, 22, `${label}: ${f(r.gb, 2)} GB`)
      body += `<text class="t1 num" x="${x(r.gb) + 8}" y="${y + 16}">${f(r.gb, 2)} GB</text>`
    })
    const yEnd = y0 + rows.length * 32 - 6
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 - 4}" y2="${yEnd}"/>`
    body += refLine('ref', x(ceiling), y0 - 6, yEnd, `${t.ceiling} ${f(ceiling, 0)} GB (NFR-004)`)
    const h = yEnd + 30
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, true))}</text>`
    const desc = rows.map((r) => `${r.n} ${t.photos}: ${f(r.gb, 2)} GB`).join('. ')
    out['bench-memory'] = doc(t.memTitle, `${desc}. ${t.ceiling} ${ceiling} GB.`, h, body)
  }

  // 3. Fotollection vs Chromium; the browser is the reference.
  {
    const panels = [
      { title: t.cExport, unit: t.cExportUnit, better: t.higher, key: 'exportPerSecond', fmt: (v) => f(v, 1) },
      { title: t.cPreview, unit: t.cPreviewUnit, better: t.lower, key: 'previewWarmP50Ms', fmt: (v) => f(v, 1) },
      { title: t.cMemory, unit: t.cMemoryUnit, better: t.lower, key: 'peakRssBytes', fmt: (v) => f(v / 1e9, 2) },
    ]
    const rows = [
      { name: t.rust, cls: 's1', data: rust },
      { name: t.web, cls: 's2', data: web },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      const max = Math.max(...rows.map((r) => r.data[p.key]))
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)} · ${esc(p.better)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = r.data[p.key]
        const w = Math.max(4, (PLOT * v) / max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)} ${p.unit}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${p.fmt(v)}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * 30 - 4}"/>`
    })
    const h = 34 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, false))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(r.data[p.key])} ${p.unit}`).join(', ')}`)
      .join('. ')
    out.bench = doc(t.compareTitle(rust.photos), desc, h, body)
  }

  // 4. Preview breakdown: stacked segments, against the budget and Chromium.
  {
    const segs = [
      { name: t.resize, cls: 's3', v: rust.previewDrawMs },
      { name: t.stamp, cls: 's4', v: Math.max(0, rust.previewStampMs) },
      { name: t.encode, cls: 's5', v: rust.previewEncodeMs },
    ]
    const total = segs.reduce((s, x) => s + x.v, 0)
    const ms = (v) => (v < 0.05 ? `< ${f(0.1, 1)} ms` : `${f(v, 1)} ms`)
    const budget = budgets['NFR-002'].value
    const max = budget * 1.1
    const x = (v) => LEFT + (PLOT * v) / max
    let body = `<text class="t3" x="0" y="30">${esc(t.ms)} · ${esc(t.lower)}</text>`
    // Legend: swatch + name in text ink, so identity never relies on colour alone.
    let lx = 0
    for (const s of segs) {
      body += `<rect class="${s.cls}" x="${lx}" y="44" width="10" height="10" rx="2"/><text class="t2" x="${lx + 16}" y="53">${esc(`${s.name} ${ms(s.v)}`)}</text>`
      lx += 40 + (s.name.length + 7) * 7
    }
    const y = 100
    body += `<text class="t2" x="${LEFT - 10}" y="${y + 17}" text-anchor="end">${esc(t.rust)}</text>`
    let cx = LEFT
    for (const s of segs) {
      const w = (PLOT * s.v) / max
      if (w <= 0.5) continue
      body += `<rect class="${s.cls}" x="${cx}" y="${y}" width="${w}" height="26"><title>${esc(`${s.name}: ${f(s.v, 1)} ms`)}</title></rect>`
      if (w > 48) body += `<text class="t1 num in" x="${cx + w / 2}" y="${y + 17}" text-anchor="middle">${f(s.v, 1)}</text>`
      cx += w
      body += `<line class="gap" x1="${cx}" x2="${cx}" y1="${y}" y2="${y + 26}"/>`
    }
    // The total sits after the bar, or after the Chromium line when that is just past the bar.
    const xWeb = x(web.previewWarmP50Ms)
    const webInsideBar = xWeb < cx
    const totalX = !webInsideBar && xWeb - cx < 80 ? xWeb + 8 : cx + 8
    body += `<text class="t1 num" x="${totalX}" y="${y + 18}">${f(total, 1)} ms</text>`
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y - 6}" y2="${y + 32}"/>`
    body += refLine('refw', xWeb, y - 10, y + 36, `${t.webShort} ${f(web.previewWarmP50Ms, 1)}`, 'end', webInsideBar ? [y - 2, y + 28] : null)
    body += refLine('ref', x(budget), y - 10, y + 34, `${t.budget} ${budgetText(budgets['NFR-002'])}`, 'end')
    const h = y + 66
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, false))}</text>`
    const desc = `${segs.map((s) => `${s.name} ${f(s.v, 1)} ms`).join(', ')}; ${f(total, 1)} ms. ${t.web} ${f(web.previewWarmP50Ms, 1)} ms. ${t.budget} ${budget} ms.`
    out['bench-preview'] = doc(t.previewTitle, desc, h, body)
  }
  // 6. Print quality: Fotollection's export against a typical web-editor export.
  {
    const S = quality.sets.srgb
    const P = quality.sets.displayP3
    const rows = [
      { name: t.rust, cls: 's1', who: 'fotollection' },
      { name: t.qWeb, cls: 's2', who: 'web_editor' },
    ]
    const panels = [
      { title: t.qDetail, unit: t.qPercent, v: (w) => S[w].detail_kept * 100, max: 100, fmt: (v) => `${f(v, 0)} %` },
      { title: t.qChroma, unit: t.qPercent, v: (w) => P[w].chroma_kept * 100, max: 100, fmt: (v) => `${f(v, 1)} %` },
      { title: t.qDe, unit: t.qDeUnit, v: (w) => S[w].de_mean, max: 2, fmt: (v) => f(v, 2), ref: 1 },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = p.v(r.who)
        const w = Math.max(4, (PLOT * Math.min(v, p.max)) / p.max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.fmt(v))}</text>`
      })
      const yEnd = y0 + 26 + rows.length * 30 - 4
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${yEnd}"/>`
      if (p.ref !== undefined) {
        body += refLine('ref', LEFT + (PLOT * p.ref) / p.max, y0 + 30, yEnd, t.qVisible, 'middle')
      }
    })
    const h = 34 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.qFooter(f(S.photos, 0), f(P.photos, 0)))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(p.v(r.who))}`).join(', ')}`)
      .join('. ')
    out['bench-quality'] = doc(t.qTitle, desc, h, body)
  }

  // 7. Share export (FR-053): after Facebook's recompression, Fotollection vs the usual way.
  {
    const fb = share.presets.facebook
    const S = fb.srgb
    const P = fb.displayP3
    const rows = [
      { name: t.rust, cls: 's1', who: 'fotollection_after_facebook' },
      { name: t.shUsual, cls: 's2', who: 'usual_after_facebook' },
    ]
    const panels = [
      { title: t.shChroma, unit: t.qPercent, v: (w) => P[w].chroma_kept * 100, max: 100, fmt: (v) => `${f(v, 1)} %` },
      { title: t.shDiff, unit: t.qDeUnit, v: (w) => (1 - S[w].ssim) * 100, max: 5, fmt: (v) => `${f(v, 1)} %` },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = p.v(r.who)
        const w = Math.max(4, (PLOT * Math.min(v, p.max)) / p.max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.fmt(v))}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * 30 - 4}"/>`
    })
    // File size: the camera file, then what each preset writes.
    const sizes = [
      { name: t.shSource, cls: 's3', v: share.presets.original.srgb.source_mb },
      { name: t.shOriginal, cls: 's1', v: share.presets.original.srgb.export_mb },
      { name: t.shFacebook, cls: 's1', v: S.export_mb },
    ]
    const y0 = 34 + panels.length * ph
    const maxMb = Math.max(...sizes.map((s) => s.v))
    body += `<text class="t1" x="0" y="${y0 + 14}">${esc(t.shSize)}</text>`
    body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(t.shMb)}</text>`
    sizes.forEach((s, j) => {
      const y = y0 + 26 + j * 30
      const w = Math.max(4, (PLOT * s.v) / maxMb)
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(s.name)}</text>`
      body += bar(s.cls, LEFT, y, w, 22, `${s.name}: ${f(s.v, 2)} MB`)
      body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(`${f(s.v, 2)} MB`)}</text>`
    })
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + sizes.length * 30 - 4}"/>`
    const h = y0 + 26 + sizes.length * 30 + 22
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.shFooter(f(S.photos, 0), f(P.photos, 0)))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(p.v(r.who))}`).join(', ')}`)
      .concat(sizes.map((s) => `${s.name} ${f(s.v, 2)} MB`))
      .join('. ')
    out['bench-share'] = doc(t.shTitle, desc, h, body)
  }

  // 5. Best-shot accuracy per improvement step (higher is better), against the labels.
  {
    const steps = bestshot.steps
    const panels = [
      {
        title: t.bsGroups(bestshot.labelledGroups),
        max: bestshot.labelledGroups,
        v: (s) => s.foundGroups,
        text: (s) => `${f(s.foundGroups, 0)}/${f(bestshot.labelledGroups, 0)}`,
      },
      {
        title: t.bsPicks,
        max: 1,
        v: (s) => s.picksRight / s.picksScored,
        text: (s) => `${f(s.picksRight, 0)}/${f(s.picksScored, 0)}`,
      },
    ]
    const ph = 40 + steps.length * 30
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 30 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(t.higher)}</text>`
      steps.forEach((s, j) => {
        const y = y0 + 26 + j * 30
        const w = Math.max(4, (PLOT * p.v(s)) / p.max)
        const last = j === steps.length - 1
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(s[lang])}</text>`
        body += bar(last ? 's1' : 's1 old', LEFT, y, w, 22, `${s[lang]}: ${p.text(s)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.text(s))}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + steps.length * 30 - 4}"/>`
    })
    const h = 30 + panels.length * ph + 30
    const [line1, line2] = t.bsFooter(f(bestshot.photos, 0), f(bestshot.labelledGroups, 0))
    body += `<text class="t3" x="0" y="${h - 20}">${esc(line1)}</text>`
    body += `<text class="t3" x="0" y="${h - 4}">${esc(line2)}</text>`
    const desc = steps
      .map((s) => `${s[lang]}: ${panels.map((p) => p.text(s)).join(', ')}`)
      .join('. ')
    out['bench-bestshot'] = doc(t.bsTitle, desc, h, body)
  }
  return out
}

for (const lang of ['en', 'vi']) {
  for (const [name, svg] of Object.entries(charts(lang))) {
    writeFileSync(join(images, `${name}${lang === 'en' ? '' : '-vi'}.svg`), svg)
  }
}
// The website shows these in its own colours: the home page and the performance page (site/README.md).
STYLE = styleFor(THEMES.site)
const SITE_CHARTS = ['bench-budgets', 'bench-quality', 'bench-share', 'bench-bestshot', 'bench-memory', 'bench']
mkdirSync(join(root, 'site/assets/img/charts'), { recursive: true })
for (const lang of ['en', 'vi']) {
  const all = charts(lang)
  for (const name of SITE_CHARTS) writeFileSync(join(root, 'site/assets/img/charts', `${name}-${lang}.svg`), all[name])
}
console.log(JSON.stringify({ measured, rust, web, inflight8, batch }, null, 2))
