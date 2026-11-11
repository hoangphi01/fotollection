// SPDX-License-Identifier: GPL-3.0-only
// Writes how often the installers were downloaded to site/_data/downloads.json, at build time
// in pages.yml, for the footer badge and the Download page; the page loads nothing for it.
// GitHub counts every download of a release file: from the site's buttons, from the app's
// update check (FR-061, which opens the new installer in the browser) and from install.sh.
// The clicks on the site's own links (scripts/ga-visits.mjs, site/_data/web_downloads.json)
// split the latest release: what is left of its macOS and Windows downloads came through the
// app's update, an estimate. The APT repository is served by GitHub Pages and is not counted.
// No dependencies. Without a token, or on any error, it writes nothing and the page leaves
// the lines out; the site still deploys.
//
//   GH_TOKEN=… node scripts/download-stats.mjs

import { existsSync, readFileSync, writeFileSync } from 'node:fs'
import { languages } from '../eleventy.registry.mjs'

const REPO = process.env.GITHUB_REPOSITORY?.trim() || 'hoangphi01/fotollection'
const OUT = new URL('../site/_data/downloads.json', import.meta.url)
const CLICKS = new URL('../site/_data/web_downloads.json', import.meta.url)

function skip(why) {
  console.log(`downloads: skipped (${why})`)
  process.exit(0)
}

/** The system a release file installs on; `null` for the checksums and anything else. */
function systemOf(name) {
  if (/\.dmg$/i.test(name)) return 'mac'
  if (/\.(exe|msi)$/i.test(name)) return 'win'
  if (/\.(AppImage|deb)$/i.test(name)) return 'linux'
  return null
}

async function releases(token) {
  const all = []
  for (let page = 1; page <= 20; page++) {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100&page=${page}`, {
      headers: { authorization: `Bearer ${token}`, accept: 'application/vnd.github+json' },
      signal: AbortSignal.timeout(15_000),
    })
    if (!res.ok) skip(`GitHub answered ${res.status}`)
    const batch = await res.json()
    if (!Array.isArray(batch)) skip('GitHub did not return a list')
    all.push(...batch)
    if (batch.length < 100) break
  }
  return all
}

function readClicks() {
  if (!existsSync(CLICKS)) return null
  try {
    const data = JSON.parse(readFileSync(CLICKS, 'utf8'))
    return data && typeof data.files === 'object' ? data : null
  } catch {
    return null
  }
}

const token = process.env.GH_TOKEN?.trim()
if (!token) skip('GH_TOKEN not set')

try {
  const list = (await releases(token)).filter((r) => !r.draft)
  const totals = { mac: 0, win: 0, linux: 0 }
  const perRelease = list.map((r) => {
    const counts = { mac: 0, win: 0, linux: 0 }
    const files = {}
    for (const asset of r.assets ?? []) {
      const system = systemOf(String(asset.name))
      const n = Number(asset.download_count)
      if (!system || !Number.isSafeInteger(n) || n < 0) continue
      counts[system] += n
      files[asset.name] = { system, count: n }
    }
    for (const k of Object.keys(totals)) totals[k] += counts[k]
    return { tag: r.tag_name, published: String(r.published_at ?? '').slice(0, 10), prerelease: !!r.prerelease, ...counts, files }
  })
  const total = totals.mac + totals.win + totals.linux
  const fmt = (n, locale) => n.toLocaleString(locale)
  const downloads = {
    total,
    ...totals,
    // The count as each language of the website writes it (site/_data/languages.yml).
    ...Object.fromEntries(languages.map((l) => [l.code, fmt(total, l.html_lang)])),
    updated: new Date().toISOString().slice(0, 10),
  }
  // The latest release, published after the clicks were first counted, split by where its macOS and
  // Windows downloads came from; Linux is left out, as install.sh also fetches from GitHub.
  const clicks = readClicks()
  const latest = perRelease.find((r) => !r.prerelease)
  // Only a release published after the day clicks were first counted: on that day the clicks
  // before the tracking went live are missing.
  if (clicks && latest && latest.published > clicks.since) {
    let site = 0
    let all = 0
    for (const [name, { system, count }] of Object.entries(latest.files)) {
      if (system === 'linux') continue
      all += count
      site += Math.min(count, Number(clicks.files[name] ?? 0))
    }
    downloads.latest = { tag: latest.tag, site, updates: Math.max(0, all - site) }
  }
  writeFileSync(OUT, `${JSON.stringify(downloads, null, 2)}\n`)
  console.log(`downloads: ${total} (macOS ${totals.mac}, Windows ${totals.win}, Linux ${totals.linux})`)
} catch (e) {
  skip(e?.name === 'TimeoutError' ? 'timed out' : 'request failed')
}
