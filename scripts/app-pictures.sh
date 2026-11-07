#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# The manual inside the app (FR-110): every screenshot the user guide shows, in English and
# Vietnamese, light and dark, made smaller (1,100 px wide, WebP q78) from the website's own
# screenshots into ui/public/manual/, so the app shows them offline. Run by
# site/screenshots/build.sh after it shoots the site's pictures, and by hand:
#
#   scripts/app-pictures.sh
#
# check-docs.sh fails when a screenshot the guide names has no picture here.
set -euo pipefail
cd "$(dirname "$0")/.."

out=ui/public/manual
mkdir -p "$out"
names=$(grep -ohE '^<!-- shot: [a-z0-9-]+' docs/USER_GUIDE.md | sed 's/.* //' | sort -u)
# Pictures of shots the guide no longer names go.
for f in "$out"/*.webp; do
  [ -e "$f" ] || continue
  name=$(basename "$f" .webp | sed -E 's/-(en|vi)-(light|dark)$//')
  grep -qx "$name" <<<"$names" || rm "$f"
done
for name in $names; do
  for v in en-light en-dark vi-light vi-dark; do
    src="site/assets/img/$name-$v.webp"
    dst="$out/$name-$v.webp"
    [ -f "$src" ] || { echo "app-pictures: $src is missing" >&2; exit 1; }
    # Only when the site's picture changed since.
    [ "$dst" -nt "$src" ] && continue
    width=$(sips -g pixelWidth "$src" 2>/dev/null | awk '/pixelWidth/ { print $2 }')
    if [ "${width:-0}" -gt 1100 ]; then
      cwebp -quiet -q 78 -resize 1100 0 "$src" -o "$dst"
    else
      cp "$src" "$dst"
    fi
  done
done
du -sh "$out"
