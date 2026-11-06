// SPDX-License-Identifier: GPL-3.0-only
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

type TickCircleProps = {
  checked: boolean
  className?: string
}

/**
 * The tick of select mode (FR-050), the same on gallery tiles and collage sheets: a 20 px
 * circle as a folder tab's ✕ (owner, 2026-11-06), raised with a lip as every button, ticked in
 * the theme's colour (its lip the colour made deeper), unticked a dark chip on the photo.
 */
export function TickCircle({ checked, className }: TickCircleProps) {
  return (
    <span
      className={cn(
        'flex size-5 items-center justify-center rounded-full [--lip:2px] raised-lip',
        checked
          ? 'bg-primary text-primary-foreground [--lip-color:color-mix(in_oklch,var(--primary),black_35%)]'
          : 'bg-stage-chip text-stage-ink ring-1 ring-stage-edge [--lip-color:var(--stage-edge)]',
        className,
      )}
      aria-hidden
    >
      {checked && <Check className="size-3.5" strokeWidth={3} />}
    </span>
  )
}
