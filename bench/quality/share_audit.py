#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Quality of the share export (FR-053), next to the usual way of posting a photo.

    python3 bench/quality/share_audit.py <source-folder> <share-folder> [--json out.json]

The share folder is what `quality_export --share <preset>` wrote. For each photo:

- reference: the source in sRGB (through its ICC profile), resized to the export's size
  with Lanczos and never compressed: the best that size can look;
- Fotollection: the exported file, compared with the reference;
- after Facebook: both the export and the usual way (a web editor saving 2000 px at q80,
  4:2:0, profile dropped) recompressed as Facebook does (q71, 4:2:0, at most 2048 px),
  then compared with the reference at the export's size.

Measures are those of audit.py: mean and 95th-percentile CIEDE2000, chroma kept, contrast
kept, detail kept (variance of the Laplacian of L*), tonal levels kept, plus file sizes.
"""

from __future__ import annotations

import io
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageOps

sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit import compare, to_srgb  # noqa: E402  (same folder)


def jpeg(img: Image.Image, quality: int, max_edge: int | None = None) -> Image.Image:
    img = img.convert("RGB")
    if max_edge and max(img.size) > max_edge:
        img = img.copy()
        img.thumbnail((max_edge, max_edge), Image.Resampling.BICUBIC)
    buf = io.BytesIO()
    img.save(buf, "JPEG", quality=quality, subsampling=2)  # 4:2:0, no profile
    buf.seek(0)
    return Image.open(buf).convert("RGB")


def facebook(img: Image.Image) -> Image.Image:
    return jpeg(img, 71, 2048)


def main() -> None:
    src_dir, share_dir = Path(sys.argv[1]), Path(sys.argv[2])
    json_path = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    rows = []
    for out_path in sorted(share_dir.rglob("*.jpg")):
        src_path = next(src_dir.glob(out_path.stem + ".*"), None)
        if src_path is None:
            continue
        src_img = Image.open(src_path)
        icc_src = src_img.info.get("icc_profile")
        src = ImageOps.exif_transpose(src_img)
        out = Image.open(out_path)
        icc_out = out.info.get("icc_profile")
        reference = to_srgb(src, icc_src).resize(out.size, Image.Resampling.LANCZOS)
        fot = compare(reference, out, None, icc_out)
        size = out.size
        fot_fb = compare(reference, facebook(out).resize(size, Image.Resampling.LANCZOS), None, None)
        # The usual way: profile ignored, 2000 px q80, then Facebook.
        web = jpeg(src.convert("RGB"), 80, 2000)
        web_fb = compare(reference, facebook(web).resize(size, Image.Resampling.LANCZOS), None, None)
        rows.append(
            {
                "photo": src_path.name,
                "size": list(size),
                "source_bytes": src_path.stat().st_size,
                "export_bytes": out_path.stat().st_size,
                "fotollection": fot,
                "fotollection_after_facebook": fot_fb,
                "usual_after_facebook": web_fb,
            }
        )
    keys = ["de_mean", "de_p95", "chroma_kept", "contrast_kept", "detail_kept", "levels_kept"]
    med = lambda who, k: float(np.median([r[who][k] for r in rows]))
    summary = {who: {k: med(who, k) for k in keys} for who in ["fotollection", "fotollection_after_facebook", "usual_after_facebook"]}
    summary["photos"] = len(rows)
    summary["source_mb"] = float(np.median([r["source_bytes"] for r in rows])) / 1e6
    summary["export_mb"] = float(np.median([r["export_bytes"] for r in rows])) / 1e6
    summary["size_ratio"] = float(np.median([r["export_bytes"] / r["source_bytes"] for r in rows]))
    print(f"{len(rows)} photos (median), source {summary['source_mb']:.2f} MB -> export {summary['export_mb']:.2f} MB ({summary['size_ratio']:.0%})")
    print(f"{'':16}{'Fotollection':>14}{'Fot.+FB':>12}{'usual+FB':>12}")
    for k in keys:
        print(f"{k:16}" + "".join(f"{summary[w][k]:>12.3f}" for w in ["fotollection", "fotollection_after_facebook", "usual_after_facebook"]))
    if json_path:
        Path(json_path).write_text(json.dumps({"summary": summary, "photos": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
