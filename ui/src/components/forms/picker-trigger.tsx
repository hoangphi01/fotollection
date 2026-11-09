// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDownIcon } from 'lucide-react'
import type * as React from 'react'
import { cn } from 'cn'
import { triggerClass } from '@/components/ui/select'

/**
 * The button of a picker that opens a popover (a search, a list of chapters): drawn as a
 * Select's, its icon, its words cut short and the same chevron, so every dropdown in the app
 * looks and opens alike (owner, 2026-11-08: "every dropdown is coded its own way"). Put it in
 * a `PopoverTrigger asChild`; the popover is a `PopoverContent align="start"` as wide as it.
 * A hand-made combobox button is refused by `check-lego`. It is a `combobox` to assistive
 * technology, as a Select's trigger is (Radix sets its `aria-expanded`).
 */
export function PickerTrigger({
  icon,
  children,
  className,
  ...props
}: React.ComponentProps<'button'> & { icon?: React.ReactNode }) {
  return (
    <button type="button" role="combobox" data-slot="picker-trigger" data-size="default" className={cn(triggerClass, 'w-full', className)} {...props}>
      <span className="flex min-w-0 flex-1 items-center gap-1.5 text-left">
        {icon}
        <span className="min-w-0 flex-1 truncate">{children}</span>
      </span>
      <ChevronDownIcon className="pointer-events-none size-4 text-muted-foreground" />
    </button>
  )
}
