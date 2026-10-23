// SPDX-License-Identifier: GPL-3.0-only
// Mirrors `crop_rect` in crates/core/src/plan.rs so the crop editor can drag at 60 fps
// without IPC. Rust re-sanitizes and stays authoritative when the crop is saved.

export type Rect = { x: number; y: number; width: number; height: number }
export type Frame = { cx: number; cy: number; zoom: number }

export const MAX_ZOOM = 8

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v))

export function sanitize(frame: Frame): Frame {
  const unit = (v: number) => (Number.isFinite(v) ? clamp(v, 0, 1) : 0.5)
  return {
    cx: unit(frame.cx),
    cy: unit(frame.cy),
    zoom: Number.isFinite(frame.zoom) ? clamp(frame.zoom, 1, MAX_ZOOM) : 1,
  }
}

/** Largest crop of `w × h` at ratio `tw : th`, then scaled/moved by `frame`, kept inside. */
export function cropRect(w: number, h: number, tw: number, th: number, frame?: Frame | null): Rect {
  const [bw, bh] = w * th >= h * tw ? [Math.round((h * tw) / th), h] : [w, Math.round((w * th) / tw)]
  if (!frame) return { x: Math.floor((w - bw) / 2), y: Math.floor((h - bh) / 2), width: bw, height: bh }
  const f = sanitize(frame)
  const cw = Math.max(1, Math.round(bw / f.zoom))
  const ch = Math.max(1, Math.round(bh / f.zoom))
  return {
    x: Math.round(clamp(f.cx * w - cw / 2, 0, w - cw)),
    y: Math.round(clamp(f.cy * h - ch / 2, 0, h - ch)),
    width: cw,
    height: ch,
  }
}

/** The frame whose centre matches a (clamped) rectangle, so dragging never drifts. */
export function frameOf(rect: Rect, w: number, h: number, zoom: number): Frame {
  return { cx: (rect.x + rect.width / 2) / w, cy: (rect.y + rect.height / 2) / h, zoom }
}

const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b))

/** `2:3`, `1:1`, or `1:1.44` when the reduced ratio isn't made of small numbers. */
export function ratioLabel(width: number, height: number): string {
  const w = Math.round(width)
  const h = Math.round(height)
  if (w <= 0 || h <= 0) return ''
  const d = gcd(w, h)
  const [a, b] = [w / d, h / d]
  if (a <= 20 && b <= 20) return `${a}:${b}`
  const r = Math.max(w, h) / Math.min(w, h)
  return w <= h ? `1:${r.toFixed(2)}` : `${r.toFixed(2)}:1`
}

export type DpiBand = 'excellent' | 'good' | 'fair' | 'poor'

/** Same thresholds as `effective_dpi` (FR-026, PRD §5). */
export function dpiBand(dpi: number): DpiBand {
  if (dpi >= 300) return 'excellent'
  if (dpi >= 200) return 'good'
  if (dpi >= 150) return 'fair'
  return 'poor'
}

export const DPI_STYLE: Record<DpiBand, { label: string; className: string }> = {
  excellent: { label: 'Excellent', className: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400' },
  good: { label: 'Good', className: 'bg-sky-500/15 text-sky-600 dark:text-sky-400' },
  fair: { label: 'Fair', className: 'bg-amber-500/15 text-amber-600 dark:text-amber-400' },
  poor: { label: 'Too low', className: 'bg-red-500/15 text-red-600 dark:text-red-400' },
}
