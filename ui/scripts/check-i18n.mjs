// SPDX-License-Identifier: GPL-3.0-only
// FR-020 gate, run by `npm run lint`:
//  1. every locale has the same keys as en.json (plural suffixes normalized), none empty;
//  2. no hard-coded user-visible text in components (JSX text or title / aria-label /
//     placeholder / label / alt attributes), outside the shadcn primitives in components/ui.
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const root = new URL('../src', import.meta.url).pathname
const locales = join(root, 'locales')
const PLURAL = /_(zero|one|two|few|many|other)$/

function flatten(obj, prefix = '', out = new Map()) {
  for (const [k, v] of Object.entries(obj)) {
    const key = prefix ? `${prefix}.${k}` : k
    if (v && typeof v === 'object') flatten(v, key, out)
    else out.set(key, v)
  }
  return out
}
const base = (keys) => new Set([...keys].map((k) => k.replace(PLURAL, '')))

const problems = []
const en = flatten(JSON.parse(readFileSync(join(locales, 'en.json'), 'utf8')))
const enBase = base(en.keys())
for (const file of readdirSync(locales).filter((f) => f.endsWith('.json') && f !== 'en.json')) {
  const other = flatten(JSON.parse(readFileSync(join(locales, file), 'utf8')))
  const otherBase = base(other.keys())
  for (const k of enBase) if (!otherBase.has(k)) problems.push(`${file}: missing key ${k}`)
  for (const k of otherBase) if (!enBase.has(k)) problems.push(`${file}: key ${k} is not in en.json`)
  for (const [k, v] of other) if (typeof v !== 'string' || v.trim() === '') problems.push(`${file}: empty value for ${k}`)
}
for (const [k, v] of en) if (typeof v !== 'string' || v.trim() === '') problems.push(`en.json: empty value for ${k}`)

// Words that are the same in every language: names, units, formats.
const ALLOWED = new Set(['Fotollection', 'DPI', 'MP', 'px', 'ISO', 'JPEG', 'PNG', 'EXIF', 'GPS', 'Esc', 'Enter', 'cm', 'in', 'KB', 'MB', 'GB'])
const visible = (text) =>
  (text.match(/[\p{L}]{2,}/gu) ?? []).some((word) => !ALLOWED.has(word))

function walk(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) return name === 'ui' && dir.endsWith('components') ? [] : walk(path)
    return path.endsWith('.tsx') ? [path] : []
  })
}
for (const file of walk(root)) {
  const src = readFileSync(file, 'utf8')
    // Comments are not user-visible.
    .replace(/\{\/\*[\s\S]*?\*\/\}/g, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/^\s*\/\/.*$/gm, '')
  const rel = relative(root, file)
  for (const m of src.matchAll(/>([^<>{}]*)</g)) {
    // Between a `>` and a `<` there is also TypeScript (arrows, comparisons, generics);
    // JSX text never contains these.
    if (/[=;[\]`]|&&|\|\||\breturn\b|\)\s*:|\?\s*\(/.test(m[1])) continue
    if (visible(m[1])) problems.push(`${rel}: hard-coded text "${m[1].trim()}"`)
  }
  for (const m of src.matchAll(/\b(title|aria-label|placeholder|label|alt)="([^"]*)"/g)) {
    if (visible(m[2])) problems.push(`${rel}: hard-coded ${m[1]}="${m[2]}"`)
  }
}

if (problems.length) {
  for (const p of problems) console.error(`  ✗ ${p}`)
  console.error(`check-i18n: ${problems.length} problem(s)`)
  process.exit(1)
}
console.log(`check-i18n: ${enBase.size} keys in every locale, no hard-coded text`)
