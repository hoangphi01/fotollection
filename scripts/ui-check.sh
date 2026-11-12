#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Runs every machine-judged UI test case of docs/UI_TESTS.md, in one command:
#
#   scripts/ui-check.sh <demo-photo-folder> [owner-folder]     # the whole matrix
#   QUICK=1 scripts/ui-check.sh <demo-photo-folder>            # English, light, Chromium only
#   UPDATE_BASELINES=1 scripts/ui-check.sh …                   # approve the pictures after an intended change
#
# The demo photos come from `python3 -I site/screenshots/fetch_photos.py <folder>`. The demo
# app's recording is made once into target/ui-check/dump and kept; delete it to record again
# (after a change to what Rust answers). Quit `tauri dev` first: the app runs once at a time.
# Exits non-zero on any failure, each line starting with its test ID.
set -euo pipefail
cd "$(dirname "$0")/.."

[ $# -ge 1 ] || { echo "usage: scripts/ui-check.sh <demo-photo-folder> [owner-folder]" >&2; exit 2; }
if pgrep -f 'target/debug/fotollection' >/dev/null || pgrep -f 'tauri dev' >/dev/null; then
  echo "ui-check: quit the running app (tauri dev) first: the demo recording would go to it" >&2
  exit 2
fi
scripts/resources.sh | tail -3 || true  # a warning, never a stop

demo=$(cd "$1" && pwd)
owner=${2:-}
work=target/ui-check
dump="$work/dump"
if [ ! -f "$dump/fixtures.json" ]; then
  folder="$work/demo-src"
  rm -rf "$folder" && mkdir -p "$folder"
  while read -r stem; do
    case "$stem" in ''|'#'*) continue ;; esac
    for dir in "$demo" ${owner:+"$owner"}; do
      for f in "$dir/$stem".jpg "$dir/$stem".JPG; do
        [ -f "$f" ] && { cp -c "$f" "$folder/" 2>/dev/null || cp "$f" "$folder/"; }
      done
    done
  done < site/screenshots/demo_set.txt
  CARGO_BUILD_JOBS=4 cargo build -q -p fotollection
  rm -rf "$dump"
  FOTOLLECTION_DEMO_FOLDER="$PWD/$folder" FOTOLLECTION_DEMO_DUMP="$PWD/$dump" target/debug/fotollection
fi
# VITE_FAULTS=1: the parts the harness breaks on purpose (UI-STATE-010); VITE_PSEUDO=1: the
# pseudo-locale en-XA, English 40 % longer (FR-125). Releases set neither.
VITE_FAULTS=1 VITE_PSEUDO=1 npm run ui:build >/dev/null

status=0
run() {
  echo "── $*"
  env MEASURE=1 "$@" node site/screenshots/shoot.mjs "$dump" "$work/out" || status=1
}
mkdir -p "$work/out"
if [ -n "${QUICK:-}" ]; then
  run MEASURE_LANG=en MEASURE_THEME=light
else
  # The two corners of the matrix (docs/UI_TESTS.md §1): English in light in Chromium, and
  # Vietnamese, the longer words, in dark in WebKit, the engine of the Mac app.
  run MEASURE_LANG=en MEASURE_THEME=light
  run MEASURE_LANG=vi MEASURE_THEME=dark MEASURE_ENGINE=webkit
  # Hungarian, and the pseudo-locale as long as German (FR-123, FR-125): every word fits, smaller
  # or cut with its hint; the words fitted are listed for the translator.
  run MEASURE_LANG=hu MEASURE_THEME=light
  run MEASURE_LANG=en-XA MEASURE_THEME=dark
fi
scripts/resources.sh | tail -3 || true  # a warning, never a stop
[ "$status" -eq 0 ] && echo "ui-check: every UI test case passes" || echo "ui-check: failures above, each with its UI- test ID" >&2
exit "$status"
