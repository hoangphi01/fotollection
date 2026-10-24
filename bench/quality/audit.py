#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Image-quality audit of the export (NFR-009): compares every exported photo with its
source, pixel for pixel, and does the same for a "typical web editor" export so the two
can be read side by side.

    python3 bench/quality/audit.py <source-folder> <export-folder> [--json out.json]

The export must be made at the photo's own size with no stamp or frame
(`cargo run --release -p fotollection-core --example quality_export`). Both images are
decoded with libjpeg (Pillow), converted to CIELAB through their embedded ICC profiles
(sRGB when there is none), and compared:

- PSNR (dB) of the RGB pixels;
- CIEDE2000 colour difference: mean, 95th percentile (1 is the threshold of a visible
  difference for a trained eye side by side);
- colourfulness kept: mean chroma C* of the output / of the source;
- contrast kept: standard deviation of L* output / source;
- detail kept: variance of the Laplacian of L*, output / source;
- tonal levels kept: distinct 8-bit luma levels, output / source (lower = banding);
- ICC profile kept, chroma subsampling, and JPEG quality estimated from the tables.

The baseline re-encodes each source the way many web editors do: sRGB assumed, the ICC
profile dropped, chroma subsampled 4:2:0, quality 80, long edge 2000 px (upscaled back to
the source size for the comparison, as a print lab would).
"""

from __future__ import annotations

import io
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageCms, ImageOps
from PIL.JpegImagePlugin import get_sampling

SRGB = ImageCms.createProfile("sRGB")


def to_srgb(img: Image.Image, icc: bytes | None) -> Image.Image:
    """The image in sRGB. Photos with another embedded profile (Display P3, Adobe RGB) are
    converted with the same transform on both sides, so the comparison stays fair."""
    img = img.convert("RGB")
    if not icc:
        return img
    src = ImageCms.ImageCmsProfile(io.BytesIO(icc))
    if "srgb" in ImageCms.getProfileDescription(src).lower():
        return img
    return ImageCms.profileToProfile(img, src, SRGB, renderingIntent=1, outputMode="RGB")


def to_lab(img: Image.Image, icc: bytes | None) -> np.ndarray:
    """CIELAB (D65) in floating point from sRGB: no 8-bit rounding in the measurement."""
    rgb = np.asarray(to_srgb(img, icc), dtype=np.float32) / 255
    lin = np.where(rgb <= 0.04045, rgb / 12.92, ((rgb + 0.055) / 1.055) ** 2.4)
    m = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]], dtype=np.float32)
    xyz = lin @ m.T / np.array([0.95047, 1.0, 1.08883], dtype=np.float32)
    f = np.where(xyz > (6 / 29) ** 3, np.cbrt(xyz), xyz / (3 * (6 / 29) ** 2) + 4 / 29)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1)


def ciede2000(lab1: np.ndarray, lab2: np.ndarray) -> np.ndarray:
    L1, a1, b1 = lab1[..., 0], lab1[..., 1], lab1[..., 2]
    L2, a2, b2 = lab2[..., 0], lab2[..., 1], lab2[..., 2]
    C1, C2 = np.hypot(a1, b1), np.hypot(a2, b2)
    Cm = (C1 + C2) / 2
    G = 0.5 * (1 - np.sqrt(Cm**7 / (Cm**7 + 25**7)))
    a1p, a2p = (1 + G) * a1, (1 + G) * a2
    C1p, C2p = np.hypot(a1p, b1), np.hypot(a2p, b2)
    h1p = np.degrees(np.arctan2(b1, a1p)) % 360
    h2p = np.degrees(np.arctan2(b2, a2p)) % 360
    dLp, dCp = L2 - L1, C2p - C1p
    dh = h2p - h1p
    dh = np.where(dh > 180, dh - 360, np.where(dh < -180, dh + 360, dh))
    dh = np.where(C1p * C2p == 0, 0, dh)
    dHp = 2 * np.sqrt(C1p * C2p) * np.sin(np.radians(dh / 2))
    Lpm, Cpm = (L1 + L2) / 2, (C1p + C2p) / 2
    hs = h1p + h2p
    hpm = np.where(
        C1p * C2p == 0,
        hs,
        np.where(np.abs(h1p - h2p) <= 180, hs / 2, np.where(hs < 360, (hs + 360) / 2, (hs - 360) / 2)),
    )
    T = (
        1
        - 0.17 * np.cos(np.radians(hpm - 30))
        + 0.24 * np.cos(np.radians(2 * hpm))
        + 0.32 * np.cos(np.radians(3 * hpm + 6))
        - 0.20 * np.cos(np.radians(4 * hpm - 63))
    )
    dtheta = 30 * np.exp(-(((hpm - 275) / 25) ** 2))
    Rc = 2 * np.sqrt(Cpm**7 / (Cpm**7 + 25**7))
    Sl = 1 + 0.015 * (Lpm - 50) ** 2 / np.sqrt(20 + (Lpm - 50) ** 2)
    Sc, Sh = 1 + 0.045 * Cpm, 1 + 0.015 * Cpm * T
    Rt = -np.sin(np.radians(2 * dtheta)) * Rc
    return np.sqrt((dLp / Sl) ** 2 + (dCp / Sc) ** 2 + (dHp / Sh) ** 2 + Rt * (dCp / Sc) * (dHp / Sh))


def laplacian_var(L: np.ndarray) -> float:
    lap = -4 * L[1:-1, 1:-1] + L[:-2, 1:-1] + L[2:, 1:-1] + L[1:-1, :-2] + L[1:-1, 2:]
    return float(lap.var())


def quality_estimate(img: Image.Image) -> int | None:
    """IJG quality that would produce the luminance table (approximate)."""
    tables = getattr(img, "quantization", None)
    if not tables:
        return None
    q0 = np.array(tables[0], dtype=float)
    base = np.array([16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
                     14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92,
                     49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99], dtype=float)
    best, err = None, 1e18
    for q in range(1, 101):
        s = 5000 / q if q < 50 else 200 - 2 * q
        t = np.clip(np.floor((base * s + 50) / 100), 1, 255)
        e = float(np.abs(np.sort(t) - np.sort(q0)).sum())
        if e < err:
            best, err = q, e
    return best


def compare(src: Image.Image, out: Image.Image, icc_src: bytes | None, icc_out: bytes | None) -> dict:
    a = np.asarray(src.convert("RGB"), dtype=np.float32)
    b = np.asarray(out.convert("RGB"), dtype=np.float32)
    mse = float(((a - b) ** 2).mean())
    la, lb = to_lab(src, icc_src), to_lab(out, icc_out)
    de = ciede2000(la, lb)
    luma = lambda x: np.round(0.299 * x[..., 0] + 0.587 * x[..., 1] + 0.114 * x[..., 2])
    return {
        "psnr": 99.0 if mse == 0 else 10 * np.log10(255**2 / mse),
        "de_mean": float(de.mean()),
        "de_p95": float(np.percentile(de, 95)),
        "chroma_kept": float(np.hypot(lb[..., 1], lb[..., 2]).mean() / max(np.hypot(la[..., 1], la[..., 2]).mean(), 1e-9)),
        "contrast_kept": float(lb[..., 0].std() / max(la[..., 0].std(), 1e-9)),
        "detail_kept": laplacian_var(lb[..., 0]) / max(laplacian_var(la[..., 0]), 1e-9),
        "levels_kept": len(np.unique(luma(b))) / max(len(np.unique(luma(a))), 1),
    }


def web_editor(src: Image.Image) -> Image.Image:
    small = src.convert("RGB")
    small.thumbnail((2000, 2000), Image.Resampling.BICUBIC)
    buf = io.BytesIO()
    small.save(buf, "JPEG", quality=80, subsampling=2)  # 4:2:0, no ICC
    buf.seek(0)
    return Image.open(buf).convert("RGB").resize(src.size, Image.Resampling.BICUBIC)


def main() -> None:
    src_dir, out_dir = Path(sys.argv[1]), Path(sys.argv[2])
    json_path = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    rows = []
    for out_path in sorted(out_dir.rglob("*.jpg")):
        src_path = next(src_dir.glob(out_path.stem + ".*"), None)
        if src_path is None:
            continue
        icc_src = Image.open(src_path).info.get("icc_profile")
        src = ImageOps.exif_transpose(Image.open(src_path))
        out = Image.open(out_path)
        icc_out = out.info.get("icc_profile")
        if out.size != src.size:
            continue  # Original keeps the size; anything else is not comparable pixel for pixel
        fot = compare(src, out, icc_src, icc_out)
        # The web editor drops the profile: its pixels are read as sRGB.
        web = compare(src, web_editor(src), icc_src, None)
        rows.append(
            {
                "photo": src_path.name,
                "fotollection": fot,
                "web_editor": web,
                "icc_kept": icc_out == icc_src,
                "source_profile": ImageCms.getProfileDescription(ImageCms.ImageCmsProfile(io.BytesIO(icc_src))).strip()
                if icc_src
                else "none (sRGB)",
                "sampling": {0: "4:4:4", 1: "4:2:2", 2: "4:2:0"}.get(get_sampling(out), "?"),
                "quality": quality_estimate(out),
            }
        )
    keys = ["psnr", "de_mean", "de_p95", "chroma_kept", "contrast_kept", "detail_kept", "levels_kept"]
    summary = {
        who: {k: float(np.median([r[who][k] for r in rows])) for k in keys} for who in ["fotollection", "web_editor"]
    }
    summary["photos"] = len(rows)
    summary["icc_kept"] = sum(r["icc_kept"] for r in rows)
    summary["sampling"] = sorted({r["sampling"] for r in rows})
    summary["quality"] = sorted({r["quality"] for r in rows})
    print(f"{len(rows)} photos (median)")
    print(f"{'':16}{'Fotollection':>14}{'web editor':>14}")
    for k in keys:
        print(f"{k:16}{summary['fotollection'][k]:14.3f}{summary['web_editor'][k]:14.3f}")
    print(f"ICC kept {summary['icc_kept']}/{len(rows)}, sampling {summary['sampling']}, quality {summary['quality']}")
    if json_path:
        Path(json_path).write_text(json.dumps({"summary": summary, "photos": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
