// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { errorText } from '@/lib/errors'

type Shown = { key: string | number; src: string } | { key: string | number; error: string }

/**
 * The last finished render stays on screen until the next one has loaded, so a new setting
 * or a drag never blanks the photo; another photo (another `key`) starts empty, so the
 * caller can show a spinner. A render that fails gives `error`, the reason with its `ERR-`
 * code from Rust's answer, so the caller shows it instead of a spinner that never stops.
 */
export function useShownSrc(key: string | number | undefined, src: string): { src: string | null; error: string | null } {
  const [shown, setShown] = useState<Shown | null>(null)
  useEffect(() => {
    if (key === undefined || !src) return
    let live = true
    const img = new Image()
    img.onload = () => live && setShown({ key, src })
    img.onerror = () => void failure(src).then((error) => live && setShown({ key, error }))
    img.src = src
    return () => {
      live = false
    }
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
