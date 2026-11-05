// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState } from 'react'
import { errorText } from '@/lib/errors'

type Shown = { key: string | number; src: string } | { key: string | number; error: string }

/**
 * The last finished render stays on screen until the next one has loaded, so a new setting
 * or a drag never blanks the photo; another photo (another `key`) starts empty, so the
 * caller can show a spinner. A render that fails gives `error`, the reason with its `ERR-`
 * code from Rust's answer, so the caller shows it instead of a spinner that never stops.
 *
 * One render of a photo loads at a time: while it does, newer sources of the same photo only
 * replace the one waiting, and the newest starts when it lands. A slider's drag changes the
 * source every frame; starting each would let none finish before the next, and the photo
 * would stand still until the drag ends (LESSONS, FR-079). Another photo never waits: the
 * old load is let go and the new one starts at once, so a RAW still developing cannot hold
 * the next photo back. The loading image is kept in a ref: an `Image` nobody holds can be
 * collected before its `load` fires, and the spinner would never stop (LESSONS 2026-11-05).
 */
export function useShownSrc(key: string | number | undefined, src: string): { src: string | null; error: string | null } {
  const [shown, setShown] = useState<Shown | null>(null)
  const loading = useRef<{ key: string | number; src: string; img: HTMLImageElement } | null>(null)
  const wanted = useRef<{ key: string | number; src: string } | null>(null)
  const alive = useRef(true)

  useEffect(() => {
    alive.current = true
    return () => {
      alive.current = false
      loading.current = null
    }
  }, [])

  useEffect(() => {
    if (key === undefined || !src) {
      // Closed: nothing is wanted, and a load still running is let go.
      wanted.current = null
      loading.current = null
      return
    }
    wanted.current = { key, src }
    if (loading.current !== null && loading.current.key === key) return
    const start = (next: { key: string | number; src: string }) => {
      const img = new Image()
      const mine = { ...next, img }
      loading.current = mine
      const done = (result: Shown) => {
        // A load let go for another photo answers nobody.
        if (loading.current !== mine) return
        loading.current = null
        // Shown only while it is still this photo; a newer source loads next.
        if (alive.current && wanted.current?.key === next.key) setShown(result)
        const after = wanted.current
        if (alive.current && after && after.src !== next.src) start(after)
      }
      img.onload = () => done({ key: next.key, src: next.src })
      img.onerror = () => void failure(next.src).then((error) => done({ key: next.key, error }))
      img.src = next.src
    }
    start({ key, src })
  }, [key, src])

  if (!shown || shown.key !== key) return { src: null, error: null }
  return 'src' in shown ? { src: shown.src, error: null } : { src: null, error: shown.error }
}

/** Why `src` failed: Rust answers a failed render with its `IpcError` as JSON. */
async function failure(src: string): Promise<string> {
  const fallback = errorText({ code: 'render.failed', message: 'The preview could not be rendered', err: 'ERR-RUS-0004' })
  try {
    const body: unknown = await (await fetch(src)).json()
    if (body && typeof body === 'object' && 'code' in body && 'message' in body) {
      const { code, message, err } = body as { code: unknown; message: unknown; err?: unknown }
      if (typeof code === 'string' && typeof message === 'string') {
        return errorText({ code, message, err: typeof err === 'string' ? err : undefined })
      }
    }
  } catch {
    // The answer was not JSON: the generic message.
  }
  return fallback
}
