// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef } from 'react'
import { useDarkTheme } from '@/lib/theme'

export type Channels = { r: readonly number[]; g: readonly number[]; b: readonly number[]; l: readonly number[] }

/** Levels 0–255 at 0–1000 high (Rust's scale) to device pixels of a w × h canvas. */
function trace(ctx: CanvasRenderingContext2D, levels: readonly number[], w: number, h: number) {
  const last = levels.length - 1
  levels.forEach((v, i) => {
    const x = (i / last) * w
    const y = h - (v / 1000) * h
    if (i === 0) ctx.moveTo(x, y)
    else ctx.lineTo(x, y)
  })
}

/**
 * The histogram's curves drawn on a canvas sized in device pixels (FR-098): sharp at every
 * display scale. An SVG stretched by `preserveAspectRatio="none"` and blended with
 * `mix-blend-mode` is raster-cached by Chromium as a layer and scaled, so at Windows'
 * 125 % or 150 % it came out soft (LESSONS). The channels blend among themselves, multiply on
 * light and screen on dark, in the theme's `--hist-*` colours; the light is a line on top.
 */
export function HistogramPlot({ channels, label, className }: { channels: Channels | null; label: string; className?: string }) {
  const ref = useRef<HTMLCanvasElement>(null)
  const dark = useDarkTheme()

  useEffect(() => {
    const canvas = ref.current
    if (!canvas) return
    const draw = () => {
      const ratio = window.devicePixelRatio || 1
      const w = Math.max(1, Math.round(canvas.clientWidth * ratio))
      const h = Math.max(1, Math.round(canvas.clientHeight * ratio))
      if (canvas.width !== w) canvas.width = w
      if (canvas.height !== h) canvas.height = h
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      ctx.clearRect(0, 0, w, h)
      if (!channels) return
      const style = getComputedStyle(canvas)
      ctx.globalCompositeOperation = dark ? 'screen' : 'multiply'
      for (const [levels, token] of [
        [channels.r, '--hist-r'],
        [channels.g, '--hist-g'],
        [channels.b, '--hist-b'],
      ] as const) {
        ctx.beginPath()
        ctx.moveTo(0, h)
        trace(ctx, levels, w, h)
        ctx.lineTo(w, h)
        ctx.closePath()
        ctx.fillStyle = style.getPropertyValue(token).trim()
        ctx.fill()
      }
      ctx.globalCompositeOperation = 'source-over'
      ctx.beginPath()
      trace(ctx, channels.l, w, h)
      // The canvas's own `color` (text-foreground/70), so the line follows the theme.
      ctx.strokeStyle = style.color
      ctx.lineWidth = 1.5 * ratio
      ctx.lineJoin = 'round'
      ctx.stroke()
    }
    draw()
    // A new width (the column resized) or a new display scale (the window moved to another
    // screen) redraws at the new device size.
    const observer = new ResizeObserver(draw)
    observer.observe(canvas)
    let scale: MediaQueryList | null = null
    const onScale = () => {
      draw()
      arm()
    }
    // A media query matches one scale only, so it is set again after each change.
    const arm = () => {
      scale?.removeEventListener('change', onScale)
      scale = window.matchMedia(`(resolution: ${window.devicePixelRatio || 1}dppx)`)
      scale.addEventListener('change', onScale)
    }
    arm()
    return () => {
      observer.disconnect()
      scale?.removeEventListener('change', onScale)
    }
  }, [channels, dark])

  return <canvas ref={ref} role="img" aria-label={label} className={className} />
}
