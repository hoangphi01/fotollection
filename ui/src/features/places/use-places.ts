// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type PlaceUse } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * The places of the library's photos and the user's names for them (FR-005). A rename shows
 * on every stamp and frame at once, through `LibraryChanged`.
 */
export function usePlaces() {
  const [places, setPlaces] = useState<PlaceUse[]>([])
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(
    () =>
      commands.listPlaces().then((r) => {
        if (r.status === 'ok') setPlaces(r.data)
      }),
    [],
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
