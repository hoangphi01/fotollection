// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDown, Clock } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { FieldLabel, Stack } from '@/components/forms/field'
import { ROW_LAYOUT } from '@/components/forms/list-row'
import { Spinner } from '@/components/forms/spinner'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { SortingQueue } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

/**
 * RollSense's queue on the progress bar (owner, 2026-11-20): shown when more than one scope is
 * asked for, a row for each, the one being read with its stop, the waiting ones with their
 * remove, and *Stop all* at the foot.
 */
export function SortingQueueMenu({
  queue,
  name,
  onStop,
  onRemove,
  onStopAll,
}: {
  queue: SortingQueue
  /** A scope's words: its folders' names, or the Workspace. */
  name: (folders: number[]) => string
  onStop: () => void
  onRemove: (folders: number[]) => void
  onStopAll: () => void
}) {
  const { t } = useTranslation()
  return (
    <Popover>
      <PopoverTrigger asChild>
        {/* Words, as small as the stop beside it, so the bar keeps its height when a second
            folder waits (nothing jumps). */}
        <Button variant="outline" size="xs" data-hint={t('rollsense.queue.show')}>
          {t('rollsense.queue.waitingCount', { count: queue.waiting.length })}
          <ChevronDown data-icon="inline-end" aria-hidden />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-panel">
        <Stack>
          <FieldLabel>{t('rollsense.queue.title')}</FieldLabel>
          <ul className="flex flex-col gap-1">
            {queue.running && (
              <li className={cn('flex h-8 items-center', ROW_LAYOUT)}>
                <Spinner className="size-4 shrink-0 text-muted-foreground" />
                <span className="min-w-0 flex-1 truncate">{name(queue.running.folders)}</span>
                <span className="shrink-0 text-xs text-muted-foreground">{t('rollsense.queue.reading')}</span>
                <RemoveButton kind="stop" iconOnly small label={t('rollsense.stop')} onClick={onStop} />
              </li>
            )}
            {queue.waiting.map((scope) => (
              <li key={scope.folders.join(',')} className={cn('flex h-8 items-center', ROW_LAYOUT)}>
                <Clock className="text-muted-foreground" aria-hidden />
                <span className="min-w-0 flex-1 truncate">{name(scope.folders)}</span>
                <span className="shrink-0 text-xs text-muted-foreground">{t('rollsense.queue.waiting')}</span>
                <RemoveButton iconOnly small label={t('rollsense.queue.remove')} onClick={() => onRemove(scope.folders)} />
              </li>
            ))}
          </ul>
          <RemoveButton kind="stop" wide label={t('rollsense.queue.stopAll')} onClick={onStopAll} />
        </Stack>
      </PopoverContent>
    </Popover>
  )
}
