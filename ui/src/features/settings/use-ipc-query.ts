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
