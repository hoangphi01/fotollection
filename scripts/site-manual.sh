#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Turns the user guide into the website's manual (ADR-0031). Run by pages.yml before Jekyll,
# and by hand to preview:
#
#   scripts/site-manual.sh
#
# For each language, docs/USER_GUIDE[.vi].md becomes site/_includes/manual-<lang>.md (from
# the first chapter on) and manual-toc-<lang>.md (one link per chapter). Neither is committed.
#   ## 4. Camera RAW <!-- raw -->     ->  ## 4. Camera RAW {#raw}
#   <!-- shot: grid | alt text -->    ->  the screenshot, dark or light (site/_includes/shot.html)
#   [PRD](PRD.md#fr-016--raw-input)   ->  [PRD](docs/PRD.md#…), linked to GitHub by manual.html
set -euo pipefail
cd "$(dirname "$0")/.."

for lang in en vi; do
  src=docs/USER_GUIDE.md
  [ "$lang" = vi ] && src=docs/USER_GUIDE.vi.md
  awk '/^## /{p=1} p' "$src" | sed -E \
    -e 's/^(##+ .*[^ ]) *<!-- ([a-z0-9-]+) -->$/\1 {#\2}/' \
    -e 's#^<!-- shot: ([a-z0-9-]+) \| (.*) -->$#<figure class="shot">{% include shot.html name="\1" alt="\2" %}</figure>#' \
    -e 's#\]\(([^):]+\.md)#](docs/\1#g' \
    > "site/_includes/manual-$lang.md"
  grep -E '^## ' "$src" | sed -E 's/^## (.*[^ ]) *<!-- ([a-z0-9-]+) -->$/- [\1](#\2)/' \
    > "site/_includes/manual-toc-$lang.md"
done
