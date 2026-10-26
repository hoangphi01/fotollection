# Website

The project's website, <https://fotollection.org/>, in English (`/`) and Vietnamese (`/vi/`). It is built by GitHub Pages' own Jekyll ([`.github/workflows/pages.yml`](../.github/workflows/pages.yml)) whenever `site/` changes on `main`, and after every release, so the download buttons always point at the latest installers ([docs/RELEASE.md](../docs/RELEASE.md)).

| Path | What it holds |
|------|---------------|
| `_config.yml` | Address (`url`, `baseurl`) and the `jekyll-github-metadata` plugin, which reads the latest release |
| `_data/i18n.yml` | Every string, in `en` and `vi`, with the same keys |
| `_layouts/default.html` | Page shell: meta tags, Open Graph and Twitter cards, `hreflang`, navigation, footer |
| `_includes/home.html` | The page itself; `shot.html` (a screenshot, dark or light), `star.html` (the GitHub star link), `icons/` (Lucide, ISC) |
| `index.html`, `vi/index.html` | The two languages |
| `performance.html`, `vi/performance.html`, `_includes/performance.html` | The Performance page: every measurement with its chart, what it means, and plain-language explanations (ΔE, SSIM, JPEG bit depth, 4:4:4, colour profiles, DPI). Its strings are the `perf_*` keys; pages other than home set `path_en`, `path_vi`, `title_key` and `description_key` in their front matter |
| `assets/css/site.css` | One hand-written stylesheet, mobile first, light and dark |
| `assets/fonts/` | Be Vietnam Pro (text) and Bitter (headings, a slab serif like the logo’s wordmark), self-hosted under the [OFL](assets/fonts/OFL.txt), so the page loads nothing from third parties |
| `assets/img/` | Screenshots and link previews, made by [`screenshots/`](screenshots/README.md). `swipe`, `select` and `estimate` are zoomed-in parts of the window (16:10, at three to five times the resolution) for the print-basket section |
| `_includes/logos/` | The dm and Rossmann logos, inline in the print-shop heading. Made from files on Wikimedia Commons marked public domain (`PD-shape` / `PD-textlogo`) and trademarked, checked 2026-10-25 like the app's brand logos ([ADR-0020](../docs/adr/0020-bundled-brand-logos.md)): [Dm-drogerie markt logo.svg](https://commons.wikimedia.org/wiki/File:Dm-drogerie_markt_logo.svg) (SHA-256 `b081849d…`) and [Logo rossmann 2024.svg](https://commons.wikimedia.org/wiki/File:Logo_rossmann_2024.svg) (`6448378a…`). The shapes are unchanged; editor metadata is removed, the colours become the theme colour (`currentColor`) and the white parts the page background, so they sit in the text in both themes. The marks belong to dm-drogerie markt and Dirk Rossmann GmbH; the page says it is not affiliated with them, and a logo is removed on request |
| `_includes/changelog.md` | Written by `pages.yml` from `CHANGELOG.md` at build time (not committed), for the "What's new" box |
| `_data/visits.json` | Written by `pages.yml` through [`scripts/ga-visits.mjs`](../scripts/ga-visits.mjs) at build time (not committed): total GA sessions since 2026-10-24 for the landing page footer. `pages.yml` also runs once a day so the number stays current. Without the `GA_PROPERTY_ID` and `GA_SERVICE_ACCOUNT` secrets the file is not written and the footer leaves the line out |
| `assets/img/charts/` | The benchmark charts in the site's colours (budgets, quality, share, best shot, memory, browser), written by `node bench/chart.mjs` next to the README's own |

**Rules.** The only third-party script is Google Analytics (`google_analytics` in `_config.yml`, production builds only), the owner's choice for counting visits; the footer says so, and the visit total in the landing page footer is read from it at build time, not by a script in the page, and the app itself has no analytics. No other third-party scripts, fonts or trackers. Text is plain Liquid output, never raw HTML from data. A new string is added to both languages.

**Address.** The domain `fotollection.org` is registered at Cloudflare (2026-10-25). Its DNS points the apex at GitHub Pages (A `185.199.108–111.153`, AAAA `2606:50c0:8000–8003::153`) and `www` at `hoangphi01.github.io`, all *DNS only* so GitHub can issue the HTTPS certificate. The custom domain is set in the repository's Pages settings (the site deploys through Actions, so no `CNAME` file is used), and the old `hoangphi01.github.io/fotollection/` address redirects to it. Moving it again means changing `url` and `baseurl` in `_config.yml` and the Pages setting.

## Preview locally

```sh
docker run --rm --platform linux/amd64 -v "$PWD/site":/work/site:ro -v /tmp/site-out:/out \
  -e GITHUB_WORKSPACE=/work -e INPUT_SOURCE=site -e INPUT_DESTINATION=../out \
  -e INPUT_TOKEN="$(gh auth token)" -e GITHUB_REPOSITORY=hoangphi01/fotollection \
  -e GITHUB_API_URL=https://api.github.com ghcr.io/actions/jekyll-build-pages:v1.0.13
```

The output in `/tmp/site-out` is served at the root of the domain.
