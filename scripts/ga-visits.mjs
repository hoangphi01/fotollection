// SPDX-License-Identifier: GPL-3.0-only
// Writes the website's total visits (Google Analytics sessions since GA was added) to
// site/_data/visits.json for the footer badge on every page, and the clicks on the site's
// download links by file (the page's `app_download` event) to site/_data/web_downloads.json
// for scripts/download-stats.mjs, at build time in pages.yml, so the page loads nothing for it. No dependencies: a service-account JWT signed with node:crypto, then
// one call to the GA Data API. Without the secrets, or on any error, it writes nothing and the
// footer simply leaves the line out; the site still deploys.
//
//   GA_PROPERTY_ID=123456789 GA_SERVICE_ACCOUNT="$(cat key.json)" node scripts/ga-visits.mjs

import { createSign } from 'node:crypto'
import { writeFileSync } from 'node:fs'
import { languages } from '../eleventy.registry.mjs'

// The day Google Analytics went on the site (commit 1294ce5).
const SINCE = '2026-10-24'
const OUT = new URL('../site/_data/visits.json', import.meta.url)
// The day the download links started sending `app_download` (default.html).
const CLICKS_SINCE = '2026-11-04'
const CLICKS_OUT = new URL('../site/_data/web_downloads.json', import.meta.url)

// Google's error message says which step is missing (API off, no access); it holds no secret.
async function reason(res) {
  try {
    const message = (await res.json())?.error?.message
    return typeof message === 'string' ? `: ${message.slice(0, 300)}` : ''
  } catch {
    return ''
  }
}

function skip(why) {
  console.log(`visits: skipped (${why})`)
  process.exit(0)
}

const property = process.env.GA_PROPERTY_ID?.trim()
const keyJson = process.env.GA_SERVICE_ACCOUNT
if (!property || !keyJson) skip('GA_PROPERTY_ID or GA_SERVICE_ACCOUNT not set')
if (!/^\d{1,20}$/.test(property)) skip('GA_PROPERTY_ID is not a numeric property ID')

let key
try {
  key = JSON.parse(keyJson)
} catch {
  skip('GA_SERVICE_ACCOUNT is not JSON')
}
if (!key.client_email || !key.private_key) skip('the key has no client_email or private_key')

const b64 = (v) => Buffer.from(typeof v === 'string' ? v : JSON.stringify(v)).toString('base64url')

async function accessToken() {
  const now = Math.floor(Date.now() / 1000)
  const claims = {
    iss: key.client_email,
    scope: 'https://www.googleapis.com/auth/analytics.readonly',
    aud: 'https://oauth2.googleapis.com/token',
    iat: now,
    exp: now + 600,
  }
  const unsigned = `${b64({ alg: 'RS256', typ: 'JWT' })}.${b64(claims)}`
  const signature = createSign('RSA-SHA256').update(unsigned).sign(key.private_key, 'base64url')
  const res = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      grant_type: 'urn:ietf:params:oauth:grant-type:jwt-bearer',
      assertion: `${unsigned}.${signature}`,
    }),
    signal: AbortSignal.timeout(15_000),
  })
  if (!res.ok) skip(`token request answered ${res.status}${await reason(res)}`)
  return (await res.json()).access_token
}

async function sessions(token) {
  const res = await fetch(`https://analyticsdata.googleapis.com/v1beta/properties/${property}:runReport`, {
    method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify({ dateRanges: [{ startDate: SINCE, endDate: 'today' }], metrics: [{ name: 'sessions' }] }),
    signal: AbortSignal.timeout(15_000),
  })
  if (!res.ok) skip(`report request answered ${res.status}${await reason(res)}`)
  const value = (await res.json()).rows?.[0]?.metricValues?.[0]?.value ?? '0'
  const n = Number(value)
  if (!Number.isSafeInteger(n) || n < 0) skip('the report did not return a whole number')
  return n
}

// Clicks on the site's download links, by the installer's file name. A failure here leaves
// the visits in place and the download split out.
async function clicks(token) {
  const res = await fetch(`https://analyticsdata.googleapis.com/v1beta/properties/${property}:runReport`, {
    method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify({
      dateRanges: [{ startDate: CLICKS_SINCE, endDate: 'today' }],
      dimensions: [{ name: 'linkUrl' }],
      metrics: [{ name: 'eventCount' }],
      dimensionFilter: { filter: { fieldName: 'eventName', stringFilter: { value: 'app_download' } } },
      limit: 1000,
    }),
    signal: AbortSignal.timeout(15_000),
  })
  if (!res.ok) throw new Error(`clicks report answered ${res.status}${await reason(res)}`)
  const files = {}
  for (const row of (await res.json()).rows ?? []) {
    const file = String(row.dimensionValues?.[0]?.value ?? '').split('/').pop() ?? ''
    const n = Number(row.metricValues?.[0]?.value ?? 0)
    if (/^Fotollection_[\w.-]+$/.test(file) && Number.isSafeInteger(n) && n > 0) files[file] = (files[file] ?? 0) + n
  }
  return files
}

try {
  const token = await accessToken()
  const total = await sessions(token)
  const visits = {
    total,
    // The count as each language of the website writes it (site/_data/languages.yml).
    ...Object.fromEntries(languages.map((l) => [l.code, total.toLocaleString(l.html_lang)])),
    since: SINCE,
    updated: new Date().toISOString().slice(0, 10),
  }
  writeFileSync(OUT, `${JSON.stringify(visits, null, 2)}\n`)
  console.log(`visits: ${total} sessions since ${SINCE}`)
  try {
    const files = await clicks(token)
    writeFileSync(CLICKS_OUT, `${JSON.stringify({ since: CLICKS_SINCE, files }, null, 2)}\n`)
    console.log(`web downloads: ${Object.values(files).reduce((a, b) => a + b, 0)} clicks since ${CLICKS_SINCE}`)
  } catch (e) {
    console.log(`web downloads: skipped (${e instanceof Error ? e.message : 'request failed'})`)
  }
} catch (e) {
  skip(e?.name === 'TimeoutError' ? 'timed out' : 'request failed')
}
