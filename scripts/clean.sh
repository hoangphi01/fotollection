#!/usr/bin/env bash
# Reclaim disk space from build artifacts. See docs/DEVELOPMENT.md §4.
#
# Usage: scripts/clean.sh [--dry-run] [sweep|release|full|deep]
#   sweep   (default) remove artifacts unused for 7 days / from old toolchains
#   release remove release-profile artifacts and bundles
#   full    remove all of target/, ui/dist, Vite cache
#   deep    full + ui/node_modules + trim ~/.cargo cache
set -euo pipefail

cd "$(dirname "$0")/.."

dry_run=0
if [ "${1:-}" = "--dry-run" ]; then
  dry_run=1
  shift
fi
level="${1:-sweep}"
sweep_days=7

size_of() {
  if [ -e "$1" ]; then du -sh "$1" 2>/dev/null | cut -f1; else echo "0"; fi
}

remove() {
  for path in "$@"; do
    [ -e "$path" ] || continue
    if [ "$dry_run" -eq 1 ]; then
      echo "  would remove $path ($(size_of "$path"))"
    else
      echo "  removing $path ($(size_of "$path"))"
      rm -rf "$path"
    fi
  done
}

echo "target/ before: $(size_of target)"

case "$level" in
  sweep)
    if ! command -v cargo-sweep >/dev/null 2>&1; then
      echo "cargo-sweep not installed: cargo install cargo-sweep" >&2
      exit 1
    fi
    [ -d target ] || { echo "nothing to sweep"; exit 0; }
    if [ "$dry_run" -eq 1 ]; then
      cargo sweep --dry-run --time "$sweep_days"
      cargo sweep --dry-run --installed
    else
      cargo sweep --time "$sweep_days"
      cargo sweep --installed
    fi
    ;;
  release)
    remove target/release target/*/release
    ;;
  full)
    remove target ui/dist ui/node_modules/.vite
    ;;
  deep)
    remove target ui/dist ui/node_modules
    if command -v cargo-cache >/dev/null 2>&1; then
      if [ "$dry_run" -eq 1 ]; then
        echo "  would run: cargo cache --autoclean"
      else
        cargo cache --autoclean
      fi
    else
      echo "  (cargo-cache not installed; skipping ~/.cargo trim)"
    fi
    ;;
  *)
    echo "unknown level: $level (expected sweep|release|full|deep)" >&2
    exit 2
    ;;
esac

echo "target/ after: $(size_of target)"
