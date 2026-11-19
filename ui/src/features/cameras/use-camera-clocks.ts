// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type CameraClock } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * The camera bodies with photos in `folders` (all when empty: the Workspace) and their clock
 * corrections (FR-030): the right column is the folders shown's (owner, 2026-11-19). A
 * correction moves every photo of that body; the grid, groups and stamps follow through
 * `LibraryChanged`.
 */
export function useCameraClocks(folders: number[]) {
  const scope = folders.join()
  const [clocks, setClocks] = useState<CameraClock[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const load = () =>
      commands.listCameras(scope ? scope.split(',').map(Number) : []).then((r) => {
        if (r.status === 'ok') setClocks(r.data)
      })
    void load()
    // New bodies appear after an import; corrections made elsewhere show up too.
    const unlisten = events.libraryChanged.listen(() => void load())
    return () => void unlisten.then((fn) => fn())
  }, [scope])

  const setOffset = useCallback(async (clock: CameraClock, offsetS: number) => {
    const result = await commands.setCameraClock(clock.id, offsetS, clock.tzMinutes)
    if (result.status === 'ok') {
      // The answer lists every body: keep the folders shown's, updated.
      setClocks((shown) => shown.map((c) => result.data.find((d) => d.id === c.id) ?? c))
      setError(null)
    } else {
      setError(errorText(result.error))
    }
    return result.status === 'ok'
  }, [])

  return { clocks, error, setOffset }
}
