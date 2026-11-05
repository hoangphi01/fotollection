#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Exports the Performance page's diagrams (docs/diagrams/*.drawio, one page per language:
page 1 English, page 2 Vietnamese) to site/assets/img/diagrams/<name>-<lang>.svg with the
draw.io desktop app's command line, then gives each SVG the website's dark palette.

    python3 -I scripts/site-diagrams.py

It also writes their sizes to site/_data/diagrams.json for the page's <img> tags.
The .drawio files are the source: open them in draw.io, edit, save, run this again. Colours
are the site's light tokens (site/assets/css/site.css); each one maps to its dark token here.
Needs draw.io desktop (Apache-2.0); it is never run by CI, and the SVGs are committed.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "docs" / "diagrams"
OUT = ROOT / "site" / "assets" / "img" / "diagrams"
APP = "/Applications/draw.io.app/Contents/MacOS/draw.io"
LANGS = ("en", "vi")

# Light token -> dark token (site.css `:root` and its dark media query).
DARK = {
    "#fffaee": "#262019",  # surface
    "#efe5cc": "#211b15",  # tint
    "#2b2219": "#f3ead6",  # ink
    "#514435": "#d8cab0",  # ink-2
    "#7a6a55": "#a8977c",  # muted
    "#e2d4b3": "#3b3127",  # line
    "#2c836d": "#c98a52",  # brand
    "#dcebe3": "#3a2c1f",  # brand, soft
    "#b5652a": "#e3a067",  # accent
    "#f4dfcc": "#3a2a1c",  # accent, soft
}

TITLES = {
    "layers": {"en": "Where the work happens: Fotollection against a typical web editor", "vi": "Công việc diễn ra ở đâu: Fotollection so với một trình sửa ảnh web thông thường"},
    "raw-develop": {"en": "A RAW photo, from the file to the print", "vi": "Một ảnh RAW, từ tệp tới ảnh in"},
    "gpu-cpu": {"en": "Editing on the graphics card, exporting on every processor core", "vi": "Chỉnh trên card đồ hoạ, xuất ảnh trên mọi nhân bộ xử lý"},
}


def palette() -> str:
    rules = []
    for light, dark in DARK.items():
        rules.append(f'[fill="{light}"]{{fill:{dark}}}')
        rules.append(f'[stroke="{light}"]{{stroke:{dark}}}')
    return "<style>@media (prefers-color-scheme: dark){" + "".join(rules) + "}</style>"


def finish(svg: str, title: str) -> str:
    # draw.io repeats each colour as an inline style, which a stylesheet cannot override.
    svg = re.sub(r' style="(?:fill|stroke)[^"]*"', "", svg)
    svg = svg.replace("color-scheme: light;", "")
    svg = re.sub(r"<\?xml[^>]*>\s*<!DOCTYPE[^>]*>\s*", "", svg)
    svg = re.sub(r"<svg ", '<svg role="img" aria-label="' + title.replace('"', "&quot;") + '" ', svg, count=1)
    return svg.replace("<defs/>", f"<title>{title}</title>{palette()}<defs/>", 1)


def main() -> int:
    if not Path(APP).exists():
        print(f"draw.io desktop not found at {APP} (brew install --cask drawio)", file=sys.stderr)
        return 1
    OUT.mkdir(parents=True, exist_ok=True)
    sizes = {}
    with tempfile.TemporaryDirectory() as tmp:
        for source in sorted(SRC.glob("*.drawio")):
            name = source.stem
            for page, lang in enumerate(LANGS, start=1):
                raw = Path(tmp) / f"{name}-{lang}.svg"
                subprocess.run(
                    [APP, "-x", "-f", "svg", "-p", str(page), "-b", "16", "--theme", "light",
                     "--embed-svg-fonts", "false", "-o", str(raw), str(source)],
                    check=True, capture_output=True,
                )
                title = TITLES.get(name, {}).get(lang, name)
                svg = finish(raw.read_text(), title)
                (OUT / raw.name).write_text(svg)
                size = re.search(r'width="(\d+)px" height="(\d+)px"', svg)
                if size:
                    sizes[name] = {"width": int(size[1]), "height": int(size[2])}
                print(f"wrote {(OUT / raw.name).relative_to(ROOT)}")
    # Lazy images need their size (LESSONS: a lazy image with no height moves the anchors).
    data = ROOT / "site" / "_data" / "diagrams.json"
    data.write_text(json.dumps(sizes, indent=2, sort_keys=True) + "\n")
    print(f"wrote {data.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
