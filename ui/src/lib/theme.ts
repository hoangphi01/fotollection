// SPDX-License-Identifier: GPL-3.0-only
// Follows the OS light/dark setting using shadcn's `.dark` class (STYLE_RULE §4).

export function followSystemTheme(): void {
  const query = window.matchMedia('(prefers-color-scheme: dark)')
  const apply = () => document.documentElement.classList.toggle('dark', query.matches)
  apply()
  query.addEventListener('change', apply)
}
