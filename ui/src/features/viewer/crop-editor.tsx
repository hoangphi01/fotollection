// SPDX-License-Identifier: GPL-3.0-only
import { Check, RotateCcw, Ruler, X, ZoomIn, ZoomOut } from 'lucide-react'
import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { CropFrame, CropGeometry } from '@/ipc/bindings'
import {
  DPI_STYLE,
  MAX_STRAIGHTEN,
  MAX_ZOOM,
  dpiBand,
  frameOf,
  ratioLabel,
  straightenedRect,
  type Frame,
} from '@/lib/crop-geometry'
import { formatNumber } from '@/lib/format'
import { cn } from '@/lib/utils'

type CropEditorProps = {
  geometry: CropGeometry
  sourceSrc: string
  /** The crop (`null` = centred) and the straightening in degrees, saved together. */
  onCommit: (crop: CropFrame | null, straighten: number) => void
  onCancel: () => void
}

const CENTRED: Frame = { cx: 0.5, cy: 0.5, zoom: 1 }

function toFrame(crop: CropFrame | null): Frame {
  return crop ? { cx: crop.cx ?? 0.5, cy: crop.cy ?? 0.5, zoom: crop.zoom ?? 1 } : CENTRED
}

/**
 * Interactive crop (FR-006): the whole photo with everything outside the crop dimmed,
 * drag to move, wheel/pinch or slider to zoom, and a live label with the printed size in
 * cm, the crop's pixel size and its print DPI (FR-026).
 */
