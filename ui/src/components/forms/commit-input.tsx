// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { Input } from '@/components/ui/input'

type CommitInputProps = {
  value: string
  placeholder: string
  numeric?: boolean
  maxLength?: number
  ariaLabel?: string
  onCommit(value: string): void
}

/**
 * A text field that commits on blur or Enter, so typing doesn't send IPC per keystroke.
 * Remount it (via `key`) when the stored value changes.
 */
export function CommitInput({
  value,
  placeholder,
  numeric,
  maxLength,
  ariaLabel,
  onCommit,
}: CommitInputProps) {
  const [draft, setDraft] = useState(value)
  return (
    <Input
      value={draft}
      placeholder={placeholder}
      maxLength={maxLength}
      aria-label={ariaLabel}
      inputMode={numeric ? 'decimal' : undefined}
      className="h-8 w-full min-w-0 text-sm"
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => draft !== value && onCommit(draft)}
      onKeyDown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
    />
  )
}
