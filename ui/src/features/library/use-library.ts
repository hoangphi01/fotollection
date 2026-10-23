// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import {
  commands,
  events,
  type CullFilter,
  type ImportSummary,
  type PhotoSummary,
} from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/** Library state from Rust (FR-001, FR-003), filtered in Rust (FR-022). The UI only caches it. */
export function useLibrary(filter: CullFilter) {
  const [photos, setPhotos] = useState<PhotoSummary[]>([])
  const [total, setTotal] = useState(0)
  const [thumbVersions, setThumbVersions] = useState<Record<number, number>>({})
  const [lastImport, setLastImport] = useState<ImportSummary | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const load = () =>
      commands.listPhotos(filter).then((result) => {
        if (result.status === 'ok') {
          setPhotos(result.data.photos)
          setTotal(result.data.total)
        } else setError(errorText(result.error))
      })
    void load()
    const unlisteners = [
      events.libraryChanged.listen(() => void load()),
      events.thumbReady.listen(({ payload }) =>
        setThumbVersions((prev) => ({ ...prev, [payload.id]: (prev[payload.id] ?? 0) + 1 })),
      ),
    ]
    return () => {
      for (const unlisten of unlisteners) void unlisten.then((fn) => fn())
    }
  }, [filter])

  const importFolder = useCallback(async () => {
    setError(null)
    const result = await commands.pickAndImportFolder()
    if (result.status === 'error') setError(errorText(result.error))
    else if (result.data) setLastImport(result.data)
  }, [])

  return { photos, total, thumbVersions, lastImport, error, importFolder }
}
