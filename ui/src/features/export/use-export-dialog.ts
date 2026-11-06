// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'
import type { CullFilter, PhotoId } from '@/ipc/bindings'

/** What one press of *Export* asked for, as the panel that asked knows it (FR-093). */
export type ExportRequest = {
  filter: CullFilter
  /** Photos (or sheets) the export writes, for the button's label. */
  count: number
  only: number | null
  ticked: readonly PhotoId[]
  /** Collage sheets, two photos to a print (FR-035). */
  collage: boolean
  /** Source folders in the export, for *together / apart* (FR-056). */
  folderCount: number
}

let request: ExportRequest | null = null
const listeners = new Set<() => void>()
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Opens the export dialog for `next`, or closes it (`null`). One dialog for every panel. */
export function showExportDialog(next: ExportRequest | null) {
  request = next
  for (const listener of listeners) listener()
}

/** The export dialog's open request, or null while it is closed. */
export function useExportDialog(): ExportRequest | null {
  return useSyncExternalStore(subscribe, () => request)
}
