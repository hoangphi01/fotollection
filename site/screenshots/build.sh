#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the website's screenshots from the demo photos (site/screenshots/README.md).
#
#   site/screenshots/build.sh <demo-photo-folder> <work-dir> [best-shot-photo] [owner-folder]
#   SHOT_LANGS="ja ko" DEMO_DIRS_JA=<dir>[:<dir>] DEMO_DIRS_KO=<dir>[:<dir>] site/screenshots/build.sh …
#
# Each language in SHOT_LANGS (en vi by default) is recorded and shot on its own: its photos
# are those listed in demo_set.<lang>.txt (else demo_set.txt), from DEMO_DIRS_<LANG> (else
# the CC0 demo photos in <demo-photo-folder> and, with [owner-folder], the owner's own).
#
# 1. The real app (debug build) records its answers for the language's photos, in that
#    language (src-tauri/src/demo.rs): a Japanese recording shows a はがき, a Korean one a 우편엽서.
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
langs=${SHOT_LANGS:-en vi}

cargo build -q -p fotollection
npm run ui:build >/dev/null
mkdir -p "$out"
# Best shot shows a burst made from the owner's photo, recorded once on its own (README.md).
best=()
if [ -n "${3:-}" ]; then
  rm -rf "$work/best-src" "$work/best-dump"
  uv run python -I site/screenshots/best_shot_photos.py "$3" "$work/best-src"
  FOTOLLECTION_DEMO_FOLDER="$work/best-src" FOTOLLECTION_DEMO_DUMP="$work/best-dump" target/debug/fotollection
  best=("$work/best-dump")
fi

for lang in $langs; do
  # The language's shown set, gathered into one folder (copies on APFS cost no space).
  list=site/screenshots/demo_set.txt
  [ -f "site/screenshots/demo_set.$lang.txt" ] && list="site/screenshots/demo_set.$lang.txt"
  var="DEMO_DIRS_$(echo "$lang" | tr '[:lower:]' '[:upper:]')"
  if [ -n "${!var:-}" ]; then
    IFS=: read -r -a dirs <<< "${!var}"
  else
    dirs=("$demo" ${owner:+"$owner"})
  fi
  folder="$work/demo-src-$lang"
  # The folder is emptied below: a source inside it would be deleted before it is read (LESSONS 2026-11-15).
  for dir in "${dirs[@]}"; do
    if [ -d "$dir" ] && [ "$(cd "$dir" && pwd)" = "$folder" ]; then
      echo "build.sh: $dir is the folder this script empties for $lang; point DEMO_DIRS_$(echo "$lang" | tr '[:lower:]' '[:upper:]') at the original photos" >&2
      exit 1
    fi
  done
  rm -rf "$folder" && mkdir -p "$folder"
  while read -r stem; do
    case "$stem" in ''|'#'*) continue ;; esac
    for dir in "${dirs[@]}"; do
      for f in "$dir/$stem".jpg "$dir/$stem".JPG "$dir/$stem".heic "$dir/$stem".HEIC; do
        [ -f "$f" ] && { cp -c "$f" "$folder/" 2>/dev/null || cp "$f" "$folder/"; }
      done
    done
  done < "$list"
  echo "demo set ($lang): $(ls "$folder" | wc -l | tr -d ' ') photos"
  rm -rf "$work/dump-$lang" "$work/png-$lang"
  mkdir -p "$work/png-$lang"
  FOTOLLECTION_DEMO_LANGUAGE="$lang" FOTOLLECTION_DEMO_FOLDER="$folder" FOTOLLECTION_DEMO_DUMP="$work/dump-$lang" target/debug/fotollection
  SHOT_LANGS="$lang" node site/screenshots/shoot.mjs "$work/dump-$lang" "$work/png-$lang" ${best[@]+"${best[@]}"}
  for png in "$work/png-$lang"/*.png; do
    name=$(basename "$png" .png)
    case "$name" in
      check-*) ;; # for checking controls by eye; not on the page
      *-phone-*) cwebp -quiet -q 82 -resize 640 0 "$png" -o "$out/$name.webp" ;; # FR-135, 640 × 800
      *) cwebp -quiet -q 82 -resize 1600 0 "$png" -o "$out/$name.webp" ;;
    esac
  done
  # The phone crops' measures (FR-135), merged into the site's data, for shot.html and the check.
  node -e 'const fs = require("fs"); const [to, from] = process.argv.slice(1); const old = fs.existsSync(to) ? JSON.parse(fs.readFileSync(to, "utf8")) : {}; const all = { ...old, ...JSON.parse(fs.readFileSync(from, "utf8")) }; fs.writeFileSync(to, JSON.stringify(Object.fromEntries(Object.entries(all).sort()), null, 2) + "\n")' site/_data/phone.json "$work/png-$lang/phone.json"
  # The finished prints at full size for the landing page (FR-135), from the same recording.
  uv run python -I site/screenshots/results.py "$work/dump-$lang" "$lang"
done
# The link previews of every page, from the WebP screenshots just written (og.mjs).
node site/screenshots/og.mjs
# The landing page's showcase tiles, the catalog images and the manual inside the app (FR-110)
# come from the English and Vietnamese shots.
case " $langs " in
  *" en "*|*" vi "*)
    node site/screenshots/showcase.mjs "$work/dump-$(echo "$langs" | awk '{print $1}')"
    scripts/app-pictures.sh
    ;;
esac
du -sh "$out"
