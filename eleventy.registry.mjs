// SPDX-License-Identifier: GPL-3.0-only
// The website's languages and pages (site/_data/languages.yml, site/_data/pages.yml), read once
// for the build (eleventy.config.mjs, site/site.11tydata.mjs) and the scripts that loop over
// languages, so that no file names a language on its own.
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import yaml from 'js-yaml'

const data = path.join(path.dirname(fileURLToPath(import.meta.url)), 'site', '_data')
const load = (name) => yaml.load(readFileSync(path.join(data, name), 'utf8'))

export const languages = load('languages.yml')
export const pages = load('pages.yml')
export const published = languages.filter((l) => l.published)

/** The path of page `id` in language `code`; English's when that language lacks the page. */
export function urlFor(id, code) {
  if (!(id in pages)) throw new Error(`url_for: no page "${id}" in site/_data/pages.yml`)
  const language = published.find((l) => l.code === code && l.pages.includes(id))
  return `${language ? language.prefix : ''}${pages[id]}`
}
