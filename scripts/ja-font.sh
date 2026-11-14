#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The Japanese face the app bundles (FR-128, docs/LOCALIZATION.md §5): Noto Sans JP's variable
# font (OFL 1.1, google/fonts), cut to the characters the interface and its manual use, plus
# every kana, the CJK punctuation and the full-width forms, as one woff2 for weights 400–700,
# and beside it the list of its characters from U+3000 on.
# Run again whenever ui/src/locales/ja.json or site/_includes/manual-short-ja.md gains a
# character: `check-i18n` fails until then. Needs curl and uv.
#
#   scripts/ja-font.sh
set -euo pipefail
cd "$(dirname "$0")/.."

url='https://github.com/google/fonts/raw/main/ofl/notosansjp/NotoSansJP%5Bwght%5D.ttf'
sha1='55c06f745ac2893afbe3296fbc6a98787865ea59'
out=ui/src/assets/fonts/noto-sans-jp-subset.woff2
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

curl -sSfL -o "$work/source.ttf" "$url"
echo "$sha1  $work/source.ttf" | shasum -c - >/dev/null
cat ui/src/locales/ja.json site/_includes/manual-short-ja.md > "$work/text.txt"
uv run -q --no-project --with fonttools --with brotli pyftsubset "$work/source.ttf" \
  --text-file="$work/text.txt" \
  --unicodes='U+3000-303F,U+3040-30FF,U+FF00-FFEF' \
  --layout-features='kern,palt,vkrn,liga,ccmp,locl' \
  --no-hinting --flavor=woff2 --output-file="$out"
 # The characters it covers, for check-i18n to test ja.json against without parsing a woff2.
uv run -q --no-project --with fonttools --with brotli python -I -c \
  'import sys; from fontTools.ttLib import TTFont; print("".join(sorted(chr(c) for c in TTFont(sys.argv[1]).getBestCmap() if c >= 0x3000)))' \
  "$out" > "${out%.woff2}.txt"
ls -l "$out" | awk '{printf "%s: %.0f KB\n", $NF, $5 / 1024}'
