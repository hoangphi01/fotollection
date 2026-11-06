// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type CullFilter, type ExportSummary, type PhotoId } from '@/ipc/bindings'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { errorText } from '@/lib/errors'

export type ExportState =
  | { kind: 'idle' }
  | { kind: 'running'; jobId: number; done: number; total: number }
  | { kind: 'finished'; summary: ExportSummary }
  | { kind: 'error'; message: string }

/**
 * Export flow (FR-008, FR-009, FR-011): destination dialog → job → progress events. Called
 * once in the app, so the settings panel and the preview share one destination and job.
 */
export function useExport() {
  const [state, setState] = useState<ExportState>({ kind: 'idle' })
  // Kept in the per-machine settings, so it survives a restart (FR-093).
  const { ui, update } = useUiSettings()
  const destination = ui.exportDestination
  const setDestination = useCallback((d: string) => void update({ exportDestination: d }), [update])

  useEffect(() => {
    const unlisteners = [
      events.exportProgress.listen(({ payload }) =>
        setState((prev) =>
          prev.kind === 'running' && prev.jobId === payload.jobId
            ? { ...prev, done: payload.done, total: payload.total }
            : prev,
        ),
      ),
      events.exportFinished.listen(({ payload }) =>
        setState({ kind: 'finished', summary: payload.summary }),
      ),
    ]
    return () => {
      for (const unlisten of unlisteners) void unlisten.then((fn) => fn())
    }
  }, [])

  const chooseDestination = useCallback(async () => {
    const result = await commands.pickExportFolder()
    if (result.status === 'ok') {
      if (result.data) setDestination(result.data)
      return result.data
    }
    setState({ kind: 'error', message: errorText(result.error) })
    return null
  }, [setDestination])

  const start = useCallback(
    async (
      filter: CullFilter,
      total: number,
      only: number | null,
      ticked: PhotoId[],
      separate: boolean,
      sheets: boolean,
    ) => {
    const result = await commands.startExport(filter, only, ticked, separate, sheets)
    if (result.status === 'ok') setState({ kind: 'running', jobId: result.data, done: 0, total })
    else setState({ kind: 'error', message: errorText(result.error) })
    },
    [],
  )

  /**
   * Picks a destination if none yet, then exports the photos `filter` keeps, or only the
   * photo `only` (the one open in the preview, FR-055). `separate` gives each source folder
   * its own subfolder (FR-056). With `sheets`, the photos go out two to a print and
   * `total` counts sheets (FR-035). Photos `ticked` in select mode go out instead of the
   * whole view (FR-050).
   */
  const exportPhotos = useCallback(
    async (
      filter: CullFilter,
      total: number,
      only: number | null = null,
      separate = false,
      sheets = false,
      ticked: readonly PhotoId[] = [],
    ) => {
      const dest = destination ?? (await chooseDestination())
      if (dest) await start(filter, only === null ? total : 1, only, [...ticked], separate, sheets)
    },
    [destination, chooseDestination, start],
  )

  const cancel = useCallback(async () => {
    if (state.kind === 'running') await commands.cancelJob(state.jobId)
  }, [state])

  const dismiss = useCallback(() => setState({ kind: 'idle' }), [])

  return { state, destination, chooseDestination, exportPhotos, cancel, dismiss }
}

export type ExportApi = ReturnType<typeof useExport>
