// SPDX-License-Identifier: GPL-3.0-only
import { FilterX, RotateCcw, Trash2, X, type LucideIcon } from 'lucide-react'
import type { MouseEventHandler } from 'react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

/** What the button undoes, which picks its icon: one meaning, one look (STYLE_RULE §4 "Red"). */
const ICONS = {
  /** Takes something out of a list (a folder tab, the logo, a remembered text). */
  remove: X,
  /** Deletes something for good (a font, a print size, the cache, a camera session). */
  delete: Trash2,
  /** Clears what was chosen (the filters, the selection). */
  clear: FilterX,
  /** Stops what is running, or leaves an edit without keeping it (an export, the crop). */
  stop: X,
  /** Brings a default back (a card's sliders, a camera clock, a place's name). */
  reset: RotateCcw,
} satisfies Record<string, LucideIcon>

type DangerButtonProps = {
  kind: keyof typeof ICONS
  /** Its words: shown, or the hint and the screen reader's name when `iconOnly`. */
  label: string
  /** A longer hint than the label, when it needs one. */
  hint?: string
  /** Only the icon, 32 px square, in a row of controls. */
  iconOnly?: boolean
  /** 24 px, inside a line or a row's value; with `round`, a 20 px circle (a folder tab's ✕). */
  small?: boolean
  round?: boolean
  /** The width of its column (a panel's footer). */
  wide?: boolean
  disabled?: boolean
  onClick: MouseEventHandler<HTMLButtonElement>
  /** Placement only (position, visibility); its look comes from here. */
  className?: string
}

/**
 * Every button that removes, deletes, clears, stops or resets (owner, 2026-11-06: "every
 * button reused, one class"): deep red, white ink, the burgundy lip (`destructive`), 32 px as
 * every control. Use `RemoveButton` or `ResetButton`; `check-lego` refuses a hand-made one.
 */
function DangerButton({ kind, label, hint, iconOnly, small, round, wide, disabled, onClick, className }: DangerButtonProps) {
  const Icon = ICONS[kind]
  const iconSize = iconOnly ? (small ? 'icon-xs' : 'icon-sm') : small ? 'xs' : 'sm'
  // The hint on a wrapper, so a disabled button still shows it (no pointer events on it).
  return (
    <span data-hint={iconOnly ? (hint ?? label) : hint} className={cn('inline-flex shrink-0', wide && 'w-full', className)}>
      <Button
        variant="destructive"
        size={iconSize}
        disabled={disabled}
        aria-label={iconOnly ? label : undefined}
        onClick={onClick}
        className={cn(wide && 'w-full', round && 'size-5 rounded-full [--lip:2px] [&_svg]:size-3.5')}
      >
        <Icon data-icon={iconOnly ? undefined : 'inline-start'} aria-hidden />
        {!iconOnly && label}
      </Button>
    </span>
  )
}

/** Removes, deletes, clears or stops: `kind` says which, and picks the icon. */
export function RemoveButton(props: Omit<DangerButtonProps, 'kind'> & { kind?: Exclude<keyof typeof ICONS, 'reset'> }) {
  return <DangerButton {...props} kind={props.kind ?? 'remove'} />
}

/** Brings a default back. */
export function ResetButton(props: Omit<DangerButtonProps, 'kind'>) {
  return <DangerButton {...props} kind="reset" />
}
