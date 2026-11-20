// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'
import type { WelcomeSettingKey } from './welcome-settings'

/**
 * Whether the welcome screen (FR-109) is open, and how: the whole welcome at start when the
 * settings say so, or after an update only the settings the old version lacked (FR-146), decided
 * once when the file's values arrive (never a flash for someone who has seen it); the whole
 * welcome again when Settings → General asks for it.
 */
export type WelcomeMode = { kind: 'full' } | { kind: 'settings'; keys: WelcomeSettingKey[] } | null

let mode: WelcomeMode = null
let decided = false
const listeners = new Set<() => void>()
const emit = () => listeners.forEach((l) => l())
const subscribe = (l: () => void) => {
  listeners.add(l)
  return () => listeners.delete(l)
}

/** Opens the whole welcome again from Settings → General. */
export function showWelcome() {
  mode = { kind: 'full' }
  emit()
}

/** The start's decision, once: the whole welcome when never seen or asked for at every start;
 * else, the settings still unchosen alone, their list fixed for the dialog's life. */
export function decideWelcome(seen: boolean, everyStart: boolean, pending: WelcomeSettingKey[]) {
  if (decided) return
  decided = true
  if (!seen || everyStart) mode = { kind: 'full' }
  else if (pending.length > 0) mode = { kind: 'settings', keys: pending }
  else return
  queueMicrotask(emit)
}

export function closeWelcome() {
  mode = null
  emit()
}

export function useWelcomeMode(): WelcomeMode {
  return useSyncExternalStore(subscribe, () => mode)
}
