#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Shared helpers for bench/run.sh and bench/batch.sh. Source it; do not run it.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/bench/results"
measure="$root/target/release/examples/measure_folder"

build_measure() {
  cargo build --release -q -p fotollection-core --example measure_folder --manifest-path "$root/Cargo.toml"
}

# rust_run <result-json> <measure_folder args…>: runs measure_folder --json under
# /usr/bin/time -l and adds the peak RSS as `peakRssBytes`.
rust_run() {
  local dest="$1"
  shift
  local raw err
  raw="$(mktemp)"
  err="$(mktemp)"
  /usr/bin/time -l "$measure" "$@" --json >"$raw" 2>"$err"
  local rss
  rss="$(awk '/maximum resident set size/ {print $1}' "$err")"
  node -e 'const fs=require("fs");const d=JSON.parse(fs.readFileSync(process.argv[1]));d.peakRssBytes=+process.argv[2];fs.writeFileSync(process.argv[3],JSON.stringify(d,null,2)+"\n")' "$raw" "$rss" "$dest"
  rm -f "$raw" "$err"
}

# Machine facts for chart footers; no paths or file names.
write_machine() {
  local folder="$1" mp
  mp="$(node -e 'const s=require("child_process").execFileSync("sips",["-g","pixelWidth","-g","pixelHeight",process.argv[1]]).toString();const [w,h]=[...s.matchAll(/: (\d+)/g)].map(m=>+m[1]);console.log(Math.round(w*h/1e6))' "$(find "$folder" -maxdepth 1 -iname '*.jp*g' | head -1)")"
  cat >"$out/machine.json" <<JSON
{
  "machine": "$(sysctl -n machdep.cpu.brand_string), $(( $(sysctl -n hw.memsize) / 1073741824 )) GB, macOS $(sw_vers -productVersion)",
  "megapixels": $mp,
  "date": "$(date +%Y-%m-%d)"
}
JSON
}
