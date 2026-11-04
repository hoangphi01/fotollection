// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { cn } from '@/lib/utils'

export type Segment<T extends string> = {
  value: T
  label: ReactNode
  /** For an icon-only segment. */
  ariaLabel?: string
  hint?: string
}

type SegmentedToggleProps<T extends string> = {
  value: T | undefined
  options: readonly Segment<T>[]
  onChange: (value: T) => void
  ariaLabel?: string
  disabled?: boolean
  /** Dates and other figures in the monospaced face. */
  mono?: boolean
  /** Tighter items, for four or more in one row. */
  dense?: boolean
  /** As wide as its items, beside other controls in a row (a unit next to two numbers). */
  fit?: boolean
}

/**
 * One choice among a few, as a row of buttons the card's width, each the same width, 32 px
 * tall like every control in a panel (STYLE_RULE §4, the lego rule).
 */
export function SegmentedToggle<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  disabled = false,
  mono = false,
  dense = false,
  fit = false,
}: SegmentedToggleProps<T>) {
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      className={fit ? 'shrink-0' : 'w-full'}
      aria-label={ariaLabel}
      disabled={disabled}
      value={value ?? ''}
      onValueChange={(v) => v && onChange(v as T)}
    >
      {options.map((o) => (
        <ToggleGroupItem
          key={o.value}
          value={o.value}
          aria-label={o.ariaLabel}
          data-hint={o.hint}
          className={cn('text-xs', !fit && 'min-w-0 flex-1', (dense || mono) && 'px-1', mono && 'font-mono tracking-tight')}
        >
          {o.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
