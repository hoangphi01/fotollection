#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the bundled offline places (crates/core/assets/geo/places.bin, ADR-0003) from a
# fresh GeoNames dump. GeoNames updates its dumps daily, so the result is pinned by the
# BLAKE3 sum the packer prints (see GEODATA.md), not by the download. The app itself never
# downloads anything.
#
#   scripts/geodata/fetch-and-pack.sh [out=crates/core/assets/geo/places.bin]
#   scripts/geodata/fetch-and-pack.sh countries   # RollSense's country outlines (FR-141)
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
if [ "${1:-}" = countries ]; then
  # Natural Earth 1:50m admin-0 countries (public domain), pinned to a release.
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' EXIT
  curl -sSfL -o "$work/countries.geojson" \
    "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/v5.1.2/geojson/ne_50m_admin_0_countries.geojson"
  cargo run -q --release -p fotollection-geodata --bin countries -- \
    "$work/countries.geojson" "$root/crates/core/assets/geo/countries.bin"
  echo "Update the BLAKE3 sum in GEODATA.md and crates/core/src/geo_shapes.rs." >&2
  exit 0
fi
out="${1:-$root/crates/core/assets/geo/places.bin}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

base="https://download.geonames.org/export/dump"
echo "downloading GeoNames cities1000 and countryInfo" >&2
curl -sSfL -o "$work/cities1000.zip" "$base/cities1000.zip"
curl -sSfL -o "$work/countryInfo.txt" "$base/countryInfo.txt"
unzip -q -o "$work/cities1000.zip" cities1000.txt -d "$work"

cargo run -q --release -p fotollection-geodata -- \
  "$work/cities1000.txt" "$work/countryInfo.txt" "$(date -u +%F)" "$out"
echo "written to $out. Update the BLAKE3 sum in GEODATA.md and crates/core/src/geocode.rs." >&2
