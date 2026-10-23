// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, type RenderSettings } from '@/ipc/bindings'

/** Render settings live in Rust; `revision` bumps on every change so previews refresh. */
export function useRenderSettings() {
  const [settings, setLocal] = useState<RenderSettings | null>(null)
  const [revision, setRevision] = useState(0)

  useEffect(() => {
    void commands.getSettings().then((result) => {
      if (result.status === 'ok') {
        setLocal(result.data.settings)
        setRevision(result.data.revision)
      }
    })
  }, [])

  const update = useCallback(
    async (change: (current: RenderSettings) => RenderSettings) => {
      if (!settings) return
      const next = change(settings)
      setLocal(next) // optimistic; Rust validates
      const result = await commands.setSettings(next)
      if (result.status === 'ok') {
        setLocal(result.data.settings)
        setRevision(result.data.revision)
      }
    },
    [settings],
  )

  return { settings, revision, update }
}
