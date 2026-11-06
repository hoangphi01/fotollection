// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useSyncExternalStore } from 'react'
import { commands, type UiSettings } from '@/ipc/bindings'

// One shared copy of the per-machine interface preferences (FR-020, FR-052, FR-009), so the
// language picker and the theme switch never overwrite each other's choice.
let current: Required<UiSettings> = {
  language: 'en',
  theme: 'system',
  mode: 'professional',
  exportFolders: 'oneFolder',
  // The memory and cache settings (FR-086), as Rust's defaults.
  previewMemoryPercent: 25,
  cacheLimitGb: 5,
  preparePreviews: true,
  // The filmstrip under the photo in Pro mode's preview (FR-090).
  filmstrip: true,
  // The develop view's layout (FR-095): the dialog unless the user chose the whole window.
  previewLayout: 'dialog',
  // The export dialog's choices (FR-092, FR-093), as Rust's defaults.
  exportQuality: 90,
  exportAsk: true,
  exportSeparate: false,
  exportDestination: null,
  // Lightroom's file rules (FR-009, FR-010, FR-104, FR-105), as Rust's defaults.
  exportSubfolder: null,
  exportNaming: { template: 'original', text: '', start: 1 },
  exportExisting: 'addNumber',
  exportMetadata: 'camera',
  exportLimitKb: null,
  // The update check at start (FR-103): on until the user turns it off.
  checkUpdatesAtStart: true,
}
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
    // Rust keeps every value within what the app allows: its answer is the truth.
    const result = await commands.setUiSettings(current)
    if (result.status === 'ok') {
      current = { ...current, ...result.data }
      emit()
    }
  }, [])
  return { ui, update }
}
