// SPDX-License-Identifier: GPL-3.0-only
import { CircleCheck, Download, FolderOutput, OctagonX, TriangleAlert, X } from 'lucide-react'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { useExport } from './use-export'

type ExportSectionProps = {
  photoCount: number
}

function folderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}

export function ExportSection({ photoCount }: ExportSectionProps) {
  const { state, destination, chooseDestination, exportAll, cancel, dismiss } = useExport()
  const running = state.kind === 'running'

  return (
    <div className="space-y-3">
      {state.kind === 'running' && (
        <div className="space-y-1.5">
          <div className="flex justify-between text-xs text-muted-foreground tabular-nums">
            <span>Exporting…</span>
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
            {state.summary.cancelled ? 'Export cancelled' : 'Export finished'}
          </AlertTitle>
          <AlertDescription>
            <p>
              {state.summary.exported} exported
              {state.summary.failed.length > 0 && `, ${state.summary.failed.length} failed`} →{' '}
              {folderName(state.summary.destination)}
            </p>
            {state.summary.failed.slice(0, 3).map((f) => (
              <p key={f.id} className="truncate text-xs" title={f.message}>
                {f.fileName}: {f.message}
              </p>
            ))}
          </AlertDescription>
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Dismiss"
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
          <AlertTitle>Couldn't export</AlertTitle>
          <AlertDescription>{state.message}</AlertDescription>
        </Alert>
      )}

      <button
        type="button"
        onClick={() => void chooseDestination()}
        disabled={running}
        className="flex w-full items-center gap-2 rounded-md text-left text-xs text-muted-foreground hover:text-foreground disabled:opacity-50"
        title={destination ?? undefined}
      >
        <FolderOutput className="size-3.5 shrink-0" />
        <span className="truncate">
          {destination ? folderName(destination) : 'Choose destination…'}
        </span>
      </button>

      {running ? (
        <Button variant="outline" className="w-full" onClick={() => void cancel()}>
          <X data-icon="inline-start" />
          Cancel
        </Button>
      ) : (
        <Button
          className="w-full"
          disabled={photoCount === 0}
          onClick={() => void exportAll(photoCount)}
        >
          <Download data-icon="inline-start" />
          Export {photoCount > 0 ? `${photoCount} photos` : ''}
        </Button>
      )}
    </div>
  )
}
