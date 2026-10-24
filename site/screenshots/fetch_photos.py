#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Downloads the website's demo photos (photos.json) from Wikimedia Commons into a folder,
checks each against its pinned SHA-256 and its licence (CC0 or public domain), and stops on
any mismatch.

    python3 -I site/screenshots/fetch_photos.py <empty-folder>

The photos are not kept in the repository; only the screenshots made from them are.
"""

from __future__ import annotations

import hashlib
import json
import sys
import urllib.parse
import urllib.request
from pathlib import Path

API = "https://commons.wikimedia.org/w/api.php"
HEADERS = {"User-Agent": "fotollection-site-demo/0.1 (https://github.com/hoangphi01/fotollection)"}


def get(url: str) -> bytes:
    with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS), timeout=120) as r:
        return r.read()


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    photos = json.loads((Path(__file__).parent / "photos.json").read_text())
    for p in photos:
        target = out / p["file"]
        if target.exists() and hashlib.sha256(target.read_bytes()).hexdigest() == p["sha256"]:
            continue
        query = urllib.parse.urlencode(
            {
                "action": "query",
                "titles": p["title"],
                "prop": "imageinfo",
                "iiprop": "url|extmetadata",
                "format": "json",
                "formatversion": "2",
            }
        )
        info = json.loads(get(f"{API}?{query}"))["query"]["pages"][0]["imageinfo"][0]
        licence = info["extmetadata"]["LicenseShortName"]["value"]
        if "CC0" not in licence and not licence.lower().startswith("public domain"):
            sys.exit(f"{p['title']}: licence is now {licence!r}")
        data = get(info["url"])
        if hashlib.sha256(data).hexdigest() != p["sha256"]:
            sys.exit(f"{p['title']}: checksum differs from photos.json")
        target.write_bytes(data)
        print(p["file"])
    print(f"{len(photos)} demo photos in {out}")


if __name__ == "__main__":
    main()
