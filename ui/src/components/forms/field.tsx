// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'

/** A field's label: one style in every panel (STYLE_RULE §4). */
export function FieldLabel({ htmlFor, children }: { htmlFor?: string; children: ReactNode }) {
  return (
    <Label htmlFor={htmlFor} className="text-xs font-normal text-muted-foreground">
      {children}
    </Label>
  )
}

/** The line that explains a control, under it. */
export function Hint({ children, className }: { children: ReactNode; className?: string }) {
  return <p className={cn('text-xs text-muted-foreground', className)}>{children}</p>
}

type FieldProps = {
  label?: ReactNode
  /** Ties the label to its control. */
  htmlFor?: string
  hint?: ReactNode
  children?: ReactNode
}

/**
 * One field of a card: its label, its control and the line that explains it, 6 px apart; the
 * card spaces the fields 12 px apart (STYLE_RULE §4, the lego rule). A field owns no margin.
 */
export function Field({ label, htmlFor, hint, children }: FieldProps) {
  return (
    <div className="space-y-1.5 [--stack-gap:--spacing(1.5)]">
      {label && <FieldLabel htmlFor={htmlFor}>{label}</FieldLabel>}
      {children}
      {hint && <Hint>{hint}</Hint>}
    </div>
  )
}

/** A small form inside a card (a custom size being added): its own tinted box, fields 12 px apart. */
export function Inset({ children }: { children: ReactNode }) {
  return <div className="space-y-3 rounded-lg border bg-muted/30 p-3 [--stack-gap:--spacing(3)]">{children}</div>
}

/** A fact and its value on one line, the value at the right (a print's DPI, how it is cropped). */
export function ValueRow({ label, icon, children }: { label: ReactNode; icon?: ReactNode; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-2">
      <span className="flex items-center gap-1.5 text-xs text-muted-foreground [&_svg]:size-3.5">
        {icon}
        {label}
      </span>
      <span className="flex items-center gap-1.5 text-xs tabular-nums">{children}</span>
    </div>
  )
}

/** Rows that each hold a control (a camera's clock, a place's name, an address), 12 px apart as
 * every row of controls (STYLE_RULE §4). */
export function RowList({ children }: { children: ReactNode }) {
  return <ul className="space-y-3">{children}</ul>
}

/** Lines of facts about a photo, 4 px apart, as an info card shows them. */
export function FactLines({ children }: { children: ReactNode }) {
  return <div className="space-y-1">{children}</div>
}

/** Blocks spaced as a card spaces them, 12 px, where no card holds them (a panel's footer). */
export function Stack({ children }: { children: ReactNode }) {
  return <div className="space-y-3 [--stack-gap:--spacing(3)]">{children}</div>
}

/**
 * Text a user may select and copy in an app that otherwise selects none (owner, 2026-11-08):
 * a photo's facts, an error's `ERR-` code. A right click on it keeps the system's Copy.
 */
export function Selectable({ children, className, block = false }: { children: ReactNode; className?: string; block?: boolean }) {
  return block ? <div className={cn('selectable', className)}>{children}</div> : <span className={cn('selectable', className)}>{children}</span>
}
