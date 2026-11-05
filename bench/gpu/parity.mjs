// SPDX-License-Identifier: GPL-3.0-only
// The GPU look against Rust's (ADR-0039): the app's own shader (ui/src/features/viewer/
// gpu-look.ts, compiled here) draws plain.png through table.bin with mask.png, in WebKit and
// Chromium, and writes gpu-<engine>.png next to them for bench/quality/look_parity.py.
//
//   cargo run --release -p fotollection-core --example look_parity -- <photo.jpg> <dir>
//   node bench/gpu/parity.mjs <dir>
//   uv run python -I bench/quality/look_parity.py <dir>
import { execFileSync } from 'node:child_process'
import { createServer } from 'node:http'
import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium, webkit } from 'playwright'

const here = path.dirname(fileURLToPath(import.meta.url))
const root = path.join(here, '../..')
const dir = path.resolve(process.argv[2] ?? '')
if (!process.argv[2]) {
  console.error('usage: node bench/gpu/parity.mjs <dir written by look_parity>')
  process.exit(2)
}
// The shader exactly as the app ships it.
execFileSync(path.join(root, 'node_modules/.bin/tsc'), [
  '--ignoreConfig', '--target', 'es2022', '--module', 'es2022', '--lib', 'es2022,dom',
  '--outDir', path.join(dir, 'js'), path.join(root, 'ui/src/features/viewer/gpu-look.ts'),
])
const page = `<!doctype html><meta charset="utf-8"><canvas id="c"></canvas>
<script type="module">
import { GpuLook } from './js/gpu-look.js'
const bitmap = async (u) => createImageBitmap(await (await fetch(u)).blob(), { colorSpaceConversion: 'none', premultiplyAlpha: 'none' })
window.probe = () => GpuLook.worthIt()
window.run = async () => {
  const canvas = document.getElementById('c')
  const look = GpuLook.create(canvas)
  if (!look) return null
  look.setPhoto(await bitmap('plain.png'), await bitmap('mask.png'))
  const table = new Uint8Array(await (await fetch('table.bin')).arrayBuffer())
  if (!look.draw(table)) return null
  return canvas.toDataURL('image/png')
}
</script>`
const server = createServer(async (req, res) => {
  const name = req.url === '/' ? null : decodeURIComponent(req.url.slice(1))
  if (!name) return res.end(page)
  try {
    res.setHeader('content-type', name.endsWith('.js') ? 'text/javascript' : 'application/octet-stream')
    res.end(await readFile(path.join(dir, name)))
  } catch {
    res.statusCode = 404
    res.end()
  }
}).listen(8778)
for (const [name, type, options] of [
  ['webkit', webkit, {}],
  ['chromium-metal', chromium, { args: ['--use-angle=metal', '--enable-gpu', '--ignore-gpu-blocklist'] }],
  ['chromium-software', chromium, {}],
]) {
  const browser = await type.launch(options)
  const p = await browser.newPage()
  await p.goto('http://localhost:8778/')
  // What the app decides on this engine (GpuLook.worthIt): the GPU, or Rust's path.
  console.log(name, 'uses the GPU:', await p.evaluate(() => window.probe()))
  const url = await p.evaluate(() => window.run())
  if (url) await writeFile(path.join(dir, `gpu-${name}.png`), Buffer.from(url.split(',')[1], 'base64'))
  console.log(name, url ? 'drawn' : 'no WebGL2')
  await browser.close()
}
server.close()
