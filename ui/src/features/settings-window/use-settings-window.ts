// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useSyncExternalStore } from 'react'
import { events } from '@/ipc/bindings'
import { isMac } from '@/lib/platform'

/** The sections of the Settings window, in the order of its list (FR-086). */
export const SECTIONS = ['general', 'export', 'memory', 'updates', 'report', 'whatsNew'] as const
export type Section = (typeof SECTIONS)[number]

// Which section is open, or none: one copy for the app, so the gear, the version badge and
// Cmd+, open the same window from any mode.
let open: Section | null = null
const listeners = new Set<() => void>()
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Opens the Settings window at `section`, or closes it with `null`. */
export function showSettings(section: Section | null) {
  open = section
  listeners.forEach((listener) => listener())
}

/** The section the Settings window shows, or `null` while it is closed. */
export function useSettingsWindow(): Section | null {
  return useSyncExternalStore(subscribe, () => open)
}

/**
 * Cmd+, opens the Settings window: the app menu's *Settings…* on macOS, the key itself on
 * Windows and Linux, which have no menu bar (FR-086).
 */
export function useSettingsShortcut() {
  useEffect(() => {
    const unlisten = events.menuAction.listen(({ payload }) => {
      if (payload.action === 'settings') showSettings('general')
    })
    return () => void unlisten.then((fn) => fn())
  }, [])
  useEffect(() => {
    if (isMac) return
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.altKey || e.shiftKey || e.key !== ',') return
      e.preventDefault()
      showSettings('general')
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])
}
