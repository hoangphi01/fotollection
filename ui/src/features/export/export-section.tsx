// SPDX-License-Identifier: GPL-3.0-only
import { CircleCheck, Download, FolderOutput, OctagonX, TriangleAlert, X } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CullFilter } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { formatBytes } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { ExportApi } from './use-export'

type ExportSectionProps = {
  exporter: ExportApi
  /** Photos kept by `filter`: exactly what will be exported (FR-022). */
  photoCount: number
  filter: CullFilter
  /** Export only this photo (the preview, FR-055) instead of the whole filtered set. */
  only?: number
  /** Source folders in the export (FR-056); from two on, they can go out together or apart. */
  folderCount?: number
}

function folderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}

/** The export controls, the same in the settings panel and in the preview (FR-055). */
export function ExportSection({
  exporter,
  photoCount,
  filter,
  only,
  folderCount = 1,
}: ExportSectionProps) {
  const { t } = useTranslation()
  const { state, destination, chooseDestination, exportPhotos, cancel, dismiss } = exporter
  const single = only !== undefined
  const [separate, setSeparate] = useState(false)
  const canSplit = !single && folderCount > 1
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
            {state.summary.sourceBytes && state.summary.outputBytes ? (
              <SizeChange from={state.summary.sourceBytes} to={state.summary.outputBytes} />
            ) : null}
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

      {canSplit && (
        <div className="space-y-1">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            className="w-full"
            disabled={running}
            value={separate ? 'separate' : 'together'}
            onValueChange={(v) => v && setSeparate(v === 'separate')}
            aria-label={t('export.folders.label')}
          >
            <ToggleGroupItem value="together" className="flex-1 text-xs">
              {t('export.folders.together')}
            </ToggleGroupItem>
            <ToggleGroupItem value="separate" className="flex-1 text-xs">
              {t('export.folders.separate')}
            </ToggleGroupItem>
          </ToggleGroup>
          <p className="text-xs text-muted-foreground">
            {t(separate ? 'export.folders.separateHint' : 'export.folders.togetherHint', { count: folderCount })}
          </p>
        </div>
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
          disabled={!single && photoCount === 0}
          onClick={() => void exportPhotos(filter, photoCount, only ?? null, canSplit && separate)}
        >
          <Download data-icon="inline-start" />
          {single
            ? t('export.buttonThis')
            : photoCount > 0
              ? t('export.buttonCount', { count: photoCount })
              : t('export.button')}
        </Button>
      )}
      {!single && (filter.flag !== 'all' || filter.minRating > 0) && !running && (
        <p className="text-xs text-muted-foreground">{t('export.filteredHint')}</p>
      )}
    </div>
  )
}

/** How much lighter the export is than the originals (FR-053); print files can be larger. */
function SizeChange({ from, to }: { from: number; to: number }) {
  const { t } = useTranslation()
  const lighter = to < from
  return (
    <p
      className={cn(
        'mt-1.5 rounded-md px-2 py-1 text-xs tabular-nums',
        lighter ? 'bg-primary/10 font-medium text-primary' : 'bg-muted text-muted-foreground',
      )}
    >
      {lighter
        ? t('export.smaller', {
            from: formatBytes(from),
            to: formatBytes(to),
            percent: Math.round((1 - to / from) * 100),
          })
        : t('export.larger', { from: formatBytes(from), to: formatBytes(to) })}
    </p>
  )
}
