// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState, type ReactNode } from 'react'
import { Slider } from '@/components/ui/slider'
import { setDragging } from '@/lib/drag-state'
import { Field, ValueRow } from './field'

type SliderFieldProps = {
  label: string
  value: number
  /** `first` is true for the first value of a drag or a click, so one gesture is one undo step. */
  onChange: (value: number, first: boolean) => void
  min?: number
  max?: number
  /** Where a double-click puts it back. */
  reset?: number
  /** The value as shown at the right of the label; signed values get their sign by default. */
  format?: (value: number) => ReactNode
  /** What it does, shown on hover, so a card of sliders stays short (FR-079). */
  hint?: string
  /** `false` when its drag cannot be drawn on the GPU (a RAW's linear sliders, FR-088). */
  gpu?: boolean
}

/** The thumb Radix renders, the element that takes the arrow keys. */
const THUMB = '[role=slider]'

const signed = (v: number) => (v > 0 ? `+${v}` : v < 0 ? `−${-v}` : '0')

/**
 * A slider with its label and value on one line (STYLE_RULE §4, the lego rule); what it does
 * shows on hover. The value moves at once; while dragging, changes go out at most once a
 * frame, the last one always, so a preview follows without a request per pixel. A
 * double-click resets it. As in a RAW editor, a press a little above or below the track still
 * takes it (the slider's 8 px of reach), and a click on the label focuses it for the arrow
 * keys: ±1, with Shift ±10.
 */
export function SliderField({
  label,
  value,
  onChange,
  min = -100,
  max = 100,
  reset = 0,
  format = signed,
  hint,
  gpu = true,
}: SliderFieldProps) {
  const [draft, setDraft] = useState<number | null>(null)
  const dragging = useRef(false)
  const pending = useRef<number | null>(null)
  const frame = useRef<number | null>(null)
  const root = useRef<HTMLSpanElement>(null)
  const shown = draft ?? value
  const focusThumb = () => {
    const thumb = root.current?.querySelector(THUMB)
    if (thumb instanceof HTMLElement) thumb.focus()
  }

  const flush = () => {
    frame.current = null
    if (pending.current === null) return
    const next = pending.current
    pending.current = null
    onChange(next, false)
  }
  useEffect(
    () => () => {
      if (frame.current !== null) cancelAnimationFrame(frame.current)
    },
    [],
  )

  return (
    <Field>
      {/* What a slider does shows on its name only, never over the track or the value: a hint
          shown while dragging hid the number being set (owner, 2026-11-06). */}
      <div onClick={focusThumb}>
        <ValueRow label={<span data-hint={hint}>{label}</span>}>{format(shown)}</ValueRow>
      </div>
      <Slider
        ref={root}
        // A signed slider has no "on" side: no filled range from its left end at 0.
        className={min < 0 ? '[&_[data-slot=slider-range]]:bg-transparent' : undefined}
        min={min}
        max={max}
        step={1}
        value={[shown]}
        aria-label={label}
        onValueChange={([v]) => {
          if (v === undefined) return
          setDraft(v)
          if (!dragging.current) {
            dragging.current = true
            // The preview follows on the GPU while the drag lasts (ADR-0039).
            setDragging(true, gpu)
            onChange(v, true)
            return
          }
          pending.current = v
          frame.current ??= requestAnimationFrame(flush)
        }}
        onValueCommit={([v]) => {
          if (frame.current !== null) cancelAnimationFrame(frame.current)
          frame.current = null
          pending.current = null
          if (v !== undefined) onChange(v, false)
          dragging.current = false
          setDragging(false)
          setDraft(null)
        }}
        onDoubleClick={() => onChange(reset, true)}
      />
    </Field>
  )
}
