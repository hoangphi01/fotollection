// SPDX-License-Identifier: GPL-3.0-only
// The look on the GPU (ADR-0039, stage 1): one slider step in WebGL2, a new 33³ table each
// step, tetrahedral interpolation with texelFetch, in WebKit (as WKWebView) and Chromium
// (with Metal, as WebView2 uses the GPU, and with SwiftShader, as a machine without one).
//
//   node bench/gpu/run.mjs <photo.jpg>   (a 1600 px JPEG, as the preview sends)
//
// Writes bench/results/gpu.json (bench/chart.mjs draws it). Nothing leaves the machine: the
// page is served on localhost and the photo is only read.
import { createServer } from 'node:http'
import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium, webkit } from 'playwright'

const here = path.dirname(fileURLToPath(import.meta.url))
const photo = process.argv[2]
if (!photo) {
  console.error('usage: node bench/gpu/run.mjs <photo.jpg>')
  process.exit(2)
}
const server = createServer(async (req, res) => {
  const file = req.url === '/photo.jpg' ? photo : path.join(here, 'index.html')
  try {
    res.end(await readFile(file))
  } catch {
    res.statusCode = 404
    res.end()
  }
}).listen(8777)

const engines = [
  ['WebKit (Apple GPU)', webkit, {}],
  ['Chromium, ANGLE Metal', chromium, { args: ['--use-angle=metal', '--enable-gpu', '--ignore-gpu-blocklist'] }],
  ['Chromium, SwiftShader (no GPU)', chromium, {}],
]
const results = []
for (const [name, type, options] of engines) {
  const browser = await type.launch(options)
  const page = await browser.newPage()
  await page.goto('http://localhost:8777/')
  const r = await page.evaluate(() => window.run())
  console.log(name, JSON.stringify(r))
  results.push({ name, renderer: r.renderer, decodeMs: r.decode, uploadMs: r.upload, stepMs: r.step, stepMaxMs: r.stepMax })
  await browser.close()
}
server.close()
const file = path.join(here, '../results/gpu.json')
const old = JSON.parse(await readFile(file, 'utf8').catch(() => '{}'))
await writeFile(file, JSON.stringify({ ...old, date: new Date().toISOString().slice(0, 10), engines: results }, null, 2) + '\n')
