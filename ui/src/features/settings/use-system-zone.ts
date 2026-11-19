// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands } from '@/ipc/bindings'

/** The zone the system reports, asked once: a suggestion, never an anchor (ADR-0058). */
let guessed: Promise<string | null> | null = null
export function useSystemZone() {
  const [zone, setZone] = useState<string | null>(null)
  useEffect(() => {
    guessed ??= commands.homeZoneGuess()
    let current = true
    void guessed.then((z) => current && setZone(z))
    return () => {
      current = false
    }
  }, [])
  return zone
}
