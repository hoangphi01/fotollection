// SPDX-License-Identifier: GPL-3.0-only
import { useVirtualizer } from '@tanstack/react-virtual'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoSummary } from '@/ipc/bindings'
import { handleCullKey } from '@/features/culling/cull-keys'
import type { GridSection } from '@/features/rollsense/use-gallery-sections'
import { GridSectionHeader } from '@/components/layout/grid-section-header'
import { ThumbTile } from './thumb-tile'

/** Ids of the photos ticked in select mode (FR-050). */
type Checked = ReadonlySet<number>

const TILE = 168
const GAP = 8
/** A section's header row (FR-141 C2, `GridSectionHeader`), 8 px of it the gap to its photos. */
const HEADER = 32

/** A row of the grid: a section's header, or photos `start` to `end` (exclusive). */
type Row = { header: GridSection } | { start: number; end: number }

type LibraryGridProps = {
  photos: PhotoSummary[]
  thumbVersions: Record<number, number>
  generation: number
  selected: number
  onSelect: (index: number) => void
  onOpen: (index: number) => void
  /** Select mode (FR-050): clicks tick photos instead of selecting one. */
  selectMode: boolean
  checked: Checked
  /** Ticks or unticks a photo; with `range`, every photo from the last one ticked. */
  onToggle: (index: number, range: boolean) => void
  onToggleAll: () => void
  /** The photo under the pointer, for Quick-preview (FR-059). */
  onHover: (index: number | null) => void
  /** Runs of photos under a header each (RollSense's time picks, FR-141 C2); none, one grid. */
  sections?: GridSection[]
}

/** Virtualized thumbnail grid (FR-003, NFR-001): only visible rows are in the DOM. */
export function LibraryGrid({
  photos,
  thumbVersions,
  generation,
  selected,
  onSelect,
  onOpen,
  selectMode,
  checked,
  onToggle,
  onToggleAll,
  onHover,
  sections = [],
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
  // Each section starts a new row under its header; without sections, one run of every photo.
  const model: Row[] = []
  for (const run of sections.length ? sections : [null]) {
    if (run) model.push({ header: run })
    const [from, to] = run ? [run.start, run.start + run.count] : [0, photos.length]
    for (let s = from; s < to; s += columns) model.push({ start: s, end: Math.min(s + columns, to) })
  }
  const rowOf = (index: number) => model.findIndex((r) => 'start' in r && index >= r.start && index < r.end)
  // TanStack Virtual returns non-memoizable functions by design; the grid re-renders
  // on scroll anyway, so skipping React Compiler memoization here is fine.
  // oxlint-disable-next-line react/incompatible-library
  const virtualizer = useVirtualizer({
    count: model.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: (i) => (model[i] && 'header' in model[i] ? HEADER : TILE + GAP),
    overscan: 4,
  })
  // The rows' heights change with the sections.
  const shape = sections.map((s) => s.count).join()
  useEffect(() => virtualizer.measure(), [shape, columns, virtualizer])

  const selectedRow = rowOf(selected)
  // Scrolls only when the selected photo itself changes (or the columns do), never because
  // headers arrived and moved its row (owner, 2026-11-19: the grid scrolled by itself); the
  // first photo brings the top into view, its header with it.
  const selectedId = photos[selected]?.id
  const scrollTo = useRef({ selected, selectedRow, virtualizer })
  scrollTo.current = { selected, selectedRow, virtualizer }
  useEffect(() => {
    const { selected: index, selectedRow: row, virtualizer: v } = scrollTo.current
    if (index === 0) v.scrollToOffset(0)
    else if (row >= 0) v.scrollToIndex(row, { align: 'auto' })
  }, [selectedId, columns])

  /** The photo a row up or down, in the same column or the row's last, past headers. */
  const vertical = (step: number) => {
    const at = model[selectedRow]
    if (!at || !('start' in at)) return selected
    for (let r = selectedRow + step; r >= 0 && r < model.length; r += step) {
      const row = model[r]
      if (row && 'start' in row) return Math.min(row.start + (selected - at.start), row.end - 1)
    }
    return selected
  }

  const onKeyDown = (event: KeyboardEvent) => {
    const moves: Record<string, () => number> = {
      ArrowRight: () => selected + 1,
      ArrowLeft: () => selected - 1,
      ArrowDown: () => vertical(1),
      ArrowUp: () => vertical(-1),
    }
    const move = moves[event.key]
    if (move !== undefined) {
      event.preventDefault()
      onSelect(Math.min(photos.length - 1, Math.max(0, move())))
    } else if (selectMode && event.key === 'Enter') {
      event.preventDefault()
      onToggle(selected, false)
    } else if (selectMode && event.key.toLowerCase() === 'a' && (event.metaKey || event.ctrlKey)) {
      event.preventDefault()
      onToggleAll()
    } else if (selectMode && event.key === ' ') {
      // Held Space is Quick-preview in select mode (FR-059), handled app-wide.
      event.preventDefault()
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
        {virtualizer.getVirtualItems().map((item) => {
          const row = model[item.index]
          if (!row) return null
          if ('header' in row)
            return (
              <GridSectionHeader
                key={item.key}
                label={row.header.label}
                trailing={row.header.trailing}
                style={{ top: item.start, left: GAP, height: HEADER - GAP, width: `calc(100% - ${2 * GAP}px)` }}
              />
            )
          return (
            <div key={item.key} className="absolute left-0 flex w-full" style={{ top: item.start, gap: GAP, paddingLeft: GAP }}>
              {photos.slice(row.start, row.end).map((photo, i) => {
                const index = row.start + i
                return (
                  <ThumbTile
                    key={photo.id}
                    photo={photo}
                    version={thumbVersions[photo.id] ?? 0}
                    generation={generation}
                    size={TILE}
                    selected={index === selected}
                    selectMode={selectMode}
                    checked={checked.has(photo.id)}
                    onSelect={(e) => {
                      onSelect(index)
                      if (selectMode) onToggle(index, e.shiftKey)
                    }}
                    onOpen={() => onOpen(index)}
                    onHover={(over) => onHover(over ? index : null)}
                  />
                )
              })}
            </div>
          )
        })}
      </div>
    </div>
  )
}
