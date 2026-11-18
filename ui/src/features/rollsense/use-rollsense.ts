// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import { commands, events, type DeleteReport, type Facets, type PhotoId } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * RollSense (FR-141): the facets with their counts, the pixels' pass (started by itself the
 * first time the tab opens while photos are unread), and the two ends of a sort: set aside or
 * delete (to the Trash, then for good on a volume without one, ADR-0055).
 */
export function useRollSense() {
  const [facets, setFacets] = useState<Facets | null>(null)
  const [jobId, setJobId] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const started = useRef(false)

  useEffect(() => {
    const load = () =>
      commands.sortFacets().then((result) => {
        if (result.status === 'ok') setFacets(result.data)
        else setError(errorText(result.error))
      })
    void load()
    const unlisteners = [
      events.sortingFinished.listen(() => {
        setJobId(null)
        void load()
      }),
      events.libraryChanged.listen(() => void load()),
      events.documentChanged.listen(() => {
        // Another library: its photos are read again when the tab is next opened.
        started.current = false
        void load()
      }),
    ]
    return () => {
      for (const unlisten of unlisteners) void unlisten.then((fn) => fn())
    }
  }, [])

  const analyze = useCallback(async () => {
    setError(null)
    const result = await commands.analyzeSorting()
    if (result.status === 'ok') setJobId(result.data)
    else setError(errorText(result.error))
  }, [])

  /** The tab opened: the first look reads the photos not read yet, as the user asked RollSense. */
  const open = useCallback(() => {
    if (started.current || !facets || facets.total === 0) return
    started.current = true
    if (facets.analysed < facets.total) void analyze()
  }, [facets, analyze])

  const cancel = useCallback(() => {
    if (jobId !== null) void commands.cancelJob(jobId)
  }, [jobId])

  const setAside = useCallback(async (ids: PhotoId[]) => {
    const result = await commands.setPhotoQueue(ids, 'aside')
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

  return { facets, running: jobId !== null, error, analyze, open, cancel, setAside, remove }
}

export type RollSense = ReturnType<typeof useRollSense>
