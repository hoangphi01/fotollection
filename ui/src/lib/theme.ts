// SPDX-License-Identifier: GPL-3.0-only
// Light and dark through shadcn's `.dark` class (STYLE_RULE §4). "system" follows the OS.
import { useSyncExternalStore } from 'react'
import type { ThemePreference } from '@/ipc/bindings'

const query = window.matchMedia('(prefers-color-scheme: dark)')
let preference: ThemePreference = 'system'

const listeners = new Set<() => void>()

function apply() {
  const dark = preference === 'dark' || (preference === 'system' && query.matches)
  document.documentElement.classList.toggle('dark', dark)
  listeners.forEach((listener) => listener())
}

query.addEventListener('change', apply)

/** Used once at start-up, before the stored preference is known. */
export function followSystemTheme(): void {
  apply()
}

/** Shows the chosen theme now, and keeps following the OS when it is "system" (FR-052). */
export function setThemePreference(next: ThemePreference): void {
  preference = next
  apply()
}

/** Whether the dark theme is shown now, following every change: pictures made for one theme
 * (the welcome's, the manual's) switch with it. */
export function useDarkTheme(): boolean {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    () => document.documentElement.classList.contains('dark'),
  )
}
