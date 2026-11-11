// SPDX-License-Identifier: GPL-3.0-only
// The AppStream metadata's release list (software centres and Flathub show it), made from the
// CHANGELOG so the two never drift:
//
//   node scripts/metainfo-releases.mjs          rewrites <releases> in the .deb's metainfo
//   node scripts/metainfo-releases.mjs --check  fails when it is out of date (check-docs.sh)
//
// The ten newest released versions, newest first, each with its date and its Highlights as a
// list (Flathub's quality guidelines ask for release notes a user reads). Markdown is reduced to
// plain text and XML is escaped; the tags are AppStream 0.15's, which Ubuntu 22.04 validates
// (LESSONS). scripts/release.sh runs it after the CHANGELOG has the new version, then
// scripts/flatpak-files.mjs copies the list into the Flatpak's metainfo.
import { readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const file = path.join(root, 'src-tauri/linux/com.hoangphi01.fotollection.metainfo.xml')
const KEEP = 10

const escape = (s) => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
const plain = (s) =>
  s
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1')
    .replace(/`([^`]+)`/g, '$1')
    .trim()

// Each released version: its number, date and Highlights, in the CHANGELOG's order (newest first).
const releases = []
let current = null
let inHighlights = false
for (const line of readFileSync(path.join(root, 'CHANGELOG.md'), 'utf8').split('\n')) {
  const head = line.match(/^## \[(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})/)
  if (head) {
    current = { version: head[1], date: head[2], points: [] }
    releases.push(current)
    inHighlights = false
    continue
  }
  if (line.startsWith('## ')) {
    current = null
    continue
  }
  if (line.startsWith('### ')) {
    inHighlights = current !== null && line === '### Highlights'
    continue
  }
  if (inHighlights && line.startsWith('- ')) current.points.push(plain(line.slice(2)))
}
const kept = releases.slice(0, KEEP)
const missing = kept.filter((r) => r.points.length === 0).map((r) => r.version)
if (missing.length) {
  console.error(`  ✗ metainfo-releases: no Highlights in the CHANGELOG for ${missing.join(', ')}`)
  process.exit(1)
}

const block = [
  '  <releases>',
  ...kept.flatMap((r) => [
    `    <release version="${r.version}" date="${r.date}">`,
    '      <description>',
    '        <ul>',
    ...r.points.map((p) => `          <li>${escape(p)}</li>`),
    '        </ul>',
    '      </description>',
    '    </release>',
  ]),
  '  </releases>',
].join('\n')

const source = readFileSync(file, 'utf8')
const start = source.indexOf('  <releases>')
const end = source.indexOf('</releases>')
if (start < 0 || end < 0) {
  console.error('  ✗ metainfo-releases: no <releases> in the metainfo')
  process.exit(1)
}
const next = source.slice(0, start) + block + source.slice(end + '</releases>'.length)

if (process.argv.includes('--check')) {
  if (next !== source) {
    console.error('  ✗ the metainfo\'s releases differ from the CHANGELOG: run node scripts/metainfo-releases.mjs')
    process.exit(1)
  }
  console.log(`metainfo-releases: ${kept.length} releases, up to date`)
} else {
  writeFileSync(file, next)
  console.log(`metainfo-releases: wrote ${kept.length} releases into ${path.relative(root, file)}`)
}
