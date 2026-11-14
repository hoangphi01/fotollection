#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds AddAssist's bundled Japanese table (crates/core/assets/addassist/jp.bin, FR-133)
# from Japan Post's postal-code data, UTF-8 one-record-per-line edition. Japan Post updates
# it monthly; the result is pinned by the BLAKE3 sum the packer prints (see ADDASSIST.md).
# The app itself never downloads anything.
#
#   scripts/addassist/fetch-and-pack.sh [out=crates/core/assets/addassist/jp.bin]
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:-$root/crates/core/assets/addassist/jp.bin}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

url="https://www.post.japanpost.jp/service/search/zipcode/download/utf/zip/utf_ken_all.zip"
echo "downloading Japan Post's utf_ken_all.zip" >&2
curl -sSfL -o "$work/utf_ken_all.zip" "$url"
unzip -q -o "$work/utf_ken_all.zip" utf_ken_all.csv -d "$work"
# The data's own date: the CSV's date inside the zip.
snapshot="$(unzip -l "$work/utf_ken_all.zip" utf_ken_all.csv | awk 'NR==4 {print $2}')"
case "$snapshot" in
  [0-9][0-9]-[0-9][0-9]-[0-9][0-9][0-9][0-9]) snapshot="${snapshot:6:4}-${snapshot:0:2}-${snapshot:3:2}" ;;
esac
shasum -a 256 "$work/utf_ken_all.zip" >&2

cargo run -q --release -p fotollection-addassistdata -- jp "$work/utf_ken_all.csv" "$snapshot" "$out"
echo "written to $out. Update the BLAKE3 sum in ADDASSIST.md and crates/core/src/addassist/jp.rs." >&2
