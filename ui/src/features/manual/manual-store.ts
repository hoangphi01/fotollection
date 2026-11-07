// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useSyncExternalStore } from 'react'
import { events } from '@/ipc/bindings'

// The chapter the manual (FR-110) is open at, or none: one copy for the app, so Settings →
// Manual and the Help menu open the same window.
let open: string | null = null
const listeners = new Set<() => void>()
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Opens the manual at chapter `id` (the first with `''`), or closes it with `null`. */
export function showManual(id: string | null) {
  open = id
  listeners.forEach((listener) => listener())
}

/** The chapter the manual shows, `''` for the first, or `null` while it is closed. */
export function useManualChapter(): string | null {
  return useSyncExternalStore(subscribe, () => open)
}

/** The Help menu's *Fotollection Manual* (macOS) opens the manual at its first chapter. */
export function useManualMenu() {
  useEffect(() => {
    const unlisten = events.menuAction.listen(({ payload }) => {
      if (payload.action === 'manual') showManual('')
    })
    return () => void unlisten.then((fn) => fn())
  }, [])
}
