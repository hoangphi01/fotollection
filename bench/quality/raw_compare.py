#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Scores rawler against the OS codecs for camera RAW (ADR-0029, Step 0): for each sample
written by `cargo run --release -p fotollection-core --example measure_rawler`, the colour
difference (CIEDE2000, mean and 95th percentile) and the structure (SSIM of L*) between

- rawler and the OS develop (ImageIO on macOS), and
- each develop and the camera's own embedded JPEG (the look the camera intended).

    uv run bench/quality/raw_compare.py <folder>

All images are 1600 px PNGs read as sRGB. A pair whose shapes differ by more than 1 % (an
embedded preview cropped to another ratio) is skipped.

The Panasonic S5 sample shows a Datacolor SpyderCheckr 24. Its makers publish no reliable
reference values (and the card changed in 2023), so its colour patches are not scored
against a standard. Its six grey patches must be neutral whatever the card: their chroma
C* measures white balance and colour casts, and their lightness L* the tone curve.
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
from audit import ciede2000, ssim, to_lab  # noqa: E402


def load(path: Path) -> Image.Image | None:
    return Image.open(path).convert("RGB") if path.exists() else None


def score(a: Image.Image, b: Image.Image) -> tuple[float, float, float] | None:
    if abs(a.width / a.height - b.width / b.height) > 0.01:
        return None
    b = b.resize(a.size, Image.Resampling.LANCZOS)
    la, lb = to_lab(a, None), to_lab(b, None)
    de = ciede2000(la, lb)
    return float(de.mean()), float(np.percentile(de, 95)), ssim(la[..., 0], lb[..., 0])


def fmt(s: tuple[float, float, float] | None) -> str:
    return "      —            " if s is None else f"{s[0]:5.2f} {s[1]:5.2f} {s[2]:.3f}"


# Centres of the S5 sample's six grey patches (white to black) in its 1600 px image; each
# box is 40 px, well inside a patch of about 100 px, so small shifts between decoders
# (lens correction) stay inside.
S5_GREYS = [(487, 750), (612, 750), (737, 750), (862, 750), (987, 750), (1107, 750)]


def greys(img: Image.Image) -> list[tuple[float, float]]:
    lab = to_lab(img, None)
    out = []
    for x, y in S5_GREYS:
        patch = lab[y - 20 : y + 20, x - 20 : x + 20].reshape(-1, 3).mean(axis=0)
        out.append((float(patch[0]), float(np.hypot(patch[1], patch[2]))))
    return out


DECODERS = ("os", "rawler", "rawcore")


def main() -> None:
    folder = Path(sys.argv[1])
    names = sorted({p.name.split(".")[0] for p in folder.glob("*.png")})
    print(f"{'sample':<24} {'decoder':<8} {'vs JPEG: ΔE  p95  SSIM':<24} {'vs OS: ΔE  p95  SSIM':<22}")
    for name in names:
        jpg, os_ = load(folder / f"{name}.embedded.png"), load(folder / f"{name}.os.png")
        for kind in DECODERS:
            img = load(folder / f"{name}.{kind}.png")
            if img is None:
                continue
            vs_jpg = score(img, jpg) if jpg is not None else None
            vs_os = score(img, os_) if os_ is not None and kind != "os" else None
            print(f"{name:<24} {kind:<8} {fmt(vs_jpg):<24} {fmt(vs_os):<22}")
    s5 = "panasonic-dc-s5"
    print("\nGrey patches of the S5 sample (L* white → black; C* = colour cast, 0 is neutral)")
    for kind in (*DECODERS, "embedded"):
        img = load(folder / f"{s5}.{kind}.png")
        if img is None or img.size != (1600, 1067):
            continue
        g = greys(img)
        ls = " ".join(f"{l:5.1f}" for l, _ in g)
        cs = sum(c for _, c in g) / len(g)
        print(f"  {kind:<9} L* {ls}   mean C* {cs:4.2f}   max C* {max(c for _, c in g):4.2f}")


if __name__ == "__main__":
    main()
