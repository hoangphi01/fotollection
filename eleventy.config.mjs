// SPDX-License-Identifier: GPL-3.0-only
// The website (site/, site/README.md), built by Eleventy (MIT) and served by GitHub Pages
// (.github/workflows/pages.yml). The templates are the Jekyll site's, in Liquid with Jekyll's
// includes (`{% include file.html key=value %}`), so they read as they did; `site.*` is the
// same object Jekyll gave them: _config.yml's keys, every file of _data under `site.data`, and
// `site.github.latest_release` from `_data/release.json` (written by the workflow).
import { readFileSync, readdirSync, existsSync } from 'node:fs'
import path from 'node:path'
import yaml from 'js-yaml'
import MarkdownIt from 'markdown-it'
import attrs from 'markdown-it-attrs'
import anchor from 'markdown-it-anchor'

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
  eleventyConfig.setLibrary('md', md)
  eleventyConfig.addFilter('markdownify', (value) => md.render(`${value ?? ''}`))

  for (const p of ['assets', 'install.sh', 'CNAME']) {
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
