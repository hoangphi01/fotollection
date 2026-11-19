// SPDX-License-Identifier: GPL-3.0-only
import { Archive, CheckCheck, PencilLine, ScanSearch, ShoppingBasket, Trash } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ButtonGroup } from '@/components/forms/button-group'
import { RemoveButton } from '@/components/forms/danger-button'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { SubToolbar } from '@/components/layout/sub-toolbar'

type SenseBarProps = {
  /** The photos the actions take: those ticked in select mode, else those the pick shows. */
  count: number
  /** A pick made or photos ticked: the actions are live; else they wait, disabled. */
  live: boolean
  /** Photos are ticked in select mode: the count names them as ticked. */
  ticked: boolean
  /** Photos of the scope not analysed, and no pass running: *Analyse photos* is offered. */
  onAnalyze?: () => void
  onEdit(): void
  onBasket(): void
  onAside(): void
  onDelete(): void
  /** Select mode is on: *Select all* and the red clear. */
  select?: { onSelectAll(): void; onClear(): void }
}

/**
 * RollSense's bar under the gallery's toolbar (FR-141, owner 2026-11-19: "a second toolbar
 * under the main one"), always there while RollSense is open, so ticking a group never moves
 * the column or the grid: what the pick leads to (ADR-0057), edited in Photos, into the print
 * basket, set aside or marked to delete, disabled until something is picked. In select mode it
 * stands for the selection bar, its actions taking the ticked photos.
 */
export function SenseBar({ count, live, ticked, onAnalyze, onEdit, onBasket, onAside, onDelete, select }: SenseBarProps) {
  const { t } = useTranslation()
  const off = !live || count === 0
  return (
    <SubToolbar
      label={
        <span className="font-medium whitespace-nowrap tabular-nums">
          {ticked ? t('select.count', { count }) : live ? t('rollsense.picked', { count }) : t('rollsense.nonePicked')}
        </span>
      }
    >
      {onAnalyze && <ToolbarButton icon={<ScanSearch />} label={t('rollsense.analyze')} labelFrom="5xl" data-hint={t('rollsense.analyze')} onClick={onAnalyze} />}
      <ToolbarButton icon={<PencilLine />} label={t('rollsense.editInPhotos')} labelFrom="5xl" variant="default" data-hint={t('rollsense.editInPhotos')} disabled={off} onClick={onEdit} />
      <ButtonGroup ariaLabel={t('toolbar.move')}>
        <ToolbarButton icon={<ShoppingBasket />} label={t('select.toBasket')} labelFrom="5xl" data-hint={t('select.toBasket')} disabled={off} onClick={onBasket} />
        <ToolbarButton icon={<Archive />} label={t('select.toAside')} labelFrom="5xl" data-hint={t('select.toAside')} disabled={off} onClick={onAside} />
        <ToolbarButton icon={<Trash />} label={t('select.toDelete')} labelFrom="5xl" data-hint={t('select.toDelete')} disabled={off} onClick={onDelete} />
      </ButtonGroup>
      {select && (
        <>
          <ToolbarButton icon={<CheckCheck />} label={t('select.all')} labelFrom="5xl" data-hint={t('select.all')} onClick={select.onSelectAll} />
          <RemoveButton kind="clear" disabled={!ticked} label={t('select.clear')} onClick={select.onClear} />
        </>
      )}
    </SubToolbar>
  )
}
