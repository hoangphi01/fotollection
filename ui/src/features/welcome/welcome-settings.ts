// SPDX-License-Identifier: GPL-3.0-only
import { Clock } from 'lucide-react'
import type { UiSettings } from '@/ipc/bindings'

/**
 * The settings the welcome asks for (FR-146, owner 2026-11-20): after an update, a user who saw
 * the welcome meets only those still unchosen, never the whole welcome again. Each has a "not
 * chosen yet" state, its value `null` in the settings file; a new welcome setting joins this
 * list with one (DOCS_GUIDE §2).
 */
export const WELCOME_SETTINGS = [
  // The home time (FR-145): null until the welcome's Start or Done keeps a zone.
  { key: 'homeZone', icon: Clock, needed: (ui: UiSettings) => ui.homeZone == null },
] as const

export type WelcomeSettingKey = (typeof WELCOME_SETTINGS)[number]['key']

/** The welcome settings this user has not chosen yet, in the list's order. */
export function pendingSettings(ui: UiSettings): WelcomeSettingKey[] {
  return WELCOME_SETTINGS.filter((s) => s.needed(ui)).map((s) => s.key)
}
