// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useSyncExternalStore } from 'react'
import { events } from '@/ipc/bindings'
import { showSettings } from '@/features/settings-window/use-settings-window'

// The chapter the manual (FR-110) shows in Settings → Manual: one copy for the app, so the
// Help menu and the section open the same chapter; `null` until one is asked for (the first).
let open: string | null = null
const listeners = new Set<() => void>()
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Shows chapter `id` (the first with `''`) in Settings → Manual, opening the window there. */
export function showManual(id: string) {
  open = id
  listeners.forEach((listener) => listener())
  showSettings('manual')
}

/** The chapter asked for, `''` or `null` for the first. */
export function useManualChapter(): string | null {
  return useSyncExternalStore(subscribe, () => open)
}

/** The Help menu's *Fotollection Manual* (macOS) opens Settings → Manual at its first chapter. */
export function useManualMenu() {
  useEffect(() => {
    const unlisten = events.menuAction.listen(({ payload }) => {
      if (payload.action === 'manual') showManual('')
    })
    return () => void unlisten.then((fn) => fn())
  }, [])
}
