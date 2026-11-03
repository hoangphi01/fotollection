// SPDX-License-Identifier: GPL-3.0-only
// Folds bench/trip.sh's runs into bench/results/trip.json: the median of each number, and
// the median RAW develop and embedded preview from measure_raw's table.
//
//   node bench/trip.mjs <run-dir> <runs> <jpegs> <raws>
import fs from 'node:fs'
import path from 'node:path'

const [dir, runs, jpegs, raws] = process.argv.slice(2)
const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b)
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2
}
const all = Array.from({ length: Number(runs) }, (_, i) =>
  JSON.parse(fs.readFileSync(path.join(dir, `run-${i + 1}.json`), 'utf8')),
)
const trip = { jpegs: Number(jpegs), raws: Number(raws), runs: all.length }
for (const [key, value] of Object.entries(all[0])) {
  if (typeof value === 'number') trip[key] = median(all.map((r) => r[key]))
}
// measure_raw rows: | File | Developed | Header | Full develop | Preview ≤ 400 | Preview ≤ 2048 | EXIF |
const ms = (cell) => Number(/([\d.]+) ms/.exec(cell)?.[1] ?? NaN)
const rows = fs
  .readFileSync(path.join(dir, 'raw.md'), 'utf8')
  .split('\n')
  .filter((l) => l.startsWith('| ') && / ms /.test(l))
  .map((l) => l.split('|').map((c) => c.trim()))
if (rows.length) {
  trip.rawDevelopMs = median(rows.map((r) => ms(r[4])))
  trip.rawPreview400Ms = median(rows.map((r) => ms(r[5])))
  trip.rawPreview2048Ms = median(rows.map((r) => ms(r[6])))
}
trip.date = new Date().toISOString().slice(0, 10)
console.log(JSON.stringify(trip, null, 2))
