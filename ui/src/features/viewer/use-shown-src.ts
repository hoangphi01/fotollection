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
 * One render loads at a time: while it does, newer sources only replace the one waiting,
 * and the newest starts when it lands. A slider's drag changes the source every frame;
 * starting each would let none finish before the next, and the photo would stand still
 * until the drag ends (LESSONS, FR-079).
 */
export function useShownSrc(key: string | number | undefined, src: string): { src: string | null; error: string | null } {
  const [shown, setShown] = useState<Shown | null>(null)
  const loading = useRef<string | null>(null)
  const wanted = useRef<{ key: string | number; src: string } | null>(null)
  const alive = useRef(true)

  useEffect(() => {
    alive.current = true
    return () => {
      alive.current = false
    }
  }, [])

  useEffect(() => {
    if (key === undefined || !src) return
    wanted.current = { key, src }
    if (loading.current !== null) return
    const start = (next: { key: string | number; src: string }) => {
      loading.current = next.src
      const img = new Image()
      const done = (result: Shown) => {
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
