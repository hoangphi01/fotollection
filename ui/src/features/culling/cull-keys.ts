// SPDX-License-Identifier: GPL-3.0-only
import { commands, type PhotoId } from '@/ipc/bindings'
import { isTyping } from '@/lib/keyboard'

/**
 * Applies the culling or rotation shortcut in `event` to `id`, if it is one.
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
  } else if (key === '[' || key === ']') {
    // Quarter turns (FR-006): [ to the left, ] to the right.
    void commands.rotatePhotos([id], key === ']')
  } else {
    return false
  }
  return true
}
