// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle, RotateCw } from 'lucide-react'
import { useState, type PointerEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { PhotoId, SheetPair } from '@/ipc/bindings'
import { sheetUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

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
  onSelect: (cell: 0 | 1) => void
  onOpen: (cell: 0 | 1) => void
  onTurn: () => void
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
  onSelect,
  onOpen,
  onTurn,
  onDragStart,
}: SheetTileProps) {
  const { t } = useTranslation()
  const src = sheetUrl(sheet.first, sheet.second, revision, true)
  const [loaded, setLoaded] = useState<{ src: string; sideBySide: boolean } | null>(null)
  // The sheet's shape tells how its cells sit: side by side on a landscape sheet.
  const sideBySide = loaded?.sideBySide ?? true
  const cells: [PhotoId | null, 0 | 1][] = [
    [sheet.first, 0],
    [sheet.second, 1],
  ]

  return (
    <div
      className={cn(
        'group relative flex shrink-0 items-center justify-center rounded-md bg-muted/40 p-2',
        selected && 'ring-2 ring-primary',
      )}
      style={{ width: size, height: size }}
    >
      <div className="relative">
        <img
          src={src}
          alt={t('collage.sheetLabel')}
          draggable={false}
          onLoad={(e) =>
            setLoaded({ src, sideBySide: e.currentTarget.naturalWidth >= e.currentTarget.naturalHeight })
          }
          className="block rounded-sm object-contain shadow-md"
          style={{ maxWidth: size - 16, maxHeight: size - 16 }}
        />
        {loaded?.src !== src && (
          <LoaderCircle className="absolute inset-0 m-auto size-5 animate-spin text-muted-foreground" />
        )}
        <div className={cn('absolute inset-0 flex', !sideBySide && 'flex-col')}>
          {cells.map(([photo, cell]) => (
            <div
              key={cell}
              data-cell
              data-photo={photo ?? undefined}
              className={cn(
                'flex-1 cursor-grab rounded-sm transition-colors active:cursor-grabbing',
                selected && selectedCell === cell && 'ring-2 ring-primary ring-inset',
                dragging !== null && over !== undefined && over === photo && dragging !== photo && 'bg-primary/25',
                dragging === photo && photo !== null && 'outline-2 -outline-offset-2 outline-primary outline-dashed',
              )}
              onPointerDown={(e) => {
                onSelect(cell)
                if (photo !== null) onDragStart(photo, e)
              }}
              onDoubleClick={() => photo !== null && onOpen(cell)}
            />
          ))}
        </div>
      </div>
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
    </div>
  )
}
