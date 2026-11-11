// SPDX-License-Identifier: GPL-3.0-only
// Every page's front matter as one object, `pg`, as Jekyll's `page.*` was: the templates read
// `pg.lang`, `pg.title_key`… and keep their own short names (`title_key` in a loop) apart.
export default {
  eleventyComputed: {
    pg: (data) => ({
      lang: data.lang,
      title_key: data.title_key,
      description_key: data.description_key,
      og: data.og,
      path_en: data.path_en,
      path_vi: data.path_vi,
      url: data.page?.url,
    }),
  },
}
