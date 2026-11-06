// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState, type CSSProperties, type PointerEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { useSharpRegion } from './use-sharp-region'
import type { View } from './use-zoom'

/** Lightroom's before / after views (FR-096); `off` is the photo alone. */
export type Compare = 'off' | 'sideBySide' | 'split' | 'topBottom' | 'splitTopBottom'
/** One of the views, the photo alone left out. */
export type CompareMode = Exclude<Compare, 'off'>

type Pointer = {
  onPointerDown: (e: PointerEvent) => void
  onPointerMove: (e: PointerEvent) => void
  onPointerUp: (e: PointerEvent) => void
  onClick: (e: { clientX: number; clientY: number }) => void
}

type CompareStageProps = {
  mode: CompareMode
  photoId: number
  revision: number
  /** The photo with its colour, and without (`plain=1`). */
  after: string
  before: string
  view: View
  transform: string
  pointer: Pointer
  /** The After image, so the zoom can count the photo's own pixels (1:1, FR-095). */
  onAfterImage: (img: HTMLImageElement | null) => void
}

/**
 * The photo before and after its colour, in one of Lightroom's four views (FR-096): side by
 * side, one above the other, or one photo split by a line to drag. Both sides share one zoom
 * and pan, and each asks for its own sharp part at 1:1, so the comparison holds up close.
 */
export function CompareStage({ mode, photoId, revision, after, before, view, transform, pointer, onAfterImage }: CompareStageProps) {
  const { t } = useTranslation()
  const shared = { photoId, revision, view, transform, pointer }
  if (mode === 'sideBySide' || mode === 'topBottom') {
    return (
      <div className={cn('flex size-full gap-1', mode === 'topBottom' && 'flex-col')}>
        <Pane {...shared} src={before} plain label={t('compare.before')} />
        <Pane {...shared} src={after} label={t('compare.after')} onImage={onAfterImage} />
      </div>
    )
  }
  return <Split {...shared} vertical={mode === 'split'} after={after} before={before} onAfterImage={onAfterImage} />
}

type PaneProps = {
  photoId: number
  revision: number
  view: View
  transform: string
  pointer: Pointer
  src: string
  plain?: boolean
  label: string
  onImage?: (img: HTMLImageElement | null) => void
}

/** One side: the photo with the shared zoom, clipped to its half, sharp when zoomed. */
function Pane({ photoId, revision, view, transform, pointer, src, plain = false, label, onImage }: PaneProps) {
  const [box, setBox] = useState<HTMLDivElement | null>(null)
  const [img, setImg] = useState<HTMLImageElement | null>(null)
  const sharp = useSharpRegion(img, box, view, photoId, revision, src, plain)
  useEffect(() => onImage?.(img), [img, onImage])
  return (
    <div ref={setBox} className="relative flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden">
      <img
        ref={setImg}
        src={src}
        alt=""
        draggable={false}
        style={{ transform }}
        className={cn('max-h-full max-w-full object-contain stage-photo', view.scale > 1 ? 'cursor-grab active:cursor-grabbing' : 'cursor-zoom-in')}
        {...pointer}
      />
      {sharp.layer && <SharpLayer src={sharp.layer.src} style={sharp.layer.style} />}
      <CornerLabel>{label}</CornerLabel>
    </div>
  )
}

type SplitProps = Omit<PaneProps, 'src' | 'plain' | 'label' | 'onImage'> & {
  vertical: boolean
  after: string
  before: string
  onAfterImage: (img: HTMLImageElement | null) => void
}

/** One photo, Before on one side of a line and After on the other; the line is dragged. */
function Split({ photoId, revision, view, transform, pointer, vertical, after, before, onAfterImage }: SplitProps) {
  const { t } = useTranslation()
  const [box, setBox] = useState<HTMLDivElement | null>(null)
  const [afterImg, setAfterImg] = useState<HTMLImageElement | null>(null)
  const [beforeImg, setBeforeImg] = useState<HTMLImageElement | null>(null)
  // Where the line is, 0–1 across (or down) the stage.
  const [at, setAt] = useState(0.5)
  const [dragging, setDragging] = useState(false)
  const sharpAfter = useSharpRegion(afterImg, box, view, photoId, revision, after)
  const sharpBefore = useSharpRegion(beforeImg, box, view, photoId, revision, before, true)
  useEffect(() => onAfterImage(afterImg), [afterImg, onAfterImage])
  const pct = `${at * 100}%`
  const clip = vertical ? `inset(0 calc(100% - ${pct}) 0 0)` : `inset(0 0 calc(100% - ${pct}) 0)`
  const moveLine = (e: PointerEvent<HTMLDivElement>) => {
    if (!box) return
    const r = box.getBoundingClientRect()
    const f = vertical ? (e.clientX - r.left) / r.width : (e.clientY - r.top) / r.height
    setAt(Math.min(0.98, Math.max(0.02, f)))
  }
  return (
    <div ref={setBox} className="relative flex size-full items-center justify-center overflow-hidden">
      <img
        ref={setAfterImg}
        src={after}
        alt=""
        draggable={false}
        style={{ transform }}
        className={cn('max-h-full max-w-full object-contain stage-photo', view.scale > 1 ? 'cursor-grab active:cursor-grabbing' : 'cursor-zoom-in')}
        {...pointer}
      />
      {sharpAfter.layer && <SharpLayer src={sharpAfter.layer.src} style={sharpAfter.layer.style} />}
      {/* Before, the same size and place, shown up to the line. */}
      <div className="pointer-events-none absolute inset-0 flex items-center justify-center" style={{ clipPath: clip }}>
        <img ref={setBeforeImg} src={before} alt="" draggable={false} style={{ transform }} className="max-h-full max-w-full object-contain" />
        {sharpBefore.layer && <SharpLayer src={sharpBefore.layer.src} style={sharpBefore.layer.style} />}
      </div>
      <div
        role="separator"
        aria-orientation={vertical ? 'vertical' : 'horizontal'}
        aria-valuenow={Math.round(at * 100)}
        aria-label={t('compare.line')}
        data-hint={t('compare.line')}
        className={cn(
          'absolute z-10 flex touch-none items-center justify-center',
          vertical ? 'inset-y-0 w-4 -translate-x-1/2 cursor-ew-resize' : 'inset-x-0 h-4 -translate-y-1/2 cursor-ns-resize',
        )}
        style={vertical ? { left: pct } : { top: pct }}
        onPointerDown={(e) => {
          e.stopPropagation()
          e.currentTarget.setPointerCapture(e.pointerId)
          setDragging(true)
        }}
        onPointerMove={(e) => dragging && moveLine(e)}
        onPointerUp={() => setDragging(false)}
      >
        <div className={cn('bg-primary', vertical ? 'h-full w-px' : 'h-px w-full')} />
      </div>
      <CornerLabel>{t('compare.before')}</CornerLabel>
      <CornerLabel end>{t('compare.after')}</CornerLabel>
    </div>
  )
}

function SharpLayer({ src, style }: { src: string; style: CSSProperties }) {
  return <img src={src} alt="" aria-hidden draggable={false} style={style} className="pointer-events-none absolute top-1/2 left-1/2 max-w-none" />
}

/** "Before" or "After" in a corner of its side, on the neutral stage (STYLE_RULE §4). */
function CornerLabel({ children, end = false }: { children: ReactNode; end?: boolean }) {
  return (
    <span className={cn('pointer-events-none absolute top-2 rounded bg-stage-chip px-1.5 py-0.5 text-xs text-stage-ink', end ? 'right-2' : 'left-2')}>
      {children}
    </span>
  )
}
