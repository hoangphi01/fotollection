#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rust vs browser benchmark and export concurrency (docs/OPTIMIZATION.md §2.1, docs/RESULT.md).
# macOS only for now (peak RSS via /usr/bin/time -l). Photos are read from <photo-folder> and
# never copied into the repo. Let the machine cool down first: the M3 throttles under load.
#
#   bench/run.sh <photo-folder> [runs=3]
set -euo pipefail
source "$(dirname "$0")/lib.sh"

folder="${1:?usage: bench/run.sh <photo-folder> [runs]}"
runs="${2:-3}"
mkdir -p "$out"
rm -f "$out"/rust-*.json "$out"/web-*.json "$out"/inflight8-*.json

build_measure
npx --prefix "$root" playwright install chromium >/dev/null

for i in $(seq "$runs"); do
  echo "run $i/$runs: rust (6 in flight)" >&2
  rust_run "$out/rust-$i.json" "$folder"
  echo "run $i/$runs: rust (8 in flight)" >&2
  rust_run "$out/inflight8-$i.json" "$folder" --inflight 8
  echo "run $i/$runs: browser" >&2
  node "$root/bench/web/run.mjs" "$folder" >"$out/web-$i.json"
done

write_machine "$folder"
node "$root/bench/chart.mjs"
