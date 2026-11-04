// SPDX-License-Identifier: GPL-3.0-only
import { useId, type ReactNode } from 'react'
import { Collapse } from '@/components/layout/collapse'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Hint } from './field'

type SwitchFieldProps = {
  /** A Lucide icon; it is drawn at 16 px in the muted colour. */
  icon?: ReactNode
  label: ReactNode
  checked: boolean
  onChange: (checked: boolean) => void
  disabled?: boolean
  /** The switch's options, folded away while it is off. */
  children?: ReactNode
  /** Shows the options even while the switch is off (a collage sheet's paper colour). */
  alwaysOpen?: boolean
  hint?: ReactNode
}

/**
 * A switch with its label, then the options it turns on, 6 px under it and folded away while
 * it is off, so a switched-off feature leaves no gap (STYLE_RULE §4, the lego rule).
 */
export function SwitchField({ icon, label, checked, onChange, disabled, children, alwaysOpen, hint }: SwitchFieldProps) {
  const id = useId()
  const open = checked || !!alwaysOpen
  return (
    <div className="space-y-1.5 [--stack-gap:--spacing(1.5)]">
      <div className="flex items-center justify-between gap-2">
        <Label htmlFor={id} className="flex items-center gap-2 text-sm font-normal [&_svg]:size-4 [&_svg]:text-muted-foreground">
          {icon}
          {label}
        </Label>
        <Switch id={id} checked={checked} disabled={disabled} onCheckedChange={onChange} />
      </div>
      {children && (
        <Collapse open={open} stack="field">
          {children}
        </Collapse>
      )}
      {hint && <Hint>{hint}</Hint>}
    </div>
  )
}
