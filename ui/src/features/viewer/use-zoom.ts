// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState, type KeyboardEvent, type PointerEvent } from 'react'

const MIN = 1
const MAX = 8
/** One Ctrl/⌘ + or − press. */
const STEP = 1.25
/** Double-click zoom. */
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
 * zoomed photo; a double-click toggles fit and 2×; the zoom buttons step about the centre.
 * `stage` is the element that takes the wheel events; another `resetKey` (another photo, the
 * crop editor) starts again at fit.
 */
export function useZoom(stage: HTMLElement | null, resetKey: string) {
  const [state, setState] = useState<{ key: string; view: View }>({ key: resetKey, view: FIT })
  const view = state.key === resetKey ? state.view : FIT
  const [drag, setDrag] = useState<{ x: number; y: number } | null>(null)
  const zoomed = view.scale > MIN

  const setView = useCallback(
    (next: (v: View) => View) =>
      setState((s) => ({ key: resetKey, view: next(s.key === resetKey ? s.view : FIT) })),
    [resetKey],
  )

  /** Zooms by `factor`, keeping the point (`px`, `py`) from the stage centre in place. */
  const zoomAt = useCallback(
    (factor: number, px = 0, py = 0) =>
      setView((v) => {
        const scale = Math.min(MAX, Math.max(MIN, v.scale * factor))
        if (scale === MIN) return FIT
        const k = scale / v.scale
        return { scale, x: px - (px - v.x) * k, y: py - (py - v.y) * k }
      }),
    [setView],
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

  /** Ctrl/⌘ + = − 0; returns true when the key was a zoom key. */
  const onKey = useCallback(
    (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return false
      if (event.key === '+' || event.key === '=') zoomAt(STEP)
      else if (event.key === '-') zoomAt(1 / STEP)
      else if (event.key === '0') setView(() => FIT)
      else return false
      event.preventDefault()
      return true
    },
    [zoomAt, setView],
  )

  const pointer = {
    onPointerDown: (e: PointerEvent) => {
      if (view.scale === MIN || e.button !== 0) return
      setDrag({ x: e.clientX, y: e.clientY })
      e.currentTarget.setPointerCapture(e.pointerId)
    },
    onPointerMove: (e: PointerEvent) => {
      if (!drag) return
      setDrag({ x: e.clientX, y: e.clientY })
      setView((v) => ({ ...v, x: v.x + e.clientX - drag.x, y: v.y + e.clientY - drag.y }))
    },
    onPointerUp: () => setDrag(null),
    onDoubleClick: (e: { clientX: number; clientY: number }) => {
      if (view.scale > MIN) setView(() => FIT)
      else zoomAt(CLOSE_UP, ...fromCentre(e.clientX, e.clientY))
    },
  }

  return {
    zoomed,
    view,
    percent: Math.round(view.scale * 100),
    canZoomIn: view.scale < MAX,
    zoomIn: () => zoomAt(STEP),
    zoomOut: () => zoomAt(1 / STEP),
    transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
    reset: () => setView(() => FIT),
    onKey,
    pointer,
  }
}
