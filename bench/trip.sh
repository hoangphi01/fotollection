#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# A real trip (docs/RESULT.md §20): the whole pipeline on one camera's folder as it came off
# the card, JPEGs and RAWs together, median of <runs>. Writes bench/results/trip.json with
# counts, times and memory only (no paths, no file names). The photos are only read.
#
#   bench/trip.sh <photo-folder> [runs=3]
set -euo pipefail
source "$(dirname "$0")/lib.sh"

folder="${1:?usage: bench/trip.sh <photo-folder> [runs]}"
runs="${2:-3}"
mkdir -p "$out"
build_measure
cargo build --release -q -p fotollection-core --example measure_raw --manifest-path "$root/Cargo.toml"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for i in $(seq "$runs"); do
  echo "run $i/$runs" >&2
  rust_run "$tmp/run-$i.json" "$folder" --discard-output
done
shopt -s nullglob nocaseglob
raws=("$folder"/*.rw2 "$folder"/*.cr2 "$folder"/*.cr3 "$folder"/*.nef "$folder"/*.arw "$folder"/*.raf "$folder"/*.orf "$folder"/*.dng)
if [ ${#raws[@]} -gt 0 ]; then
  "$root/target/release/examples/measure_raw" "${raws[@]}" >"$tmp/raw.md"
else
  : >"$tmp/raw.md"
fi
jpegs=$(find "$folder" -maxdepth 1 -iname '*.jp*g' | wc -l | tr -d ' ')
node "$root/bench/trip.mjs" "$tmp" "$runs" "$jpegs" "${#raws[@]}" >"$out/trip.json"
cat "$out/trip.json"
