// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { Compass } from 'lucide-react'
import { useRef, type PointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SectionCard } from '@/components/layout/section-card'
import type { View } from './use-zoom'

/** The Navigator's zoom choices, as Lightroom's. */
type Choice = 'fit' | 'fill' | 'one' | 'two'

type NavigatorCardProps = {
  /** The photo as the stage shows it (already loaded: no new decode); null while the first
   * one loads, the column keeping its place. */
  src: string | null
  view: View
  /** The photo's size on the stage at fit, and the stage's inner size, in CSS px. */
  fit: { w: number; h: number } | null
  stage: { w: number; h: number } | null
  /** The scale of 1:1 (FR-095), or null when the photo's pixels are not known yet. */
  oneToOne: number | null
  onZoomTo: (scale: number) => void
  onPanTo: (x: number, y: number) => void
}

/**
 * The develop view's Navigator (FR-095), as Lightroom's: the photo small, the part the stage
 * shows outlined; a click or a drag moves the zoomed photo there; Fit, Fill, 1:1 and 2:1.
 */
export function NavigatorCard({ src, view, fit, stage, oneToOne, onZoomTo, onPanTo }: NavigatorCardProps) {
  const { t } = useTranslation()
  const dragging = useRef(false)
  const ready = fit && stage && fit.w > 0 && fit.h > 0
  const fill = ready ? Math.max(stage.w / fit.w, stage.h / fit.h) : 1
  const scales: Record<Choice, number | null> = {
    fit: 1,
    fill,
    one: oneToOne,
    two: oneToOne === null ? null : 2 * oneToOne,
  }
  const close = (a: number | null) => a !== null && Math.abs(a - view.scale) < 0.01 * view.scale
  const current = (Object.keys(scales) as Choice[]).find((c) => close(scales[c]))

  // The part shown, as fractions of the photo.
  const box = ready
    ? (() => {
        const w = Math.min(1, stage.w / (view.scale * fit.w))
        const h = Math.min(1, stage.h / (view.scale * fit.h))
        const cx = 0.5 - view.x / (view.scale * fit.w)
        const cy = 0.5 - view.y / (view.scale * fit.h)
        return { left: cx - w / 2, top: cy - h / 2, w, h }
      })()
    : null

  const moveTo = (e: PointerEvent<HTMLDivElement>) => {
    if (!ready) return
    const r = e.currentTarget.getBoundingClientRect()
    const cx = (e.clientX - r.left) / r.width
    const cy = (e.clientY - r.top) / r.height
    onPanTo((0.5 - cx) * view.scale * fit.w, (0.5 - cy) * view.scale * fit.h)
  }

  return (
    <SectionCard icon={<Compass />} title={t('navigator.title')}>
      <div className="flex justify-center rounded-md bg-neutral-950 p-2">
        <div
          className="relative cursor-crosshair touch-none overflow-hidden"
          onPointerDown={(e) => {
            dragging.current = true
            e.currentTarget.setPointerCapture(e.pointerId)
            moveTo(e)
          }}
          onPointerMove={(e) => dragging.current && moveTo(e)}
          onPointerUp={() => (dragging.current = false)}
        >
          {src ? (
            <img src={src} alt="" draggable={false} className="block max-h-40 max-w-full select-none" />
          ) : (
            <div className="h-40 w-60" />
          )}
          {box && view.scale > 1 && (
            <div
              aria-hidden
              className="pointer-events-none absolute rounded-xs ring-2 ring-primary shadow-[0_0_0_9999px_rgb(0_0_0/0.45)]"
              style={{
                left: `${box.left * 100}%`,
                top: `${box.top * 100}%`,
                width: `${box.w * 100}%`,
                height: `${box.h * 100}%`,
              }}
            />
          )}
        </div>
      </div>
      <SegmentedToggle<Choice>
        value={current}
        dense
        ariaLabel={t('navigator.zoom')}
        options={[
          { value: 'fit', label: t('viewer.fit') },
          { value: 'fill', label: t('navigator.fill') },
          { value: 'one', label: '1:1' },
          { value: 'two', label: '2:1' },
        ]}
        onChange={(c) => {
          const scale = scales[c]
          if (scale !== null) onZoomTo(scale)
        }}
      />
    </SectionCard>
  )
}
