// SPDX-License-Identifier: GPL-3.0-only
// The landing page's keynote video, from keynote.html and the real app's screenshots.
//
//   node site/screenshots/keynote.mjs <out-dir> [desktop|mobile ...] [--text] [--frame <seconds>]
//
// Steps keynote.html through every frame (no real-time recording, so each run gives the same
// video), pipes the frames to ffmpeg as a lossless master, then encodes AV1 and H.264 MP4s
// with no audio. --text bakes the headlines in (for review; the page lays them over the video).
// --frame writes one PNG instead. No network: every request outside keynote.localhost is refused.

import { readFile, mkdir, stat } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
import yaml from 'js-yaml'
import { chromium } from 'playwright'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const args = process.argv.slice(2)
const outDir = path.resolve(args[0] ?? '')
if (!args[0]) {
  console.error('usage: node site/screenshots/keynote.mjs <out-dir> [desktop|mobile ...] [--text] [--frame <seconds>]')
  process.exit(2)
}
const text = args.includes('--text')
const frameAt = args.includes('--frame') ? Number(args[args.indexOf('--frame') + 1]) : null
const CUTS = { desktop: { w: 1280, h: 720 }, mobile: { w: 720, h: 900 } }
const cuts = args.filter((a) => a in CUTS)
const FPS = 30
const DURATION = 20

const TYPES = { '.html': 'text/html', '.webp': 'image/webp', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2' }
const files = {
  '/keynote.html': path.join(root, 'site/screenshots/keynote.html'),
  '/img/': path.join(root, 'site/assets/img/'),
  '/fonts/': path.join(root, 'site/assets/fonts/'),
  '/brand/': path.join(root, 'ui/public/brand/'),
  '/logos/': path.join(root, 'site/_includes/logos/'),
}

// The headlines and their times, the same as the landing page lays over the video
// (site/_data/keynote.yml), in English for review.
const en = yaml.load(readFileSync(path.join(root, 'site/_data/i18n/en.yml'), 'utf8'))
const LINES = yaml.load(readFileSync(path.join(root, 'site/_data/keynote.yml'), 'utf8')).map((l) => ({
  from: l.from,
  to: l.to,
  text: en[l.key],
  sub: l.sub ? en[l.sub] : undefined,
}))

const run = (cmd, argv, input) =>
  new Promise((resolve, reject) => {
    const p = spawn(cmd, argv, { stdio: [input ? 'pipe' : 'ignore', 'inherit', 'inherit'] })
    p.on('error', reject)
    p.on('exit', (code) => (code === 0 ? resolve() : reject(new Error(`${cmd} exited ${code}`))))
    if (input) input(p.stdin)
  })

await mkdir(outDir, { recursive: true })
const browser = await chromium.launch()
for (const name of cuts.length ? cuts : ['desktop', 'mobile']) {
  const { w, h } = CUTS[name]
  const page = await browser.newPage({ viewport: { width: w, height: h }, deviceScaleFactor: 1 })
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url())
    if (url.host !== 'keynote.localhost') return route.abort()
    const prefix = Object.keys(files).find((k) => url.pathname === k || (k.endsWith('/') && url.pathname.startsWith(k)))
    if (!prefix) return route.fulfill({ status: 404 })
    const file = prefix.endsWith('/') ? path.join(files[prefix], decodeURIComponent(url.pathname.slice(prefix.length))) : files[prefix]
    if (!file.startsWith(root)) return route.fulfill({ status: 403 })
    let body = await readFile(file).catch(() => null)
    if (!body) return route.fulfill({ status: 404 })
    // The logos are inline on the site and take the page's --bg; standalone their cut-outs stay open, and the system marks take the headline's cream.
    if (file.endsWith('.svg')) body = Buffer.from(body.toString().replaceAll('var(--bg)', 'none').replaceAll('currentColor', '#fffaee'))
    return route.fulfill({ body, contentType: TYPES[path.extname(file)] ?? 'application/octet-stream' })
  })
  await page.addInitScript((lines) => { window.KEYNOTE = lines }, LINES)
  await page.goto(`http://keynote.localhost/keynote.html?cut=${name}&text=${text ? 1 : 0}`)
  await page.evaluate(() => window.ready)

  if (frameAt !== null) {
    await page.evaluate((t) => window.seek(t), frameAt)
    const file = path.join(outDir, `frame-${name}-${frameAt}.png`)
    await page.screenshot({ path: file })
    console.log(file)
    continue
  }

  // The master: lossless, so each encode below starts from the same pixels.
  const tag = `keynote-${name}${text ? '-text' : ''}`
  const master = path.join(outDir, `${tag}-master.mkv`)
  const started = Date.now()
  await run('ffmpeg', ['-v', 'error', '-y', '-f', 'image2pipe', '-framerate', String(FPS), '-c:v', 'png', '-i', '-', '-c:v', 'ffv1', '-pix_fmt', 'yuv444p', master], async (stdin) => {
    for (let i = 0; i < DURATION * FPS; i++) {
      await page.evaluate((t) => window.seek(t), i / FPS)
      const png = await page.screenshot({ type: 'png' })
      if (!stdin.write(png)) await new Promise((r) => stdin.once('drain', r))
    }
    stdin.end()
  })
  console.log(`${tag}: ${DURATION * FPS} frames in ${((Date.now() - started) / 1000).toFixed(1)} s`)

  const common = ['-v', 'error', '-y', '-i', master, '-an', '-pix_fmt', 'yuv420p', '-movflags', '+faststart']
  const av1 = path.join(outDir, `${tag}-av1.mp4`)
  const h264 = path.join(outDir, `${tag}-h264.mp4`)
  await run('ffmpeg', [...common, '-c:v', 'libsvtav1', '-preset', '4', '-crf', '40', '-g', '300', av1])
  await run('ffmpeg', [...common, '-c:v', 'libx264', '-preset', 'veryslow', '-crf', '27', '-profile:v', 'high', '-tune', 'animation', h264])
  for (const f of [av1, h264]) console.log(`${path.basename(f)}: ${((await stat(f)).size / 1024).toFixed(0)} KB`)
  await page.close()
}
await browser.close()
