// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands, type CropGeometry, type PhotoId, type SheetPair } from '@/ipc/bindings'

/** The crop editor's geometry for a photo of a sheet, while it is open (FR-035, FR-006). */
export function useCellGeometry(photo: PhotoId | null, sheet: SheetPair | undefined, revision: number) {
  const [geometry, setGeometry] = useState<{ key: string; value: CropGeometry } | null>(null)
  const first = sheet?.first
  const second = sheet?.second ?? null
  const key = `${photo}:${first}:${second}:${revision}`
  useEffect(() => {
    if (photo === null || first === undefined) return
    let live = true
    void commands.getCropGeometry(photo, { first, second }).then((result) => {
      if (live && result.status === 'ok') setGeometry({ key, value: result.data })
    })
    return () => {
      live = false
    }
  }, [photo, first, second, key])
  return geometry?.key === key ? geometry.value : null
}
