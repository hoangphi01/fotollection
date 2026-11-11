# ADR-0049: The website is built by Eleventy, still served by GitHub Pages

- **Status:** Accepted
- **Date:** 2026-11-11
- **Related:** FR-021, ADR-0031, ADR-0044

## Context

The website was built by GitHub Pages' own Jekyll (`actions/jekyll-build-pages`), which runs Jekyll 3.10 in safe mode: no plugin of our own and no Gemfile. The owner asked on 2026-11-11 for the site in Hungarian, German, Polish and Czech, with a way to manage languages that scales to every subpage, the visitor's language found by itself, and later keynote-style videos. In safe mode each language needs its own copy of every page (seven ten-line stubs per language today), each page knows only `path_en` and `path_vi`, and the templates branch on `page.lang == 'vi'` seventeen times; nothing can generate a page per language. A local preview needed Docker, which the owner does not want to run for this. A VPS costs money the project does not spend.

## Options considered

1. **Jekyll with our own build** (`bundle exec`, a Gemfile, a Ruby plugin generating pages per language). Pros: the fewest template changes. Cons: Ruby added to a Rust and Node project; the preview still needs Ruby or Docker.
2. **Eleventy** (MIT), Node, with Liquid. Pros: the templates stay Liquid with Jekyll's includes (`jekyllInclude`), so they read as they did; one template can make every language (pagination over a language list); Node is already the site's toolchain (charts, screenshots, checks); a preview is `npm run site:serve`. Cons: Jekyll's `page.*`, `site.*`, kramdown and `jekyll-github-metadata` replaced by hand.
3. **Astro.** Pros: i18n routing, image and video handling built in. Cons: every template rewritten, the most risk and work.

## Decision

Option 2. `eleventy.config.mjs` builds `site/` into `_site/` in the pages workflow, which uploads it to GitHub Pages as before (`actions/upload-pages-artifact`, `actions/deploy-pages`): the hosting, the domain, HTTPS and the APT repository do not change, and nothing is paid for.

- **The same `site.*`:** `_config.yml`'s keys, every `_data` file under `site.data`, `site.time`, and `site.github.latest_release` from `_data/release.json`, which the workflow writes with `gh api` (what `jekyll-github-metadata` gave).
- **The same `page.*`, named `pg.*`:** each page's front matter as one object (`site/site.11tydata.mjs`), so a template's own short names (`title_key` in a loop) never shadow it, as they did on the first try.
- **The same Markdown:** markdown-it (MIT) with markdown-it-attrs (MIT, the manual's `{#id}`) and markdown-it-anchor (Unlicense, kramdown's GFM header ids), typographic quotes, `<hr />`, and a rule that wraps a list item's first paragraph in `<p>` only where kramdown did (a blank line after it and another item to come).
- **Proven the same:** every page built both ways and compared after normalising white space; the only differences left are what only the workflow has (the visit count) and `&` written as `&amp;` in two `alt` attributes.

## Consequences

- **Languages from one list:** `site/_data/languages.yml` names every language (its prefix, the pages it has, its screenshots, its date format) and `site/_data/pages.yml` every page; each page is one template built per published language, links come from `url_for`, texts in data files from `tr`, and the words from `site/_data/i18n/<code>.yml`, English the fallback. The seventeen `== 'vi'` branches and the seven Vietnamese stubs are gone; `scripts/check-site-i18n.mjs` keeps a language's name out of the templates, and its `--build` builds the site with every borrowed English word marked, so a published language showing English fails (the Proxy tried first saw nothing: Eleventy deep-copies global data before rendering).
- **SEO per language:** each page's `hreflang`, `og:locale:alternate` and JSON-LD `inLanguage` come from the registry, `sitemap.xml` is written from it after the build (`robots.txt` names it), and each published language has its own link previews (`og_card_<page>` words, `og.mjs`).
- **Subpages per language (2026-11-11):** Czech, German, Hungarian and Polish have the Performance page (its explanations `perf/<part>-<code>.md`, charts from `bench/chart.mjs`'s words per language, diagrams from label tables), the Wiki, the code signing policy and a short guide in place of the full manual; the story stays English and Vietnamese, the author's own words (owner).
- The site's build is `npm ci` and `npx @11ty/eleventy` in about a second; the preview needs no Docker.
- Templates read front matter as `pg.lang`, `pg.title_key`, `pg.og`; `page.url` is Eleventy's own.
