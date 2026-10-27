// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, type Place } from '@/ipc/bindings'

/** How long typing pauses before the bundled places are searched. */
const DEBOUNCE_MS = 120
const RESULTS = 20

/**
 * Places whose name starts with `query`, largest first (FR-032). The search runs offline in
 * Rust over the bundled places; nothing goes online.
 */
export function usePlaceSearch(query: string) {
  const [places, setPlaces] = useState<Place[]>([])
  useEffect(() => {
    const text = query.trim()
    let current = true
    const timer = setTimeout(() => {
      if (text === '') {
        setPlaces([])
        return
      }
      void commands.searchPlaces(text, RESULTS).then((r) => {
        if (current && r.status === 'ok') setPlaces(r.data)
      })
    }, DEBOUNCE_MS)
    return () => {
      current = false
      clearTimeout(timer)
    }
  }, [query])
  return places
}
