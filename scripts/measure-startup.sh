#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Cold start (NFR-010): launches the release app N times (default 5) with FOTOLLECTION_TIMING
# set, waits for the first thumbnail, quits it, and prints each run's milestones. Times
# start at `run()`, a few ms after the process starts. The app reopens your last session,
# as it does for you; it is closed before it can save anything.
#
#   npx tauri build --no-bundle && scripts/measure-startup.sh [runs]
set -euo pipefail

app="target/release/fotollection"
runs="${1:-5}"
[ -x "$app" ] || { echo "build first: npx tauri build --no-bundle" >&2; exit 1; }
log="$(mktemp)"
trap 'rm -f "$log"' EXIT

for i in $(seq "$runs"); do
  : >"$log"
  FOTOLLECTION_TIMING=1 "$app" 2>"$log" &
  pid=$!
  for _ in $(seq 200); do
    grep -q "first thumbnail" "$log" && break
    sleep 0.1
  done
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  echo "run $i: $(grep '^timing:' "$log" | sed 's/^timing: //' | paste -sd ';' - | sed 's/;/ · /g')"
done
