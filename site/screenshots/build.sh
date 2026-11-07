#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the website's screenshots from the demo photos (site/screenshots/README.md).
#
#   site/screenshots/build.sh <demo-photo-folder> <work-dir> [best-shot-photo] [owner-folder]
#
# The photos shown are those listed in demo_set.txt: the CC0 demo photos from
# <demo-photo-folder> and, with [owner-folder], the owner's own (README.md).
#
# 1. The real app (debug build) records its answers for the demo folder (src-tauri/src/demo.rs).
# 2. Playwright shoots the real interface with those answers (shoot.mjs).
# 3. Screenshots become 1600 px WebP in site/assets/img/; og.mjs then renders each page's
#    link preview, 1200 × 630 JPEG, from them.
# Needs: cargo, npm, cwebp (libwebp), uv (Python with Pillow, from pyproject.toml).
set -euo pipefail
cd "$(dirname "$0")/../.."

demo=$(cd "$1" && pwd)
work=$(mkdir -p "$2" && cd "$2" && pwd)
out=site/assets/img
owner=${4:-}

# The shown set, gathered into one folder (copies on APFS cost no space).
folder="$work/demo-src"
rm -rf "$folder" && mkdir -p "$folder"
while read -r stem; do
  case "$stem" in ''|'#'*) continue ;; esac
  for dir in "$demo" ${owner:+"$owner"}; do
    for f in "$dir/$stem".jpg "$dir/$stem".JPG; do
      [ -f "$f" ] && cp -c "$f" "$folder/" 2>/dev/null || { [ -f "$f" ] && cp "$f" "$folder/"; }
    done
  done
done < site/screenshots/demo_set.txt
echo "demo set: $(ls "$folder" | wc -l | tr -d ' ') photos"

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
    *) cwebp -quiet -q 82 -resize 1600 0 "$png" -o "$out/$name.webp" ;;
  esac
done
# The link previews of every page, from the WebP screenshots just written (og.mjs).
node site/screenshots/og.mjs
# The landing page's showcase tiles and the catalog images, from the same screenshots.
node site/screenshots/showcase.mjs "$work/dump"
# The manual inside the app (FR-110): the guide's screenshots, smaller, in ui/public/manual/.
scripts/app-pictures.sh
du -sh "$out"
