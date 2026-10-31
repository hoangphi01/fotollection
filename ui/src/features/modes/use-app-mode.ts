// SPDX-License-Identifier: GPL-3.0-only
import type { AppMode } from '@/ipc/bindings'
import { useUiSettings } from '@/features/settings/use-ui-settings'

/** Simple, Collage or Professional, remembered on this machine (FR-048). */
export function useAppMode() {
  const { ui, update } = useUiSettings()
  return { mode: ui.mode, setMode: (mode: AppMode) => void update({ mode }) }
}
