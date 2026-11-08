// SPDX-License-Identifier: GPL-3.0-only
import { Spinner } from '@/components/forms/spinner'
import { useTranslation } from 'react-i18next'
import { NoticeCard } from '@/components/layout/notice-stack'
import { Progress } from '@/components/ui/progress'
import type { TaskProgress } from '@/ipc/bindings'

/** A task's progress with a percentage, as a block of the notice stack. */
export function TaskProgressBar({ progress }: { progress: TaskProgress }) {
  const { t } = useTranslation()
  const percent = Math.round((progress.done / progress.total) * 100)

  return (
    <NoticeCard className="flex items-center gap-2" role="status">
      <Spinner className="size-3.5 shrink-0 text-muted-foreground" />
      <span className="shrink-0 text-xs font-medium">{t(`task.${progress.task}` as const)}</span>
      <Progress value={percent} className="h-1.5 flex-1" />
      <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
        {progress.done}/{progress.total} · {percent}%
      </span>
    </NoticeCard>
  )
}
