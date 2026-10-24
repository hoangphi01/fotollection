// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useSyncExternalStore } from 'react'
import { commands, type UiSettings } from '@/ipc/bindings'

// One shared copy of the per-machine interface preferences (FR-020, FR-052), so the
// language picker and the theme switch never overwrite each other's choice.
let current: Required<UiSettings> = { language: 'en', theme: 'system' }
let requested = false
const listeners = new Set<() => void>()
const emit = () => listeners.forEach((listener) => listener())
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

function load() {
  if (requested) return
  requested = true
  void commands.getUiSettings().then((result) => {
    if (result.status === 'ok') {
      current = { ...current, ...result.data }
      emit()
    }
  })
}

/** The interface preferences stored in `settings.json`, and a way to change some of them. */
export function useUiSettings() {
  const ui = useSyncExternalStore(subscribe, () => current)
  useEffect(load, [])
  const update = useCallback(async (change: Partial<UiSettings>) => {
    current = { ...current, ...change }
    emit()
    await commands.setUiSettings(current)
  }, [])
  return { ui, update }
}
