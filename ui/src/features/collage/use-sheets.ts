// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import { commands, events, type CullFilter, type PhotoId, type SheetPair } from '@/ipc/bindings'
import { sheetOrder } from './sheet-cells'

/**
 * The collage sheets of the photos shown, in order (FR-035), refreshed after every edit (and
 * when postcards are switched, which Rust announces as a library change, FR-073), and
 * the way to reorder them (only listed while `enabled`, in Collage mode): `move` drops a photo on another photo's cell, or on an empty one.
 */
export function useSheets(filter: CullFilter, enabled: boolean) {
  const [sheets, setSheets] = useState<SheetPair[]>([])
  const key = JSON.stringify(filter)
  useEffect(() => {
    if (!enabled) return
    let live = true
    const load = () =>
      void commands.listSheets(JSON.parse(key) as CullFilter).then((result) => {
        if (live && result.status === 'ok') setSheets(result.data)
      })
    load()
    const unlisten = events.libraryChanged.listen(load)
    return () => {
      live = false
      void unlisten.then((off) => off())
    }
  }, [key, enabled])

  const move = useCallback(
    (photo: PhotoId, target: PhotoId | null) => {
      const order = sheetOrder(sheets).map((c) => c.photo)
      const from = order.indexOf(photo)
      if (from < 0 || photo === target) return
      if (target === null) {
        // An empty cell is the last one: the photo goes to the end.
        order.splice(from, 1)
        order.push(photo)
      } else {
        const to = order.indexOf(target)
        if (to < 0) return
        order[from] = target
        order[to] = photo
      }
      void commands.setCollageOrder(order)
    },
    [sheets],
  )

  const turn = useCallback((first: PhotoId) => void commands.turnSheet(first), [])

  return { sheets, move, turn }
}
