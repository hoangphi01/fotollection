// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'

/**
 * Whether a colour slider is being dragged, app-wide, and which drag: the preview shows the
 * look on the GPU meanwhile, from a photo loaded at each drag's start, and Rust's render
 * again once it ends (ADR-0039). `0` while nothing is dragged.
 */
let drag = 0
let count = 0
/** Whether the drag in progress can be drawn on the GPU: a RAW's sliders made in linear
 * light (FR-088) are not in the look's table, so Rust renders each of their steps. */
let gpu = true
const listeners = new Set<() => void>()

export function setDragging(next: boolean, onGpu = true) {
  if (next === (drag !== 0)) return
  drag = next ? ++count : 0
  gpu = onGpu
  listeners.forEach((l) => l())
}

/** Whether the drag in progress may be drawn on the GPU (see `setDragging`). */
export function useDragOnGpu(): boolean {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => gpu,
  )
}

/** The drag in progress, numbered from 1, or 0. */
export function useDrag(): number {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => drag,
  )
}

/**
 * The look of the library under the pointer in *Presets & LUTs* (FR-089), as Lightroom shows
 * a preset on the photo while it is hovered: the preview draws that look on the GPU, as during
 * a drag, and goes back once the pointer leaves. `null` while nothing is hovered.
 */
let hover: string | null = null

export function setHoverLook(id: string | null) {
  if (id === hover) return
  hover = id
  // A hover is a drag of its own for the GPU path: the photo loaded once, the table changed.
  if (id !== null && drag === 0) {
    drag = ++count
    gpu = true
  } else if (id === null) {
    drag = 0
  }
  listeners.forEach((l) => l())
}

/** The look hovered in *Presets & LUTs*, or `null`. */
export function useHoverLook(): string | null {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l)
      return () => listeners.delete(l)
    },
    () => hover,
  )
}
