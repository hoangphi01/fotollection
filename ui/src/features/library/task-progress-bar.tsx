// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Progress } from '@/components/ui/progress'
import { events, type TaskProgress } from '@/ipc/bindings'

/** Background task progress with a percentage (import, thumbnail generation). */
export function TaskProgressBar() {
  const { t } = useTranslation()
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

  if (!progress || progress.total === 0) return null
  const percent = Math.round((progress.done / progress.total) * 100)

  return (
    <div className="flex items-center gap-3 border-b bg-muted/40 px-4 py-2" role="status">
      <LoaderCircle className="size-3.5 shrink-0 animate-spin text-muted-foreground" />
      <span className="shrink-0 text-xs font-medium">{t(`task.${progress.task}` as const)}</span>
      <Progress value={percent} className="h-1.5 flex-1" />
      <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
        {progress.done}/{progress.total} · {percent}%
      </span>
    </div>
  )
}
