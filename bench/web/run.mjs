// SPDX-License-Identifier: GPL-3.0-only
// Runs the browser baseline in headless Chromium on a local photo folder and prints JSON.
// Photos are served from their folder and never copied into the repository.
//
//   node bench/web/run.mjs <photo-folder> > bench/results/web.json

import { execFileSync } from 'node:child_process'
import { createServer } from 'node:http'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { extname, join, resolve } from 'node:path'
import { chromium } from 'playwright'

const folder = resolve(process.argv[2] ?? '')
const here = new URL('.', import.meta.url).pathname
const photos = readdirSync(folder).filter((f) => /\.jpe?g$/i.test(f)).sort()
if (photos.length === 0) throw new Error(`no JPEGs in ${folder}`)

const types = { '.html': 'text/html', '.js': 'text/javascript', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg' }
const server = createServer((req, res) => {
  const url = decodeURIComponent(new URL(req.url, 'http://x').pathname)
  const file = url.startsWith('/photos/') ? join(folder, url.slice(8)) : join(here, url === '/' ? 'index.html' : url)
  if (!file.startsWith(folder) && !file.startsWith(here)) return res.writeHead(403).end()
  try {
    res.writeHead(200, { 'content-type': types[extname(file).toLowerCase()] ?? 'application/octet-stream' })
    res.end(readFileSync(file))
  } catch {
    res.writeHead(404).end()
  }
})
await new Promise((r) => server.listen(0, '127.0.0.1', r))
const base = `http://127.0.0.1:${server.address().port}`

// Peak resident memory of the browser and all its child processes, sampled every 100 ms.
function treeRss(rootPid) {
  const rows = execFileSync('ps', ['-A', '-o', 'pid=,ppid=,rss=']).toString().trim().split('\n')
  const procs = rows.map((r) => r.trim().split(/\s+/).map(Number))
  const pids = new Set([rootPid])
  let grew = true
  while (grew) {
    grew = false
    for (const [pid, ppid] of procs) if (pids.has(ppid) && !pids.has(pid)) (pids.add(pid), (grew = true))
  }
  return procs.filter(([pid]) => pids.has(pid)).reduce((sum, [, , rss]) => sum + rss * 1024, 0)
}

// The browser is a child of this runner; its own RSS is subtracted.
const browser = await chromium.launch()
let peakRss = 0
const sampler = setInterval(() => {
  peakRss = Math.max(peakRss, treeRss(process.pid) - process.memoryUsage().rss)
}, 100)
const page = await browser.newPage()
await page.goto(base)
const result = await page.evaluate((args) => window.runBench(args), {
  urls: photos.map((p) => `${base}/photos/${encodeURIComponent(p)}`),
  repeats: 20,
})
clearInterval(sampler)
const version = browser.version()
await browser.close()
server.close()

const bytes = photos.reduce((s, p) => s + statSync(join(folder, p)).size, 0)
console.log(
  JSON.stringify(
    { engine: `Chromium ${version} (headless)`, ...result, peakRssBytes: peakRss, inputBytes: bytes },
    null,
    2,
  ),
)
