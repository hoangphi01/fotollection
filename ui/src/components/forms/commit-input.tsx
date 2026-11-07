// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState } from 'react'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'

type CommitInputProps = {
  value: string
  placeholder: string
  numeric?: boolean
  maxLength?: number
  ariaLabel?: string
  /** Also commit while typing, this long after the last key (live preview). */
  liveMs?: number
  /** Several lines (a postcard's text): Enter breaks the line, blur commits. */
  multiline?: boolean
  /** Keeps a draft within limits as it is typed (e.g. a line count). */
  limit?: (value: string) => string
  onCommit(value: string): void
  /** The field was left: typing is done (a postcard's address is remembered then, FR-077). */
  onDone?(value: string): void
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
  multiline,
  limit = (v) => v,
  onCommit,
  onDone,
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
  // already match. While the field has focus, typing wins: a late echo of an earlier commit
  // must never pull the text back (LESSONS).
  const focused = useRef(false)
  useEffect(() => {
    if (!focused.current && value !== committed.current) {
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
  const events = {
    onFocus: () => {
      focused.current = true
    },
    onBlur: () => {
      focused.current = false
      onDone?.(draft)
      if (draft !== committed.current) commit(draft)
      else if (value !== committed.current) {
        // Nothing typed, but the value changed meanwhile (an undo): show it now.
        committed.current = value
        setDraft(value)
      }
    },
  }
  if (multiline) {
    return (
      <Textarea
        value={draft}
        placeholder={placeholder}
        maxLength={maxLength}
        aria-label={ariaLabel}
        className="min-h-20 w-full min-w-0 text-sm"
        onChange={(e) => setDraft(limit(e.target.value))}
        {...events}
      />
    )
  }
  return (
    <Input
      value={draft}
      placeholder={placeholder}
      maxLength={maxLength}
      aria-label={ariaLabel}
      inputMode={numeric ? 'decimal' : undefined}
      className="h-8 w-full min-w-0 text-sm"
      onChange={(e) => setDraft(limit(e.target.value))}
      {...events}
      onKeyDown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
    />
  )
}
