// SPDX-License-Identifier: GPL-3.0-only
// Every page is one template, built once per language that has it (site/README.md): the
// languages and their pages are in _data/languages.yml, each page's path in _data/pages.yml.
// A page names its id (`page_id`); its front matter becomes one object, `pg`, as Jekyll's
// `page.*` was, so the templates read `pg.lang`, `pg.title_key`… and keep their own short
// names (`title_key` in a loop) apart.
import { homes, languages, pages, published, urlFor } from '../eleventy.registry.mjs'

const has = (language, id) => language.published && language.pages.includes(id)

export default {
  pagination: {
    data: 'site.data.languages',
    size: 1,
    alias: 'language',
    before: (items, data) => items.filter((l) => has(l, data.page_id)),
  },
  eleventyComputed: {
    permalink: (data) => `${data.language.prefix}${pages[data.page_id]}`,
    pg: (data) => ({
      lang: data.language.code,
      // The screenshots a language shows: its own, or English's while the app lacks it (FR-084).
      assets: data.language.assets,
      prefix: data.language.prefix,
      date_format: data.language.date_format,
      guide: data.language.guide,
      support: data.language.support,
      html_lang: data.language.html_lang,
      og_locale: data.language.og_locale,
      page_id: data.page_id,
      title_key: data.title_key,
      description_key: data.description_key,
      og: data.og,
      url: data.page?.url,
      // Every language this page is in, in the registry's order.
      versions: languages
        .filter((l) => has(l, data.page_id))
        .map((l) => ({ ...l, url: urlFor(data.page_id, l.code) })),
      // Each published language's home page by its code (the first-visit redirect).
      homes: homes(),
      // The language menu: every published language, at this page if it has it, else at its home.
      menu: published.map((l) => ({
        ...l,
        url: urlFor(has(l, data.page_id) ? data.page_id : 'home', l.code),
        current: l.code === data.language.code,
      })),
    }),
  },
}
