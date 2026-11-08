// SPDX-License-Identifier: GPL-3.0-only
import { useEffect } from 'react'
import { setMotionPreference } from '@/lib/motion'
import { useUiSettings } from './use-ui-settings'

/** Settings → Motion (FR-100), stored per machine in `settings.json`, applied in `App` so every
 * view and dialog follows it (LESSONS: per-machine preferences apply in `App`). */
export function useMotion() {
  const { ui } = useUiSettings()
  useEffect(() => setMotionPreference(ui.motion), [ui.motion])
}