export function CropEditor({ geometry, sourceSrc, onCommit, onCancel }: CropEditorProps) {
  const { t } = useTranslation()
  const g = geometry
  const stageRef = useRef<HTMLDivElement>(null)
  const boxRef = useRef<HTMLDivElement>(null)
  const [stage, setStage] = useState({ width: 0, height: 0 })
  const [frame, setFrame] = useState<Frame>(() => toFrame(g.crop))
  const [angle, setAngle] = useState(() => g.straighten ?? 0)
  const [leveling, setLeveling] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)
  // Take the keyboard on open, so the arrow keys level the photo right away.
  useEffect(() => rootRef.current?.focus(), [])
  const drag = useRef<{ x: number; y: number; frame: Frame } | null>(null)

  useEffect(() => {
    const el = stageRef.current
    if (!el) return
    const observer = new ResizeObserver(([e]) =>
      setStage({ width: e?.contentRect.width ?? 0, height: e?.contentRect.height ?? 0 }),
    )
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const rectAt = (f: Frame, degrees: number) =>
    straightenedRect(g.width, g.height, g.targetWidth, g.targetHeight, f, degrees)
  const rect = rectAt(frame, angle)
  const setClamped = (next: Frame, degrees = angle) => {
    const r = rectAt(next, degrees)
    setFrame(frameOf(r, g.width, g.height, Math.min(MAX_ZOOM, Math.max(1, next.zoom))))
  }
  const setStraighten = (degrees: number) => {
    // Rounded to the slider's 0.1° steps so repeated key presses don't drift.
    const d = Math.round(Math.max(-MAX_STRAIGHTEN, Math.min(MAX_STRAIGHTEN, degrees)) * 10) / 10
    setAngle(d)
    setClamped(frame, d)
  }

  // Wheel and trackpad pinch (ctrlKey wheel) zoom; needs a non-passive listener.
  useEffect(() => {
    const el = boxRef.current
    if (!el) return
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      setFrame((f) => {
        const zoom = Math.min(MAX_ZOOM, Math.max(1, f.zoom * Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.0015))))
        const r = straightenedRect(g.width, g.height, g.targetWidth, g.targetHeight, { ...f, zoom }, angle)
        return frameOf(r, g.width, g.height, zoom)
      })
    }
    el.addEventListener('wheel', onWheel, { passive: false })
    return () => el.removeEventListener('wheel', onWheel)
  }, [g.width, g.height, g.targetWidth, g.targetHeight, angle])

  const scale = Math.min(stage.width / g.width, stage.height / g.height) || 0
  const box = { width: g.width * scale, height: g.height * scale }

  const onPointerDown = (e: ReactPointerEvent) => {
    e.currentTarget.setPointerCapture(e.pointerId)
    drag.current = { x: e.clientX, y: e.clientY, frame }
  }
  const onPointerMove = (e: ReactPointerEvent) => {
    const d = drag.current
    if (!d || box.width === 0) return
    setClamped({
      ...d.frame,
      cx: d.frame.cx + (e.clientX - d.x) / box.width,
      cy: d.frame.cy + (e.clientY - d.y) / box.height,
    })
  }
  const onPointerUp = () => {
    drag.current = null
  }

  const onKeyDown = (e: React.KeyboardEvent) => {
    // Sliders keep their own arrow keys (the straighten slider steps 0.1°).
    if (e.key.startsWith('Arrow') && e.target instanceof HTMLInputElement) return
    const step = e.shiftKey ? 0.05 : 0.01
    const moves: Record<string, [number, number]> = {
      ArrowUp: [0, -step],
      ArrowDown: [0, step],
    }
    const move = moves[e.key]
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      // ← → level the photo a degree at a time, ⌥ for a tenth (FR-006).
      e.preventDefault()
      e.stopPropagation()
      const delta = (e.key === 'ArrowLeft' ? -1 : 1) * (e.altKey ? 0.1 : 1)
      setLeveling(true)
      setStraighten(angle + delta)
    } else if (move) {
      e.preventDefault()
      e.stopPropagation()
      setClamped({ ...frame, cx: frame.cx + move[0], cy: frame.cy + move[1] })
    } else if (e.key === '+' || e.key === '=') {
      setClamped({ ...frame, zoom: frame.zoom * 1.1 })
    } else if (e.key === '-') {
      setClamped({ ...frame, zoom: frame.zoom / 1.1 })
    } else if (e.key === 'Enter') {
      e.preventDefault()
      onCommit(frame, angle)
    } else if (e.key === '0') {
      setFrame(CENTRED)
      setAngle(0)
    }
  }

  const pct = (v: number, total: number) => `${(v / total) * 100}%`
  const cmW = g.printWidthCm ?? 0
  const cmH = g.printHeightCm ?? 0
  const longCm = Math.max(cmW, cmH)
  const dpi = longCm > 0 ? Math.round(Math.max(rect.width, rect.height) / (longCm / 2.54)) : 0
  const band = dpiBand(dpi)
  const unit = g.unit ?? 'cm'
  const lenW = unit === 'in' ? cmW / 2.54 : cmW
  const lenH = unit === 'in' ? cmH / 2.54 : cmH
  const fmtLen = (v: number, withUnit = true) =>
    `${formatNumber(v, 1)}${withUnit ? ` ${unit}` : ''}`
  const ratio = ratioLabel(g.targetWidth, g.targetHeight)
  const atTop = rect.y / g.height < 0.06
  const atLeft = rect.x / g.width < 0.06

  return (
    // oxlint-disable-next-line jsx-a11y/no-static-element-interactions -- keyboard handled, focusable
    <div
      ref={rootRef}
      className="flex size-full flex-col gap-3 outline-none"
      tabIndex={0}
      onKeyDown={onKeyDown}
      onKeyUp={(e) => e.key.startsWith('Arrow') && setLeveling(false)}
    >
      <div ref={stageRef} className="relative flex min-h-0 flex-1 items-center justify-center">
        <div
          ref={boxRef}
          className="relative cursor-move touch-none select-none"
          style={{ width: box.width, height: box.height }}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onDoubleClick={() => setFrame(CENTRED)}
        >
          {/* The photo turns under a fixed crop box (FR-006); only the straightened canvas shows. */}
          <div className="absolute inset-0 overflow-hidden rounded-sm">
            <img
              src={sourceSrc}
              alt=""
              draggable={false}
              className="size-full"
              style={angle ? { transform: `rotate(${angle}deg)` } : undefined}
            />
          </div>
          {/* Dim everything outside the crop: four bands around the crop rectangle. */}
          {[
            { left: 0, top: 0, width: '100%', height: pct(rect.y, g.height) },
            {
              left: 0,
              top: pct(rect.y + rect.height, g.height),
              width: '100%',
              height: pct(g.height - rect.y - rect.height, g.height),
            },
            {
              left: 0,
              top: pct(rect.y, g.height),
              width: pct(rect.x, g.width),
              height: pct(rect.height, g.height),
            },
            {
              left: pct(rect.x + rect.width, g.width),
              top: pct(rect.y, g.height),
              width: pct(g.width - rect.x - rect.width, g.width),
              height: pct(rect.height, g.height),
            },
          ].map((band, i) => (
            // oxlint-disable-next-line react/no-array-index-key -- fixed set of four bands
            <div key={i} className="pointer-events-none absolute bg-black/60" style={band} />
          ))}
          <div
            className="pointer-events-none absolute border border-white/90 shadow-[0_0_0_1px_rgba(0,0,0,0.4)]"
            style={{
              left: pct(rect.x, g.width),
              top: pct(rect.y, g.height),
              width: pct(rect.width, g.width),
              height: pct(rect.height, g.height),
            }}
          >
            {/* A fine grid while levelling, to line up horizons and walls. */}
            {(leveling || angle !== 0) &&
              [1, 2, 3, 4, 5, 6, 7].map((i) => (
                <div key={i} className="contents">
                  <div className="absolute inset-y-0 border-l border-white/15" style={{ left: `${(i * 100) / 8}%` }} />
                  <div className="absolute inset-x-0 border-t border-white/15" style={{ top: `${(i * 100) / 8}%` }} />
                </div>
              ))}
            {/* Rule of thirds */}
            <div className="absolute inset-y-0 left-1/3 border-l border-white/35" />
            <div className="absolute inset-y-0 left-2/3 border-l border-white/35" />
            <div className="absolute inset-x-0 top-1/3 border-t border-white/35" />
            <div className="absolute inset-x-0 top-2/3 border-t border-white/35" />
            {/* Corner handles */}
            {['-top-1 -left-1', '-top-1 -right-1', '-bottom-1 -left-1', '-bottom-1 -right-1'].map((c) => (
              <span key={c} className={cn('absolute size-3 border-white', c, c.includes('top') ? 'border-t-2' : 'border-b-2', c.includes('left') ? 'border-l-2' : 'border-r-2')} />
            ))}
            {/* Width along the top edge, height along the left edge (inside when at the photo's edge). */}
            <div
              className={cn(
                'absolute left-1/2 -translate-x-1/2 rounded bg-black/75 px-2 py-0.5 text-[11px] whitespace-nowrap text-white tabular-nums',
                atTop ? 'top-1.5' : '-top-6',
              )}
            >
              <span className="font-medium">{fmtLen(lenW)}</span>
              <span className="text-white/60"> · {rect.width} px</span>
            </div>
            <div
              className={cn(
                'absolute top-1/2 origin-center -translate-y-1/2 -rotate-90 rounded bg-black/75 px-2 py-0.5 text-[11px] whitespace-nowrap text-white tabular-nums',
                atLeft ? 'left-0 translate-x-[-30%]' : '-left-2 -translate-x-full',
              )}
              style={{ transformOrigin: 'center' }}
            >
              <span className="font-medium">{fmtLen(lenH)}</span>
              <span className="text-white/60"> · {rect.height} px</span>
            </div>
            {/* Ratio and print quality in the middle of the crop. */}
            <div className="absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 items-center gap-2 rounded-full bg-black/70 px-3 py-1 text-xs whitespace-nowrap text-white tabular-nums">
              <span className="font-semibold">{ratio}</span>
              <span className="text-white/60">
                {fmtLen(lenW, false)} × {fmtLen(lenH)}
              </span>
              <span className={cn('rounded px-1.5 py-0.5 font-medium', DPI_STYLE[band].className)}>
                {dpi} DPI
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="flex items-center justify-center gap-3 pb-1">
        <ZoomOut className="size-4 text-white/60" />
        <input
          type="range"
          min={1}
          max={MAX_ZOOM}
          step={0.01}
          value={frame.zoom}
          aria-label={t('crop.zoom')}
          onChange={(e) => setClamped({ ...frame, zoom: Number(e.target.value) })}
          className="w-56 accent-white"
        />
        <ZoomIn className="size-4 text-white/60" />
        <span className="w-12 text-xs text-white/70 tabular-nums">{formatNumber(frame.zoom, 2, 2)}×</span>
        <Ruler className="size-4 text-white/60" aria-hidden />
        <input
          type="range"
          min={-MAX_STRAIGHTEN}
          max={MAX_STRAIGHTEN}
          step={0.1}
          value={angle}
          aria-label={t('crop.straighten')}
          data-hint={t('crop.straightenHint')}
          onChange={(e) => setStraighten(Number(e.target.value))}
          onPointerDown={() => setLeveling(true)}
          onPointerUp={() => setLeveling(false)}
          onDoubleClick={() => setStraighten(0)}
          className="w-40 accent-white"
        />
        <span className="w-12 text-xs text-white/70 tabular-nums">{formatNumber(angle, 1, 1)}°</span>
        <Button variant="ghost" size="sm" className="text-white/80" onClick={() => onCommit(null, 0)}>
          <RotateCcw data-icon="inline-start" /> {t('common.reset')}
        </Button>
        <Button variant="secondary" size="sm" onClick={onCancel}>
          <X data-icon="inline-start" /> {t('common.cancel')}
        </Button>
        <Button size="sm" onClick={() => onCommit(frame, angle)}>
          <Check data-icon="inline-start" /> {t('common.done')}
        </Button>
      </div>
    </div>
  )
}
