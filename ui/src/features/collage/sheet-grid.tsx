// SPDX-License-Identifier: GPL-3.0-only
import { useVirtualizer } from '@tanstack/react-virtual'
import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoId, SheetPair } from '@/ipc/bindings'
import { thumbUrl } from '@/lib/photo-url'
import { SheetTile } from './sheet-tile'

const TILE = 248
const GAP = 12
/** How far the pointer moves before a press becomes a drag, in pixels. */
const DRAG_SLOP = 5

type Drag = { photo: PhotoId; startX: number; startY: number; x: number; y: number; moved: boolean }

type SheetGridProps = {
  sheets: SheetPair[]
  revision: number
  generation: number
  selected: number
  selectedCell: 0 | 1
  onSelect: (sheet: number, cell: 0 | 1) => void
  onEdit: (photo: PhotoId, sheet: SheetPair) => void
  onMove: (photo: PhotoId, target: PhotoId | null) => void
  onTurn: (first: PhotoId) => void
}

/**
 * The collage sheets (FR-035), virtualized like the gallery (NFR-001). Drag a photo onto
 * another cell to swap them; drag it onto an empty cell to send it to the end. Pointer
 * events, not HTML drag and drop, so the window's file drop never interferes (ADR-0032).
 */
export function SheetGrid({
  sheets,
  revision,
  generation,
  selected,
  selectedCell,
  onSelect,
  onEdit,
  onMove,
  onTurn,
}: SheetGridProps) {
  const { t } = useTranslation()
  const scrollRef = useRef<HTMLDivElement>(null)
  const [width, setWidth] = useState(0)
  const [drag, setDrag] = useState<Drag | null>(null)
  // `undefined`: over no cell; `null`: over an empty one.
  const [over, setOver] = useState<PhotoId | null | undefined>(undefined)

  useEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0))
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const columns = Math.max(1, Math.floor((width - GAP) / (TILE + GAP)))
  const rows = Math.ceil(sheets.length / columns)
  // TanStack Virtual returns non-memoizable functions by design, as in the gallery.
  // oxlint-disable-next-line react/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => TILE + GAP,
    overscan: 3,
  })

  useEffect(() => {
    virtualizer.scrollToIndex(Math.floor(selected / columns), { align: 'auto' })
  }, [selected, columns, virtualizer])

  // The drag follows the pointer anywhere in the window, and ends wherever it is released.
  useEffect(() => {
    if (!drag) return
    const target = (x: number, y: number) => {
      const cell = document.elementFromPoint(x, y)?.closest<HTMLElement>('[data-cell]')
      if (!cell) return undefined
      const photo = cell.dataset.photo
      return photo === undefined ? null : Number(photo)
    }
    const onMoveEvent = (e: globalThis.PointerEvent) => {
      const moved = drag.moved || Math.hypot(e.clientX - drag.startX, e.clientY - drag.startY) > DRAG_SLOP
      setDrag({ ...drag, x: e.clientX, y: e.clientY, moved })
      if (moved) setOver(target(e.clientX, e.clientY))
    }
    const onUp = (e: globalThis.PointerEvent) => {
      if (drag.moved) {
        const to = target(e.clientX, e.clientY)
        if (to !== undefined && to !== drag.photo) onMove(drag.photo, to)
      }
      setDrag(null)
      setOver(undefined)
    }
    window.addEventListener('pointermove', onMoveEvent)
    window.addEventListener('pointerup', onUp)
    return () => {
      window.removeEventListener('pointermove', onMoveEvent)
      window.removeEventListener('pointerup', onUp)
    }
  }, [drag, onMove])

  const startDrag = (photo: PhotoId, e: PointerEvent) => {
    if (e.button !== 0) return
    setDrag({ photo, startX: e.clientX, startY: e.clientY, x: e.clientX, y: e.clientY, moved: false })
  }

  // The photos in order, each with its sheet and cell, for ← → and ⌥/Alt ← →.
  const cells = sheets.flatMap((s, i) =>
    s.second === null ? [{ photo: s.first, sheet: i, cell: 0 as const }] : [
      { photo: s.first, sheet: i, cell: 0 as const },
      { photo: s.second, sheet: i, cell: 1 as const },
    ],
  )
  const at = cells.findIndex((c) => c.sheet === selected && c.cell === selectedCell)

  const onKeyDown = (event: KeyboardEvent) => {
    const sheet = sheets[selected]
    const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
    const row = event.key === 'ArrowDown' ? columns : event.key === 'ArrowUp' ? -columns : 0
    if (step !== 0) {
      event.preventDefault()
      const next = cells[at + step]
      if (!next) return
      // ⌥/Alt ← → moves the chosen photo one place in the order; ← → alone moves the choice.
      if (event.altKey) onMove(cells[at]?.photo ?? next.photo, next.photo)
      onSelect(next.sheet, next.cell)
    } else if (row !== 0) {
      event.preventDefault()
      onSelect(Math.min(sheets.length - 1, Math.max(0, selected + row)), 0)
    } else if (event.key === 'Enter' && cells[at]) {
      event.preventDefault()
      const cell = cells[at]
      const pair = sheets[cell.sheet]
      if (pair) onEdit(cell.photo, pair)
    } else if (event.key.toLowerCase() === 'r' && sheet && !event.metaKey && !event.ctrlKey && !event.altKey) {
      event.preventDefault()
      onTurn(sheet.first)
    }
  }

  return (
    <div
      ref={scrollRef}
      tabIndex={0}
      onKeyDown={onKeyDown}
      className="flex-1 overflow-y-auto p-3 outline-none select-none"
      aria-label={t('collage.gridLabel')}
    >
      <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((row) => (
          <div
            key={row.key}
            className="absolute left-0 flex w-full"
            style={{ top: row.start, gap: GAP, paddingLeft: GAP }}
          >
            {sheets.slice(row.index * columns, row.index * columns + columns).map((sheet, i) => {
              const index = row.index * columns + i
              return (
                <SheetTile
                  key={`${sheet.first}:${sheet.second}`}
                  sheet={sheet}
                  revision={revision}
                  size={TILE}
                  selected={index === selected}
                  selectedCell={selectedCell}
                  dragging={drag?.moved ? drag.photo : null}
                  over={over}
                  onSelect={(cell) => onSelect(index, cell)}
                  onEdit={(photo) => onEdit(photo, sheet)}
                  onTurn={() => onTurn(sheet.first)}
                  onDragStart={startDrag}
                />
              )
            })}
          </div>
        ))}
      </div>
      {drag?.moved && (
        <img
          src={thumbUrl(drag.photo, 0, generation)}
          alt=""
          aria-hidden
          className="pointer-events-none fixed z-50 size-20 rounded-md object-cover shadow-xl ring-2 ring-primary"
          style={{ left: drag.x - 40, top: drag.y - 40 }}
        />
      )}
    </div>
  )
}
