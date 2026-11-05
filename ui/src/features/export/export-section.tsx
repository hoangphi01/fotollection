// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CircleCheck, Download, FolderOutput, OctagonX, TriangleAlert, X } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Field, Hint, Stack } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import type { CullFilter, ExportFolders, PhotoId } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { formatBytes } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { ExportApi } from './use-export'
import { useExportEstimate } from './use-export-estimate'

type ExportSectionProps = {
  exporter: ExportApi
  /** Photos kept by `filter`: exactly what will be exported (FR-022). */
  photoCount: number
  filter: CullFilter
  /** Export only this photo (the preview, FR-055) instead of the whole filtered set. */
  only?: number
  /** Source folders in the export (FR-056); from two on, they can go out together or apart. */
  folderCount?: number
  /** Settings revision, so the size estimate follows every change. */
  revision: number
  /** Collage sheets the photos make, when they go out two to a print (FR-035). */
  sheets?: number
  /** Photos ticked in select mode: when there are some, only they go out (FR-050); in
   * Collage, the photos of the ticked sheets, with `sheets` counting those sheets. */
  ticked?: readonly PhotoId[]
}

const NO_TICKS: readonly PhotoId[] = []

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
  revision,
  sheets,
  ticked = NO_TICKS,
}: ExportSectionProps) {
  const { t } = useTranslation()
  const { state, destination, chooseDestination, exportPhotos, cancel, dismiss } = exporter
  const single = only !== undefined
  const [separate, setSeparate] = useState(false)
  // Into one folder or a folder per day (FR-009), remembered on this computer.
  const { ui, update } = useUiSettings()
  const byDay = ui.exportFolders === 'byDay'
  const collage = sheets !== undefined
  const picked = single ? NO_TICKS : ticked
  const count = picked.length > 0 ? picked.length : photoCount
  const canSplit = !single && !collage && folderCount > 1
  const running = state.kind === 'running'
  const { estimate, stale } = useExportEstimate(
    filter,
    only ?? null,
    revision,
    !running && (single || count > 0),
    collage,
    picked,
  )

  return (
    <Stack>
      {state.kind === 'running' && (
        <Field>
          <div className="flex justify-between text-xs text-muted-foreground tabular-nums">
            <span>{t('export.running')}</span>
            <span>
              {state.done} / {state.total}
            </span>
          </div>
          <Progress value={state.total ? (state.done / state.total) * 100 : 0} />
        </Field>
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
        <Field
          hint={t(separate ? 'export.folders.separateHint' : 'export.folders.togetherHint', { count: folderCount })}
        >
          <SegmentedToggle
            disabled={running}
            value={separate ? 'separate' : 'together'}
            options={[
              { value: 'together', label: t('export.folders.together') },
              { value: 'separate', label: t('export.folders.separate') },
            ]}
            onChange={(v) => setSeparate(v === 'separate')}
            ariaLabel={t('export.folders.label')}
          />
        </Field>
      )}

      <Field hint={t(byDay ? 'export.days.byDayHint' : 'export.days.oneFolderHint')}>
        <SegmentedToggle<ExportFolders>
          disabled={running}
          value={ui.exportFolders}
          options={[
            { value: 'oneFolder', label: t('export.days.oneFolder') },
            { value: 'byDay', label: t('export.days.byDay') },
          ]}
          onChange={(exportFolders) => void update({ exportFolders })}
          ariaLabel={t('export.days.label')}
        />
      </Field>

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

      {!running && estimate && estimate.photos > 0 && estimate.sourceBytes && estimate.estimatedBytes ? (
        <SizeChange
          from={estimate.sourceBytes}
          to={estimate.estimatedBytes}
          estimate={estimate.sampled !== estimate.photos}
          before
          stale={stale}
        />
      ) : !running && !estimate && count > 0 ? (
        // The figure's room, kept while the first estimate is computed, so the footer never grows.
        <p className="h-6" aria-hidden />
      ) : null}

      {running ? (
        <Button variant="outline" className="w-full" onClick={() => void cancel()}>
          <X data-icon="inline-start" />
          {t('common.cancel')}
        </Button>
      ) : (
        <Button
          className="w-full"
          disabled={!single && count === 0}
          onClick={() =>
            void exportPhotos(filter, sheets ?? count, only ?? null, canSplit && separate, collage, picked)
          }
        >
          <Download data-icon="inline-start" />
          {single
            ? t(collage ? 'export.buttonThisSheet' : 'export.buttonThis')
            : sheets !== undefined && sheets > 0
              ? t(picked.length > 0 ? 'export.buttonSheetsTicked' : 'export.buttonSheets', { count: sheets })
              : picked.length > 0
              ? t('export.buttonTicked', { count })
              : photoCount > 0
              ? t('export.buttonCount', { count: photoCount })
              : t('export.button')}
        </Button>
      )}
      {!single && picked.length === 0 && (filter.flag !== 'all' || filter.minRating > 0) && !running && (
        <Hint>{t('export.filteredHint')}</Hint>
      )}
    </Stack>
  )
}

/**
 * How much lighter the export is than the originals (FR-053), before it runs (`before`,
 * an estimate unless every photo was sampled) or after; print files can be larger.
 */
function SizeChange({
  from,
  to,
  before = false,
  estimate = false,
  stale = false,
}: {
  from: number
  to: number
  before?: boolean
  estimate?: boolean
  stale?: boolean
}) {
  const { t } = useTranslation()
  const lighter = to < from
  const values = {
    from: formatBytes(from),
    to: `${estimate ? '≈ ' : ''}${formatBytes(to)}`,
    percent: Math.round((1 - to / from) * 100),
  }
  const key = before
    ? lighter
      ? 'estimate.smaller'
      : 'estimate.larger'
    : lighter
      ? 'export.smaller'
      : 'export.larger'
  return (
    <p
      className={cn(
        'rounded-md px-2 py-1 text-xs tabular-nums transition-opacity',
        before ? '' : 'mt-1.5',
        lighter ? 'bg-primary/10 font-medium text-primary' : 'bg-muted text-muted-foreground',
        // Dimmed only when the new figure is slow to come: a quick refresh while typing
        // never blinks (STYLE_RULE §4).
        stale && 'opacity-50 delay-700',
      )}
    >
      {t(key, values)}
    </p>
  )
}
