// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type ImportSummary, type PhotoSummary } from '@/ipc/bindings'

/** Library state from Rust (FR-001, FR-003). The UI only caches it. */
export function useLibrary() {
  const [photos, setPhotos] = useState<PhotoSummary[]>([])
  const [thumbVersions, setThumbVersions] = useState<Record<number, number>>({})
  const [lastImport, setLastImport] = useState<ImportSummary | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const load = () =>
      commands.listPhotos().then((result) => {
        if (result.status === 'ok') setPhotos(result.data)
        else setError(result.error.message)
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
  }, [])

  const importFolder = useCallback(async () => {
    setError(null)
    const result = await commands.pickAndImportFolder()
    if (result.status === 'error') setError(result.error.message)
    else if (result.data) setLastImport(result.data)
  }, [])

  return { photos, thumbVersions, lastImport, error, importFolder }
}
