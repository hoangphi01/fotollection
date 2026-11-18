// SPDX-License-Identifier: GPL-3.0-only
import { ScanSearch } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Button } from '@/components/ui/button'
import type { Facets, SortPick } from '@/ipc/bindings'
import { usePickLabel } from './pick-label'

type PickChipProps = {
  picks: SortPick[]
  facets: Facets | null
  /** Back to RollSense, its ticks kept. */
  onOpen(): void
  onClear(): void
}

/**
 * RollSense's pick while Photos shows it (FR-141, ADR-0057, owner 2026-11-18): the grid, colour,
 * export and print follow the pick, so it is always named in the toolbar, with RollSense's
 * violet key, and cleared with one press. Never a filter left behind unseen.
 */
export function PickChip({ picks, facets, onOpen, onClear }: PickChipProps) {
  const { t } = useTranslation()
  const label = usePickLabel(facets)
  const [first] = picks
  if (!first) return null
  const words = picks.length > 1 ? `${label(first)} +${picks.length - 1}` : label(first)
  return (
    <span className="theme-sense inline-flex min-w-0 shrink items-center gap-1">
      <Button variant="outline" size="sm" className="min-w-0" onClick={onOpen} data-hint={t('rollsense.backToPick')}>
        <span data-slot="card-icon">
          <ScanSearch />
        </span>
        {words}
      </Button>
      <RemoveButton kind="clear" iconOnly small label={t('rollsense.clearPick')} onClick={onClear} />
    </span>
  )
}
