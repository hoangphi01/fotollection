// SPDX-License-Identifier: GPL-3.0-only
import { useVirtualizer } from '@tanstack/react-virtual'
import { useEffect, useRef, type ReactNode } from 'react'
import { cn } from '@/lib/utils'

/** An open strip with its header line, less its own line: 28 px of header, 96 px of cells. The
 * viewer's footer beside it keeps this height, open or folded (owner, 2026-11-07). */
export const STRIP_OPEN = 124

/** A cell's side and the gap between cells, in px (STYLE_RULE §4). */
const CELL = 80
const GAP = 8
/** Room for the current cell's ring (2 px) and its offset (2 px) at either end, with 2 to spare. */
const EDGE = 6

type FilmstripProps<T> = {
  /** What the strip holds, in order. */
  items: readonly T[]
  /** The index of the one shown, outlined and kept in sight. */
  current?: number | null
  onPick: (item: T, index: number) => void
  /** A cell's content (its thumbnail and badges), filling an 80 px square. */
  cell: (item: T) => ReactNode
  /** A cell's name, for its hint and screen readers. */
  hint: (item: T) => string
  /** What screen readers call a cell when it differs from its hint: the words it shows, so the
   * name contains them (UI-A11Y-002). */
  name?: (item: T) => string
  /** A cell's own key (`aria-keyshortcuts`), told to screen readers instead of in its name. */
  keys?: (item: T) => string | undefined
  /** A short label before the cells (the capture screen's shot count). */
  label?: ReactNode
  /** Shown in place of the cells when there are none. */
  empty?: ReactNode
  /** A line above the cells, as Lightroom's: where the shown one is in the set, its name. */
  header?: ReactNode
  /** Only the header line, the cells folded away (the strip hidden, FR-090). */
  collapsed?: boolean
}

/**
 * A strip of thumbnails under a viewer's stage, in `ViewerFrame`'s `below` slot: the capture
 * screen's shots and Pro mode's filmstrip (FR-090), one block so they look alike. Fixed at
 * 112 px (STYLE_RULE §4); only the cells in sight are drawn, so a set of 500 scrolls as
 * smoothly as one of 5.
 */
export function Filmstrip<T>({ items, current = null, onPick, cell, hint, name, keys, label, empty, header, collapsed = false }: FilmstripProps<T>) {
  const count = items.length
  const scrollRef = useRef<HTMLDivElement>(null)
  // As the gallery's grid does: the virtualizer's own functions stay fresh by design.
  // oxlint-disable-next-line react/incompatible-library
  const strip = useVirtualizer({
    horizontal: true,
    count,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => CELL + GAP,
    paddingStart: EDGE,
    paddingEnd: EDGE,
    scrollPaddingStart: EDGE,
    scrollPaddingEnd: EDGE,
    overscan: 6,
  })
  // The photo shown stays in sight as ← → move it, without moving a strip it is already in.
  useEffect(() => {
    if (current !== null && current < count) strip.scrollToIndex(current, { align: 'auto' })
  }, [current, count, strip])

  return (
    <div className="flex shrink-0 flex-col border-t border-stage-line bg-stage">
      {header !== undefined && (
        <div className="flex h-7 items-center gap-2 truncate px-4 text-xs text-stage-ink-muted tabular-nums">{header}</div>
      )}
      {/* The cells fold under the header (FR-100): their height, as Collapse's (`motion-fold`);
          kept drawn and `inert` while folded, so opening is one move, never a load. */}
      <div className={cn('grid motion-fold', collapsed ? 'grid-rows-[0fr] opacity-0' : 'grid-rows-[1fr]')} inert={collapsed}>
      <div className="min-h-0 overflow-hidden">
      {/* Under a header the row is the strip's 96 px: the cells, 4 px for the chosen ring above
          and below (2 px ring, 2 px offset), and the scroll bar's 8 px; the box that folds clips
          anything more (LESSONS, UI-LAYOUT-016). */}
      <div className={cn('flex shrink-0 items-center gap-3 px-4', header === undefined ? 'h-28' : 'h-24')}>
        {label !== undefined && <span className="w-16 shrink-0 text-xs text-stage-ink-muted tabular-nums">{label}</span>}
        {count === 0 ? (
          <p className="min-w-0 flex-1 text-xs text-stage-ink-muted">{empty}</p>
        ) : (
          <div ref={scrollRef} className={cn('min-w-0 flex-1 overflow-x-auto', header === undefined ? 'py-2' : 'py-1')}>
            <div className="relative" style={{ width: strip.getTotalSize(), height: CELL }}>
              {strip.getVirtualItems().map((item) => {
                const it = items[item.index]
                if (it === undefined) return null
                return (
                  <button
                    key={item.key}
                    type="button"
                    aria-label={name ? name(it) : hint(it)}
                    aria-keyshortcuts={keys?.(it)}
                    aria-current={item.index === current ? 'true' : undefined}
                    data-hint={hint(it)}
                    onClick={() => onPick(it, item.index)}
                    className={cn(
                      'absolute top-0 overflow-hidden rounded-sm bg-stage-raised ring-offset-2 ring-offset-stage motion-control outline-none hover:ring-2 hover:ring-primary/60 focus-visible:ring-2 focus-visible:ring-primary',
                      item.index === current && 'ring-2 ring-primary hover:ring-primary',
                    )}
                    style={{ left: item.start, width: CELL, height: CELL }}
                  >
                    {cell(it)}
                  </button>
                )
              })}
            </div>
          </div>
        )}
      </div>
      </div>
      </div>
    </div>
  )
}
