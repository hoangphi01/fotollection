// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type PlaceUse } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * The places of the photos in `folders` (all when empty: the Workspace) and the user's names
 * for them (FR-005): the right column is the folders shown's (owner, 2026-11-19). A rename
 * shows on every stamp and frame at once, through `LibraryChanged`.
 */
export function usePlaces(folders: number[]) {
  const scope = folders.join()
  const [places, setPlaces] = useState<PlaceUse[]>([])
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(
    () =>
      commands.listPlaces(scope ? scope.split(',').map(Number) : []).then((r) => {
        if (r.status === 'ok') setPlaces(r.data)
      }),
    [scope],
  )

  useEffect(() => {
    void load()
    const unlisten = events.libraryChanged.listen(() => void load())
    return () => void unlisten.then((fn) => fn())
  }, [load])

  const rename = useCallback(async (place: PlaceUse, name: string | null) => {
    const r = await commands.renamePlace(place.geonamesId, name)
    setError(r.status === 'ok' ? null : errorText(r.error))
  }, [])

  return { places, error, rename }
}
