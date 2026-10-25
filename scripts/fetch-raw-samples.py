#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Downloads the camera RAW samples used to test and measure RAW input (FR-016) from
raw.pixls.us into a folder, checks each one's licence (CC0) against the site's current list
and its pinned SHA-256 (bench/raw-samples.json), and stops on any mismatch.

    python3 -I scripts/fetch-raw-samples.py <folder>

About 270 MB. The samples are never kept in the repository.
"""

from __future__ import annotations

import hashlib
import json
import sys
import urllib.parse
import urllib.request
from pathlib import Path

SITE = "https://raw.pixls.us"
HEADERS = {"User-Agent": "fotollection-raw-samples/0.1 (https://github.com/hoangphi01/fotollection)"}


def get(url: str) -> bytes:
    with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS), timeout=300) as r:
        return r.read()


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    samples = json.loads((Path(__file__).parent.parent / "bench/raw-samples.json").read_text())
    # The site's own list says which files are CC0; a licence change stops the download.
    rows = json.loads(get(f"{SITE}/json/getrepository.php?set=all"))["data"]
    cc0 = {row[7].split("getfile.php/")[1].split("/")[0] for row in rows if "publicdomain/zero" in row[5]}
    for s in samples:
        if str(s["id"]) not in cc0:
            sys.exit(f"{s['file']}: no longer listed as CC0 on {SITE}")
        target = out / s["file"]
        if target.exists() and hashlib.sha256(target.read_bytes()).hexdigest() == s["sha256"]:
            continue
        data = get(f"{SITE}/getfile.php/{s['id']}/nice/{urllib.parse.quote(s['name'])}")
        if hashlib.sha256(data).hexdigest() != s["sha256"]:
            sys.exit(f"{s['file']}: checksum differs from bench/raw-samples.json")
        target.write_bytes(data)
        print(f"{s['file']} ({s['make']} {s['model']}, {s['megapixels']} MP)")
    print(f"{len(samples)} RAW samples in {out}")


if __name__ == "__main__":
    main()
