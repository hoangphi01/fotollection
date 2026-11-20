// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, events, type SortingQueue } from '@/ipc/bindings'

/**
 * RollSense's queue as Rust keeps it (FR-141; owner, 2026-11-20: folders pressed in turn are all
 * read, one after another): the scope being read and those waiting. One store for the progress
 * bar, its list and the folder rows; the UI never holds a queue of its own.
 */
export function useSortingQueue(): SortingQueue | null {
  const [queue, setQueue] = useState<SortingQueue | null>(null)
  useEffect(() => {
    let live = true
    void commands.sortingQueue().then((q) => live && setQueue(q))
    const unlisten = events.sortingQueueChanged.listen(({ payload }) => setQueue(payload))
    return () => {
      live = false
      void unlisten.then((fn) => fn())
    }
  }, [])
  return queue
}

/** Two scopes name the same folders (none: the Workspace). */
export function sameScope(a: number[], b: number[]): boolean {
  return a.length === b.length && [...a].sort().every((f, i) => f === [...b].sort()[i])
}
