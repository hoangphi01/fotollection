# Website

The project's website, <https://hoangphi01.github.io/fotollection/>, in English (`/`) and Vietnamese (`/vi/`). It is built by GitHub Pages' own Jekyll ([`.github/workflows/pages.yml`](../.github/workflows/pages.yml)) whenever `site/` changes on `main`, and after every release, so the download buttons always point at the latest installers ([docs/RELEASE.md](../docs/RELEASE.md)).

| Path | What it holds |
|------|---------------|
| `_config.yml` | Address (`url`, `baseurl`) and the `jekyll-github-metadata` plugin, which reads the latest release |
| `_data/i18n.yml` | Every string, in `en` and `vi`, with the same keys |
| `_layouts/default.html` | Page shell: meta tags, Open Graph and Twitter cards, `hreflang`, navigation, footer |
| `_includes/home.html` | The page itself; `shot.html` (a screenshot, dark or light), `star.html` (the GitHub star link), `icons/` (Lucide, ISC) |
| `index.html`, `vi/index.html` | The two languages |
| `assets/css/site.css` | One hand-written stylesheet, mobile first, light and dark |
| `assets/fonts/` | Be Vietnam Pro (text) and Bitter (headings, a slab serif like the logo’s wordmark), self-hosted under the [OFL](assets/fonts/OFL.txt), so the page loads nothing from third parties |
| `assets/img/` | Screenshots and link previews, made by [`screenshots/`](screenshots/README.md) |
| `assets/logos/` | The print shops' logos in the kiosk section, unchanged from Wikimedia Commons, where each is marked public domain (`PD-textlogo` / `PD-shape`) and trademarked, checked 2026-10-25 like the app's brand logos ([ADR-0020](../docs/adr/0020-bundled-brand-logos.md)): `dm.svg` [Dm-drogerie markt logo.svg](https://commons.wikimedia.org/wiki/File:Dm-drogerie_markt_logo.svg) (SHA-256 `b081849d…`), `rossmann.svg` [Logo rossmann 2024.svg](https://commons.wikimedia.org/wiki/File:Logo_rossmann_2024.svg) (`6448378a…`), `cewe.svg` [Cewe new.svg](https://commons.wikimedia.org/wiki/File:Cewe_new.svg) (`feb5152f…`). The marks belong to dm-drogerie markt, Dirk Rossmann GmbH and CEWE Stiftung; the page says it is not affiliated with them, and a logo is removed on request |
| `_includes/changelog.md` | Written by `pages.yml` from `CHANGELOG.md` at build time (not committed), for the "What's new" box |
| `assets/img/charts/` | Three benchmark charts in the site's colours, written by `node bench/chart.mjs` next to the README's own |

**Rules.** The only third-party script is Google Analytics (`google_analytics` in `_config.yml`, production builds only), the owner's choice for counting visits; the footer says so, and the app itself has no analytics. No other third-party scripts, fonts or trackers. Text is plain Liquid output, never raw HTML from data. A new string is added to both languages.

**Moving the address** (for example to `fotollection.github.io`, which needs a GitHub organisation named `fotollection`): change `url` and `baseurl` in `_config.yml`.

## Preview locally

```sh
docker run --rm --platform linux/amd64 -v "$PWD/site":/work/site:ro -v /tmp/site-out:/out \
  -e GITHUB_WORKSPACE=/work -e INPUT_SOURCE=site -e INPUT_DESTINATION=../out \
  -e INPUT_TOKEN="$(gh auth token)" -e GITHUB_REPOSITORY=hoangphi01/fotollection \
  -e GITHUB_API_URL=https://api.github.com ghcr.io/actions/jekyll-build-pages:v1.0.13
```

The output in `/tmp/site-out` expects to be served under `/fotollection/`.
