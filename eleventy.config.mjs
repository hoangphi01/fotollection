// SPDX-License-Identifier: GPL-3.0-only
// The website (site/, site/README.md), built by Eleventy (MIT) and served by GitHub Pages
// (.github/workflows/pages.yml). The templates are the Jekyll site's, in Liquid with Jekyll's
// includes (`{% include file.html key=value %}`), so they read as they did; `site.*` is the
// same object Jekyll gave them: _config.yml's keys, every file of _data under `site.data`, and
// `site.github.latest_release` from `_data/release.json` (written by the workflow).
import { readFileSync, readdirSync, existsSync, mkdirSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import yaml from 'js-yaml'
import MarkdownIt from 'markdown-it'
import attrs from 'markdown-it-attrs'
import anchor from 'markdown-it-anchor'
import { homes, languages, pages, published, urlFor } from './eleventy.registry.mjs'

const SITE = 'site'

function siteData() {
  const config = yaml.load(readFileSync(path.join(SITE, '_config.yml'), 'utf8'))
  const data = {}
  const dir = path.join(SITE, '_data')
  for (const file of readdirSync(dir)) {
    const name = file.replace(/\.(ya?ml|json)$/, '')
    if (name === file) continue
    const text = readFileSync(path.join(dir, file), 'utf8')
    data[name] = file.endsWith('.json') ? JSON.parse(text) : yaml.load(text)
  }
  // The words, one file per language (i18n/<code>.yml). English is the source and the
  // fallback: a key another language lacks shows in English. With I18N_MARK=1 (the check build of
  // scripts/check-site-i18n.mjs) each borrowed text carries its key between invisible marks, so
  // the check finds every English word a language's pages really show.
  const words = path.join(dir, 'i18n')
  const lang = (code) => {
    const file = path.join(words, `${code}.yml`)
    return existsSync(file) ? (yaml.load(readFileSync(file, 'utf8')) ?? {}) : {}
  }
  const mark = process.env.I18N_MARK === '1'
  // Fields of a list that are names, not words (an icon's file, a logo, an anchor, a command)
  // stay as they are: they are the same in every language.
  const names = new Set(['icon', 'logo', 'id', 'code', 'code2'])
  const marked = (key, value) =>
    typeof value === 'string' ? `\u2063${key}\u2062${value}\u2064`
      : Array.isArray(value) ? value.map((v) => marked(key, v))
        : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k, v]) => [k, names.has(k) ? v : marked(key, v)]))
          : value
  const en = lang('en')
  data.i18n = { en }
  for (const { code } of languages) {
    if (code === 'en') continue
    const own = lang(code)
    const borrowed = Object.fromEntries(Object.entries(en).filter(([k]) => !(k in own)).map(([k, v]) => [k, mark ? marked(k, v) : v]))
    data.i18n[code] = { ...borrowed, ...own }
  }
  // The check build renders every branch the workflow's data opens (LESSONS): what only CI has
  // (the visit count, the latest release's split of downloads) is stood in for when absent.
  if (mark) {
    const count = Object.fromEntries(languages.map((l) => [l.code, '1']))
    data.visits ??= { total: 1, ...count }
    data.downloads ??= { total: 1, mac: 1, win: 1, linux: 1, ...count }
    data.downloads.latest ??= { tag: 'v0.0.0', site: '1', updates: '1' }
  }
  const release = data.release ?? null
  return { ...config, time: new Date(), data, github: { latest_release: release } }
}

