# ADR-0031: The manual on the website is built from the user guide

- **Status:** Accepted
- **Date:** 2026-10-30
- **Related:** FR-020, ADR-0006, [USER_GUIDE.md](../USER_GUIDE.md), [site/README.md](../../site/README.md)

## Context

The owner wants an electronic manual on <https://fotollection.org/manual/>, in English and Vietnamese, that matches the app one to one: the same labels, the same steps, and screenshots of every step taken the way the landing page's are (the real app on the CC0 demo photos). It must stay correct as the app changes, and be easy to extend chapter by chapter.

[USER_GUIDE.md](../USER_GUIDE.md) already holds the steps and the shortcuts, and DOCS_GUIDE already makes every change a user would notice update it in the same commit. A second copy of the same text on the website would drift from it.

## Options considered

1. **Write the manual as website data** (`site/_data/manual/*.yml`), with the user guide kept beside it. Full control of the layout, but every step is written twice in English, and the two copies drift.
2. **Build the manual from the user guide** at build time, like the "What's new" box is built from the CHANGELOG. One English text; a Vietnamese mirror `USER_GUIDE.vi.md` held to the same chapters by `scripts/check-docs.sh`, as `README.vi.md` is.
3. **Link to the user guide on GitHub.** Nothing to build, but GitHub cannot show a screenshot per language and theme, and it is not a page a photographer would read.

## Decision

Option 2. `docs/USER_GUIDE.md` (English) and `docs/USER_GUIDE.vi.md` (Vietnamese, the second exception to "docs are in English" after `README.vi.md`) are the manual. `scripts/site-manual.sh` turns them into `site/_includes/manual-{en,vi}.md` (not committed) when the website is built, and `site/_includes/manual.html` lays them out with the landing page's parts.

The guide carries two kinds of markers, HTML comments that GitHub does not show:

- `<!-- chapter: raw -->` on its own line before `## 4. Camera RAW`: the chapter's anchor, the same in both languages, so `/manual/#raw` and `/vi/manual/#raw` show the same chapter and a link survives a renumbering. It sits above the title, not in it, so the title's own anchor on GitHub (`#4-camera-raw`) does not change.
- `<!-- shot: grid | The library, culled with stars and picks -->` on a line of its own: a screenshot (`site/assets/img/grid-{en,vi}-{light,dark}.webp`, dark or light to follow the reader's theme) with its alt text in the file's language.

## Consequences

- A change a user would notice updates `USER_GUIDE.md` **and** `USER_GUIDE.vi.md` in the same commit, using the app's own labels from `ui/src/locales/{en,vi}.json`; when the change is visible, its screenshots are shot again with `site/screenshots/build.sh` (DOCS_GUIDE §2).
- `scripts/check-docs.sh` checks that both guides have the same chapters and sections, the same anchors and the same screenshots in the same order, and that every screenshot exists in both languages and themes.
- Manual-only screenshots are named `manual-*` in `site/screenshots/shoot.mjs`; the landing page's are reused where they show the step.
- Links to PRD, STATUS or an ADR become links to the file on GitHub, so the manual can keep the guide's references.
- Chapters the app has but the guide does not yet cover (the date stamp, the info frame, crop and straighten, best shot, export, settings) are added to both guides as they are written, each with its screenshots; [STATUS.md](../STATUS.md) lists them.
