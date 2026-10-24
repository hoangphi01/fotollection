// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import {
  commands,
  events,
  type AnalysisFinished,
  type Group,
  type PhotoSummary,
} from '@/ipc/bindings'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { errorText } from '@/lib/errors'

/** Best-shot suggestions (FR-041): the analysis job, its groups, and Keep best. */
export function useBestShots() {
  const [groups, setGroups] = useState<Group[]>([])
  // Every photo, unfiltered: the Groups view shows names and flags whatever the grid filter.
  const [photosById, setPhotosById] = useState<Map<number, PhotoSummary>>(new Map())
  const [jobId, setJobId] = useState<number | null>(null)
  const [finished, setFinished] = useState<AnalysisFinished | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const load = () =>
      Promise.all([commands.listGroups(), commands.listPhotos(ALL_PHOTOS)]).then(
        ([groupsResult, photosResult]) => {
          if (groupsResult.status === 'ok') setGroups(groupsResult.data)
          else setError(errorText(groupsResult.error))
          if (photosResult.status === 'ok')
            setPhotosById(new Map(photosResult.data.photos.map((p) => [p.id, p])))
        },
      )
    void load()
    const unlisteners = [
      events.analysisFinished.listen(({ payload }) => {
        setJobId(null)
        setFinished(payload)
        void load()
      }),
      // Flags change with Keep best and undo; another project clears the groups.
      events.libraryChanged.listen(() => void load()),
      events.documentChanged.listen(() => void load()),
    ]
    return () => {
      for (const unlisten of unlisteners) void unlisten.then((fn) => fn())
    }
  }, [])

  const analyze = useCallback(async () => {
    setError(null)
    const result = await commands.analyzeLibrary()
    if (result.status === 'ok') setJobId(result.data)
    else setError(errorText(result.error))
  }, [])

  const cancel = useCallback(() => {
    if (jobId !== null) void commands.cancelJob(jobId)
  }, [jobId])

  const keepBest = useCallback(async (ids: number[]) => {
    if (ids.length === 0) return
    const result = await commands.keepBest(ids)
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  const keepOne = useCallback(async (group: number, photo: number) => {
    const result = await commands.keepOne(group, photo)
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  return {
    groups,
    photosById,
    running: jobId !== null,
    finished,
    error,
    analyze,
    cancel,
    keepBest,
    keepOne,
  }
}

export type BestShotsApi = ReturnType<typeof useBestShots>
