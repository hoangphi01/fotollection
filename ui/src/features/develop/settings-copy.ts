// SPDX-License-Identifier: GPL-3.0-only
import { useSyncExternalStore } from 'react'
import { commands, events, type PhotoId, type SettingsParts } from '@/ipc/bindings'

/**
 * Copy, paste and reset develop settings (FR-097), one store for every surface: the develop
 * view and the collage sheet's preview say which photo they show (`setSettingsTarget`), the
 * gallery and Collage pass their ticked photos, and one set of keys and menu items acts on
 * them (`useSettingsKeys`). What was copied lives in Rust; this mirrors which parts.
 */
export type SettingsTarget = { photo: PhotoId; collage: boolean }

type State = {
  copied: SettingsParts | null
  target: SettingsTarget | null
  /** The Copy dialog, open for this photo. */
  dialog: SettingsTarget | null
  /** The parts ticked last time, kept for the session (not saved: no settings change). */
  lastParts: SettingsParts
}

let state: State = {
  copied: null,
  target: null,
  dialog: null,
  lastParts: { look: true, rawLook: true, crop: false, stamp: false },
}
const listeners = new Set<() => void>()
let loaded = false
const set = (next: Partial<State>) => {
  state = { ...state, ...next }
  for (const l of listeners) l()
}
const refresh = () =>
  void commands.copiedSettings().then((r) => {
    if (r.status === 'ok') set({ copied: r.data })
  })
const subscribe = (listener: () => void) => {
  listeners.add(listener)
  if (!loaded) {
    loaded = true
    refresh()
    // Another project forgets the copy in Rust (FR-097); follow it.
    void events.libraryChanged.listen(refresh)
  }
  return () => listeners.delete(listener)
}

export function useSettingsCopy(): State {
  return useSyncExternalStore(subscribe, () => state)
}

/** The photo a surface shows now, or none when it closes (only the one that set it clears it). */
export function setSettingsTarget(next: SettingsTarget | null, owner?: SettingsTarget | null) {
  if (next === null && owner && state.target?.photo !== owner.photo) return
  if (next?.photo === state.target?.photo && next?.collage === state.target?.collage) return
  set({ target: next })
}

export function openCopySettings(target: SettingsTarget) {
  set({ dialog: target })
}

export function closeCopySettings() {
  set({ dialog: null })
}

export async function copySettings(target: SettingsTarget, parts: SettingsParts) {
  const result = await commands.copyPhotoSettings(target.photo, parts, target.collage)
  if (result.status === 'ok') set({ copied: result.data, lastParts: parts, dialog: null })
  return result
}

export function pasteSettings(ids: readonly PhotoId[], collage: boolean) {
  if (ids.length === 0 || !state.copied) return
  void commands.pastePhotoSettings([...ids], collage)
}

export function resetSettings(ids: readonly PhotoId[], collage: boolean) {
  if (ids.length === 0) return
  void commands.resetPhotoSettings([...ids], collage)
}

export const anyPart = (p: SettingsParts) => p.look || p.rawLook || p.crop || p.stamp
