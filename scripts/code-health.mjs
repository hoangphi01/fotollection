// SPDX-License-Identifier: GPL-3.0-only
// Code health in numbers (docs/CODE_HEALTH.md, STYLE_RULE §1 "Quality is a number"): the
// length of every source file git tracks, split into production lines and test lines, against
// the thresholds the owner set on 2026-11-17, and against the watch list in
// scripts/code-health.json, so a file grows past a threshold only with a reason written down.
//
//   node scripts/code-health.mjs            # check, then print the summary
//   node scripts/code-health.mjs --list     # also every file over 800 lines, largest first
//
// Production lines: a Rust file's lines before its `#[cfg(test)]`; every line of other code.
// Test lines: the rest, and every line of a file under tests/, examples/, benches/ or fuzz/.
// Levels: green up to 800 production lines; orange above 800, or above 1,500 lines in all
// with the tests; red above 1,500 production lines. Exits non-zero on any problem.
import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'

const ORANGE = 800
const RED = 1500
const TOTAL_ORANGE = 1500
const GROWTH = 1.1 // an orange file may grow 10 % past its recorded size, a red one not at all
const EXTENSIONS = /\.(rs|ts|tsx|mjs|py|sh)$/
const GENERATED = new Set(['ui/src/ipc/bindings.ts'])
const TEST_PATH = /(^|\/)(tests|examples|benches|fuzz)\//

const root = new URL('..', import.meta.url).pathname
const watch = JSON.parse(readFileSync(`${root}scripts/code-health.json`, 'utf8'))
const files = execFileSync('git', ['ls-files'], { cwd: root, encoding: 'utf8' })
  .split('\n')
  .filter((f) => EXTENSIONS.test(f) && !GENERATED.has(f))

let allows = 0
const measured = files.map((path) => {
  const lines = readFileSync(`${root}${path}`, 'utf8').split('\n')
  if (lines.at(-1) === '') lines.pop()
  if (path.endsWith('.rs')) allows += lines.filter((l) => /clippy::too_many_lines/.test(l) && !/^\s*\/\//.test(l)).length
  const total = lines.length
  let production = total
  if (TEST_PATH.test(path)) production = 0
  else if (path.endsWith('.rs')) {
    const at = lines.findIndex((l) => /^#\[cfg\(test\)\]/.test(l))
    if (at >= 0) production = at
  }
  const level = production > RED ? 'red' : production > ORANGE || total > TOTAL_ORANGE ? 'orange' : 'green'
  return { path, production, tests: total - production, total, level }
})

const problems = []
for (const f of measured) {
  const w = watch.files[f.path]
  if (f.level === 'green') {
    if (w) problems.push(`${f.path} is green now (${f.production} lines): take it off scripts/code-health.json`)
    continue
  }
  if (!w) {
    problems.push(`${f.path} is ${f.level} (${f.production} production, ${f.total} in all) and not on the watch list: analyse it (docs/CODE_HEALTH.md, the checklist) and add it to scripts/code-health.json`)
    continue
  }
  if (w.level === 'orange' && f.level === 'red') problems.push(`${f.path} turned red (${f.production} production lines > ${RED}): split it before it grows`)
  const limit = (n) => (w.level === 'red' ? n : Math.floor(n * GROWTH))
  if (f.production > limit(w.production)) problems.push(`${f.path} grew to ${f.production} production lines (recorded ${w.production}, limit ${limit(w.production)})`)
  if (f.total > limit(w.total)) problems.push(`${f.path} grew to ${f.total} lines in all (recorded ${w.total}, limit ${limit(w.total)})`)
}
for (const path of Object.keys(watch.files)) {
  if (!measured.some((f) => f.path === path)) problems.push(`${path} is on the watch list but no longer exists: take it off scripts/code-health.json`)
}
if (allows > watch.allowTooManyLines) problems.push(`${allows} #[allow(clippy::too_many_lines)] (budget ${watch.allowTooManyLines}, which only goes down): split the function instead`)

const production = measured.reduce((s, f) => s + f.production, 0)
const greenShare = measured.filter((f) => f.level === 'green').reduce((s, f) => s + f.production, 0) / production
const count = (l) => measured.filter((f) => f.level === l).length
const largest = measured.reduce((a, b) => (b.production > a.production ? b : a))
console.log(
  `code-health: ${measured.length} files, ${production} production lines, ${(100 * greenShare).toFixed(1)} % of them in green files; `
    + `${count('orange')} orange, ${count('red')} red; ${allows} too_many_lines allows (budget ${watch.allowTooManyLines}); `
    + `largest ${largest.path} (${largest.production})`,
)
if (process.argv.includes('--list')) {
  for (const f of measured.filter((m) => m.level !== 'green').sort((a, b) => b.production - a.production)) {
    console.log(`  ${f.level.padEnd(6)} ${String(f.production).padStart(5)} production ${String(f.tests).padStart(5)} tests  ${f.path}`)
  }
}
for (const p of problems) console.log(`  ✗ ${p}`)
process.exit(problems.length ? 1 : 0)
