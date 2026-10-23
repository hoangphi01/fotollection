// SPDX-License-Identifier: GPL-3.0-only
import { commands, type PhotoId } from '@/ipc/bindings'
import { isTyping } from '@/lib/keyboard'

/** Lightroom-style culling keys (FR-022): 0–5 stars, P pick, X reject, U unflag. */
export const CULL_KEYS_HINT = '0–5 stars · P pick · X reject · U unflag'

/**
 * Applies the culling shortcut in `event` to `id`, if it is one.
 * Returns true when the key was handled, so callers can stop other handlers.
 */
export function handleCullKey(
  event: { key: string; target: EventTarget | null; metaKey: boolean; ctrlKey: boolean; altKey: boolean },
  id: PhotoId | undefined,
): boolean {
  if (id === undefined || event.metaKey || event.ctrlKey || event.altKey || isTyping(event.target)) {
    return false
  }
  const key = event.key.toLowerCase()
  if (/^[0-5]$/.test(key)) {
    void commands.setPhotoRating([id], Number(key))
  } else if (key === 'p') {
    void commands.setPhotoFlag([id], 'pick')
  } else if (key === 'x') {
    void commands.setPhotoFlag([id], 'reject')
  } else if (key === 'u') {
    void commands.setPhotoFlag([id], null)
  } else {
    return false
  }
  return true
}
