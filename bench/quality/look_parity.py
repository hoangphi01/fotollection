#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Scores the GPU look against Rust's (ADR-0039): CIEDE2000 between cpu.png and each
gpu-<engine>.png written by bench/gpu/parity.mjs, mean, 99th percentile and maximum, over
the photo's own pixels (mask.png) and over the whole image. Budget: mean and p99 ≤ 1.

    uv run python -I bench/quality/look_parity.py <dir>

Prints one JSON object; bench/results/gpu.json keeps it as `parity`.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
from audit import ciede2000, to_lab  # noqa: E402


def main() -> None:
    folder = Path(sys.argv[1])
    cpu = Image.open(folder / "cpu.png").convert("RGB")
    mask = np.asarray(Image.open(folder / "mask.png").convert("L")) > 127
    lab_cpu = to_lab(cpu, None)
    out = {}
    for gpu_path in sorted(folder.glob("gpu-*.png")):
        gpu = Image.open(gpu_path).convert("RGB")
        de = ciede2000(lab_cpu, to_lab(gpu, None))
        photo = de[mask]
        out[gpu_path.stem.removeprefix("gpu-")] = {
            "photoMean": round(float(photo.mean()), 3),
            "photoP99": round(float(np.percentile(photo, 99)), 3),
            "photoMax": round(float(photo.max()), 3),
            "allMean": round(float(de.mean()), 3),
            "allP99": round(float(np.percentile(de, 99)), 3),
        }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
