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

import { readFileSync, readdirSync, writeFileSync } from 'node:fs'
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
const STYLE = `<style>
  .bg{fill:#fcfcfb} .t1{fill:#1f1f1e;font:600 13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t2{fill:#52514c;font:13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t3{fill:#6f6e68;font:12px system-ui,-apple-system,Segoe UI,sans-serif}
  .h{font-size:15px} .num{font-variant-numeric:tabular-nums} .axis{stroke:#c9c8c1;stroke-width:1}
  .ref{stroke:#52514c;stroke-width:1.5;stroke-dasharray:4 3} .refw{stroke:#eb6834;stroke-width:1.5;stroke-dasharray:4 3}
  .s1{fill:#2a78d6} .s2{fill:#eb6834} .s3{fill:#1baf7a} .s4{fill:#eda100} .s5{fill:#e87ba4} .gap{stroke:#fcfcfb;stroke-width:2}
  .in{fill:#fff}
  @media (prefers-color-scheme: dark){
    .bg{fill:#1a1a19} .t1{fill:#f0efec} .t2{fill:#c3c2b7} .t3{fill:#9d9c93} .axis{stroke:#45443f}
    .ref{stroke:#c3c2b7} .refw{stroke:#d95926}
    .s1{fill:#3987e5} .s2{fill:#d95926} .s3{fill:#199e70} .s4{fill:#c98500} .s5{fill:#d55181} .gap{stroke:#1a1a19}
  }
</style>`

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
    const budget = budgets['NFR-002'].value
    const max = budget * 1.1
    const x = (v) => LEFT + (PLOT * v) / max
    let body = `<text class="t3" x="0" y="30">${esc(t.ms)} · ${esc(t.lower)}</text>`
    // Legend: swatch + name in text ink, so identity never relies on colour alone.
    let lx = 0
    for (const s of segs) {
      body += `<rect class="${s.cls}" x="${lx}" y="44" width="10" height="10" rx="2"/><text class="t2" x="${lx + 16}" y="53">${esc(`${s.name} ${f(s.v, 1)} ms`)}</text>`
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
    body += `<text class="t1 num" x="${cx + 8}" y="${y + 18}">${f(total, 1)} ms</text>`
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y - 6}" y2="${y + 32}"/>`
    body += refLine('refw', x(web.previewWarmP50Ms), y - 10, y + 36, `${t.webShort} ${f(web.previewWarmP50Ms, 1)}`, 'end', [y - 2, y + 28])
    body += refLine('ref', x(budget), y - 10, y + 34, `${t.budget} ${budgetText(budgets['NFR-002'])}`, 'end')
    const h = y + 66
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, false))}</text>`
    const desc = `${segs.map((s) => `${s.name} ${f(s.v, 1)} ms`).join(', ')}; ${f(total, 1)} ms. ${t.web} ${f(web.previewWarmP50Ms, 1)} ms. ${t.budget} ${budget} ms.`
    out['bench-preview'] = doc(t.previewTitle, desc, h, body)
  }
  return out
}

for (const lang of ['en', 'vi']) {
  for (const [name, svg] of Object.entries(charts(lang))) {
    writeFileSync(join(images, `${name}${lang === 'en' ? '' : '-vi'}.svg`), svg)
  }
}
console.log(JSON.stringify({ measured, rust, web, inflight8, batch }, null, 2))
