// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type CullFilter, type ExportSummary } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

export type ExportState =
  | { kind: 'idle' }
  | { kind: 'running'; jobId: number; done: number; total: number }
  | { kind: 'finished'; summary: ExportSummary }
  | { kind: 'error'; message: string }

/** Export flow (FR-008, FR-009, FR-011): destination dialog → job → progress events. */
export function useExport() {
  const [state, setState] = useState<ExportState>({ kind: 'idle' })
  const [destination, setDestination] = useState<string | null>(null)

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
  }, [])

  const start = useCallback(async (filter: CullFilter, total: number) => {
    const result = await commands.startExport(filter)
    if (result.status === 'ok') setState({ kind: 'running', jobId: result.data, done: 0, total })
    else setState({ kind: 'error', message: errorText(result.error) })
  }, [])

  /** Picks a destination if none yet, then exports. */
  const exportAll = useCallback(
    async (filter: CullFilter, total: number) => {
      const dest = destination ?? (await chooseDestination())
      if (dest) await start(filter, total)
    },
    [destination, chooseDestination, start],
  )

  const cancel = useCallback(async () => {
    if (state.kind === 'running') await commands.cancelJob(state.jobId)
  }, [state])

  const dismiss = useCallback(() => setState({ kind: 'idle' }), [])

  return { state, destination, chooseDestination, exportAll, cancel, dismiss }
}
