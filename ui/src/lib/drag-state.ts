// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'

/**
 * Whether a colour slider is being dragged, app-wide, and which drag: the preview shows the
 * look on the GPU meanwhile, from a photo loaded at each drag's start, and Rust's render
 * again once it ends (ADR-0039). `0` while nothing is dragged.
 */
let drag = 0
let count = 0
const listeners = new Set<() => void>()

export function setDragging(next: boolean) {
  if (next === (drag !== 0)) return
  drag = next ? ++count : 0
  listeners.forEach((l) => l())
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
