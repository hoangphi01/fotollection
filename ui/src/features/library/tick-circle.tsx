// SPDX-License-Identifier: GPL-3.0-only
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

type TickCircleProps = {
  checked: boolean
  className?: string
}

/** The tick of select mode (FR-050), the same on gallery tiles and collage sheets. */
export function TickCircle({ checked, className }: TickCircleProps) {
  return (
    <span
      className={cn(
        'flex size-5 items-center justify-center rounded-full border-2 shadow',
        checked ? 'border-primary bg-primary text-primary-foreground' : 'border-white bg-black/30',
        className,
      )}
      aria-hidden
    >
      {checked && <Check className="size-3" strokeWidth={3} />}
    </span>
  )
}
