// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState, type ReactNode } from 'react'
import { Slider } from '@/components/ui/slider'
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
}

const signed = (v: number) => (v > 0 ? `+${v}` : v < 0 ? `−${-v}` : '0')

/**
 * A slider with its label and value on one line (STYLE_RULE §4, the lego rule); what it does
 * shows on hover. The value moves at once; while dragging, changes go out at most once a
 * frame, the last one always, so a preview follows without a request per pixel. A
 * double-click resets it.
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
}: SliderFieldProps) {
  const [draft, setDraft] = useState<number | null>(null)
  const dragging = useRef(false)
  const pending = useRef<number | null>(null)
  const frame = useRef<number | null>(null)
  const shown = draft ?? value

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
      <div data-hint={hint}>
        <ValueRow label={label}>{format(shown)}</ValueRow>
      </div>
      <Slider
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
          setDraft(null)
        }}
        onDoubleClick={() => onChange(reset, true)}
        data-hint={hint}
      />
    </Field>
  )
}
