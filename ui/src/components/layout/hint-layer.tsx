// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useLayoutEffect, useRef, useState } from 'react'

const DELAY_MS = 450
const GAP = 6
/** Space kept between a hint and the window's edges. */
const EDGE = 8

type Shown = { text: string; x: number; y: number; above: boolean }

/**
 * App-wide hover hints for any element with `data-hint`, styled like shadcn's tooltip.
 * Replaces the native `title` attribute, whose square system tooltip looks out of place on
 * Windows (STYLE_RULE §4). One listener serves the whole app, so long lists stay cheap.
 */
export function HintLayer() {
  const [shown, setShown] = useState<Shown | null>(null)
  const ref = useRef<HTMLDivElement>(null)
  // Centred under its element, but moved back inside the window near an edge.
  useLayoutEffect(() => {
    const el = ref.current
    if (!el || !shown) return
    const width = el.offsetWidth
    const left = Math.min(Math.max(EDGE, shown.x - width / 2), window.innerWidth - width - EDGE)
    el.style.left = `${Math.max(EDGE, left)}px`
  }, [shown])

  useEffect(() => {
    let timer: number | undefined
    let target: HTMLElement | null = null
    const hide = () => {
      window.clearTimeout(timer)
      target = null
      setShown(null)
    }
    const over = (e: Event) => {
      const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-hint]') : null
      if (el === target) return
      hide()
      const text = el?.dataset.hint
      if (!el || !text) return
      target = el
      timer = window.setTimeout(() => {
        const r = el.getBoundingClientRect()
        const above = r.bottom + 40 > window.innerHeight
        setShown({ text, x: r.left + r.width / 2, y: above ? r.top - GAP : r.bottom + GAP, above })
      }, DELAY_MS)
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
      className="pointer-events-none fixed z-[100] w-max max-w-xs animate-in rounded-md bg-foreground px-3 py-1.5 text-xs break-words text-background fade-in-0 zoom-in-95"
      style={{
        left: EDGE,
        top: shown.y,
        transform: shown.above ? 'translateY(-100%)' : undefined,
      }}
    >
      {shown.text}
    </div>
  )
}
