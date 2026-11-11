#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Writes each diagram's pages for the website's other languages (site/README.md) from its
English page and a table of words:

    python3 -I scripts/diagram-pages.py

docs/diagrams/<name>.drawio has page 1 in English and page 2 in Vietnamese, both drawn by
hand in draw.io. Every other language gets a page made here: English's page with each label
replaced from docs/diagrams/labels/<code>.json (English label -> that language's), named by
the code. Edit the English page in draw.io, add any new label to each table, and run this,
then scripts/site-diagrams.py. A label missing from a table stops it, naming the label.
"""

from __future__ import annotations

import html
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "docs" / "diagrams"
LABELS = SRC / "labels"


def main() -> int:
    tables = {p.stem: json.loads(p.read_text(encoding="utf-8")) for p in sorted(LABELS.glob("*.json"))}
    missing = []
    for drawio in sorted(SRC.glob("*.drawio")):
        text = drawio.read_text(encoding="utf-8")
        pages = re.findall(r"<diagram\b.*?</diagram>", text, re.S)
        # The hand-drawn pages stay: English (1) and Vietnamese (2); the rest are made again.
        kept = pages[:2]
        english = pages[0]
        made = []
        for code, table in tables.items():
            def swap(m: re.Match[str]) -> str:
                label = html.unescape(m.group(1))
                if not label.strip():
                    return m.group(0)
                if label not in table:
                    missing.append(f"{drawio.name}: no {code} words for {label!r}")
                    return m.group(0)
                return 'value="' + html.escape(table[label], quote=True).replace("\n", "&#10;") + '"'

            page = re.sub(r'\bvalue="([^"]*)"', swap, english)
            page = re.sub(r'<diagram id="([^"]+)-0" name="EN">', rf'<diagram id="\1-{code}" name="{code.upper()}">', page, count=1)
            made.append(page)
        head = text[: text.index(pages[0])]
        tail = text[text.index(pages[-1]) + len(pages[-1]) :]
        drawio.write_text(head + "".join(kept + made) + tail, encoding="utf-8")
    if missing:
        print("\n".join(missing), file=sys.stderr)
        return 1
    print(f"{len(tables)} languages written into {len(list(SRC.glob('*.drawio')))} diagrams")
    return 0


if __name__ == "__main__":
    sys.exit(main())
