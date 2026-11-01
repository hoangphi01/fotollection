// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useEffectEvent, useState } from 'react'
import { commands, type PhotoId, type Queue } from '@/ipc/bindings'

/**
 * Select mode (FR-050), like the Select button on an iPhone, shared by the gallery and the
 * collage sheets: photos are ticked by id (Shift for a range from the last one), the ticks
 * move to the basket, aside or back as one undoable step, and Esc leaves, unless a dialog,
 * or what `isBlocked` says (Quick-preview), has it first.
 */
export function useSelectMode(photos: readonly { id: PhotoId }[], isBlocked: () => boolean = () => false) {
  const [active, setActive] = useState(false)
  const [checked, setChecked] = useState<ReadonlySet<PhotoId>>(new Set())
  const [anchor, setAnchor] = useState<number | null>(null)

  const toggle = useCallback(
    (index: number, range = false) => {
      const id = photos[index]?.id
      if (id === undefined) return
      setChecked((prev) => {
        const next = new Set(prev)
        if (range && anchor !== null) {
          const [a, b] = [Math.min(anchor, index), Math.max(anchor, index)]
          for (const p of photos.slice(a, b + 1)) next.add(p.id)
        } else if (next.has(id)) next.delete(id)
        else next.add(id)
        return next
      })
      setAnchor(index)
    },
    [photos, anchor],
  )
  const toggleId = (id: PhotoId) => toggle(photos.findIndex((p) => p.id === id))
  /** Ticks or unticks photos together, as one collage sheet (FR-035): all ticked → none. */
  const toggleGroup = (ids: readonly PhotoId[]) =>
    setChecked((prev) => {
      const next = new Set(prev)
      const all = ids.every((id) => next.has(id))
      for (const id of ids) {
        if (all) next.delete(id)
        else next.add(id)
      }
      return next
    })
  // Ticks in other folder tabs stay (FR-056).
  const tickAll = () => setChecked((prev) => new Set([...prev, ...photos.map((p) => p.id)]))
  const clear = () => setChecked(new Set())
  /** A new view starts again with no ticks. */
  const reset = () => {
    setChecked(new Set())
    setAnchor(null)
  }
  const leave = () => {
    setActive(false)
    reset()
  }
  const move = (queue: Queue | null) => {
    const ids = [...checked]
    if (ids.length > 0) void commands.setPhotoQueue(ids, queue)
    setChecked(new Set())
  }

  const onEscape = useEffectEvent((e: KeyboardEvent) => {
    if (e.key !== 'Escape' || isBlocked() || document.querySelector('[role="dialog"]')) return
    leave()
  })
  useEffect(() => {
    if (!active) return
    window.addEventListener('keydown', onEscape)
    return () => window.removeEventListener('keydown', onEscape)
  }, [active])

  /** The ticks in the order the gallery shows them, then those in other folder tabs (FR-056). */
  const shown = photos.filter((p) => checked.has(p.id)).map((p) => p.id)
  const inView = new Set(shown)
  const ordered = [...shown, ...[...checked].filter((id) => !inView.has(id))]

  return {
    active,
    start: () => setActive(true),
    leave,
    checked,
    ordered,
    anchor,
    toggle,
    toggleId,
    toggleGroup,
    tickAll,
    clear,
    reset,
    move,
  }
}

export type SelectModeApi = ReturnType<typeof useSelectMode>
