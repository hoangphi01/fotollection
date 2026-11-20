// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import { commands, events, type Facets, type PhotoId, type SortingQueue } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * RollSense (FR-141): the facets with their counts, the pixels' pass (started by itself the
 * first time the tab opens while photos are unread), and the two ends of a sort: set aside or
 * delete (to the Trash, then for good on a volume without one, ADR-0055).
 */
export function useRollSense(folders: number[], shown: boolean, queue: SortingQueue | null) {
  // Only the tab chosen (owner, 2026-11-18): a folder's photos, or the Workspace's when none.
  const scope = JSON.stringify(folders)
  const [facets, setFacets] = useState<Facets | null>(null)
  const [error, setError] = useState<string | null>(null)
  const shownRef = useRef(shown)
  useEffect(() => {
    shownRef.current = shown
  }, [shown])

  useEffect(() => {
    // Folders ticked or unticked while RollSense is shown: the new set's unread photos are asked
    // for at once, as a press would (owner, 2026-11-18). Once per set, so a set stopped by the
    // user is not asked again by itself; Rust queues each set and reads them one after another
    // (owner, 2026-11-20), so the UI keeps no job of its own.
    let wanted = true
    const load = () =>
      commands.sortFacets(JSON.parse(scope) as number[]).then(async (result) => {
        if (result.status !== 'ok') return setError(errorText(result.error))
        setFacets(result.data)
        if (wanted && shownRef.current && result.data.analysed < result.data.total) {
          wanted = false
          const asked = await commands.analyzeSorting(JSON.parse(scope) as number[])
          if (asked.status === 'error') setError(errorText(asked.error))
        }
        wanted = false
      })
    void load()
    const unlisteners = [
      events.sortingFinished.listen(() => void load()),
      // The pass's photos as they are read, so the cards fill (owner, 2026-11-18).
      events.sortingRead.listen(() => void load()),
      events.libraryChanged.listen(() => void load()),
      events.documentChanged.listen(() => void load()),
    ]
    return () => {
      for (const unlisten of unlisteners) void unlisten.then((fn) => fn())
    }
  }, [scope])

  const analyze = useCallback(async () => {
    setError(null)
    const result = await commands.analyzeSorting(JSON.parse(scope) as number[])
    if (result.status === 'error') setError(errorText(result.error))
  }, [scope])

  /** The tab opened: the photos of the scope not read yet are read, as the user asked RollSense. */
  const open = useCallback(() => {
    if (!facets || facets.total === 0) return
    if (facets.analysed < facets.total) void analyze()
  }, [facets, analyze])

  /** *Stop all*: nothing waits any more and the scope being read stops. */
  const cancel = useCallback(() => void commands.cancelSorting(), [])

  const setAside = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'aside')
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  /** Into the print basket (FR-050): a pick gathered to print. */
  const toBasket = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'print')
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  /** Marked to delete (FR-141): out of the gallery, undoable; the files go to the Trash from
   * the sidebar's *To delete*, all at once. */
  const markDelete = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'delete')
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  return { folders, facets, running: queue?.running != null, error, analyze, open, cancel, setAside, toBasket, markDelete }
}

export type RollSense = ReturnType<typeof useRollSense>
