// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { isTyping } from '@/lib/keyboard'

/**
 * Quick-preview (FR-059), like macOS Quick Look: in select mode, holding Space shows the
 * photo under the pointer (or `fallback`) and releasing it closes the view. While it is
 * held, ← → move to the next photo and Enter ticks the one shown (`onToggle`).
 */
export function useQuickPreview(
  enabled: boolean,
  count: number,
  hovered: number | null,
  fallback: number,
  onToggle: (index: number) => void,
) {
  const [index, setIndex] = useState<number | null>(null)
  const shown = enabled && index !== null && index < count ? index : null

  useEffect(() => {
    if (!enabled) return
    const down = (e: KeyboardEvent) => {
      if (isTyping(e.target) || document.querySelector('[role="dialog"]')) return
      if (e.key === ' ') {
        e.preventDefault()
        if (!e.repeat) setIndex((open) => open ?? hovered ?? fallback)
      } else if (shown !== null && (e.key === 'ArrowRight' || e.key === 'ArrowLeft')) {
        e.preventDefault()
        const step = e.key === 'ArrowRight' ? 1 : -1
        setIndex((i) => (i === null ? i : Math.min(count - 1, Math.max(0, i + step))))
      } else if (shown !== null && e.key === 'Enter') {
        e.preventDefault()
        onToggle(shown)
      } else if (shown !== null && e.key === 'Escape') {
        setIndex(null)
      }
    }
    const up = (e: KeyboardEvent) => {
      if (e.key === ' ') setIndex(null)
    }
    const close = () => setIndex(null)
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    window.addEventListener('blur', close)
    return () => {
      window.removeEventListener('keydown', down)
      window.removeEventListener('keyup', up)
      window.removeEventListener('blur', close)
    }
  }, [enabled, count, hovered, fallback, shown, onToggle])

  return { index: shown, close: () => setIndex(null) }
}
