// SPDX-License-Identifier: GPL-3.0-only
import { useVirtualizer } from '@tanstack/react-virtual'
import { CalendarX2, MapPinOff } from 'lucide-react'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import type { PhotoSummary } from '@/ipc/bindings'
import { cn } from '@/lib/utils'
import { thumbUrl } from '@/lib/photo-url'

const TILE = 168
const GAP = 8

type LibraryGridProps = {
  photos: PhotoSummary[]
  thumbVersions: Record<number, number>
  selected: number
  onSelect: (index: number) => void
  onOpen: (index: number) => void
}

function formatCapture(capture: string): string {
  return capture.replace('T', ' ').slice(0, 16)
}

/** Virtualized thumbnail grid (FR-003, NFR-001): only visible rows are in the DOM. */
export function LibraryGrid({ photos, thumbVersions, selected, onSelect, onOpen }: LibraryGridProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [width, setWidth] = useState(0)

  useEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0))
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const columns = Math.max(1, Math.floor((width - GAP) / (TILE + GAP)))
  const rows = Math.ceil(photos.length / columns)
  // TanStack Virtual returns non-memoizable functions by design; the grid re-renders
  // on scroll anyway, so skipping React Compiler memoization here is fine.
  // oxlint-disable-next-line react/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => TILE + GAP,
    overscan: 4,
  })

  useEffect(() => {
    virtualizer.scrollToIndex(Math.floor(selected / columns), { align: 'auto' })
  }, [selected, columns, virtualizer])

  const onKeyDown = (event: KeyboardEvent) => {
    const moves: Record<string, number> = {
      ArrowRight: 1,
      ArrowLeft: -1,
      ArrowDown: columns,
      ArrowUp: -columns,
    }
    const delta = moves[event.key]
    if (delta !== undefined) {
      event.preventDefault()
      onSelect(Math.min(photos.length - 1, Math.max(0, selected + delta)))
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      onOpen(selected)
    }
  }

  return (
    <div
      ref={scrollRef}
      tabIndex={0}
      onKeyDown={onKeyDown}
      className="flex-1 overflow-y-auto p-2 outline-none"
      aria-label="Photo library"
    >
      <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((row) => (
          <div
            key={row.key}
            className="absolute left-0 flex w-full"
            style={{ top: row.start, gap: GAP, paddingLeft: GAP }}
          >
            {photos.slice(row.index * columns, row.index * columns + columns).map((photo, i) => {
              const index = row.index * columns + i
              return (
                <button
                  key={photo.id}
                  type="button"
                  title={`${photo.fileName} · ${formatCapture(photo.capture)}`}
                  onClick={() => onSelect(index)}
                  onDoubleClick={() => onOpen(index)}
                  className={cn(
                    'group relative shrink-0 overflow-hidden rounded-md bg-muted outline-none',
                    index === selected && 'ring-2 ring-primary ring-offset-2 ring-offset-background',
                  )}
                  style={{ width: TILE, height: TILE }}
                >
                  <img
                    src={thumbUrl(photo.id, thumbVersions[photo.id])}
                    alt=""
                    loading="lazy"
                    decoding="async"
                    draggable={false}
                    className="size-full object-cover"
                  />
                  <div className="absolute inset-x-0 bottom-0 flex items-center gap-1 bg-gradient-to-t from-black/60 to-transparent px-1.5 pt-4 pb-1 text-[10px] text-white opacity-0 transition-opacity group-hover:opacity-100">
                    <span className="truncate">{formatCapture(photo.capture)}</span>
                  </div>
                  <div className="absolute top-1 right-1 flex gap-0.5">
                    {photo.dateFromFile && (
                      <span className="rounded bg-black/60 p-0.5 text-amber-300" aria-label="No capture date">
                        <CalendarX2 className="size-3" />
                      </span>
                    )}
                    {!photo.hasGps && (
                      <span className="rounded bg-black/60 p-0.5 text-white/80" aria-label="No location">
                        <MapPinOff className="size-3" />
                      </span>
                    )}
                  </div>
                </button>
              )
            })}
          </div>
        ))}
      </div>
    </div>
  )
}
