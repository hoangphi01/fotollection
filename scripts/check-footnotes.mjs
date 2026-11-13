// SPDX-License-Identifier: GPL-3.0-only
// Checks the website's footnotes (site/README.md, owner 2026-11-07: per page, as Apple does).
// Each page lists its notes in site/_data/footnotes.yml, per language; site/_includes/fn.html
// numbers a mark by its place there. This fails when a listed note has no text in English or
// Vietnamese, when a list names a note twice, when a note is cited in the site but listed on
// no page, or listed but cited nowhere. The order itself is checked by rendering (site/README.md).
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const site = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../site')
const problems = []

// footnotes.yml: "page:" then "  en: [a, b]" lines.
const lists = []
let page = null
for (const line of readFileSync(path.join(site, '_data/footnotes.yml'), 'utf8').split('\n')) {
  const head = line.match(/^(\w+):\s*$/)
  if (head) page = head[1]
  const list = line.match(/^ {2}([a-z]{2}): \[(.*)\]\s*$/)
  if (list) lists.push({ page, lang: list[1], keys: list[2].split(',').map((k) => k.trim()).filter(Boolean) })
}
if (!lists.length) problems.push('site/_data/footnotes.yml lists no notes')

// The text of every note, per language (top-level keys of i18n/<code>.yml).
// English and Vietnamese always; a language with a list of its own (Japanese's home page, its
// own stores, owner 2026-11-14) is checked in its own words too, once its file exists.
const own = [...new Set(lists.map((l) => l.lang))].filter((l) => !['en', 'vi'].includes(l))
const wordsFile = (l) => path.join(site, `_data/i18n/${l}.yml`)
const words = Object.fromEntries(['en', 'vi', ...own].filter((l) => l === 'en' || l === 'vi' || existsSync(wordsFile(l))).map((l) => [l, readFileSync(wordsFile(l), 'utf8')]))
const has = (lang, key) => new RegExp(`^${key}: \\S`, 'm').test(words[lang])

const listed = new Set()
for (const { page, lang, keys } of lists) {
  const seen = new Set()
  for (const key of keys) {
    if (seen.has(key)) problems.push(`${page} (${lang}) lists ${key} twice`)
    seen.add(key)
    listed.add(key)
    if (!has('en', key)) problems.push(`${key}, listed for ${page}, has no text in English`)
    else if (['en', 'vi'].includes(lang) && !has('vi', key)) problems.push(`${key}, listed for ${page}, has no text in Vietnamese`)
    if (words[lang] && !['en', 'vi'].includes(lang) && !has(lang, key)) problems.push(`${key}, listed for ${page} (${lang}), has no text in ${lang}.yml`)
  }
}

// Every note cited in the site's templates and data: keys="…" and fn lists.
const cited = new Set()
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const file = path.join(dir, name)
    if (statSync(file).isDirectory()) {
      if (!['assets', 'screenshots', '_site'].includes(name)) walk(file)
    } else if (/\.(html|md|yml)$/.test(name) && name !== 'footnotes.yml') {
      const text = readFileSync(file, 'utf8')
      for (const m of text.matchAll(/keys="([^"]+)"/g)) m[1].split(',').forEach((k) => cited.add(k.trim()))
      // YAML lets a list's names be quoted (`fn: ["fn_raw"]`); the name is the same.
      for (const m of text.matchAll(/\b\w*fn: \[([^\]]+)\]/g)) m[1].split(',').forEach((k) => cited.add(k.trim().replace(/^["']|["']$/g, '')))
    }
  }
}
walk(site)
for (const key of cited) if (!listed.has(key)) problems.push(`${key} is cited but listed on no page in site/_data/footnotes.yml`)
for (const key of listed) if (!cited.has(key)) problems.push(`${key} is listed in site/_data/footnotes.yml but cited nowhere`)

if (problems.length) {
  for (const p of problems) console.error(`  ✗ ${p}`)
  process.exit(1)
}
console.log(`check-footnotes: ${listed.size} notes, ${lists.length} page lists, all cited and written in both languages`)
