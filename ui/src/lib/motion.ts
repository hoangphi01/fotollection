// SPDX-License-Identifier: GPL-3.0-only
// The motion system's script side (FR-100, docs/TRANS_ANIMATION.md): whether motion is reduced
// (the system's *Reduce motion*, or Settings → Motion), and the paces as numbers for the few
// places a script times motion. The values live once, in `index.css`; nothing here repeats them.
import { useEffect, useState, useSyncExternalStore } from 'react'
import type { MotionPreference } from '@/ipc/bindings'

const query = window.matchMedia('(prefers-reduced-motion: reduce)')
let preference: MotionPreference = 'system'
const listeners = new Set<() => void>()

function apply() {
  const reduce = preference === 'reduce' || query.matches
  if (reduce) document.documentElement.dataset.motion = 'reduce'
  else delete document.documentElement.dataset.motion
  listeners.forEach((listener) => listener())
}

query.addEventListener('change', apply)
apply()

/** Settings → Motion (FR-100): `system` follows the OS, `reduce` keeps motion to short fades. */
export function setMotionPreference(next: MotionPreference): void {
  preference = next
  apply()
}

/** Whether motion is reduced now, for scripts that slide, zoom or scroll. */
export function reducedMotion(): boolean {
  return document.documentElement.dataset.motion === 'reduce'
}

/** The same, following every change. */
export function useReducedMotion(): boolean {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    reducedMotion,
  )
}

export type MotionToken =
  | 'motion-fast'
  | 'motion-base'
  | 'motion-slow'
  | 'motion-base-out'
  | 'motion-slow-out'
  | 'delay-hint'
  | 'delay-stale'

/** A pace or a delay in milliseconds, read from `index.css` (shorter while reduced). */
export function motionMs(token: MotionToken): number {
  const value = getComputedStyle(document.documentElement).getPropertyValue(`--${token}`).trim()
  const ms = value.endsWith('ms') ? Number.parseFloat(value) : Number.parseFloat(value) * 1000
  return Number.isFinite(ms) ? ms : 0
}

/** A curve for the Web Animations API (`element.animate`), the same as the CSS one. */
export function motionEase(curve: 'enter' | 'exit' | 'move'): string {
  return getComputedStyle(document.documentElement).getPropertyValue(`--ease-${curve}`).trim() || 'ease'
}

/**
 * Something that comes and goes (an overlay, a hint): while `value` is set it shows; once it is
 * `null`, the last value stays drawn with `leaving` for its exit (`token`, the exit pace), then
 * goes. Nothing waits for it: a new value shows at once, ending the exit.
 */
export function usePresence<T>(value: T | null, token: MotionToken = 'motion-base-out'): { shown: T | null; leaving: boolean } {
  const [shown, setShown] = useState<T | null>(value)
  const [leaving, setLeaving] = useState(false)
  if (value !== null && (value !== shown || leaving)) {
    setShown(value)
    setLeaving(false)
  } else if (value === null && shown !== null && !leaving) {
    setLeaving(true)
  }
  useEffect(() => {
    if (!leaving) return
    const timer = window.setTimeout(() => {
      setShown(null)
      setLeaving(false)
    }, motionMs(token))
    return () => window.clearTimeout(timer)
  }, [leaving, token])
  return { shown, leaving }
}
