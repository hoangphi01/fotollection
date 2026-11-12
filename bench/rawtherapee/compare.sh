#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-only
# Highlight recovery, Fotollection against RawTherapee, on the same frames with the same truth
# and the same scorer (RESULT §41). Each raw is developed whole and with its sensor clipped at
# 60 % of white (RawTherapee: PreExposure 1/0.6 and the exposure brought back), two stops
# down, as 16-bit sRGB; score.py scores both alike. Needs rawtherapee-cli (5.13 measured),
# uv, and a release build of highlights_bench. Writes <out>/rt.jsonl and <out>/ours.jsonl.
#
#   bench/rawtherapee/compare.sh <out-folder> <raw>…
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out=$1
shift
mkdir -p "$out/pp3"
score() { uv run -q --no-project --with numpy --with tifffile python -I "$here/score.py" "$@"; }

# RawTherapee's own neutral profile, written by itself, then one profile per run.
first=$1
rawtherapee-cli -q -Y -O "$out/neutral.tif" -t -b16 -c "$first" >/dev/null
mv "$out/neutral.tif.pp3" "$out/pp3/base.pp3"
rm -f "$out/neutral.tif"
cut=$(python3 -I -c "import math;print(-2-math.log2(1/0.6))")
profile() { # name pre-exposure compensation enabled method
  python3 -I - "$out/pp3" "$@" <<'EOF'
import re, sys
d, name, pre, comp, on, method = sys.argv[1:7]
t = open(f"{d}/base.pp3").read()
t = re.sub(r"(\[Exposure\][^\[]*?)Compensation=[^\n]*", r"\g<1>Compensation=" + comp, t, count=1, flags=re.S)
t = re.sub(r"(\[HLRecovery\]\n)Enabled=[^\n]*\nMethod=[^\n]*", r"\g<1>Enabled=" + on + "\nMethod=" + method, t)
t = re.sub(r"PreExposure=[^\n]*", "PreExposure=" + pre, t)
open(f"{d}/{name}.pp3", "w").write(t)
EOF
}
profile truth_off 1 -2 false Coloropp
profile truth_opp 1 -2 true Coloropp
profile cut_off 1.6666667 "$cut" false Coloropp
for m in Luminance "CIELab blending" Color Blend Coloropp; do
  profile "cut_${m// /_}" 1.6666667 "$cut" true "$m"
done

: >"$out/rt.jsonl"
: >"$out/ours.jsonl"
for raw in "$@"; do
  n=$(basename "$raw")
  d="$out/rt/$n"
  mkdir -p "$d"
  for pp in "$out"/pp3/*.pp3; do
    p=$(basename "$pp" .pp3)
    [ "$p" = base ] && continue
    /usr/bin/time -l rawtherapee-cli -q -Y -o "$d/$p.tif" -p "$pp" -t -b16 -c "$raw" 2>"$d/$p.time" >/dev/null
    echo "$n $p $(awk '/real/{print $1" s"} /maximum resident/{printf "%.0f MB\n", $1/1048576}' "$d/$p.time" | tr '\n' ' ')"
  done
  echo "$n $(score "$d" off Luminance CIELab_blending Color Blend Coloropp)" >>"$out/rt.jsonl"
  rm -f "$d"/*.tif
  if "$root/target/release/examples/highlights_bench" --export "$out/ours" "$raw" >/dev/null; then
    echo "$n $(score "$out/ours/$n" off Global Regions Local)" >>"$out/ours.jsonl"
    rm -f "$out/ours/$n"/*.ppm
  fi
done
