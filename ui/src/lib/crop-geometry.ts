// SPDX-License-Identifier: GPL-3.0-only
import { formatNumber } from "./format";
// Mirrors `crop_rect` in crates/core/src/plan.rs so the crop editor can drag at 60 fps
// without IPC. Rust re-sanitizes and stays authoritative when the crop is saved.

export type Rect = { x: number; y: number; width: number; height: number };
export type Frame = { cx: number; cy: number; zoom: number };

export const MAX_ZOOM = 8;

const clamp = (v: number, lo: number, hi: number) =>
  Math.min(hi, Math.max(lo, v));

export function sanitize(frame: Frame): Frame {
  const unit = (v: number) => (Number.isFinite(v) ? clamp(v, 0, 1) : 0.5);
  return {
    cx: unit(frame.cx),
    cy: unit(frame.cy),
    zoom: Number.isFinite(frame.zoom) ? clamp(frame.zoom, 1, MAX_ZOOM) : 1,
  };
}

/** Largest crop of `w × h` at ratio `tw : th`, then scaled/moved by `frame`, kept inside. */
export function cropRect(
  w: number,
  h: number,
  tw: number,
  th: number,
  frame?: Frame | null,
): Rect {
  const [bw, bh] =
    w * th >= h * tw
      ? [Math.round((h * tw) / th), h]
      : [w, Math.round((w * th) / tw)];
  if (!frame)
    return {
      x: Math.floor((w - bw) / 2),
      y: Math.floor((h - bh) / 2),
      width: bw,
      height: bh,
    };
  const f = sanitize(frame);
  const cw = Math.max(1, Math.round(bw / f.zoom));
  const ch = Math.max(1, Math.round(bh / f.zoom));
  return {
    x: Math.round(clamp(f.cx * w - cw / 2, 0, w - cw)),
    y: Math.round(clamp(f.cy * h - ch / 2, 0, h - ch)),
    width: cw,
    height: ch,
  };
}

export const MAX_STRAIGHTEN = 15;

/** Mirrors `straighten::fits` in crates/core/src/straighten.rs. */
function fits(
  cx: number,
  cy: number,
  w: number,
  h: number,
  hw: number,
  hh: number,
  cos: number,
  sin: number,
) {
  return [
    [-1, -1],
    [1, -1],
    [1, 1],
    [-1, 1],
  ].every(([sx = 0, sy = 0]) => {
    const qx = cx + (sx * w) / 2;
    const qy = cy + (sy * h) / 2;
    const px = cos * qx + sin * qy;
    const py = -sin * qx + cos * qy;
    return Math.abs(px) <= hw + 1e-6 && Math.abs(py) <= hh + 1e-6;
  });
}

/**
 * Mirrors `straighten::crop_rect`: the crop in the straightened frame, as large as it can be
 * without an empty corner, moved toward the frame's centre as far as it stays inside.
 */
export function straightenedRect(
  w: number,
  h: number,
  tw: number,
  th: number,
  frame: Frame | null | undefined,
  degrees: number,
): Rect {
  if (Math.abs(degrees) < 0.01) return cropRect(w, h, tw, th, frame);
  const theta =
    (clamp(degrees, -MAX_STRAIGHTEN, MAX_STRAIGHTEN) * Math.PI) / 180;
  const cos = Math.cos(theta);
  const sin = Math.sin(theta);
  const r = tw / th;
  const [c, s] = [Math.abs(cos), Math.abs(sin)];
  const wMax = Math.min(w / (c + s / r), h / (s + c / r));
  const f = sanitize(frame ?? { cx: 0.5, cy: 0.5, zoom: 1 });
  const cw = Math.max(1, wMax / f.zoom);
  const ch = Math.max(1, cw / r);
  const [hw, hh] = [w / 2, h / 2];
  const want = [f.cx * w - hw, f.cy * h - hh] as const;
  let [lo, hi] = [0, 1];
  if (fits(want[0], want[1], cw, ch, hw, hh, cos, sin)) lo = 1;
  else
    for (let i = 0; i < 30; i++) {
      const mid = (lo + hi) / 2;
      if (fits(want[0] * mid, want[1] * mid, cw, ch, hw, hh, cos, sin))
        lo = mid;
      else hi = mid;
    }
  const [x, y] = [want[0] * lo + hw, want[1] * lo + hh];
  return {
    x: Math.max(0, Math.round(x - cw / 2)),
    y: Math.max(0, Math.round(y - ch / 2)),
    width: Math.round(cw),
    height: Math.round(ch),
  };
}

/** The frame whose centre matches a (clamped) rectangle, so dragging never drifts. */
export function frameOf(rect: Rect, w: number, h: number, zoom: number): Frame {
  return {
    cx: (rect.x + rect.width / 2) / w,
    cy: (rect.y + rect.height / 2) / h,
    zoom,
  };
}

const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));

/** `2:3`, `1:1`, or `1:1.44` when the reduced ratio isn't made of small numbers. */
export function ratioLabel(width: number, height: number): string {
  const w = Math.round(width);
  const h = Math.round(height);
  if (w <= 0 || h <= 0) return "";
  const d = gcd(w, h);
  const [a, b] = [w / d, h / d];
  if (a <= 20 && b <= 20) return `${a}:${b}`;
  // Sizes entered in inches are stored in whole mm (4 × 6 in → 102 × 152), so snap to
  // the nearest small ratio within 1 %.
  for (let q = 1; q <= 20; q++) {
    for (let p = 1; p <= 20; p++) {
      if (gcd(p, q) === 1 && Math.abs((p / q) * h - w) <= 0.01 * w)
        return `${p}:${q}`;
    }
  }
  const r = formatNumber(Math.max(w, h) / Math.min(w, h), 2, 2);
  return w <= h ? `1:${r}` : `${r}:1`;
}

export type DpiBand = "excellent" | "good" | "fair" | "poor";

/** Same thresholds as `effective_dpi` (FR-026, PRD §5). */
export function dpiBand(dpi: number): DpiBand {
  if (dpi >= 300) return "excellent";
  if (dpi >= 200) return "good";
  if (dpi >= 150) return "fair";
  return "poor";
}

/** Badge colours per band; the words come from the `dpi.*` translations (FR-020). */
export const DPI_STYLE: Record<DpiBand, { className: string }> = {
  excellent: {
    className: "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400",
  },
  good: { className: "bg-sky-500/15 text-sky-600 dark:text-sky-400" },
  fair: { className: "bg-amber-500/15 text-amber-600 dark:text-amber-400" },
  poor: { className: "bg-red-500/15 text-red-600 dark:text-red-400" },
};
