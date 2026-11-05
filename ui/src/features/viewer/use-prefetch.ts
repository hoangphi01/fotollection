// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef } from 'react'
import { commands, type PhotoId } from '@/ipc/bindings'

/**
 * Once the photo at `index` is on screen (`landed`), Rust loads the next two in the
 * direction of travel and the one behind into memory, from their standard previews, so ←
 * and → show at once (FR-091). A newer request stops the older one in Rust; it never runs
 * before the photo on screen.
 */
export function usePrefetch(photos: readonly { id: PhotoId }[], index: number | null, landed: boolean) {
  const direction = useRef(1)
  const last = useRef<number | null>(null)
  // The ids two back to two ahead, as a string: the effect runs when they change, not when
  // the caller builds a new array.
  const around = index === null ? '' : [-2, -1, 1, 2].map((o) => photos[index + o]?.id ?? '').join(',')
  useEffect(() => {
    if (!landed || index === null) return
    if (last.current !== null && index !== last.current) direction.current = index > last.current ? 1 : -1
    last.current = index
    const [back2, back1, next1, next2] = around.split(',')
    const order = direction.current > 0 ? [next1, next2, back1] : [back1, back2, next1]
    const ids = order.filter((id) => !!id).map(Number)
    if (ids.length > 0) void commands.prefetchPreviews(ids)
  }, [landed, index, around])
}
