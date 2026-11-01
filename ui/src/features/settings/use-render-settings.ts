// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import { commands, type RenderSettings } from '@/ipc/bindings'

/**
 * Render settings live in Rust; `revision` bumps on every change so previews refresh.
 * Changes apply at once on screen. Only the answer to the latest change replaces them: an
 * older answer arriving late would undo what was typed since (LESSONS).
 */
export function useRenderSettings() {
  const [settings, setLocal] = useState<RenderSettings | null>(null)
  const [revision, setRevision] = useState(0)
  // The settings each change builds on, and the number of the latest change.
  const current = useRef<RenderSettings | null>(null)
  const latest = useRef(0)

  const show = useCallback((next: RenderSettings) => {
    current.current = next
    setLocal(next)
  }, [])

  const reload = useCallback(() => {
    const asked = latest.current
    void commands.getSettings().then((result) => {
      if (result.status !== 'ok') return
      setRevision((r) => Math.max(r, result.data.revision))
      if (asked === latest.current) show(result.data.settings)
    })
  }, [show])

  useEffect(() => reload(), [reload])

  const update = useCallback(
    async (change: (current: RenderSettings) => RenderSettings) => {
      if (!current.current) return
      show(change(current.current)) // optimistic; Rust validates
      const asked = ++latest.current
      const result = await commands.setSettings(current.current)
      if (result.status !== 'ok') return
      setRevision((r) => Math.max(r, result.data.revision))
      if (asked === latest.current) show(result.data.settings)
    },
    [show],
  )

  return { settings, revision, update, reload }
}
