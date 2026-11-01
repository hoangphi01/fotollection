// SPDX-License-Identifier: GPL-3.0-only
import type { PhotoId, SheetPair } from '@/ipc/bindings'

/** A sheet's two halves: the photo in each (`null` for an empty one) and which half it is. */
export function sheetCells(sheet: SheetPair): [PhotoId | null, 0 | 1][] {
  return [
    [sheet.first, 0],
    [sheet.second, 1],
  ]
}

/** The photos of the sheets in collage order, each with its sheet and half (FR-035). */
export function sheetOrder(sheets: SheetPair[]): { photo: PhotoId; sheet: number; cell: 0 | 1 }[] {
  return sheets.flatMap((s, sheet) =>
    sheetCells(s).flatMap(([photo, cell]) => (photo === null ? [] : [{ photo, sheet, cell }])),
  )
}
