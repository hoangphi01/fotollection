// SPDX-License-Identifier: GPL-3.0-only
import { Archive, CheckCheck, ClipboardPaste, MapPin, ShoppingBasket, Undo2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ButtonGroup } from '@/components/forms/button-group'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import type { Queue, QueueView } from '@/ipc/bindings'
import { RemoveButton } from '@/components/forms/danger-button'

type SelectionBarProps = {
  count: number
  /** What is counted: photos, or collage sheets (FR-035), each holding its photos. */
  unit?: 'photos' | 'sheets'
  /** Ticked photos in folder tabs not shown now (FR-056): they are moved too. */
  elsewhere: number
  /** The view the photos are ticked in: the actions move them out of it. */
  view: QueueView
  onMove: (queue: Queue | null) => void
  /** Set the date and place of the ticked photos by hand (FR-032). */
  onDatePlace: () => void
  /** Paste the copied develop settings to the ticked photos (FR-097); absent with nothing copied. */
  onPasteSettings?: () => void
  onSelectAll: () => void
  onClear: () => void
}

/**
 * Select mode's actions (FR-050), like the Select button on an iPhone: put the ticked photos
 * in the print basket, set them aside, or bring them back to the gallery, and set their date
 * and place by hand (FR-032). Each action is undoable; nothing is ever deleted.
 */
export function SelectionBar({ count, unit = 'photos', elsewhere, view, onMove, onDatePlace, onPasteSettings, onSelectAll, onClear }: SelectionBarProps) {
  const { t } = useTranslation()
  const none = count === 0
  // One line of a fixed height (STYLE_RULE §4: a toolbar never wraps, nothing jumps): the
  // actions are toolbar buttons that keep only their icons in a narrow window.
  return (
    <div className="@container flex h-12 shrink-0 items-center gap-2 overflow-x-auto overflow-y-hidden border-b bg-primary/5 px-4 text-sm [scrollbar-width:thin] [&>*]:shrink-0">
      <span className="font-medium whitespace-nowrap tabular-nums">{t(unit === 'sheets' ? 'select.countSheets' : 'select.count', { count })}</span>
      {elsewhere > 0 && (
        <span className="text-xs whitespace-nowrap text-muted-foreground tabular-nums">{t('select.elsewhere', { count: elsewhere })}</span>
      )}
      <div className="ml-auto flex items-center gap-2">
        {/* Where the ticked photos go, then what they get, each joined (owner, 2026-11-08);
            the red clear stands apart. */}
        <ButtonGroup ariaLabel={t('toolbar.move')}>
        {view !== 'print' && (
          <ToolbarButton icon={<ShoppingBasket />} label={t('select.toBasket')} labelFrom="5xl" variant="default" data-hint={t('select.toBasket')} disabled={none} onClick={() => onMove('print')} />
        )}
        {view !== 'aside' && (
          <ToolbarButton icon={<Archive />} label={t('select.toAside')} labelFrom="5xl" data-hint={t('select.toAside')} disabled={none} onClick={() => onMove('aside')} />
        )}
        {view !== 'gallery' && (
          <ToolbarButton icon={<Undo2 />} label={t('select.toGallery')} labelFrom="5xl" data-hint={t('select.toGallery')} disabled={none} onClick={() => onMove(null)} />
        )}
        </ButtonGroup>
        <ButtonGroup ariaLabel={t('toolbar.apply')}>
        <ToolbarButton icon={<MapPin />} label={t('places.set')} labelFrom="5xl" data-hint={t('places.set')} disabled={none} onClick={onDatePlace} />
        {onPasteSettings && (
          <ToolbarButton icon={<ClipboardPaste />} label={t('develop.pasteToTicked')} labelFrom="5xl" data-hint={t('develop.pasteToTicked')} disabled={none} onClick={onPasteSettings} />
        )}
        </ButtonGroup>
        <ToolbarButton icon={<CheckCheck />} label={t('select.all')} labelFrom="5xl" variant="default" data-hint={t('select.all')} onClick={onSelectAll} />
        <RemoveButton kind="clear" disabled={none} label={t('select.clear')} onClick={onClear} />
      </div>
    </div>
  )
}
