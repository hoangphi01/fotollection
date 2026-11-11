// SPDX-License-Identifier: GPL-3.0-only
// Checks the website's languages (site/README.md, ADR-0049). Without arguments (check-docs.sh,
// no packages needed) it fails when:
//   - a template picks a language by name (`== 'vi'`, a `/vi/` path, `x[lang]` on data) instead
//     of the registry, url_for and tr, or defaults a filter's argument (`, x | default: y`);
//   - a words file is not a language of site/_data/languages.yml, or a published language has none;
//   - a language has a key English lacks (a typo), or a text lost or gained a placeholder
//     ([n], [tag]…) English has.
// With --build (the pages workflow, and check-docs.sh when the packages are installed) it also
// builds the site with every borrowed English text marked (I18N_MARK=1, eleventy.config.mjs) and
// fails when a published language's pages show any English, naming the keys and the pages.
import { execFileSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const site = path.join(root, 'site')
const problems = []

// 1. No language named in a template.
const banned = [
  [/==\s*['"](vi|en|hu|de|pl|cs)['"]/, 'compares the language with a name; use pg.* from the registry'],
  [/['"]\/(vi|hu|de|pl|cs)\//, 'writes a language\'s path; use url_for'],
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
if (!existsSync(registry)) problems.push('site/_data/languages.yml is missing')
else {
  let code = null
  for (const line of readFileSync(registry, 'utf8').split('\n')) {
    const c = line.match(/^- code: (\w+)/)
    if (c) codes.push((code = c[1]))
    if (/^ {2}published: true\b/.test(line) && code) published.push(code)
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
const marks = (text) => [...text.matchAll(/\[[a-z]+\]/g)].map((m) => m[0]).sort().join(' ')
if (files.includes('en')) {
  const en = read('en')
  for (const code of files.filter((f) => f !== 'en')) {
    for (const [key, text] of read(code)) {
      if (!en.has(key)) { problems.push(`${code}.yml has ${key}, which en.yml lacks`); continue }
      if (text && en.get(key) && marks(text) !== marks(en.get(key))) problems.push(`${code}.yml ${key}: placeholders "${marks(text)}" but English has "${marks(en.get(key))}"`)
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
