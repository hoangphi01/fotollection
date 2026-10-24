// SPDX-License-Identifier: GPL-3.0-only
// Light and dark through shadcn's `.dark` class (STYLE_RULE §4). "system" follows the OS.
import type { ThemePreference } from '@/ipc/bindings'

const query = window.matchMedia('(prefers-color-scheme: dark)')
let preference: ThemePreference = 'system'

function apply() {
  const dark = preference === 'dark' || (preference === 'system' && query.matches)
  document.documentElement.classList.toggle('dark', dark)
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
