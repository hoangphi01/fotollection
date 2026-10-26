// SPDX-License-Identifier: GPL-3.0-only

/** The largest correction Rust accepts (FR-030, `MAX_CLOCK_OFFSET_S`): a month. */
export const MAX_OFFSET_S = 31 * 24 * 3600

/** A clock offset as `+1:02:00` (hours, minutes, seconds); hours may pass 24. */
export function formatOffset(seconds: number): string {
  const sign = seconds < 0 ? '−' : '+'
  const abs = Math.abs(Math.round(seconds))
  const h = Math.floor(abs / 3600)
  const m = Math.floor((abs % 3600) / 60)
  const s = abs % 60
  return `${sign}${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

/**
 * Reads what the user typed: `+1:02`, `-0:45:30`, `2` (hours), with `+`, `-` or `−`.
 * `null` when it is not an offset or is larger than a month.
 */
export function parseOffset(text: string): number | null {
  const match = /^\s*([+\-−]?)\s*(\d{1,3})(?::(\d{1,2}))?(?::(\d{1,2}))?\s*$/.exec(text)
  if (!match) return null
  const [, sign, h, m = '0', s = '0'] = match
  if (Number(m) > 59 || Number(s) > 59) return null
  const total = Number(h) * 3600 + Number(m) * 60 + Number(s)
  if (total > MAX_OFFSET_S) return null
  return sign === '-' || sign === '−' ? -total : total
}
