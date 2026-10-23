#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rust vs browser benchmark (docs/OPTIMIZATION.md §2). macOS only for now (peak RSS via
# /usr/bin/time -l). Photos are read from <photo-folder> and never copied into the repo.
#
#   bench/run.sh <photo-folder> [runs=3]
set -euo pipefail

folder="${1:?usage: bench/run.sh <photo-folder> [runs]}"
runs="${2:-3}"
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/bench/results"
mkdir -p "$out"
rm -f "$out"/rust-*.json "$out"/web-*.json

cargo build --release -q -p fotollection-core --example measure_folder --manifest-path "$root/Cargo.toml"
npx --prefix "$root" playwright install chromium >/dev/null

for i in $(seq "$runs"); do
  echo "run $i/$runs: rust" >&2
  err="$(mktemp)"
  /usr/bin/time -l "$root/target/release/examples/measure_folder" "$folder" --json >"$out/rust-$i.raw" 2>"$err"
  rss="$(awk '/maximum resident set size/ {print $1}' "$err")"
  node -e 'const f=process.argv[1],d=JSON.parse(require("fs").readFileSync(f+".raw"));d.peakRssBytes=+process.argv[2];require("fs").writeFileSync(f+".json",JSON.stringify(d,null,2)+"\n")' "$out/rust-$i" "$rss"
  rm -f "$out/rust-$i.raw" "$err"
  echo "run $i/$runs: browser" >&2
  node "$root/bench/web/run.mjs" "$folder" >"$out/web-$i.json"
done

# Machine facts for the chart footer; no paths or file names.
mp="$(node -e 'const s=require("child_process").execFileSync("sips",["-g","pixelWidth","-g","pixelHeight",process.argv[1]]).toString();const [w,h]=[...s.matchAll(/: (\d+)/g)].map(m=>+m[1]);console.log(Math.round(w*h/1e6))' "$(find "$folder" -maxdepth 1 -iname '*.jp*g' | head -1)")"
cat >"$out/machine.json" <<EOF
{
  "machine": "$(sysctl -n machdep.cpu.brand_string), $(( $(sysctl -n hw.memsize) / 1073741824 )) GB, macOS $(sw_vers -productVersion)",
  "megapixels": $mp,
  "date": "$(date +%Y-%m-%d)"
}
EOF

node "$root/bench/chart.mjs"
