// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, events, type RevisionChanged } from '@/ipc/bindings'

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))
  )
}

/**
 * Render revision + undo/redo state from Rust (FR-023), and the global shortcuts
 * Cmd/Ctrl+Z (undo), Shift+Cmd/Ctrl+Z or Ctrl+Y (redo).
 */
export function useHistory(): RevisionChanged {
  const [state, setState] = useState<RevisionChanged>({ revision: 0, canUndo: false, canRedo: false })

  useEffect(() => {
    const unlisten = events.revisionChanged.listen(({ payload }) => setState(payload))
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || isTyping(e.target)) return
      const key = e.key.toLowerCase()
      if (key === 'z' && !e.shiftKey) {
        e.preventDefault()
        void commands.undo()
      } else if ((key === 'z' && e.shiftKey) || key === 'y') {
        e.preventDefault()
        void commands.redo()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => {
      window.removeEventListener('keydown', onKey)
      void unlisten.then((fn) => fn())
    }
  }, [])

  return state
}
