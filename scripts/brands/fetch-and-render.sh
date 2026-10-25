#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the bundled brand logos (crates/core/assets/brands, ADR-0020) from Wikimedia
# Commons. Every SVG is checked against a pinned SHA-256 before it is rendered. macOS only
# (AppKit renders the SVG). The app itself never downloads anything.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/crates/core/assets/brands"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# brand  url  sha256  height-px
logos=(
  "apple https://upload.wikimedia.org/wikipedia/commons/f/fa/Apple_logo_black.svg 9d00ea77a3240f291356c36261c5f45d7fa456f29c97a2f060caf0ed4b9c3231 512"
  "canon https://upload.wikimedia.org/wikipedia/commons/0/0a/Canon_wordmark.svg 9b6acea0a1a760992b23754c0be41eaaa2fdac8d9b75ae3cc5218a1ade278192 320"
  "fujifilm https://upload.wikimedia.org/wikipedia/commons/a/a1/Fujifilm_logo.svg d86cf1e0208b9898846f9ccd93bc514be3130fcb64492e5fc915a4e08d7dd8f2 320"
  "hasselblad https://upload.wikimedia.org/wikipedia/commons/6/6b/Hasselblad_Logo.svg 631f89027756186cf900a1af1aceb8496f8aa120e881a828ad4c9ebfc425cb9b 320"
  "leica https://upload.wikimedia.org/wikipedia/commons/e/ec/Leica_Camera_logo.svg 5fcb8f72ae1640fa4475765ea16383dc8a2b0e4b4973057f88b5cdd8063e18bb 512"
  "lumix https://upload.wikimedia.org/wikipedia/commons/3/3e/Lumix_logo.svg affd3d5e35050c3a48c8060656a6f0e299349a90b25e8e8d19fd34ef1b936446 320"
  "nikon https://upload.wikimedia.org/wikipedia/commons/f/f3/Nikon_Logo.svg 449bab29e259c46b7264ed19886e5a0e2ef567250d01163c8e2cd1bbce486c53 512"
  "noritsu https://upload.wikimedia.org/wikipedia/commons/f/fb/Noritsu_logo.svg 19a90f835d85c9b3e441eee9c34b384c0280165509e045f6ce6573279fdba13d 320"
  "phaseone https://upload.wikimedia.org/wikipedia/commons/0/0a/Phase_one_logo.svg 133d0b1459fa967a5c1c1e0212cb09f2b3826242273041da3288e14c970cbb75 320"
  "ricoh https://upload.wikimedia.org/wikipedia/commons/8/8f/Ricoh_logo_2012.svg be0ae0727d7d94dd19c583f7fef7900ea3035f18d20f85f16526b3e0d566a56a 320"
  "samsung https://upload.wikimedia.org/wikipedia/commons/b/b4/Samsung_wordmark.svg 7c1a09ef9076d86b6a0ce37b60dac50f666db4abf016f5dea7ae790b082f2575 320"
  "sony https://upload.wikimedia.org/wikipedia/commons/c/ca/Sony_logo.svg f6191bc69f8df259c86a13dc9582ba9ed2e0101d4452ba3f1d33ec8e907ddda0 320"
)

for row in "${logos[@]}"; do
  read -r brand url sha height <<<"$row"
  curl -sSfL -A "fotollection-build (https://github.com/hoangphi01/fotollection)" -o "$work/$brand.svg" "$url"
  echo "$sha  $work/$brand.svg" | shasum -a 256 -c - >/dev/null || { echo "checksum mismatch: $brand" >&2; exit 1; }
  if grep -qiE '<script|onload=|<foreignObject|href="http' "$work/$brand.svg"; then
    echo "refusing $brand: script or external reference" >&2; exit 1
  fi
  cp "$work/$brand.svg" "$out/src/$brand.svg"
  swift "$root/scripts/brands/render.swift" "$out/src/$brand.svg" "$out/$brand.png" "$height"
done
echo "rendered ${#logos[@]} logos into $out" >&2
