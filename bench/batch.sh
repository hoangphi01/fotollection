#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Large-batch memory check (NFR-004, docs/RESULT.md): hard-links every JPEG of <photo-folder>
# <copies> times into a temporary folder (no extra disk space), runs the whole pipeline with
# --discard-output so exports need no disk either, and writes bench/results/batch-<n>.json.
# The folder must be on the same volume as $TMPDIR. Takes minutes for thousands of photos.
#
#   bench/batch.sh <photo-folder> [copies=53]
set -euo pipefail
source "$(dirname "$0")/lib.sh"

folder="${1:?usage: bench/batch.sh <photo-folder> [copies]}"
copies="${2:-53}"
mkdir -p "$out"
build_measure

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
shopt -s nullglob nocaseglob
for k in $(seq "$copies"); do
  for f in "$folder"/*.jpg "$folder"/*.jpeg; do
    name="$(basename "$f")"
    ln "$f" "$tmp/${name%.*}_$k.${name##*.}"
  done
done
n="$(find "$tmp" -type f | wc -l | tr -d ' ')"
echo "batch: $n photos" >&2
rust_run "$out/batch-$n.json" "$tmp" --discard-output
