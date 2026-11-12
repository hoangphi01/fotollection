# SPDX-License-Identifier: GPL-3.0-only
# Scores RawTherapee or Fotollection outputs the way crates/core/examples/highlights_bench.rs scores ours.
# Inputs (bench/rawtherapee/compare.sh): 16-bit sRGB TIFFs or PPMs (truth_off, truth_opp, cut_off, cut_<method>), lowered by 2 EV (truth) / 2 EV + clip compensation (cut).
import sys, json
import numpy as np, tifffile

import os
def load(p):
    q = p[:-4] + ".ppm"
    if os.path.exists(q):
        raw = open(q, "rb").read()
        parts = raw.split(b"\n", 3)
        w, h = map(int, parts[1].split())
        a = np.frombuffer(parts[3], dtype=">u2").reshape(h, w, 3).astype(np.float32) / 65535.0
        return a
    a = tifffile.imread(p).astype(np.float32) / 65535.0
    return a[..., :3]

def lin(e):
    return np.where(e <= 0.04045, e / 12.92, ((e + 0.055) / 1.055) ** 2.4)

def lab8(e):
    q = np.round(np.clip(e, 0, 1) * 255.0) / 255.0
    r, g, b = [lin(q[..., i]) for i in range(3)]
    x = 0.4124 * r + 0.3576 * g + 0.1805 * b
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = 0.0193 * r + 0.1192 * g + 0.9505 * b
    f = lambda t: np.where(t > 0.008856, np.cbrt(t), 7.787 * t + 16 / 116)
    fx, fy, fz = f(x / 0.95047), f(y), f(z / 1.08883)
    return np.stack([116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)], -1)

def de2000(l1, l2):
    L1, a1, b1 = l1[:, 0], l1[:, 1], l1[:, 2]
    L2, a2, b2 = l2[:, 0], l2[:, 1], l2[:, 2]
    C1, C2 = np.hypot(a1, b1), np.hypot(a2, b2)
    Cm = (C1 + C2) / 2
    G = 0.5 * (1 - np.sqrt(Cm**7 / (Cm**7 + 25.0**7)))
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
    hpm = np.where(C1p * C2p == 0, hs,
          np.where(np.abs(h1p - h2p) <= 180, hs / 2,
          np.where(hs < 360, (hs + 360) / 2, (hs - 360) / 2)))
    T = (1 - 0.17 * np.cos(np.radians(hpm - 30)) + 0.24 * np.cos(np.radians(2 * hpm))
         + 0.32 * np.cos(np.radians(3 * hpm + 6)) - 0.20 * np.cos(np.radians(4 * hpm - 63)))
    dth = 30 * np.exp(-(((hpm - 275) / 25) ** 2))
    Rc = 2 * np.sqrt(Cpm**7 / (Cpm**7 + 25.0**7))
    Sl = 1 + 0.015 * (Lpm - 50) ** 2 / np.sqrt(20 + (Lpm - 50) ** 2)
    Sc, Sh = 1 + 0.045 * Cpm, 1 + 0.015 * Cpm * T
    Rt = -np.sin(np.radians(2 * dth)) * Rc
    return np.sqrt((dLp / Sl) ** 2 + (dCp / Sc) ** 2 + (dHp / Sh) ** 2 + Rt * (dCp / Sc) * (dHp / Sh))

def lum(p):
    return 0.2126 * p[..., 0] + 0.7152 * p[..., 1] + 0.0722 * p[..., 2]

d = sys.argv[1]
truth_e = load(f"{d}/truth_off.tif"); truth = lin(truth_e)
opp = lin(load(f"{d}/truth_opp.tif"))
off = lin(load(f"{d}/cut_off.tif"))
unknown = (np.abs(truth - opp) > 1e-4).any(-1)
changed = (np.abs(off - truth) > 0.01 * np.maximum(truth, 0.01)).any(-1)
mask = changed & ~unknown
eps = 1e-4
res = {"pixels": int(mask.sum()), "unknown": int(unknown.sum())}
tl = lab8(truth_e[mask])
for m in sys.argv[2:]:
    ge = load(f"{d}/cut_{m}.tif"); g = lin(ge)
    gm, tm = g[mask], truth[mask]
    ratio = lambda p, c: np.log(np.maximum(p[:, c], eps) / np.maximum(p[:, 1], eps))
    chroma = ((np.abs(ratio(gm, 0) - ratio(tm, 0)) + np.abs(ratio(gm, 2) - ratio(tm, 2))) / 2).mean()
    ev = np.abs(np.log2(np.maximum(lum(gm), eps) / np.maximum(lum(tm), eps))).mean()
    de = de2000(lab8(ge[mask]), tl).mean()
    gu = g[unknown]
    guess = ((np.abs(ratio(gu, 0)) + np.abs(ratio(gu, 2))) / 2).mean() if len(gu) else 0.0
    res[m] = {"chroma": float(chroma), "ev": float(ev), "de": float(de), "guess": float(guess)}
print(json.dumps(res))
