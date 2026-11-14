#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds AddAssist's bundled tables (crates/core/assets/addassist/, FR-133). The result is
# pinned by the BLAKE3 sum the packer prints (see ADDASSIST.md). The app itself never
# downloads anything.
#
#   scripts/addassist/fetch-and-pack.sh jp
#       Japan Post's postal-code data (UTF-8 one-record-per-line edition), downloaded here.
#   scripts/addassist/fetch-and-pack.sh kr <folder> <YYYY-MM-DD>
#       Korea's 도로명주소 한글 전체분, downloaded by hand from business.juso.go.kr (주소 DB,
#       속성 › 도로명주소 한글, the month's 전체분) and unzipped into <folder>; its files are
#       CP949, turned to UTF-8 by iconv on the way in. The date is the data's month end.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
assets="$root/crates/core/assets/addassist"

case "${1:-}" in
  jp)
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    url="https://www.post.japanpost.jp/service/search/zipcode/download/utf/zip/utf_ken_all.zip"
    echo "downloading Japan Post's utf_ken_all.zip" >&2
    curl -sSfL -o "$work/utf_ken_all.zip" "$url"
    unzip -q -o "$work/utf_ken_all.zip" utf_ken_all.csv -d "$work"
    # The data's own date: the CSV's date inside the zip (MM-DD-YYYY).
    d="$(unzip -l "$work/utf_ken_all.zip" utf_ken_all.csv | awk 'NR==4 {print $2}')"
    snapshot="${d:6:4}-${d:0:2}-${d:3:2}"
    shasum -a 256 "$work/utf_ken_all.zip" >&2
    cargo run -q --release -p fotollection-addassistdata -- jp "$work/utf_ken_all.csv" "$snapshot" "$assets/jp.bin"
    ;;
  kr)
    folder="${2:?the folder of the unzipped 도로명주소 한글 전체분}"
    snapshot="${3:?the date of the data, YYYY-MM-DD}"
    cargo build -q --release -p fotollection-addassistdata
    for f in "$folder"/rnaddrkor_*.txt; do iconv -f CP949 -t UTF-8 "$f"; done |
      "$root/target/release/fotollection-addassistdata" kr - "$snapshot" "$assets/kr.bin"
    ;;
  *)
    echo "usage: $0 jp | kr <folder> <YYYY-MM-DD>" >&2
    exit 2
    ;;
esac
echo "Update the BLAKE3 sum in ADDASSIST.md and crates/core/src/addassist/$1.rs." >&2
