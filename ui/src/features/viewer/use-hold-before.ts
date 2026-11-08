// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState, type PointerEvent } from 'react'

/** How long a still press lasts before it shows the original: past a click (which zooms). */
const HOLD_MS = 250
/** A press that moves this far is a drag (a pan, the crop), never a hold (LESSONS: slop). */
const SLOP = 3

/**
 * Press and hold the photo to see it without its colour, as in the iPhone's editor (owner,
 * 2026-11-08): a still press of 250 ms shows the original, releasing shows the edit again,
 * and that release is no click (a click zooms). A press that moves is a drag, never a hold.
 * The original starts loading at the press (`warm`), so it is there when the hold begins.
 * Off while the white-balance picker, the crop editor or a compare view has the photo.
 */
export function useHoldBefore(enabled: boolean, warm: () => void) {
  const [held, setHeld] = useState(false)
  const heldRef = useRef(false)
  const press = useRef<{ x: number; y: number; timer: number } | null>(null)
  const swallow = useRef(false)

  const end = useCallback(() => {
    if (press.current) window.clearTimeout(press.current.timer)
    press.current = null
    if (heldRef.current) {
      heldRef.current = false
      swallow.current = true
      setHeld(false)
    }
  }, [])

  // A release anywhere ends it, even outside the photo or after a pointer capture elsewhere.
  useEffect(() => {
    window.addEventListener('pointerup', end)
    window.addEventListener('pointercancel', end)
    window.addEventListener('blur', end)
    return () => {
      window.removeEventListener('pointerup', end)
      window.removeEventListener('pointercancel', end)
      window.removeEventListener('blur', end)
    }
  }, [end])
  useEffect(() => {
    if (!enabled) end()
  }, [enabled, end])

  return {
    held,
    onPointerDown: (e: PointerEvent) => {
      swallow.current = false
      if (!enabled || e.button !== 0) return
      warm()
      const timer = window.setTimeout(() => {
        heldRef.current = true
        setHeld(true)
      }, HOLD_MS)
      press.current = { x: e.clientX, y: e.clientY, timer }
    },
    onPointerMove: (e: PointerEvent) => {
      const p = press.current
      if (p && !heldRef.current && Math.hypot(e.clientX - p.x, e.clientY - p.y) > SLOP) {
        window.clearTimeout(p.timer)
        press.current = null
      }
    },
    /** True once for the click that ends a hold, which must not zoom. */
    swallowClick: () => {
      const s = swallow.current
      swallow.current = false
      return s
    },
  }
}
