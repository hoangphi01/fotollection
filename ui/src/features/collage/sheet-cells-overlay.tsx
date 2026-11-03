// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import type { PhotoId, SheetPair } from '@/ipc/bindings'
import { cn } from '@/lib/utils'
import { sheetCells } from './sheet-cells'

type SheetCellsOverlayProps = {
  sheet: SheetPair
  sideBySide: boolean
  /** A postcard (FR-073): its photo is on the left, or below the card when it stands. */
  postcard?: boolean
  /** One half: its photo (`null` when empty) and which half it is. */
  children: (photo: PhotoId | null, cell: 0 | 1) => ReactNode
}

/**
 * The two halves of a sheet laid over its image (FR-035), the same in the sheet grid and the
 * sheet preview: what each half does is up to the caller.
 */
export function SheetCellsOverlay({ sheet, sideBySide, postcard = false, children }: SheetCellsOverlayProps) {
  return (
    <div className={cn('absolute inset-0 flex', !sideBySide && (postcard ? 'flex-col-reverse' : 'flex-col'))}>
      {sheetCells(sheet).map(([photo, cell]) => children(photo, cell))}
    </div>
  )
}
