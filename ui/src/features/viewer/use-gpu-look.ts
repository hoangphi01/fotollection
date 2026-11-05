// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState } from 'react'
import { useDrag } from '@/lib/drag-state'
import { lookTableUrl, maskUrl, previewUrl } from '@/lib/photo-url'
import { GpuLook } from './gpu-look'

/** Off for the rest of the session after a failure: the CPU path then serves every drag. */
let broken = false

const bitmap = async (url: string) =>
  createImageBitmap(await (await fetch(url)).blob(), { colorSpaceConversion: 'none', premultiplyAlpha: 'none' })

type GpuLookState = {
  /** Give it the canvas laid exactly where the preview image is (its `ref` callback). */
  attach: (canvas: HTMLCanvasElement | null) => void
  /** Show the canvas instead of the image. */
  active: boolean
}

/**
 * The look on the GPU while a colour slider is dragged (ADR-0039). The plain preview and its
 * mask are loaded once when the drag starts; each change of `revision` then loads only Rust's
 * table, the newest winning. The canvas stays until Rust's render of the last change is on
 * screen (`settled`), so the photo never jumps back.
 */
export function useGpuLook(photoId: number | undefined, revision: number, settled: boolean): GpuLookState {
  const drag = useDrag()
  // The drag the photo was last loaded for, so a drag that ends keeps its canvas until
  // Rust's render lands, and the next one loads the photo again (its stamp may have changed).
  const [lastDrag, setLastDrag] = useState(0)
  const dragging = drag !== 0
  const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null)
  const gpu = useRef<{ canvas: HTMLCanvasElement; look: GpuLook } | null>(null)
  const [ready, setReady] = useState<string | null>(null)
  const key = photoId === undefined ? null : `${photoId}:${drag || lastDrag}`
  const want = !broken && photoId !== undefined
  const asked = useRef(0)

  // A drag starts: the plain photo and its mask, for this photo, once.
  useEffect(() => {
    if (!want || !dragging || !canvas || photoId === undefined || ready === key) return
    let live = true
    void (async () => {
      try {
        // Decided by the machine itself, once (GpuLook.worthIt): no switch (owner,
        // 2026-11-05). A weak or software GPU keeps Rust's path for the session.
        if (!GpuLook.worthIt()) throw new Error('the CPU is quicker here')
        if (gpu.current?.canvas !== canvas) {
          const look = GpuLook.create(canvas)
          if (!look) throw new Error('no WebGL2')
          gpu.current = { canvas, look }
        }
        const [photo, mask] = await Promise.all([
          bitmap(`${previewUrl(photoId, revision)}&plain=1`),
          bitmap(maskUrl(photoId, revision)),
        ])
        if (!live) return
        gpu.current.look.setPhoto(photo, mask)
        photo.close()
        mask.close()
        setReady(key)
        setLastDrag(drag)
      } catch {
        broken = true
      }
    })()
    return () => {
      live = false
    }
    // Once loaded (`ready`), later revisions only change the table: the early return above.
  }, [want, dragging, canvas, photoId, revision, ready, key, drag])

  // Each step: Rust's table for the newest revision.
  useEffect(() => {
    if (!want || ready !== key || photoId === undefined || !gpu.current) return
    const n = ++asked.current
    void fetch(lookTableUrl(photoId, revision))
      .then((r) => r.arrayBuffer())
      .then((buffer) => {
        if (n !== asked.current || !gpu.current) return
        const table = new Uint8Array(buffer)
        // An empty table: the look changes nothing now, Rust's render shows the plain photo.
        if (table.length > 0 && !gpu.current.look.draw(table)) broken = true
      })
      .catch(() => {
        broken = true
      })
  }, [want, ready, key, photoId, revision])

  return { attach: setCanvas, active: want && ready !== null && ready === key && (dragging || !settled) }
}
