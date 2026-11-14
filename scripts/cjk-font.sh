#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The Japanese and Korean faces the app bundles (FR-128, docs/LOCALIZATION.md §5): Noto Sans JP
# and Noto Sans KR's variable fonts (OFL 1.1, google/fonts), each cut to the characters its
# interface and manual use, plus the letters typed or shown whole (every kana, the Hangul
# compatibility jamo, the CJK punctuation), as one woff2 for weights 400–700, and beside it the
# list of its characters from U+3000 on. Run again whenever ui/src/locales/<code>.json or
# site/_includes/manual-short-<code>.md gains a character: `check-i18n` fails until then.
# Needs curl and uv.
#
#   scripts/cjk-font.sh ja
#   scripts/cjk-font.sh ko
set -euo pipefail
cd "$(dirname "$0")/.."

case "${1:-}" in
  ja)
    url='https://github.com/google/fonts/raw/main/ofl/notosansjp/NotoSansJP%5Bwght%5D.ttf'
    sha1='55c06f745ac2893afbe3296fbc6a98787865ea59'
    out=ui/src/assets/fonts/noto-sans-jp-subset.woff2
    ranges='U+3000-303F,U+3040-30FF,U+FF00-FFEF'
    ;;
  ko)
    url='https://github.com/google/fonts/raw/main/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf'
    sha1='79bfd054156deb890bac8797feb898a97ca64c7f'
    out=ui/src/assets/fonts/noto-sans-kr-subset.woff2
    ranges='U+3000-303F,U+3131-318E'
    ;;
  *)
    echo "usage: scripts/cjk-font.sh ja|ko" >&2
    exit 2
    ;;
esac
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

curl -sSfL -o "$work/source.ttf" "$url"
echo "$sha1  $work/source.ttf" | shasum -c - >/dev/null
cat "ui/src/locales/$1.json" "site/_includes/manual-short-$1.md" > "$work/text.txt"
uv run -q --no-project --with fonttools --with brotli pyftsubset "$work/source.ttf" \
  --text-file="$work/text.txt" \
  --unicodes="$ranges" \
  --layout-features='kern,palt,vkrn,liga,ccmp,locl' \
  --no-hinting --flavor=woff2 --output-file="$out"
# The characters it covers, for check-i18n to test the locale against without parsing a woff2.
uv run -q --no-project --with fonttools --with brotli python -I -c \
  'import sys; from fontTools.ttLib import TTFont; print("".join(sorted(chr(c) for c in TTFont(sys.argv[1]).getBestCmap() if c >= 0x3000)))' \
  "$out" > "${out%.woff2}.txt"
ls -l "$out" | awk '{printf "%s: %.0f KB\n", $NF, $5 / 1024}'
