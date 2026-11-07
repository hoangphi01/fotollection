// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'

/**
 * Whether the welcome screen (FR-109) is open: at start when the settings say so, decided
 * once when the file's values arrive (never a flash for someone who has seen it), or when
 * Settings → General asks for it again.
 */
let open = false
let decided = false
const listeners = new Set<() => void>()
const emit = () => listeners.forEach((l) => l())
const subscribe = (l: () => void) => {
  listeners.add(l)
  return () => listeners.delete(l)
}

/** Opens it again from Settings → General. */
export function showWelcome() {
  open = true
  emit()
}

/** The start's decision, once: shown when never seen or asked for at every start. */
export function decideWelcome(seen: boolean, everyStart: boolean) {
  if (decided) return
  decided = true
  if (!seen || everyStart) {
    open = true
    queueMicrotask(emit)
  }
}

export function closeWelcome() {
  open = false
  emit()
}

export function useWelcomeOpen(): boolean {
  return useSyncExternalStore(subscribe, () => open)
}