export default function (eleventyConfig) {
  eleventyConfig.setLiquidOptions({ jekyllInclude: true, dynamicPartials: false, strictFilters: true })
  eleventyConfig.addGlobalData('site', siteData)
  eleventyConfig.addGlobalData('jekyll', { environment: process.env.JEKYLL_ENV ?? 'development' })
  // _data is read once above, as Jekyll did; Eleventy's own data folder is unused.
  eleventyConfig.addDataExtension('yml,yaml', (contents) => yaml.load(contents))

  const url = (value) => `${value ?? ''}`
  eleventyConfig.addFilter('relative_url', url)
  eleventyConfig.addFilter('absolute_url', (value) => {
    const v = `${value ?? ''}`
    return /^https?:/.test(v) ? v : `https://fotollection.org${v.startsWith('/') ? '' : '/'}${v}`
  })
  eleventyConfig.addFilter('jsonify', (value) => JSON.stringify(value))
  // `{{ l.name | tr: lang }}`: a text kept per language in a data file (looks.yml, timeline.yml),
  // English when that language lacks it; marked as borrowed in the check build (see siteData).
  eleventyConfig.addFilter('tr', (value, code) => {
    if (!value || typeof value !== 'object') return value
    if (code in value) return value[code]
    return process.env.I18N_MARK === '1' && typeof value.en === 'string' ? `\u2063data\u2062${value.en}\u2064` : value.en
  })
  // `{{ 'manual' | url_for: pg.lang }}`: a page's path in a language (eleventy.registry.mjs).
  eleventyConfig.addFilter('url_for', (id, code) => urlFor(id, code))
  // `markdownify` as Jekyll's kramdown did for these pages: raw HTML kept, `{#id}` after a
  // title (the manual's chapter anchors) through markdown-it-attrs.
  // kramdown's GFM header ids: lower case, everything but letters, digits, `_`, `-` and spaces
  // dropped, spaces as hyphens, a repeat numbered; typographic quotes and dashes as kramdown's.
  const gfmId = (text) =>
    text.toLowerCase().replace(/[^\p{L}\p{M}\p{N}_\- ]/gu, '').replace(/ /g, '-')
  const md = new MarkdownIt({ html: true, xhtmlOut: true, typographer: true })
    .use(attrs, { allowedAttributes: ['id', 'class'] })
    .use(anchor, { slugify: gfmId, tabIndex: false, uniqueSlugStartIndex: 1 })
  // kramdown wraps a list item's first paragraph in <p> only when a blank line follows it and
  // another item comes after; markdown-it decides for the whole list. Each item follows kramdown.
  md.core.ruler.push('kramdown_items', (state) => {
    const lines = state.src.split('\n')
    const t = state.tokens
    for (let i = 0; i < t.length; i++) {
      if (t[i].type !== 'list_item_open') continue
      const open = t[i + 1]
      if (!open || open.type !== 'paragraph_open' || !open.map) continue
      const after = lines[open.map[1]]
      const blank = after === undefined || after.trim() === ''
      // The item's end, and whether another item of the same list follows it.
      let depth = 0
      let last = true
      for (let j = i + 1; j < t.length; j++) {
        if (t[j].type === 'list_item_open') depth++
        if (t[j].type === 'list_item_close') {
          if (depth === 0) { last = !t[j + 1] || t[j + 1].type !== 'list_item_open'; break }
          depth--
        }
      }
      const wrap = blank && !last
      open.hidden = !wrap
      for (let j = i + 2; j < t.length; j++) {
        if (t[j].type === 'paragraph_close' && t[j].level === open.level) { t[j].hidden = !wrap; break }
      }
    }
  })
  // sitemap.xml (robots.txt names it): every page in every published language, each with the
  // other languages it is in (hreflang) and English as x-default, as the pages' own <head> says.
  // `directories.output` is the folder really written (`--output` included); `dir.output` is the
  // configured one (LESSONS).
  eleventyConfig.on('eleventy.after', ({ dir, directories }) => {
    const out = directories?.output ?? dir.output
    mkdirSync(out, { recursive: true })
    const abs = (p) => `https://fotollection.org${p}`
    const urls = []
    for (const id of Object.keys(pages)) {
      const versions = published.filter((l) => l.pages.includes(id))
      const links = versions.map((l) => `    <xhtml:link rel="alternate" hreflang="${l.html_lang}" href="${abs(urlFor(id, l.code))}"/>`)
      links.push(`    <xhtml:link rel="alternate" hreflang="x-default" href="${abs(urlFor(id, 'en'))}"/>`)
      for (const l of versions) urls.push(`  <url>\n    <loc>${abs(urlFor(id, l.code))}</loc>\n${links.join('\n')}\n  </url>`)
    }
    writeFileSync(path.join(out, 'sitemap.xml'), `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n${urls.join('\n')}\n</urlset>\n`)
    // /hi, the short link for QR codes, ads and social profiles (site/README.md, owner
    // 2026-11-11): no page to click through, it opens at once the home page in the visitor's
    // language (the menu's choice, else the browser's) and turns ?from=<source> into utm_source,
    // so Google Analytics counts where visits came from, anonymously. Never indexed.
    const homesByCode = JSON.stringify(homes())
    mkdirSync(path.join(out, 'hi'), { recursive: true })
    writeFileSync(path.join(out, 'hi', 'index.html'), `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="robots" content="noindex" />
<link rel="canonical" href="https://fotollection.org/" />
<meta http-equiv="refresh" content="0; url=/?utm_source=hi&amp;utm_medium=hi" />
<title>Fotollection</title>
<script>
;(function () {
  var homes = ${homesByCode}
  var pick = null
  try { pick = localStorage.getItem('lang') } catch (e) {}
  if (!homes[pick]) {
    pick = 'en'
    var wanted = navigator.languages || [navigator.language || '']
    for (var i = 0; i < wanted.length; i++) {
      var code = String(wanted[i]).toLowerCase().split('-')[0]
      if (homes[code]) { pick = code; break }
    }
  }
  var q = new URLSearchParams(location.search)
  var from = (q.get('from') || 'hi').replace(/[^a-z0-9_-]/gi, '').slice(0, 40) || 'hi'
  q.delete('from')
  if (!q.has('utm_source')) q.set('utm_source', from)
  if (!q.has('utm_medium')) q.set('utm_medium', 'hi')
  location.replace(homes[pick] + '?' + q.toString() + location.hash)
})()
</script>
</head>
<body><p><a href="/">Fotollection</a></p></body>
</html>
`)
  })
  eleventyConfig.setLibrary('md', md)
  eleventyConfig.addFilter('markdownify', (value) => md.render(`${value ?? ''}`))

  // `.well-known/`: Flathub's token proving the project owns fotollection.org
  // (org.flathub.VerifiedApps.txt, the owner's to add once Flathub gives it).
  for (const p of ['assets', 'install.sh', 'CNAME', 'robots.txt', '.well-known']) {
    if (existsSync(path.join(SITE, p))) eleventyConfig.addPassthroughCopy(path.join(SITE, p))
  }
  eleventyConfig.ignores.add(`${SITE}/README.md`)
  eleventyConfig.ignores.add(`${SITE}/screenshots/**`)
  eleventyConfig.ignores.add(`${SITE}/assets/**`)

  return {
    dir: { input: SITE, output: '_site', includes: '_includes', layouts: '_layouts', data: '_eleventy_data' },
    htmlTemplateEngine: 'liquid',
    markdownTemplateEngine: 'liquid',
    templateFormats: ['html', 'liquid'],
  }
}
