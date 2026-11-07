// SPDX-License-Identifier: GPL-3.0-only
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

type StageNavProps = {
  /** Which way it goes, which also places it at the stage's left or right edge. */
  to: 'previous' | 'next'
  label: string
  disabled?: boolean
  onClick: () => void
}

/**
 * The stage's previous and next buttons, for the mouse (STYLE_RULE §4): on the black stage's
 * own colours, 32 px as every control, shown while the pointer is over the stage. One block
 * for the photo preview and the sheet preview; its hint on a wrapper, so it still shows on
 * the first or last photo, when the button is off.
 */
export function StageNav({ to, label, disabled = false, onClick }: StageNavProps) {
  const Icon = to === 'previous' ? ChevronLeft : ChevronRight
  return (
    <span
      data-hint={label}
      className={cn(
        'absolute opacity-0 transition-opacity group-hover/stage:opacity-100 focus-within:opacity-100',
        to === 'previous' ? 'left-3' : 'right-3',
      )}
    >
      <Button
        variant="secondary"
        size="icon-sm"
        aria-label={label}
        disabled={disabled}
        onClick={onClick}
        className="bg-stage-chip text-stage-ink hover:bg-stage-raised"
      >
        <Icon />
      </Button>
    </span>
  )
}
