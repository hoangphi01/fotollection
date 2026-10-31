// SPDX-License-Identifier: GPL-3.0-only
import { commands, type Queue, type QueueView } from '@/ipc/bindings'

/**
 * ↑ to the print basket, ↓ set aside (FR-050). A photo that stays in the view (the basket
 * in the gallery) moves on with `next`; one that leaves it is replaced by the next photo
 * when the list refreshes.
 */
export function sendTo(photoId: number, queue: Queue | null, view: QueueView, next: () => void) {
  void commands.setPhotoQueue([photoId], queue)
  const stays = queue === null ? view === 'gallery' : view === 'gallery' ? queue === 'print' : view === queue
  if (stays) next()
}
