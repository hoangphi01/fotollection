// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import { commands, events, type DeleteReport, type Facets, type PhotoId } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * RollSense (FR-141): the facets with their counts, the pixels' pass (started by itself the
 * first time the tab opens while photos are unread), and the two ends of a sort: set aside or
 * delete (to the Trash, then for good on a volume without one, ADR-0055).
 */
export function useRollSense(folders: number[], shown: boolean) {
  // Only the tab chosen (owner, 2026-11-18): a folder's photos, or the Workspace's when none.
  const scope = JSON.stringify(folders)
  const [facets, setFacets] = useState<Facets | null>(null)
  const [jobId, setJobId] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const shownRef = useRef(shown)
  useEffect(() => {
    shownRef.current = shown
  }, [shown])

  useEffect(() => {
    // Folders ticked or unticked while RollSense is shown: the new set's unread photos are read
    // at once, as a press would (owner, 2026-11-18); only on the set's first load, so a pass
    // stopped by the user is not started again.
    let first = true
    const load = () =>
      commands.sortFacets(JSON.parse(scope) as number[]).then(async (result) => {
        if (result.status !== 'ok') return setError(errorText(result.error))
        setFacets(result.data)
        if (first && shownRef.current && result.data.analysed < result.data.total) {
          const job = await commands.analyzeSorting(JSON.parse(scope) as number[])
          if (job.status === 'ok') setJobId(job.data)
          else setError(errorText(job.error))
        }
        first = false
      })
    void load()
    const unlisteners = [
      events.sortingFinished.listen(() => {
        setJobId(null)
        void load()
      }),
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
    if (result.status === 'ok') setJobId(result.data)
    else setError(errorText(result.error))
  }, [scope])

  /** The tab opened: the photos of the scope not read yet are read, as the user asked RollSense. */
  const open = useCallback(() => {
    if (!facets || facets.total === 0) return
    if (facets.analysed < facets.total) void analyze()
  }, [facets, analyze])

  const cancel = useCallback(() => {
    if (jobId !== null) void commands.cancelJob(jobId)
  }, [jobId])

  const setAside = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'aside')
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  /** Into the print basket (FR-050): a pick gathered to print. */
  const toBasket = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'print')
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  /** Moves the files to the Trash, or deletes them for good; what could not be moved comes back. */
  const remove = useCallback(async (ids: PhotoId[], forGood: boolean): Promise<DeleteReport | null> => {
    setError(null)
    const result = forGood ? await commands.deletePhotos(ids) : await commands.trashPhotos(ids)
    if (result.status === 'ok') return result.data
    setError(errorText(result.error))
    return null
  }, [])

  return { facets, running: jobId !== null, error, analyze, open, cancel, setAside, toBasket, remove }
}

export type RollSense = ReturnType<typeof useRollSense>
