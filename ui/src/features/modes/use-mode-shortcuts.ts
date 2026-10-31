// SPDX-License-Identifier: GPL-3.0-only
import { useEffect } from 'react'
import { events, type Action, type AppMode } from '@/ipc/bindings'
import { isMac } from '@/lib/platform'
import { isTyping } from '@/lib/keyboard'

const FROM_MENU: Partial<Record<Action, AppMode>> = {
  modeSimple: 'simple',
  modeCollage: 'collage',
  modeProfessional: 'professional',
}
const FROM_KEY: Record<string, AppMode> = { '1': 'simple', '2': 'collage', '3': 'professional' }

/** Cmd/Ctrl+1, 2, 3 switch to Simple, Collage and Pro (FR-048): the View menu on macOS. */
export function useModeShortcuts(setMode: (mode: AppMode) => void) {
  useEffect(() => {
    const unlisten = events.menuAction.listen(({ payload }) => {
      const mode = FROM_MENU[payload.action]
      if (mode) setMode(mode)
    })
    return () => void unlisten.then((fn) => fn())
  }, [setMode])

  useEffect(() => {
    if (isMac) return
    const onKey = (e: KeyboardEvent) => {
      const mode = FROM_KEY[e.key]
      if (!e.ctrlKey || e.altKey || e.shiftKey || !mode || isTyping(e.target)) return
      e.preventDefault()
      setMode(mode)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [setMode])
}
