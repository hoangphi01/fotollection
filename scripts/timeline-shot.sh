#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# A screenshot of the app as it was at a release, for the website's Wiki page (its timeline,
# site/_data/timeline.yml). The picture is taken from the git history, never made again, so
# it shows that version as it shipped:
#
#   scripts/timeline-shot.sh <tag|HEAD> <path-in-that-tree> <name>
#   scripts/timeline-shot.sh v0.9.9 site/assets/img/collage-en-dark.webp 0.9-collage
#
# Writes site/assets/img/timeline/<name>.webp, at most 1600 px wide, WebP quality 82.
# At each MINOR release, after site/screenshots/build.sh: one to three of its new shots with
# HEAD, and a milestone in timeline.yml (docs/DOCS_GUIDE.md, "A release").
set -euo pipefail
cd "$(dirname "$0")/.."

[ $# -eq 3 ] || { echo "usage: scripts/timeline-shot.sh <tag|HEAD> <path> <name>" >&2; exit 2; }
rev=$1 path=$2 name=$3
case "$name" in *[!a-z0-9.-]*) echo "timeline-shot: a name of a-z, 0-9, . and - only" >&2; exit 2 ;; esac
command -v cwebp >/dev/null || { echo "timeline-shot: needs cwebp (libwebp)" >&2; exit 2; }

out=site/assets/img/timeline
mkdir -p "$out"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
src="$tmp/source.${path##*.}"
git show "${rev}:${path}" > "$src"
# cwebp reads PNG and JPEG; a WebP is decoded first.
if [ "${path##*.}" = webp ]; then
  dwebp -quiet "$src" -o "$tmp/source.png"
  src="$tmp/source.png"
fi
width=$(sips -g pixelWidth "$src" 2>/dev/null | awk '/pixelWidth/ {print $2}' || true)
# macOS bash 3.2 and `set -u` refuse an empty array: two calls instead.
if [ -n "$width" ] && [ "$width" -gt 1600 ]; then
  cwebp -quiet -q 82 -resize 1600 0 "$src" -o "$out/$name.webp"
else
  cwebp -quiet -q 82 "$src" -o "$out/$name.webp"
fi
echo "$out/$name.webp ($(du -h "$out/$name.webp" | cut -f1)) from ${rev}:${path}"
