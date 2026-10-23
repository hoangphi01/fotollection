// SPDX-License-Identifier: GPL-3.0-only
import { useVirtualizer } from '@tanstack/react-virtual'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoSummary } from '@/ipc/bindings'
import { handleCullKey } from '@/features/culling/cull-keys'
import { ThumbTile } from './thumb-tile'

const TILE = 168
const GAP = 8

type LibraryGridProps = {
  photos: PhotoSummary[]
  thumbVersions: Record<number, number>
  generation: number
  selected: number
  onSelect: (index: number) => void
  onOpen: (index: number) => void
}

/** Virtualized thumbnail grid (FR-003, NFR-001): only visible rows are in the DOM. */
export function LibraryGrid({
  photos,
  thumbVersions,
  generation,
  selected,
  onSelect,
  onOpen,
}: LibraryGridProps) {
  const { t } = useTranslation()
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
    } else if (handleCullKey(event, photos[selected]?.id)) {
      event.preventDefault()
    }
  }

  return (
    <div
      ref={scrollRef}
      tabIndex={0}
      onKeyDown={onKeyDown}
      className="flex-1 overflow-y-auto p-2 outline-none"
      aria-label={t('library.gridLabel')}
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
                <ThumbTile
                  key={photo.id}
                  photo={photo}
                  version={thumbVersions[photo.id] ?? 0}
                  generation={generation}
                  size={TILE}
                  selected={index === selected}
                  onSelect={() => onSelect(index)}
                  onOpen={() => onOpen(index)}
                />
              )
            })}
          </div>
        ))}
      </div>
    </div>
  )
}
