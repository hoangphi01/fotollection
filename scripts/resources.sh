#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Disk, memory and build-folder check, run before, during and after builds and tests
# (docs/DOCS_GUIDE.md §3). Exits non-zero when there is too little room to build safely.
#
#   scripts/resources.sh            # report; fail if free disk < 10 GB
#   scripts/resources.sh --clean    # also run `cargo clean` when target/ is larger than 12 GB
# Portable to macOS bash 3.2.
set -uo pipefail
cd "$(dirname "$0")/.."

MIN_FREE_GB=10
MAX_TARGET_GB=12

free_kb=$(df -k . | awk 'NR==2 {print $4}')
free_gb=$((free_kb / 1024 / 1024))
target_kb=$(du -sk target 2>/dev/null | awk '{print $1}')
target_gb=$(( ${target_kb:-0} / 1024 / 1024 ))
echo "disk free: ${free_gb} GB · target/: ${target_gb} GB"
if command -v memory_pressure >/dev/null; then
  memory_pressure | tail -n 1
  sysctl -n vm.swapusage 2>/dev/null | sed 's/^/swap: /'
fi

if [ "${1:-}" = "--clean" ] && [ "$target_gb" -gt "$MAX_TARGET_GB" ]; then
  echo "target/ is over ${MAX_TARGET_GB} GB: cargo clean"
  cargo clean -q
  free_kb=$(df -k . | awk 'NR==2 {print $4}')
  free_gb=$((free_kb / 1024 / 1024))
  echo "disk free after clean: ${free_gb} GB"
fi

if [ "$free_gb" -lt "$MIN_FREE_GB" ]; then
  echo "less than ${MIN_FREE_GB} GB free: clean before building (scripts/resources.sh --clean, scripts/clean.sh)" >&2
  exit 1
fi
