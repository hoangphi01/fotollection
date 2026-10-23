// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type UserFont } from '@/ipc/bindings'

/** Imported OTF/TTF fonts (FR-042). Refreshes after removals (which bump the revision). */
export function useFonts() {
  const [fonts, setFonts] = useState<UserFont[]>([])
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    const result = await commands.listFonts()
    if (result.status === 'ok') setFonts(result.data)
  }, [])

  useEffect(() => {
    let live = true
    void commands.listFonts().then((r) => live && r.status === 'ok' && setFonts(r.data))
    const unlisten = events.revisionChanged.listen(() => void refresh())
    return () => {
      live = false
      void unlisten.then((fn) => fn())
    }
  }, [refresh])

  /** Opens the font picker; resolves to the new font, or null if cancelled or invalid. */
  const importFont = useCallback(async (): Promise<UserFont | null> => {
    const result = await commands.importFont()
    if (result.status === 'error') {
      setError(result.error.message)
      return null
    }
    setError(null)
    await refresh()
    return result.data
  }, [refresh])

  const removeFont = useCallback(
    async (id: number) => {
      await commands.removeFont(id)
      await refresh()
    },
    [refresh],
  )

  return { fonts, error, importFont, removeFont }
}

export type FontsApi = ReturnType<typeof useFonts>
