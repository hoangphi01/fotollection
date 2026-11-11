#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Turns the user guide into the website's manual (ADR-0031). Run by pages.yml before Jekyll,
# and by hand to preview:
#
#   scripts/site-manual.sh
#
# For each language, its guide (docs/USER_GUIDE[.vi].md, or site/_includes/manual-short-<lang>.md)
# becomes site/_includes/manual-<lang>.md (from
# the first chapter on) and manual-toc-<lang>.md (one link per chapter). Neither is committed.
# The story page's chapter list, story-toc-<lang>.md, comes from its committed text the same way.
#   <!-- chapter: raw --> + ## 4. Camera RAW  ->  ## 4. Camera RAW {#raw}
#   <!-- shot: grid | alt text -->            ->  the screenshot, dark or light (site/_includes/shot.html)
#   [PRD](PRD.md#fr-016--raw-input)           ->  [PRD](docs/PRD.md#…), linked to GitHub by manual.html
set -euo pipefail
cd "$(dirname "$0")/.."

# Each published language with a manual page, its guide (`guide` in site/_data/languages.yml:
# the full one in docs/ for English and Vietnamese, the short one in site/_includes/ for the
# others) and whether it has a story page.
node --input-type=module -e "
  import { published } from './eleventy.registry.mjs'
  for (const l of published) if (l.pages.includes('manual')) console.log(l.code, l.guide, l.pages.includes('story') ? 1 : 0)
" | while read -r lang src story; do
  # A chapter marker names the next chapter title's anchor.
  awk '/^<!-- chapter: [a-z0-9-]+ -->$/ { id = $3; p = 1; next }
       /^## / && id != "" { print $0 " {#" id "}"; id = ""; next }
       p' "$src" | sed -E \
    -e 's#^<!-- shot: ([a-z0-9-]+) \| (.*) -->$#<figure class="shot">{% include shot.html name="\1" alt="\2" %}</figure>#' \
    -e 's#\]\(([^):]+\.md)#](docs/\1#g' \
    > "site/_includes/manual-$lang.md"
  grep -E '^## ' "site/_includes/manual-$lang.md" | sed -E 's/^## (.*) \{#([a-z0-9-]+)\}$/- [\1](#\2)/' \
    > "site/_includes/manual-toc-$lang.md"
  # The story page's chapter list, from its own text (site/_includes/story-<lang>.md).
  if [ "$story" = 1 ]; then
    grep -E '^## ' "site/_includes/story-$lang.md" | sed -E 's/^## (.*) \{#([a-z0-9-]+)\}$/- [\1](#\2)/' \
      > "site/_includes/story-toc-$lang.md"
  fi
done
