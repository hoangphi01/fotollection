// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react'

const MIN = 1
/** The closest zoom, in times the fit, unless 2:1 is closer still (FR-095). */
const MAX = 8
/** A press that moves less than this, in px, is a click: it zooms rather than drags. */
const CLICK = 4
/** One Ctrl/⌘ + or − press. */
const STEP = 1.25
/** A click's zoom when the photo's pixels are not known. */
const CLOSE_UP = 2
/** Zoom per pixel of a pinch, and of a two-finger scroll or wheel. */
const PINCH_RATE = 0.01
const SCROLL_RATE = 0.005
/** A mouse wheel's notch is ~100 px: at most this much of it counts, so one notch is ~1.35×. */
const MAX_SCROLL = 60

/** The photo's scale and its offset from the stage centre, in CSS pixels. */
export type View = { scale: number; x: number; y: number }
const FIT: View = { scale: 1, x: 0, y: 0 }

/**
 * Zoom and pan for the preview stage (FR-057): a two-finger scroll up or down (or the mouse
 * wheel), a trackpad pinch (sent by WebKit and WebView2 as a wheel event with Ctrl held) or
 * Ctrl/⌘ + − 0 zoom about the pointer; a drag, or a sideways two-finger scroll, moves the
 * zoomed photo; a click, or `Z`, goes between fit and 1:1, as in Lightroom (FR-095); the zoom
 * buttons step about the centre. `stage` is the element that takes the wheel events; another
 * `resetKey` (another photo, the crop editor) starts again at fit. `oneToOne` is the scale at
 * which one pixel of the photo meets one pixel of the screen, when known: the percentages are
 * then the photo's own (100 % is 1:1), not the fit's.
 */
