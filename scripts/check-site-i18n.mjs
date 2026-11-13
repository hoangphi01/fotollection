// SPDX-License-Identifier: GPL-3.0-only
// Checks the website's languages (site/README.md, ADR-0049). Without arguments (check-docs.sh,
// no packages needed) it fails when:
//   - a template picks a language by name (`== 'vi'`, a `/vi/` path, `x[lang]` on data) instead
//     of the registry, url_for and tr, or defaults a filter's argument (`, x | default: y`);
//   - a words file is not a language of site/_data/languages.yml, or a published language has none;
//   - a language has a key English lacks (a typo), or a text lost or gained a placeholder
//     ([n], [tag]…) English has;
//   - a short guide lacks a chapter of docs/USER_GUIDE.md, or a Performance part has no text;
//   - the languages are not in order (English first, then by their own names);
//   - a published language lacks a page's link preview: its og_card_<page> words or the image.
// With --build (the pages workflow, and check-docs.sh when the packages are installed) it also
// builds the site with every borrowed English text marked (I18N_MARK=1, eleventy.config.mjs) and
// fails when a published language's pages show any English, naming the keys and the pages.
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const site = path.join(root, 'site')
const problems = []

// 1. No language named in a template.
const banned = [
  [/==\s*['"](vi|en|hu|de|pl|cs|ja)['"]/, 'compares the language with a name; use pg.* from the registry'],
  [/['"]\/(vi|hu|de|pl|cs|ja)\//, 'writes a language\'s path; use url_for'],
  [/\w\[lang\]/, 'reads a text by [lang]; use `| tr: lang`'],
  // Liquid ends a filter's arguments at `|`: `replace: "[n]", x | default: y` defaults the whole
  // text, not x (LESSONS); assign the value first.
  [/:\s*[^|%}]*,\s*[\w.[\]]+\s*\|\s*default:/, 'puts `| default:` after a filter argument, where it applies to the whole text'],
]
for (const dir of ['_layouts', '_includes']) {
  const walk = (d) => {
    for (const name of readdirSync(d)) {
      const file = path.join(d, name)
      if (statSync(file).isDirectory()) walk(file)
      else if (name.endsWith('.html')) {
        readFileSync(file, 'utf8').split('\n').forEach((line, i) => {
          for (const [re, why] of banned) if (re.test(line)) problems.push(`${path.relative(root, file)}:${i + 1} ${why}`)
        })
      }
    }
  }
  walk(path.join(site, dir))
}

// 2. The registry and the words files agree.
const registry = path.join(site, '_data/languages.yml')
const codes = []
const published = []
const pagesOf = {}
const names = {}
const guideOf = {}
const kioskOf = {}
if (!existsSync(registry)) problems.push('site/_data/languages.yml is missing')
else {
  let code = null
  for (const line of readFileSync(registry, 'utf8').split('\n')) {
    const c = line.match(/^- code: (\w+)/)
    if (c) codes.push((code = c[1]))
    const n = line.match(/^ {2}name: (.+)$/)
    if (n && code) names[code] = n[1].trim()
    const g = line.match(/^ {2}guide: (.+)$/)
    if (g && code) guideOf[code] = g[1].trim()
    const k = line.match(/^ {2}kiosk: (\w+)/)
    if (k && code) kioskOf[code] = k[1]
    const p = line.match(/^ {2}pages: \[(.*)\]/)
    if (p && code) pagesOf[code] = p[1].split(',').map((x) => x.trim())
    if (/^ {2}published: true\b/.test(line) && code) published.push(code)
  }
}
// The order (owner, 2026-11-11): English first, then by each language's own name.
const sorted = [...codes.slice(1)].sort((a, b) => names[a].localeCompare(names[b], 'en'))
if (codes[0] !== 'en' || codes.slice(1).join() !== sorted.join()) problems.push(`site/_data/languages.yml: English first, then by own name: en, ${sorted.join(', ')}`)
// Each published language's own texts for its pages: a short guide with English's chapters,
// in order (a chapter added to USER_GUIDE.md is added to every short guide), and the
// Performance page's explanations, one per part.
const chapters = (file) => [...readFileSync(file, 'utf8').matchAll(/^<!-- chapter: ([a-z0-9-]+) -->$/gm)].map((m) => m[1]).join(' ')
const parts = readdirSync(path.join(site, '_includes/perf')).filter((f) => f.endsWith('-en.md'))
for (const c of published) {
  const g = guideOf[c]
  if ((pagesOf[c] ?? []).includes('manual') && g && !g.startsWith('docs/')) {
    const file = path.join(root, g)
    if (!existsSync(file)) problems.push(`${c}: its short guide ${g} is missing`)
    else if (spawnSync('git', ['check-ignore', '-q', g], { cwd: root }).status === 0) problems.push(`${g} is ignored by .gitignore, so it would never reach the repository`)
    else if (chapters(file) !== chapters(path.join(root, 'docs/USER_GUIDE.md'))) problems.push(`${g}: its chapters must be docs/USER_GUIDE.md's, in order (${chapters(path.join(root, 'docs/USER_GUIDE.md'))})`)
  }
  if ((pagesOf[c] ?? []).includes('performance')) {
    for (const f of parts) if (!existsSync(path.join(site, '_includes/perf', f.replace('-en.md', `-${c}.md`)))) problems.push(`${c}: no site/_includes/perf/${f.replace('-en.md', `-${c}.md`)}`)
  }
}
// Each published language's link previews: its words (og_card_<page>) and the image og.mjs made.
for (const c of published) {
  for (const id of pagesOf[c] ?? []) {
    const file = id === 'home' ? `og-${c}.jpg` : `og-${id}-${c}.jpg`
    if (!existsSync(path.join(site, 'assets/img', file))) problems.push(`${c}: no site/assets/img/${file}; run node site/screenshots/og.mjs`)
  }
}
const wordsDir = path.join(site, '_data/i18n')
const files = existsSync(wordsDir) ? readdirSync(wordsDir).filter((f) => f.endsWith('.yml')).map((f) => f.slice(0, -4)) : []
if (!files.includes('en')) problems.push('site/_data/i18n/en.yml, the source of every word, is missing')
for (const f of files) if (!codes.includes(f)) problems.push(`site/_data/i18n/${f}.yml is not a language of site/_data/languages.yml`)
for (const c of published) if (!files.includes(c)) problems.push(`${c} is published but has no site/_data/i18n/${c}.yml`)

// 3. Keys and placeholders follow English. A top-level key is `name:` at the line's start; a
// one-line text's placeholders are its [word] marks.
const read = (code) => {
  const keys = new Map()
  for (const line of readFileSync(path.join(wordsDir, `${code}.yml`), 'utf8').split('\n')) {
    const m = line.match(/^([a-z0-9_]+):\s?(.*)$/)
    if (m) keys.set(m[1], m[2])
  }
  return keys
}
// A text's placeholders: `[n]`, and `{dm}` / `{rossmann}`, the stores' marks in the keynote's lines;
// a language whose home page shows Japan's stores (languages.yml kiosk: jp) has theirs instead.
const marks = (text) => [...text.matchAll(/\[[a-z]+\]|\{[a-z]+\}/g)].map((m) => m[0]).sort().join(' ')
const STORES = { eu: ['{dm}', '{rossmann}'], jp: ['{familymart}', '{lawson}', '{seven}'] }
const asIn = (code, text) => {
  const own = STORES[kioskOf[code] ?? 'eu']
  const m = marks(text).split(' ').filter(Boolean)
  return m.some((x) => STORES.eu.includes(x)) ? [...m.filter((x) => !STORES.eu.includes(x)), ...own].sort().join(' ') : m.join(' ')
}
if (files.includes('en')) {
  const en = read('en')
  for (const c of published.filter((x) => x !== 'en' && files.includes(x))) {
    const own = read(c)
    for (const id of pagesOf[c] ?? []) {
      const key = `og_card_${id.replace(/-/g, '_')}`
      if (!own.has(key)) problems.push(`${c}.yml lacks ${key}, its ${id} page's link preview`)
    }
    if (!own.has('og_vnlf')) problems.push(`${c}.yml lacks og_vnlf, its link previews' badge`)
  }
  for (const code of files.filter((f) => f !== 'en')) {
    for (const [key, text] of read(code)) {
      if (!en.has(key)) { problems.push(`${code}.yml has ${key}, which en.yml lacks`); continue }
      if (text && en.get(key) && marks(text) !== asIn(code, en.get(key))) problems.push(`${code}.yml ${key}: placeholders "${marks(text)}" but English has "${marks(en.get(key))}"`)
    }
  }
}

// 4. A build with the borrowed words marked: no English on a published language's pages.
if (process.argv.includes('--build') && !problems.length) {
  const { published: langs, pages } = await import(path.join(root, 'eleventy.registry.mjs'))
  const out = mkdtempSync(path.join(tmpdir(), 'site-i18n-'))
  try {
    execFileSync('npx', ['@11ty/eleventy', `--output=${out}`, '--quiet'], { cwd: root, stdio: ['ignore', 'ignore', 'inherit'], env: { ...process.env, I18N_MARK: '1' } })
    for (const l of langs.filter((x) => x.code !== 'en')) {
      const shown = new Map()
      for (const id of l.pages) {
        const file = path.join(out, `${l.prefix}${pages[id]}`, 'index.html')
        if (!existsSync(file)) { problems.push(`${l.code}: its ${id} page was not built (${path.relative(out, file)})`); continue }
        for (const m of readFileSync(file, 'utf8').matchAll(/⁣([a-z0-9_]+)⁢/g)) {
          if (!shown.has(m[1])) shown.set(m[1], new Set())
          shown.get(m[1]).add(id)
        }
      }
      for (const [key, ids] of shown) problems.push(`${l.code} shows ${key} in English (${[...ids].join(', ')}): add it to site/_data/i18n/${l.code}.yml`)
    }
  } finally {
    rmSync(out, { recursive: true, force: true })
  }
}

if (problems.length) {
  for (const p of problems.slice(0, 60)) console.error(`  ✗ ${p}`)
  if (problems.length > 60) console.error(`  … and ${problems.length - 60} more`)
  process.exit(1)
}
console.log(`check-site-i18n: ${codes.length} languages, ${published.length} published (${published.join(', ')}), no language named in a template${process.argv.includes('--build') ? ', no English shown on their pages' : ''}`)
