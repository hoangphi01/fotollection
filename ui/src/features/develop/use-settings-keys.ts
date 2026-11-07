// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef } from 'react'
import { events, type PhotoId } from '@/ipc/bindings'
import { isTyping } from '@/lib/keyboard'
import { openCopySettings, pasteSettings, resetSettings, useSettingsCopy } from './settings-copy'

/**
 * The keys and menu items of FR-097, in one place so a key is never handled twice: ⇧⌘C copies
 * from the photo shown, ⇧⌘V pastes to it (or to the ticked photos when none is shown), ⌥⇧⌘V
 * pastes to the ticked photos, ⇧⌘R resets the photo shown (or the ticked ones). On Windows
 * WebView2 claims Ctrl+Shift+C and Ctrl+Shift+R; `preventDefault` takes them back. On macOS
 * the menu's accelerators come here as menu actions.
 */
export function useSettingsKeys(ticked: readonly PhotoId[], collage: boolean) {
  const { target } = useSettingsCopy()
  const now = useRef({ target, ticked, collage })
  useEffect(() => {
    now.current = { target, ticked, collage }
  })

  useEffect(() => {
    const act = (action: 'copy' | 'paste' | 'pasteTicked' | 'reset') => {
      const { target, ticked, collage } = now.current
      const onTarget = target ? [target.photo] : []
      const pick = (first: readonly PhotoId[], second: readonly PhotoId[]) => (first.length > 0 ? first : second)
      if (action === 'copy') {
        if (target) openCopySettings(target)
      } else if (action === 'paste') {
        pasteSettings(pick(onTarget, ticked), target?.collage ?? collage)
      } else if (action === 'pasteTicked') {
        pasteSettings(pick(ticked, onTarget), target?.collage ?? collage)
      } else {
        resetSettings(pick(onTarget, ticked), target?.collage ?? collage)
      }
    }
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || !e.shiftKey || isTyping(e.target)) return
      const key = e.code
      const action =
        key === 'KeyC' ? 'copy' : key === 'KeyV' ? (e.altKey ? 'pasteTicked' : 'paste') : key === 'KeyR' ? 'reset' : null
      if (!action) return
      e.preventDefault()
      e.stopPropagation()
      act(action)
    }
    const unlisten = events.menuAction.listen(({ payload }) => {
      const map = { copySettings: 'copy', pasteSettings: 'paste', pasteSettingsTicked: 'pasteTicked', resetSettings: 'reset' } as const
      const action = map[payload.action as keyof typeof map]
      if (action) act(action)
    })
    window.addEventListener('keydown', onKey, true)
    return () => {
      window.removeEventListener('keydown', onKey, true)
      void unlisten.then((fn) => fn())
    }
  }, [])
}
