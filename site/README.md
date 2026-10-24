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
| `assets/fonts/` | Be Vietnam Pro and Bricolage Grotesque, self-hosted under the [OFL](assets/fonts/OFL.txt), so the page loads nothing from third parties |
| `assets/img/` | Screenshots and link previews, made by [`screenshots/`](screenshots/README.md) |

**Rules.** No third-party scripts, fonts or trackers. Text is plain Liquid output, never raw HTML from data. A new string is added to both languages.

**Moving the address** (for example to `fotollection.github.io`, which needs a GitHub organisation named `fotollection`): change `url` and `baseurl` in `_config.yml`.

## Preview locally

```sh
docker run --rm --platform linux/amd64 -v "$PWD/site":/work/site:ro -v /tmp/site-out:/out \
  -e GITHUB_WORKSPACE=/work -e INPUT_SOURCE=site -e INPUT_DESTINATION=../out \
  -e INPUT_TOKEN="$(gh auth token)" -e GITHUB_REPOSITORY=hoangphi01/fotollection \
  -e GITHUB_API_URL=https://api.github.com ghcr.io/actions/jekyll-build-pages:v1.0.13
```

The output in `/tmp/site-out` expects to be served under `/fotollection/`.
