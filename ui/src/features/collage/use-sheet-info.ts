// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, type SheetInfo, type SheetPair } from '@/ipc/bindings'

/** A sheet's size, each photo's area, the suggested size and its quality (FR-035, FR-026). */
export function useSheetInfo(sheet: SheetPair | undefined, revision: number): SheetInfo | null {
  const [info, setInfo] = useState<{ key: string; value: SheetInfo } | null>(null)
  const first = sheet?.first
  const second = sheet?.second ?? null
  const key = `${first}:${second}:${revision}`
  useEffect(() => {
    if (first === undefined) return
    let live = true
    void commands.sheetInfo({ first, second }).then((result) => {
      if (live && result.status === 'ok') setInfo({ key, value: result.data })
    })
    return () => {
      live = false
    }
  }, [first, second, key])
  return info?.key === key ? info.value : null
}
