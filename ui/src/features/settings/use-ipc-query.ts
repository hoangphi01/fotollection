// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'

/** Runs an IPC call once on mount. Returns `undefined` until it resolves or if it fails. */
export function useIpcQuery<T>(call: () => Promise<T>): T | undefined {
  const [value, setValue] = useState<T>()

  useEffect(() => {
    let live = true
    call()
      .then((result) => {
        if (live) setValue(result)
      })
      .catch((error: unknown) => console.warn('IPC call failed', error))
    return () => {
      live = false
    }
  }, [call])

  return value
}

type Answer<T> = { subject: string; refresh: unknown; value: T }

/**
 * Asks Rust about a subject (a photo, a sheet) again whenever `refresh` changes (a setting,
 * an edit), and keeps the last answer on screen while the next one loads, so nothing
 * vanishes and nothing below it jumps (LESSONS). `data` is the last answer, even for another
 * subject, for text and notes that only need to hold their place; `current` is the answer
 * for this subject, for what must not show another photo's numbers (a crop editor);
 * `stale` says a newer answer is on its way.
 *
 * `subject` is plain text (JSON for several values) and `load` must be stable (defined
 * outside the component), so a caller that rebuilds its arguments never loops.
 */
export function useLiveQuery<T>(
  subject: string | null,
  refresh: unknown,
  load: (subject: string) => Promise<T | null>,
): { data: T | null; current: T | null; stale: boolean } {
  const [answer, setAnswer] = useState<Answer<T> | null>(null)
  useEffect(() => {
    if (subject === null) return
    let live = true
    void load(subject).then((value) => {
      if (live && value !== null) setAnswer({ subject, refresh, value })
    })
    return () => {
      live = false
    }
  }, [subject, refresh, load])
  const same = answer !== null && answer.subject === subject
  return {
    // No subject (no photo open): nothing to show, not the last one's answer.
    data: subject === null ? null : (answer?.value ?? null),
    current: same ? answer.value : null,
    stale: !same || answer.refresh !== refresh,
  }
}
