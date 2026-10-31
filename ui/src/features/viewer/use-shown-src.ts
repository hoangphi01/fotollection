// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'

/**
 * The last finished render stays on screen until the next one has loaded, so a new setting
 * or a drag never blanks the photo; another photo (another `key`) starts empty, so the
 * caller can show a spinner.
 */
export function useShownSrc(key: string | number | undefined, src: string): string | null {
  const [shown, setShown] = useState<{ key: string | number; src: string } | null>(null)
  useEffect(() => {
    if (key === undefined || !src) return
    let live = true
    const img = new Image()
    img.onload = () => live && setShown({ key, src })
    img.src = src
    return () => {
      live = false
    }
  }, [key, src])
  return shown && shown.key === key ? shown.src : null
}
