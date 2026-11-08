// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { motionMs } from '@/lib/motion'
import { cn } from '@/lib/utils'

const GAP = 6
/** Space kept between a hint and the window's edges. */
const EDGE = 8

/** The hint's text and its element's box. */
type Shown = { text: string; x: number; top: number; bottom: number }
/** A hint fading out after the pointer left (FR-100); a press, a scroll or a blur hide it at once. */
type Leaving = Shown & { leaving: true }

/**
 * App-wide hover hints for any element with `data-hint`, styled like shadcn's tooltip.
 * Replaces the native `title` attribute, whose square system tooltip looks out of place on
 * Windows (STYLE_RULE §4). One listener serves the whole app, so long lists stay cheap.
 */
export function HintLayer() {
  const [shown, setShown] = useState<Shown | Leaving | null>(null)
  const ref = useRef<HTMLDivElement>(null)
  // Centred under its element, or above it when the hint as measured (one line or three)
  // would pass the window's bottom; moved back inside the window near a side.
  useLayoutEffect(() => {
    const el = ref.current
    if (!el || !shown) return
    const width = el.offsetWidth
    const height = el.offsetHeight
    const left = Math.min(Math.max(EDGE, shown.x - width / 2), window.innerWidth - width - EDGE)
    const below = shown.bottom + GAP
    const top = below + height + EDGE <= window.innerHeight ? below : Math.max(EDGE, shown.top - GAP - height)
    el.style.left = `${Math.max(EDGE, left)}px`
    el.style.top = `${top}px`
    el.style.visibility = 'visible'
  }, [shown])

  useEffect(() => {
    let timer: number | undefined
    let target: HTMLElement | null = null
    let gone: number | undefined
    const hide = () => {
      window.clearTimeout(timer)
      window.clearTimeout(gone)
      target = null
      setShown(null)
    }
    // The pointer left: the hint fades out (the exit pace) instead of vanishing.
    const leave = () => {
      window.clearTimeout(timer)
      target = null
      setShown((s) => (s && !('leaving' in s) ? { ...s, leaving: true } : s))
      gone = window.setTimeout(() => setShown((s) => (s && 'leaving' in s ? null : s)), motionMs('motion-base-out'))
    }
    const over = (e: Event) => {
      const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-hint]') : null
      if (el === target) return
      leave()
      const text = el?.dataset.hint
      if (!el || !text) return
      target = el
      // The delay before a hint, the `--delay-hint` token (FR-100).
      timer = window.setTimeout(() => {
        window.clearTimeout(gone)
        const r = el.getBoundingClientRect()
        setShown({ text, x: r.left + r.width / 2, top: r.top, bottom: r.bottom })
      }, motionMs('delay-hint'))
    }
    document.addEventListener('pointerover', over)
    document.addEventListener('focusin', over)
    document.addEventListener('pointerdown', hide)
    document.addEventListener('scroll', hide, true)
    window.addEventListener('blur', hide)
    return () => {
      hide()
      document.removeEventListener('pointerover', over)
      document.removeEventListener('focusin', over)
      document.removeEventListener('pointerdown', hide)
      document.removeEventListener('scroll', hide, true)
      window.removeEventListener('blur', hide)
    }
  }, [])

  if (!shown) return null
  return (
    <div
      ref={ref}
      role="tooltip"
      className={cn(
        'pointer-events-none fixed z-[100] w-max max-w-xs rounded-md bg-foreground px-3 py-1.5 text-xs break-words whitespace-pre-line text-background',
        'leaving' in shown ? 'animate-out fill-mode-forwards fade-out-0 zoom-out-95 motion-out' : 'animate-in fade-in-0 zoom-in-95 motion-in',
      )}
      // Placed once measured, so it never shows a frame in the wrong place.
      style={{ left: EDGE, top: 0, visibility: 'hidden' }}
    >
      {shown.text}
    </div>
  )
}
