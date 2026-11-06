// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import type { CullFilter } from '@/ipc/bindings'
import { useSheetInfo } from './use-sheet-info'
import { useSheets } from './use-sheets'

/**
 * Collage mode's state (FR-035), shared by the sheet grid and the panel: the sheets, the
 * chosen sheet and cell, and what Rust says about them.
 */
export function useCollage(filter: CullFilter, revision: number, enabled: boolean) {
  const { sheets, move, turn } = useSheets(filter, enabled)
  const [chosen, setChosen] = useState<{ sheet: number; cell: 0 | 1 }>({ sheet: 0, cell: 0 })
  // Whether a photo is picked: a click on a cell picks it, a click beside the sheets lets go.
  // The colour cards edit the picked photo only, and are gone while none is (FR-079).
  const [picked, setPicked] = useState(false)
  // The list can shrink under the choice (a photo leaves the view): stay in range.
  const index = Math.max(0, Math.min(chosen.sheet, sheets.length - 1))
  const sheet = enabled ? sheets[index] : undefined
  const cellIndex: 0 | 1 = chosen.cell === 1 && sheet?.second != null ? 1 : 0
  const info = useSheetInfo(sheet, revision)
  const photo = sheet ? (cellIndex === 0 ? sheet.first : sheet.second) : null
  const cell = info?.cells.find((c) => c.id === photo) ?? info?.cells[0]
  return {
    sheets,
    index,
    cellIndex,
    sheet,
    cell,
    /** What Rust says about the chosen sheet: its cells and its pixels (FR-087). */
    info,
    picked,
    select: (sheet: number, cell: 0 | 1) => {
      setChosen({ sheet, cell })
      setPicked(true)
    },
    unpick: () => setPicked(false),
    move,
    turn,
  }
}

export type CollageApi = ReturnType<typeof useCollage>
