# SPDX-License-Identifier: GPL-3.0-only
"""The landing page's finished results (FR-135): the prints themselves, at full size.

    uv run python -I site/screenshots/results.py <dump> <lang>

Copies four renders the real app made into the recording <dump> (src-tauri/src/demo.rs: the
postcards, a photo with its date stamp, one in its info frame) to
site/assets/img/results/<name>-<lang>.webp, and records each one's size in
site/_data/results.json, so the page gives every picture its width and height (LESSONS: a lazy
image with no height moves the anchors below it). Never retouched elsewhere.
"""

import json
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]

# Per recording language: (render folder, photo stem) of each result. The postcards are the
# language's demo cards (demo.rs `postcard_demos`); the stamp and the frame use photos the
# set shows best.
PICKS = {
    "ja": {"card": ("sheet-postcard-full", "IMG_5773"), "card2": ("sheet-postcard", "Gokurakuji"),
           "stamp": ("preview/stamp", "IMG_5745"), "frame": ("preview/frame", "IMG_4329")},
    "ko": {"card": ("sheet-postcard-full", "DSCF2041"), "card2": ("sheet-postcard", "DSCF1990"),
           "stamp": ("preview/stamp", "IMG_1469"), "frame": ("preview/frame", "DSCF2022")},
    "en": {"card": ("sheet-postcard-full", "P1068073"), "card2": ("sheet-postcard", "P1067865"),
           "stamp": ("preview/stamp", "P1067865"), "frame": ("preview/frame", "P1068073")},
}
PICKS["vi"] = PICKS["en"]


def main() -> None:
    dump, lang = Path(sys.argv[1]), sys.argv[2]
    fixtures = json.loads((dump / "fixtures.json").read_text())
    ids = {p["fileName"].rsplit(".", 1)[0]: p["id"] for p in fixtures["common"]["list_photos"]["photos"]}
    out = ROOT / "site/assets/img/results"
    out.mkdir(parents=True, exist_ok=True)
    data_file = ROOT / "site/_data/results.json"
    data = json.loads(data_file.read_text()) if data_file.exists() else {}
    sizes = {}
    for name, (folder, stem) in PICKS[lang].items():
        if stem not in ids:
            sys.exit(f"results.py: {lang}: {stem} is not in the recording")
        source = dump / "img" / folder / f"{ids[stem]}.jpg"
        image = Image.open(source).convert("RGB")
        image.save(out / f"{name}-{lang}.webp", quality=84, method=6)
        sizes[name] = list(image.size)
    data[lang] = sizes
    data_file.write_text(json.dumps(dict(sorted(data.items())), indent=2) + "\n")
    print(f"results ({lang}): {', '.join(f'{k} {w}×{h}' for k, (w, h) in sizes.items())}")


if __name__ == "__main__":
    main()
