// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle } from 'lucide-react'
import type { ComponentProps, ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

type ToolbarButtonProps = Omit<ComponentProps<typeof Button>, 'children' | 'size'> & {
  icon: ReactNode
  label: string
  /** The work it started is running: a spinner in the icon's place, and no second click. */
  busy?: boolean
  /** Always the icon alone (undo, rotate): its label is for the hint and screen readers. */
  iconOnly?: boolean
}

/**
 * A toolbar's button (STYLE_RULE §4): the icon alone in a narrow toolbar, as a square with
 * the icon in its centre, and the icon with its label from the `@3xl` container width. The
 * padding follows what is shown, so the icon is never off centre when the label hides.
 */
export function ToolbarButton({
  icon,
  label,
  busy = false,
  iconOnly = false,
  variant = 'outline',
  className,
  disabled,
  ...props
}: ToolbarButtonProps) {
  return (
    <Button
      variant={variant}
      size="sm"
      aria-label={label}
      aria-busy={busy || undefined}
      disabled={disabled || busy}
      className={cn('w-7 px-0', !iconOnly && '@3xl:w-auto @3xl:pr-2.5 @3xl:pl-1.5', className)}
      {...props}
    >
      {busy ? <LoaderCircle className="animate-spin" /> : icon}
      {!iconOnly && <span className="hidden @3xl:inline">{label}</span>}
    </Button>
  )
}
