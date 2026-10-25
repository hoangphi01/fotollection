// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, type CullFilter, type ExportEstimate } from '@/ipc/bindings'

/** Settle time after a change before estimating again: a few photos are encoded each time. */
const DELAY_MS = 400

/**
 * What the export will weigh, before it runs (FR-053): Rust encodes a few photos in memory
 * with the current settings. Refreshed when the photos or the settings (`revision`) change;
 * an answer that arrives after a newer request is dropped.
 */
export function useExportEstimate(filter: CullFilter, only: number | null, revision: number, enabled: boolean) {
  const [estimate, setEstimate] = useState<{ key: string; value: ExportEstimate } | null>(null)
  const key = `${JSON.stringify(filter)}:${only}:${revision}`
  useEffect(() => {
    if (!enabled) return
    let live = true
    const timer = window.setTimeout(() => {
      void commands.estimateExport(filter, only).then((result) => {
        if (live && result.status === 'ok') setEstimate({ key, value: result.data })
      })
    }, DELAY_MS)
    return () => {
      live = false
      window.clearTimeout(timer)
    }
  }, [key, filter, only, enabled])
  return {
    estimate: estimate?.value ?? null,
    /** The shown figure belongs to older settings while a new one is computed. */
    stale: estimate?.key !== key,
  }
}
