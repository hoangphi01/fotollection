// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState } from 'react'
import { Input } from '@/components/ui/input'

type CommitInputProps = {
  value: string
  placeholder: string
  numeric?: boolean
  maxLength?: number
  ariaLabel?: string
  /** Also commit while typing, this long after the last key (live preview). */
  liveMs?: number
  onCommit(value: string): void
}

/**
 * A text field that commits on blur or Enter, so typing doesn't send IPC per keystroke.
 * Follows the stored value when it changes elsewhere.
 */
export function CommitInput({
  value,
  placeholder,
  numeric,
  maxLength,
  ariaLabel,
  liveMs,
  onCommit,
}: CommitInputProps) {
  const [draft, setDraft] = useState(value)
  const committed = useRef(value)
  const commit = (next: string) => {
    if (next !== committed.current) {
      committed.current = next
      onCommit(next)
    }
  }
  // A change from elsewhere (undo, another photo) replaces the draft; our own commits
  // already match, so typing is never interrupted.
  useEffect(() => {
    if (value !== committed.current) {
      committed.current = value
      setDraft(value)
    }
  }, [value])
  // Live mode: commit once typing pauses, so the preview follows the text.
  useEffect(() => {
    if (liveMs === undefined || draft === committed.current) return
    const timer = setTimeout(() => commit(draft), liveMs)
    return () => clearTimeout(timer)
  })
  return (
    <Input
      value={draft}
      placeholder={placeholder}
      maxLength={maxLength}
      aria-label={ariaLabel}
      inputMode={numeric ? 'decimal' : undefined}
      className="h-8 w-full min-w-0 text-sm"
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => commit(draft)}
      onKeyDown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
    />
  )
}
