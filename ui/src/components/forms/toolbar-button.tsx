// SPDX-License-Identifier: GPL-3.0-only
import { Spinner } from '@/components/forms/spinner'
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
  /** The label shows only from a wider row (`5xl`, 64 rem): a crowded toolbar, as the
   * develop view's with copy, paste and reset (owner, 2026-11-07; MEASURE at 1180 px). */
  labelFrom?: '3xl' | '5xl'
}

/**
 * A toolbar's button (STYLE_RULE §4): the icon alone in a narrow toolbar, as a 32 px square
 * with its 16 px icon in the centre, and the icon with its label from the `@3xl` container
 * width. The padding follows what is shown, so the icon is never off centre when the label
 * hides. Its hint sits on a wrapper: a disabled button gets no pointer events, and its hint
 * (undo's key, why Paste is off) must still show.
 */
export function ToolbarButton({
  icon,
  label,
  busy = false,
  iconOnly = false,
  labelFrom = '3xl',
  variant = 'outline',
  className,
  disabled,
  'data-hint': hint,
  ...props
}: ToolbarButtonProps & { 'data-hint'?: string }) {
  return (
    <span data-hint={hint} className="inline-flex shrink-0">
      <Button
        variant={variant}
        size="sm"
        aria-label={label}
        aria-busy={busy || undefined}
        disabled={disabled || busy}
        className={cn(
          'w-8 px-0',
          !iconOnly && (labelFrom === '5xl' ? '@5xl:w-auto @5xl:pr-2.5 @5xl:pl-1.5' : '@3xl:w-auto @3xl:pr-2.5 @3xl:pl-1.5'),
          className,
        )}
        {...props}
      >
        {busy ? <Spinner /> : icon}
        {!iconOnly && <span className={labelFrom === '5xl' ? 'hidden @5xl:inline' : 'hidden @3xl:inline'}>{label}</span>}
      </Button>
    </span>
  )
}
