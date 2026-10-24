// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, type PrintAdvice } from '@/ipc/bindings'

/**
 * How large the photo prints well, and a verdict per print size (FR-026). Asked again when
 * the photo or the custom sizes change; `null` while there is no photo or no answer for it.
 */
export function usePrintAdvice(photoId: number | undefined, sizesKey: string): PrintAdvice | null {
  const [answer, setAnswer] = useState<{ id: number; advice: PrintAdvice } | null>(null)
  useEffect(() => {
    if (photoId === undefined) return
    let live = true
    void commands.printAdvice(photoId).then((result) => {
      if (live && result.status === 'ok') setAnswer({ id: photoId, advice: result.data })
    })
    return () => {
      live = false
    }
  }, [photoId, sizesKey])
  return answer && answer.id === photoId ? answer.advice : null
}
