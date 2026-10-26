// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type CameraClock } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * The camera bodies in the library and their clock corrections (FR-030). A correction moves
 * every photo of that body; the grid, groups and stamps follow through `LibraryChanged`.
 */
export function useCameraClocks() {
  const [clocks, setClocks] = useState<CameraClock[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const load = () =>
      commands.listCameras().then((r) => {
        if (r.status === 'ok') setClocks(r.data)
      })
    void load()
    // New bodies appear after an import; corrections made elsewhere show up too.
    const unlisten = events.libraryChanged.listen(() => void load())
    return () => void unlisten.then((fn) => fn())
  }, [])

  const setOffset = useCallback(async (clock: CameraClock, offsetS: number) => {
    const result = await commands.setCameraClock(clock.id, offsetS, clock.tzMinutes)
    if (result.status === 'ok') {
      setClocks(result.data)
      setError(null)
    } else {
      setError(errorText(result.error))
    }
    return result.status === 'ok'
  }, [])

  return { clocks, error, setOffset }
}
