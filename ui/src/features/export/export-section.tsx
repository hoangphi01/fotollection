// SPDX-License-Identifier: GPL-3.0-only
import { CircleCheck, Download, FolderOutput, OctagonX, TriangleAlert, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import type { CullFilter } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useExport } from './use-export'

type ExportSectionProps = {
  /** Photos kept by `filter`: exactly what will be exported (FR-022). */
  photoCount: number
  filter: CullFilter
}

function folderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}

export function ExportSection({ photoCount, filter }: ExportSectionProps) {
  const { t } = useTranslation()
  const { state, destination, chooseDestination, exportAll, cancel, dismiss } = useExport()
  const running = state.kind === 'running'

  return (
    <div className="space-y-3">
      {state.kind === 'running' && (
        <div className="space-y-1.5">
          <div className="flex justify-between text-xs text-muted-foreground tabular-nums">
            <span>{t('export.running')}</span>
            <span>
              {state.done} / {state.total}
            </span>
          </div>
          <Progress value={state.total ? (state.done / state.total) * 100 : 0} />
        </div>
      )}

      {state.kind === 'finished' && (
        <Alert variant={state.summary.failed.length ? 'destructive' : 'default'}>
          {state.summary.cancelled ? (
            <TriangleAlert />
          ) : state.summary.failed.length ? (
            <OctagonX />
          ) : (
            <CircleCheck />
          )}
          <AlertTitle>
            {state.summary.cancelled ? t('export.cancelled') : t('export.finished')}
          </AlertTitle>
          <AlertDescription>
            <p>
              {t('export.exported', { count: state.summary.exported })}
              {state.summary.failed.length > 0 &&
                `, ${t('export.failed', { count: state.summary.failed.length })}`}{' '}
              →{' '}
              {folderName(state.summary.destination)}
            </p>
            {state.summary.failed.slice(0, 3).map((f) => (
              <p key={f.id} className="truncate text-xs" data-hint={f.message}>
                {f.fileName}: {errorText(f)}
              </p>
            ))}
          </AlertDescription>
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label={t('common.dismiss')}
            className="absolute top-2 right-2"
            onClick={dismiss}
          >
            <X />
          </Button>
        </Alert>
      )}

      {state.kind === 'error' && (
        <Alert variant="destructive">
          <OctagonX />
          <AlertTitle>{t('export.error')}</AlertTitle>
          <AlertDescription>{state.message}</AlertDescription>
        </Alert>
      )}

      <button
        type="button"
        onClick={() => void chooseDestination()}
        disabled={running}
        className="flex w-full items-center gap-2 rounded-md text-left text-xs text-muted-foreground hover:text-foreground disabled:opacity-50"
        data-hint={destination ?? undefined}
      >
        <FolderOutput className="size-3.5 shrink-0" />
        <span className="truncate">
          {destination ? folderName(destination) : t('export.chooseDestination')}
        </span>
      </button>

      {running ? (
        <Button variant="outline" className="w-full" onClick={() => void cancel()}>
          <X data-icon="inline-start" />
          {t('common.cancel')}
        </Button>
      ) : (
        <Button
          className="w-full"
          disabled={photoCount === 0}
          onClick={() => void exportAll(filter, photoCount)}
        >
          <Download data-icon="inline-start" />
          {photoCount > 0 ? t('export.buttonCount', { count: photoCount }) : t('export.button')}
        </Button>
      )}
      {(filter.flag !== 'all' || filter.minRating > 0) && !running && (
        <p className="text-xs text-muted-foreground">{t('export.filteredHint')}</p>
      )}
    </div>
  )
}
