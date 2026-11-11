// SPDX-License-Identifier: GPL-3.0-only

/** Room kept from every window edge by an open list or popover, in pixels. */
const MARGIN = 8

/**
 * How far an open dropdown or popover keeps from the window's edges (Radix's
 * `collisionPadding`), the title bar included: the app's own bar (`--titlebar`, FR-052) sits
 * above every popup, so a list that opened upwards to the window's top lost its first rows
 * under it (owner, 2026-11-11; UI-LAYOUT-021).
 */
export function windowEdges(): { top: number; right: number; bottom: number; left: number } {
  const root = document.documentElement
  const bar = getComputedStyle(root).getPropertyValue('--titlebar').trim()
  const rem = Number.parseFloat(getComputedStyle(root).fontSize) || 16
  const value = Number.parseFloat(bar)
  const height = Number.isFinite(value) ? (bar.endsWith('rem') ? value * rem : value) : 0
  return { top: height + MARGIN, right: MARGIN, bottom: MARGIN, left: MARGIN }
}
