// SPDX-License-Identifier: GPL-3.0-only
import { Archive, CheckCheck, MapPin, ShoppingBasket, Undo2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { Queue, QueueView } from '@/ipc/bindings'

type SelectionBarProps = {
  count: number
  /** Ticked photos in folder tabs not shown now (FR-056): they are moved too. */
  elsewhere: number
  /** The view the photos are ticked in: the actions move them out of it. */
  view: QueueView
  onMove: (queue: Queue | null) => void
  /** Set the date and place of the ticked photos by hand (FR-032). */
  onDatePlace: () => void
  onSelectAll: () => void
  onClear: () => void
}

/**
 * Select mode's actions (FR-050), like the Select button on an iPhone: put the ticked photos
 * in the print basket, set them aside, or bring them back to the gallery, and set their date
 * and place by hand (FR-032). Each action is undoable; nothing is ever deleted.
 */
export function SelectionBar({ count, elsewhere, view, onMove, onDatePlace, onSelectAll, onClear }: SelectionBarProps) {
  const { t } = useTranslation()
  const none = count === 0
  return (
    <div className="flex min-h-[45px] flex-wrap items-center gap-2 border-b bg-primary/5 px-4 py-1.5 text-sm">
      <span className="font-medium tabular-nums">{t('select.count', { count })}</span>
      {elsewhere > 0 && (
        <span className="text-xs text-muted-foreground tabular-nums">{t('select.elsewhere', { count: elsewhere })}</span>
      )}
      <div className="ml-auto flex flex-wrap gap-1.5">
        {view !== 'print' && (
          <Button size="sm" disabled={none} onClick={() => onMove('print')}>
            <ShoppingBasket data-icon="inline-start" /> {t('select.toBasket')}
          </Button>
        )}
        {view !== 'aside' && (
          <Button size="sm" variant="outline" disabled={none} onClick={() => onMove('aside')}>
            <Archive data-icon="inline-start" /> {t('select.toAside')}
          </Button>
        )}
        {view !== 'gallery' && (
          <Button size="sm" variant="outline" disabled={none} onClick={() => onMove(null)}>
            <Undo2 data-icon="inline-start" /> {t('select.toGallery')}
          </Button>
        )}
        <Button size="sm" variant="outline" disabled={none} onClick={onDatePlace}>
          <MapPin data-icon="inline-start" /> {t('places.set')}
        </Button>
        <Button size="sm" variant="ghost" onClick={onSelectAll}>
          <CheckCheck data-icon="inline-start" /> {t('select.all')}
        </Button>
        <Button size="sm" variant="ghost" disabled={none} onClick={onClear}>
          <X data-icon="inline-start" /> {t('select.clear')}
        </Button>
      </div>
    </div>
  )
}
