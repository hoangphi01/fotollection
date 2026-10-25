// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState, type CSSProperties } from 'react'
import { regionUrl } from '@/lib/photo-url'
import type { View } from './use-zoom'

/** Wait this long after the last zoom or pan before asking for pixels. */
const SETTLE_MS = 150
/** Ask a little more than the screen shows, so a small pan stays sharp. */
const MARGIN = 0.1
/** Below this many screen pixels per preview pixel, the preview is sharp enough. */
const SHARP_ENOUGH = 1.05

type Rect = { x: number; y: number; w: number; h: number }
type Loaded = { src: string; base: string; rect: Rect }

/**
 * The sharp layer of a zoomed preview (FR-057). The preview is at most 1600 px, which a
 * weak machine renders quickly; zooming past its pixels asks Rust for the visible part of
 * the full-resolution render only, once the view has settled. The zoomed preview stays on
 * screen meanwhile, so the photo is never blank.
 *
 * `img` is the preview element and `base` its source; `stage` the element that clips it.
 */
export function useSharpRegion(
  img: HTMLImageElement | null,
  stage: HTMLElement | null,
  view: View,
  photoId: number | undefined,
  revision: number,
  base: string | null,
) {
  const [size, setSize] = useState<{ w: number; h: number } | null>(null)
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  const [wanted, setWanted] = useState<string | null>(null)

  // The preview's layout size, without the zoom transform.
  useEffect(() => {
    if (!img) return
    const observer = new ResizeObserver(() => setSize({ w: img.offsetWidth, h: img.offsetHeight }))
    observer.observe(img)
    return () => observer.disconnect()
  }, [img])

  const zoomed = view.scale > 1
  useEffect(() => {
    if (!zoomed || !img || !stage || !size || !base || photoId === undefined) return
    let live = true
    const timer = window.setTimeout(() => {
      const rect = visibleRect(view, size, stage.clientWidth, stage.clientHeight)
      const dpr = window.devicePixelRatio || 1
      if (!rect || size.w * view.scale * dpr <= img.naturalWidth * SHARP_ENOUGH) return
      const px = Math.ceil(rect.w * size.w * view.scale * dpr)
      const src = regionUrl(photoId, revision, rect, px)
      setWanted(src)
      const next = new Image()
      next.onload = () => live && setLoaded({ src, base, rect })
      next.src = src
    }, SETTLE_MS)
    return () => {
      live = false
      window.clearTimeout(timer)
    }
  }, [zoomed, img, stage, size, view, photoId, revision, base])

  const current = zoomed && loaded && loaded.base === base && size ? loaded : null
  return {
    /** The sharp part and where it sits, or null while there is none for this view. */
    layer: current && size ? { src: current.src, style: layerStyle(current.rect, view, size) } : null,
    /** A sharper part has been asked for and has not arrived yet. */
    loading: zoomed && wanted !== null && wanted !== loaded?.src,
  }
}

/** The part of the preview inside the stage, in fractions of the preview, with a margin. */
function visibleRect(view: View, size: { w: number; h: number }, sw: number, sh: number): Rect | null {
  // Preview coordinates from its centre: a stage point p shows (p − offset) / scale.
  const span = (stage: number, offset: number, length: number): [number, number] | null => {
    const from = Math.max(-length / 2, (-stage / 2 - offset) / view.scale)
    const to = Math.min(length / 2, (stage / 2 - offset) / view.scale)
    const pad = (to - from) * MARGIN
    const a = Math.max(0, (from - pad + length / 2) / length)
    const b = Math.min(1, (to + pad + length / 2) / length)
    return b > a ? [a, b - a] : null
  }
  const x = span(sw, view.x, size.w)
  const y = span(sh, view.y, size.h)
  if (!x || !y) return null
  // Rounded down to 4 places, so a region never ends past the photo (Rust refuses that).
  const floor = (v: number) => Math.floor(v * 10_000) / 10_000
  const [rx, ry] = [floor(x[0]), floor(y[0])]
  return { x: rx, y: ry, w: floor(Math.min(x[1], 1 - rx)), h: floor(Math.min(y[1], 1 - ry)) }
}

/**
 * Places the sharp part over the preview at its final on-screen size. Only a translation:
 * WebKit rasterises an element at its layout size before a CSS scale, so a scaled layer
 * would throw its extra pixels away and look as soft as the preview under it.
 */
function layerStyle(rect: Rect, view: View, size: { w: number; h: number }): CSSProperties {
  const left = view.x + view.scale * (rect.x - 0.5) * size.w
  const top = view.y + view.scale * (rect.y - 0.5) * size.h
  return {
    width: rect.w * size.w * view.scale,
    height: rect.h * size.h * view.scale,
    transform: `translate(${left}px, ${top}px)`,
  }
}
