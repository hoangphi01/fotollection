// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { events, type TaskProgress } from '@/ipc/bindings'

/**
 * The background task in progress (import, thumbnail generation), or `null` when none is:
 * the caller decides whether a notice is drawn, so the notice stack knows when it is empty.
 */
export function useTaskProgress(): TaskProgress | null {
  const [progress, setProgress] = useState<TaskProgress | null>(null)

  useEffect(() => {
    let hideTimer: ReturnType<typeof setTimeout> | undefined
    const unlisten = events.taskProgress.listen(({ payload }) => {
      clearTimeout(hideTimer)
      setProgress(payload)
      if (payload.done >= payload.total) hideTimer = setTimeout(() => setProgress(null), 800)
    })
    return () => {
      clearTimeout(hideTimer)
      void unlisten.then((fn) => fn())
    }
  }, [])

  return progress && progress.total > 0 ? progress : null
}
