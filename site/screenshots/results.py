# SPDX-License-Identifier: GPL-3.0-only
"""The landing page's finished results (FR-135): the prints themselves, at full size.

    uv run python -I site/screenshots/results.py <dump> <lang>

Copies the renders the real app made into the recording <dump> (src-tauri/src/demo.rs: the
postcards, a photo with its date stamp, a two-photo collage sheet, one in its info frame) to
site/assets/img/results/<name>-<lang>.webp, and records in site/_data/results.json each one's
size, so the page gives every picture its width and height (LESSONS: a lazy image with no
height moves the anchors below it), and the rows they are laid in, from their shapes, so no
corner is left empty (owner, 2026-11-15). Never retouched elsewhere.
"""

import json
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]

# Per recording language: (render folder, photo stem or the sheet's two stems) of each result.
# The postcards are the language's demo cards (demo.rs `postcard_demos`); the stamp, the sheet
# and the frame use photos the set shows best, none repeating the postcards' places.
PICKS = {
    "ja": {"card": ("sheet-postcard-full", "IMG_5773"), "card2": ("sheet-postcard", "Gokurakuji"),
           "stamp": ("preview/stamp", "IMG_5745"), "sheet": ("sheet", ("IMG_4329", "IMG_4421")),
           "frame": ("preview/frame", "IMG_5758")},
    "ko": {"card": ("sheet-postcard-full", "DSCF2041"), "card2": ("sheet-postcard", "DSC06332"),
           "stamp": ("preview/stamp", "DSC06335"), "sheet": ("sheet", ("DSCF1972", "DSCF1975")),
           "frame": ("preview/frame", "DSCF2022"), "frame2": ("preview/frame", "DSC06325")},
    "en": {"card": ("sheet-postcard-full", "P1068073"), "card2": ("sheet-postcard", "P1067865"),
           "stamp": ("preview/stamp", "P1067865"), "sheet": ("sheet", ("P1067819", "P1067829")),
           "frame": ("preview/frame", "P1068073")},
}
PICKS["vi"] = PICKS["en"]

# Who took each photo, by file name (credited under each print, owner 2026-11-15; the page's
# fn_photos note says the rest). The camera comes from the photo's EXIF in the recording.
def author(stem: str) -> str:
    if stem.startswith("DSCF"):
        return "@9adox"
    if stem.startswith(("IMG_14", "DSC0")):  # ZV-1 files: DSC0…, the phone's IMG_14…
        return "@imyourctus"
    return "@hoangphi01"


def source(dump: Path, ids: dict, folder: str, stem) -> Path:
    if isinstance(stem, tuple):
        a, b = (ids[s] for s in stem)
        for name in (f"{a}-{b}.jpg", f"{b}-{a}.jpg"):
            if (dump / "img" / folder / name).exists():
                return dump / "img" / folder / name
        sys.exit(f"results.py: no sheet of {stem[0]} and {stem[1]} in the recording")
    return dump / "img" / folder / f"{ids[stem]}.jpg"


def rows(sizes: dict) -> list:
    """The prints after the postcard in rows of columns, by shape, so no corner stays empty:
    upright ones two to a row, wide ones two to a row (a second frame beside the first); one
    upright left over sits beside two wide ones stacked, one wide alone takes its row."""
    names = [n for n in ("stamp", "sheet", "frame", "frame2") if n in sizes]
    tall = [n for n in names if sizes[n][1] > sizes[n][0]]
    wide = [n for n in names if n not in tall]
    out = [[[a], [b]] for a, b in zip(tall[0::2], tall[1::2])]
    pairs = [[[a], [b]] for a, b in zip(wide[0::2], wide[1::2])]
    if len(tall) % 2 and len(wide) >= 2:
        out.append([[tall[-1]], wide[:2]])
        wide = wide[2:]
        pairs = [[[a], [b]] for a, b in zip(wide[0::2], wide[1::2])]
    elif len(tall) % 2:
        out.append([[tall[-1]]])
    out += pairs
    if len(wide) % 2:
        out.append([[wide[-1]]])
    return out


def main() -> None:
    dump, lang = Path(sys.argv[1]), sys.argv[2]
    fixtures = json.loads((dump / "fixtures.json").read_text())
    ids = {p["fileName"].rsplit(".", 1)[0]: p["id"] for p in fixtures["common"]["list_photos"]["photos"]}
    out = ROOT / "site/assets/img/results"
    out.mkdir(parents=True, exist_ok=True)
    data_file = ROOT / "site/_data/results.json"
    data = json.loads(data_file.read_text()) if data_file.exists() else {}
    cameras = {p["fileName"].rsplit(".", 1)[0]: p.get("camera") or "" for p in fixtures["common"]["list_photos"]["photos"]}
    sizes, credits = {}, {}
    for name, (folder, stem) in PICKS[lang].items():
        first = stem[0] if isinstance(stem, tuple) else stem
        credits[name] = " · ".join(x for x in (author(first), cameras.get(first, "")) if x)
        for s in stem if isinstance(stem, tuple) else (stem,):
            if s not in ids:
                sys.exit(f"results.py: {lang}: {s} is not in the recording")
        image = Image.open(source(dump, ids, folder, stem)).convert("RGB")
        image.save(out / f"{name}-{lang}.webp", quality=84, method=6)
        sizes[name] = list(image.size)
    data[lang] = {"sizes": sizes, "credits": credits, "rows": rows(sizes)}
    data_file.write_text(json.dumps(dict(sorted(data.items())), indent=1) + "\n")
    print(f"results ({lang}): {', '.join(f'{k} {w}×{h}' for k, (w, h) in sizes.items())}; rows {data[lang]['rows']}")


if __name__ == "__main__":
    main()