export function useZoom(stage: HTMLElement | null, resetKey: string, oneToOne: number | null = null) {
  const [state, setState] = useState<{ key: string; view: View }>({ key: resetKey, view: FIT })
  const view = state.key === resetKey ? state.view : FIT
  const [drag, setDrag] = useState<{ x: number; y: number } | null>(null)
  const pressedAt = useRef<{ x: number; y: number } | null>(null)
  const zoomed = view.scale > MIN
  const max = Math.max(MAX, 2 * (oneToOne ?? 0))
  const closeUp = oneToOne !== null && oneToOne > MIN ? oneToOne : CLOSE_UP

  /**
   * The photo kept on the stage (owner, 2026-11-08: "a safe zone, never dragged out"): a photo
   * wider or taller than the stage never shows the ground inside its edge, and one smaller
   * stays centred on that axis. The photo's fit size is its laid-out size (the transform
   * scales it), the stage's its own box.
   */
  const clamp = useCallback(
    (v: View): View => {
      const photo = stage?.querySelector('img')
      if (!stage || !photo || v.scale <= MIN) return v
      const bound = (fit: number, room: number) => Math.max(0, (v.scale * fit - room) / 2)
      const bx = bound(photo.offsetWidth, stage.clientWidth)
      const by = bound(photo.offsetHeight, stage.clientHeight)
      return { ...v, x: Math.min(bx, Math.max(-bx, v.x)), y: Math.min(by, Math.max(-by, v.y)) }
    },
    [stage],
  )

  const setView = useCallback(
    (next: (v: View) => View) =>
      setState((s) => ({ key: resetKey, view: clamp(next(s.key === resetKey ? s.view : FIT)) })),
    [resetKey, clamp],
  )

  /** Zooms by `factor`, keeping the point (`px`, `py`) from the stage centre in place. */
  const zoomAt = useCallback(
    (factor: number, px = 0, py = 0) =>
      setView((v) => {
        const scale = Math.min(max, Math.max(MIN, v.scale * factor))
        if (scale === MIN) return FIT
        const k = scale / v.scale
        return { scale, x: px - (px - v.x) * k, y: py - (py - v.y) * k }
      }),
    [setView, max],
  )

  /** Fit, or 1:1 about the point (`px`, `py`) from the stage centre. */
  const toggleCloseUp = useCallback(
    (px = 0, py = 0) => {
      if (view.scale > MIN) setView(() => FIT)
      else zoomAt(closeUp, px, py)
    },
    [view.scale, setView, zoomAt, closeUp],
  )

  const fromCentre = useCallback(
    (clientX: number, clientY: number): [number, number] => {
      const r = stage?.getBoundingClientRect()
      return r ? [clientX - r.left - r.width / 2, clientY - r.top - r.height / 2] : [0, 0]
    },
    [stage],
  )

  // A native listener: React's wheel handler is passive and cannot stop the page zooming.
  useEffect(() => {
    if (!stage) return
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const at = fromCentre(e.clientX, e.clientY)
      if (e.ctrlKey || e.metaKey) return zoomAt(Math.exp(-e.deltaY * PINCH_RATE), ...at)
      // Fingers moving up zoom in, whichever way the system scrolls: WebKit says when macOS's
      // natural scrolling has turned the deltas around (the wheel's forward notch zooms in too).
      const natural = (e as WheelEvent & { webkitDirectionInvertedFromDevice?: boolean })
        .webkitDirectionInvertedFromDevice
      const up = natural ? e.deltaY : -e.deltaY
      if (Math.abs(e.deltaY) >= Math.abs(e.deltaX)) {
        const step = Math.max(-MAX_SCROLL, Math.min(MAX_SCROLL, up))
        zoomAt(Math.exp(step * SCROLL_RATE), ...at)
      } else if (zoomed) {
        // Sideways, it moves the zoomed photo.
        setView((v) => ({ ...v, x: v.x - e.deltaX }))
      }
    }
    stage.addEventListener('wheel', onWheel, { passive: false })
    return () => stage.removeEventListener('wheel', onWheel)
  }, [stage, zoomed, zoomAt, fromCentre, setView])

  /** Ctrl/⌘ + = − 0, and `Z` for fit ↔ 1:1; returns true when the key was a zoom key. */
  const onKey = useCallback(
    (event: KeyboardEvent) => {
      if (event.key.toLowerCase() === 'z' && !(event.ctrlKey || event.metaKey || event.altKey)) {
        event.preventDefault()
        toggleCloseUp()
        return true
      }
      if (!(event.ctrlKey || event.metaKey)) return false
      if (event.key === '+' || event.key === '=') zoomAt(STEP)
      else if (event.key === '-') zoomAt(1 / STEP)
      else if (event.key === '0') setView(() => FIT)
      else return false
      event.preventDefault()
      return true
    },
    [zoomAt, setView, toggleCloseUp],
  )

  const pointer = {
    onPointerDown: (e: PointerEvent) => {
      if (e.button !== 0) return
      // A press on the photo never starts a text selection: a drag that leaves the stage
      // would otherwise select the panels beside it (owner, 2026-11-08).
      e.preventDefault()
      pressedAt.current = { x: e.clientX, y: e.clientY }
      if (view.scale === MIN) return
      setDrag({ x: e.clientX, y: e.clientY })
      e.currentTarget.setPointerCapture(e.pointerId)
    },
    onPointerMove: (e: PointerEvent) => {
      if (!drag) return
      setDrag({ x: e.clientX, y: e.clientY })
      setView((v) => ({ ...v, x: v.x + e.clientX - drag.x, y: v.y + e.clientY - drag.y }))
    },
    onPointerUp: () => setDrag(null),
    /** A click that did not move: fit ↔ 1:1 at the pointer. */
    onClick: (e: { clientX: number; clientY: number }) => {
      const from = pressedAt.current
      pressedAt.current = null
      if (from && Math.hypot(e.clientX - from.x, e.clientY - from.y) > CLICK) return
      toggleCloseUp(...fromCentre(e.clientX, e.clientY))
    },
  }

  return {
    zoomed,
    view,
    /** The photo's own percentage (100 % is 1:1) when its pixels are known, else the fit's. */
    percent: Math.round((oneToOne ? view.scale / oneToOne : view.scale) * 100),
    canZoomIn: view.scale < max,
    toggleCloseUp: () => toggleCloseUp(),
    /** Zooms to `scale` times the fit, about the stage centre (the Navigator's choices). */
    zoomTo: (scale: number) => zoomAt(scale / view.scale),
    /** Moves the zoomed photo so its offset from the stage centre is (`x`, `y`) in px. */
    panTo: (x: number, y: number) => setView((v) => (v.scale > MIN ? { ...v, x, y } : v)),
    zoomIn: () => zoomAt(STEP),
    zoomOut: () => zoomAt(1 / STEP),
    transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
    reset: () => setView(() => FIT),
    onKey,
    pointer,
  }
}
