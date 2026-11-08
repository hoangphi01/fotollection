// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { cn } from '@/lib/utils'
import { JOINED_BAR } from './joined-bar'

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
  /** In a card's header (the Navigator's zoom, FR-095): 28 px, flat choices, the chosen one
   * pressed in; always as wide as its items. */
  small?: boolean
}

/**
 * One choice among a few, as one raised bar the card's width, its choices the same width, 32 px
 * tall like every control in a panel (STYLE_RULE §4, the lego rule): joined as a button group,
 * a hairline between them, the chosen one pressed in, in a darker accent, as a console's
 * buttons (FR-099, "Depth"; owner, 2026-11-08).
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
  small = false,
}: SegmentedToggleProps<T>) {
  return (
    <ToggleGroup
      type="single"
      variant="default"
      size={small ? 'sm' : 'default'}
      spacing={0}
      // One raised bar, as the Project block's ButtonGroup (owner, 2026-11-08: "a button group
      // saves the pixels between the buttons"): flat choices, a hairline between them, the
      // chosen one pressed into the bar. The frame is a ring, so the bar stays 32 px.
      className={cn(
        JOINED_BAR,
        fit || small ? 'shrink-0' : 'w-full',
      )}
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
          // Shared rows give each word all the room there is, 4 px a side (the words stay centred):
          // three choices in an 18 rem panel had 66 px (70 at 18.75 rem), and `MEASURE=1` found *Thành phố*,
          // *Không RAW* and *One colour* spilling into a 10 px padding. Dates set mono a size
          // smaller, so four fit.
          // Important: joined, shadcn gives each item 8 px a side, which would eat the words' room.
          className={cn('text-xs focus-visible:ring-inset', small && 'px-1.5!', !fit && !small && 'min-w-0 flex-1 px-1!', (dense || mono) && 'px-1!', mono && 'font-mono text-2xs tracking-tight')}
        >
          {o.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
