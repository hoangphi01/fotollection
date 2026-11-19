// SPDX-License-Identifier: GPL-3.0-only
import type { CSSProperties } from 'react'

/**
 * A header between runs of a photo grid (STYLE_RULE §4; FR-141 C2, RollSense's events and
 * days): one line of `text-xs` words over a hairline, its count at the right, 24 px of a 32 px
 * row so 8 px lead to the photos under it, as between two rows of tiles.
 */
export function GridSectionHeader({ label, trailing, style }: { label: string; trailing: string; style: CSSProperties }) {
  return (
    <div data-slot="grid-section-header" className="absolute flex items-end justify-between gap-2 border-b pb-1 text-xs font-medium text-muted-foreground" style={style}>
      <span className="truncate">{label}</span>
      <span className="shrink-0 tabular-nums">{trailing}</span>
    </div>
  )
}
