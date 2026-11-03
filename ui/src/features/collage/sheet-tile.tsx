// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle, RotateCw } from 'lucide-react'
import type { PointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { PhotoId, SheetPair } from '@/ipc/bindings'
import { sheetUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'
import { BasketBadge } from '@/features/library/basket-badge'
import { TickCircle } from '@/features/library/tick-circle'
import { SheetCellsOverlay } from './sheet-cells-overlay'
import { useSheetShape } from './use-sheet-shape'

type SheetTileProps = {
  sheet: SheetPair
  revision: number
  size: number
  selected: boolean
  /** The cell the keyboard and the panel work on, on the selected sheet. */
  selectedCell: 0 | 1
  /** The photo being dragged, and the cell under the pointer (`null` for an empty one). */
  dragging: PhotoId | null
  over: PhotoId | null | undefined
  /** Select mode (FR-050): a click ticks the whole sheet, both photos; no dragging. */
  selectMode: boolean
  checked: ReadonlySet<number>
  /** Photos in the print basket, badged like the gallery's. */
  inBasket: (photo: PhotoId) => boolean
  onSelect: (cell: 0 | 1) => void
  onToggle: () => void
  onOpen: (cell: 0 | 1) => void
  /** Absent when the sheet does not turn (a postcard, FR-073). */
  onTurn?: () => void
  /** The sheet is a postcard (FR-073): its photo half is the left or bottom one. */
  postcard?: boolean
  onDragStart: (photo: PhotoId, event: PointerEvent) => void
}

/**
 * One collage sheet as it will print (FR-035), with its two cells on top: click picks a
 * cell, double-click opens the sheet's preview, and a cell dragged onto another swaps the
 * photos.
 */
export function SheetTile({
  sheet,
  revision,
  size,
  selected,
  selectedCell,
  dragging,
  over,
  selectMode,
  checked,
  inBasket,
  onSelect,
  onToggle,
  onOpen,
  onTurn,
  postcard = false,
  onDragStart,
}: SheetTileProps) {
  const { t } = useTranslation()
  const src = sheetUrl(sheet.first, sheet.second, revision, true)
  const shape = useSheetShape(src)
  const ticked = checked.has(sheet.first) && (sheet.second === null || checked.has(sheet.second))

  return (
    <div
      className={cn(
        'group relative flex shrink-0 items-center justify-center rounded-md bg-muted/40 p-2',
        // Framed like the gallery's selected photo (thumb-tile.tsx). In select mode the one
        // frame says the sheet is ticked: no frame per photo, no chosen cell.
        (selectMode ? ticked : selected) && 'ring-[3px] ring-primary ring-offset-2 ring-offset-background',
      )}
      style={{ width: size, height: size }}
    >
      {/* Centred in the tile, not in the image, which has no size until it loads (a sheet
          made new by a reorder). */}
      {!shape.loaded && (
        <LoaderCircle className="absolute inset-0 m-auto size-5 animate-spin text-muted-foreground" />
      )}
      <div className="relative">
        <img
          src={src}
          alt={t('collage.sheetLabel')}
          draggable={false}
          onLoad={shape.onLoad}
          className="block rounded-sm object-contain shadow-md"
          style={{ maxWidth: size - 16, maxHeight: size - 16 }}
        />
        <SheetCellsOverlay sheet={sheet} sideBySide={shape.sideBySide} postcard={postcard}>
          {(photo, cell) => (
            <div
              key={cell}
              data-cell
              data-photo={photo ?? undefined}
              className={cn(
                'relative flex-1 rounded-sm transition-colors',
                selectMode ? 'cursor-pointer' : 'cursor-grab active:cursor-grabbing',
                !selectMode && selected && selectedCell === cell && 'ring-2 ring-primary ring-inset',
                dragging !== null && over !== undefined && over === photo && dragging !== photo && 'bg-primary/25',
                dragging === photo && photo !== null && 'outline-2 -outline-offset-2 outline-primary outline-dashed',
              )}
              onPointerDown={(e) => {
                onSelect(cell)
                if (selectMode) onToggle()
                else if (photo !== null) onDragStart(photo, e)
              }}
              onDoubleClick={() => photo !== null && !selectMode && onOpen(cell)}
            >
              {photo !== null && inBasket(photo) && <BasketBadge className="absolute top-1 right-1" />}
            </div>
          )}
        </SheetCellsOverlay>
        {selectMode && <TickCircle checked={ticked} className="pointer-events-none absolute right-1.5 bottom-1.5" />}
      </div>
      {!selectMode && onTurn && (
        <Button
          variant="secondary"
          size="icon-xs"
          className={cn('absolute top-1.5 right-1.5 opacity-0 group-hover:opacity-100', selected && 'opacity-100')}
          aria-label={t('collage.turn')}
          data-hint={t('collage.turnHint')}
          onClick={onTurn}
        >
          <RotateCw />
        </Button>
      )}
    </div>
  )
}
