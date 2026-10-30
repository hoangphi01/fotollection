#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the website's screenshots from the demo photos (site/screenshots/README.md).
#
#   site/screenshots/build.sh <demo-photo-folder> <work-dir> [best-shot-photo]
#
# 1. The real app (debug build) records its answers for the demo folder (src-tauri/src/demo.rs).
# 2. Playwright shoots the real interface with those answers (shoot.mjs).
# 3. Screenshots become 1600 px WebP, link previews 1200 × 630 JPEG, in site/assets/img/.
# Needs: cargo, npm, cwebp (libwebp), uv (Python with Pillow, from pyproject.toml).
set -euo pipefail
cd "$(dirname "$0")/../.."

folder=$(cd "$1" && pwd)
work=$(mkdir -p "$2" && cd "$2" && pwd)
out=site/assets/img

cargo build -q -p fotollection
rm -rf "$work/dump" "$work/png"
FOTOLLECTION_DEMO_FOLDER="$folder" FOTOLLECTION_DEMO_DUMP="$work/dump" target/debug/fotollection
npm run ui:build >/dev/null
mkdir -p "$work/png" "$out"
# Best shot shows a burst made from the owner's photo, recorded on its own (README.md).
best=()
if [ -n "${3:-}" ]; then
  rm -rf "$work/best-src" "$work/best-dump"
  uv run python -I site/screenshots/best_shot_photos.py "$3" "$work/best-src"
  FOTOLLECTION_DEMO_FOLDER="$work/best-src" FOTOLLECTION_DEMO_DUMP="$work/best-dump" target/debug/fotollection
  best=("$work/best-dump")
fi
node site/screenshots/shoot.mjs "$work/dump" "$work/png" "${best[@]}"

for png in "$work"/png/*.png; do
  name=$(basename "$png" .png)
  case "$name" in
    check-*) ;; # for checking controls by eye; not on the page
    og-*) uv run python -c 'import sys; from PIL import Image; Image.open(sys.argv[1]).convert("RGB").save(sys.argv[2], quality=86, optimize=True)' "$png" "$out/$name.jpg" ;;
    *) cwebp -quiet -q 82 -resize 1600 0 "$png" -o "$out/$name.webp" ;;
  esac
done
du -sh "$out"
