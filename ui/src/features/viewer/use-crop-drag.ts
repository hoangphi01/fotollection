// SPDX-License-Identifier: GPL-3.0-only
import { useRef, useState, type PointerEvent } from 'react'
import { commands, type CropFrame, type CropGeometry } from '@/ipc/bindings'
import { frameOf, straightenedRect, type Frame } from '@/lib/crop-geometry'

/**
 * Drag the photo in the preview to choose what the print keeps (FR-006, FR-040 "Crop to
 * fill"): the crop moves under the pointer and the preview re-renders as it goes. Rust
 * folds one drag into a single undo step (`amend`). At most one save is in flight; the
 * newest position waits for it, so a fast drag never queues renders.
 */
export function useCropDrag(photoId: number | undefined, geometry: CropGeometry | null, enabled: boolean) {
  const [dragging, setDragging] = useState(false)
  const drag = useRef<{ x: number; y: number; start: Frame; scale: number; amend: boolean } | null>(null)
  const pending = useRef<{ frame: Frame; amend: boolean } | null>(null)
  const busy = useRef(false)

  function save(frame: Frame, amend: boolean) {
    if (photoId === undefined || !geometry) return
    if (busy.current) {
      // Keep the newest position; it amends only if every queued one did.
      pending.current = { frame, amend: amend && (pending.current?.amend ?? true) }
      return
    }
    busy.current = true
    void commands.setPhotoCrop(photoId, frame satisfies CropFrame, geometry.straighten ?? 0, amend).finally(() => {
      busy.current = false
      const next = pending.current
      pending.current = null
      if (next) save(next.frame, next.amend)
    })
  }
  /** The first move of a drag starts a new undo step; the rest amend it. */
  const send = (frame: Frame) => {
    const d = drag.current
    save(frame, d?.amend ?? true)
    if (d) d.amend = true
  }

  const moveTo = (e: PointerEvent) => {
    const d = drag.current
    if (!d || !geometry) return null
    const g = geometry
    // Dragging the picture right shows more of its left side: the crop moves the other way.
    const want = {
      ...d.start,
      cx: d.start.cx - ((e.clientX - d.x) * d.scale) / g.width,
      cy: d.start.cy - ((e.clientY - d.y) * d.scale) / g.height,
    }
    const r = straightenedRect(g.width, g.height, g.targetWidth, g.targetHeight, want, g.straighten ?? 0)
    return frameOf(r, g.width, g.height, d.start.zoom)
  }

  return {
    dragging,
    onPointerDown: (e: PointerEvent) => {
      if (!enabled || !geometry || e.button !== 0 || !(e.target instanceof HTMLImageElement)) return
      const g = geometry
      const start = { cx: g.crop?.cx ?? 0.5, cy: g.crop?.cy ?? 0.5, zoom: g.crop?.zoom ?? 1 }
      const crop = straightenedRect(g.width, g.height, g.targetWidth, g.targetHeight, start, g.straighten ?? 0)
      // The photo spans the preview's width (with a frame, the bar sits below it).
      const scale = crop.width / Math.max(1, e.target.getBoundingClientRect().width)
      drag.current = { x: e.clientX, y: e.clientY, start, scale, amend: false }
      e.currentTarget.setPointerCapture(e.pointerId)
      setDragging(true)
    },
    onPointerMove: (e: PointerEvent) => {
      const frame = moveTo(e)
      if (frame) send(frame)
    },
    onPointerUp: (e: PointerEvent) => {
      const frame = moveTo(e)
      if (frame) send(frame)
      drag.current = null
      setDragging(false)
    },
  }
}
