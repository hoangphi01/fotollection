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


def main() -> None:
    folder = Path(sys.argv[1])
    names = sorted({p.name.split(".")[0] for p in folder.glob("*.png")})
    print(f"{'sample':<26} {'rawler vs OS':^19} {'rawler vs JPEG':^19} {'OS vs JPEG':^19}")
    print(f"{'':<26} {'ΔE   p95   SSIM':^19} {'ΔE   p95   SSIM':^19} {'ΔE   p95   SSIM':^19}")
    for name in names:
        os_, rl, jpg = (load(folder / f"{name}.{k}.png") for k in ("os", "rawler", "embedded"))
        pair = lambda a, b: score(a, b) if a is not None and b is not None else None  # noqa: E731
        print(f"{name:<26} {fmt(pair(rl, os_))} {fmt(pair(rl, jpg))} {fmt(pair(os_, jpg))}")


if __name__ == "__main__":
    main()
