// SPDX-License-Identifier: GPL-3.0-only
// Turns bench/results/{rust,web}-*.json into bench/results/summary.json and
// docs/images/bench.svg (plain SVG, no dependencies). Each metric is the median of the runs.
//
//   node bench/chart.mjs

import { readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const root = new URL('..', import.meta.url).pathname
const results = join(root, 'bench/results')

function load(prefix) {
  const runs = readdirSync(results)
    .filter((f) => f.startsWith(`${prefix}-`) && f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(results, f), 'utf8')))
  if (runs.length === 0) throw new Error(`no ${prefix}-*.json in ${results}`)
  const median = (key) => {
    const v = runs.map((r) => r[key]).sort((a, b) => a - b)
    return v[Math.floor(v.length / 2)]
  }
  return {
    engine: runs[0].engine,
    threads: runs[0].threads,
    photos: runs[0].photos,
    runs: runs.length,
    exportPerSecond: median('exportPerSecond'),
    previewWarmP50Ms: median('previewWarmP50Ms'),
    previewColdMs: median('previewColdMs'),
    peakRssBytes: median('peakRssBytes'),
  }
}

const rust = load('rust')
const web = load('web')
const meta = JSON.parse(readFileSync(join(results, 'machine.json'), 'utf8'))
writeFileSync(join(results, 'summary.json'), `${JSON.stringify({ ...meta, rust, web }, null, 2)}\n`)

const panels = [
  { title: 'Batch export', unit: 'photos / s', better: 'higher is better', key: 'exportPerSecond', fmt: (v) => v.toFixed(1) },
  { title: 'Warm preview (1600 px)', unit: 'ms, p50', better: 'lower is better', key: 'previewWarmP50Ms', fmt: (v) => v.toFixed(1) },
  { title: 'Peak memory', unit: 'GB', better: 'lower is better', key: 'peakRssBytes', fmt: (v) => (v / 1e9).toFixed(2) },
]
const rows = [
  { name: 'Fotollection (Rust)', cls: 's1', data: rust },
  { name: 'Browser (Chromium)', cls: 's2', data: web },
]

const W = 720
const left = 170
const right = 70
const barH = 22
const gap = 8
const panelH = 30 + rows.length * (barH + gap) + 18
const top = 34
const H = top + panels.length * panelH + 26
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;')

let body = ''
panels.forEach((p, i) => {
  const y0 = top + i * panelH
  const max = Math.max(...rows.map((r) => r.data[p.key]))
  body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
  body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)} · ${esc(p.better)}</text>`
  rows.forEach((r, j) => {
    const y = y0 + 26 + j * (barH + gap)
    const v = r.data[p.key]
    const w = Math.max(4, ((W - left - right) * v) / max)
    body += `<text class="t2" x="${left - 10}" y="${y + barH / 2 + 4}" text-anchor="end">${esc(r.name)}</text>`
    // Square at the baseline, 4 px rounded data end.
    body += `<path class="${r.cls}" d="M${left} ${y}h${w - 4}a4 4 0 0 1 4 4v${barH - 8}a4 4 0 0 1 -4 4h${4 - w}z"><title>${esc(r.name)}: ${p.fmt(v)} ${esc(p.unit)}</title></path>`
    body += `<text class="t1 num" x="${left + w + 8}" y="${y + barH / 2 + 4}">${p.fmt(v)}</text>`
  })
  body += `<line class="axis" x1="${left}" x2="${left}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * (barH + gap) - gap + 4}"/>`
})

const date = meta.date
const note = `${rust.photos} × ${meta.megapixels} MP JPEGs · ${meta.machine} · ${date} · median of ${rust.runs} runs`
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-16 -16 ${W + 32} ${H + 32}" width="${W + 32}" height="${H + 32}" role="img" aria-labelledby="t d">
<title id="t">Fotollection vs browser: export throughput, preview latency, peak memory</title>
<desc id="d">${esc(rows.map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(r.data[p.key])} ${p.unit}`).join(', ')}`).join('. '))}. ${esc(note)}.</desc>
<style>
  .bg{fill:#fcfcfb} .t1{fill:#1f1f1e;font:600 13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t2{fill:#52514c;font:13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t3{fill:#6f6e68;font:12px system-ui,-apple-system,Segoe UI,sans-serif}
  .num{font-variant-numeric:tabular-nums} .axis{stroke:#c9c8c1;stroke-width:1}
  .s1{fill:#2a78d6} .s2{fill:#eb6834}
  @media (prefers-color-scheme: dark){
    .bg{fill:#1a1a19} .t1{fill:#f0efec} .t2{fill:#c3c2b7} .t3{fill:#9d9c93} .axis{stroke:#45443f}
    .s1{fill:#3987e5} .s2{fill:#d95926}
  }
</style>
<rect class="bg" x="-16" y="-16" width="${W + 32}" height="${H + 32}" rx="8"/>
<text class="t1" x="0" y="8" style="font-size:15px">Same ${rust.photos} photos, same job: decode → crop 2:3 → date stamp → JPEG q95</text>
${body}
<text class="t3" x="0" y="${H - 6}">${esc(note)}</text>
</svg>
`
writeFileSync(join(root, 'docs/images/bench.svg'), svg)
console.log(JSON.stringify({ rust, web }, null, 2))
