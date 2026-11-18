// SPDX-License-Identifier: GPL-3.0-only
import { Archive, Images, PencilLine, ScanSearch, ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint, Stack } from '@/components/forms/field'
import { ListRow } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SidePanel } from '@/components/layout/side-panel'
import { Button } from '@/components/ui/button'
import type { PhotoId, SortPick } from '@/ipc/bindings'
import { PanelFooter } from '@/features/about/panel-footer'
import {
  DuplicatesCard,
  SourcesCard,
  SubjectsCard,
  TimeCard,
  TripsCard,
  VisitsCard,
  WhereCard,
} from './rollsense-cards'
import type { RollSense } from './use-rollsense'

type RollSensePanelProps = {
  sense: RollSense
  picks: SortPick[]
  onPicks(picks: SortPick[]): void
  /** The photos the pick shows in the grid, what the actions take. */
  shown: PhotoId[]
  /** To Photos with the pick kept, to colour, print or export it (ADR-0057). */
  onEdit(): void
}

/**
 * RollSense's side panel (FR-141): the facets as cards, and under them the pass's state and
 * what a pick leads to (owner 2026-11-18, ADR-0057): edited in Photos with the pick kept, into
 * the print basket, set aside, or deleted.
 */
export function RollSensePanel({ sense, picks, onPicks, shown, onEdit }: RollSensePanelProps) {
  const { t } = useTranslation()
  const { facets } = sense
  const props = facets ? { facets, picks, onPicks } : null
  const count = shown.length

  return (
    // RollSense's column: violet on its small solid things, from `.theme-sense`'s tokens.
    <div className="contents theme-sense">
    <SidePanel
      title={
        // The column's name beside RollSense's violet key, the mark each card repeats.
        <span className="flex items-center gap-2">
          <span data-slot="card-icon">
            <ScanSearch />
          </span>
          {t('rollsense.title')}
        </span>
      }
      footer={
        <>
          <Stack>
            {facets && facets.analysed < facets.total && <Hint>{t('rollsense.read', { done: facets.analysed, total: facets.total })}</Hint>}
            {sense.running ? (
              <RemoveButton kind="stop" wide label={t('rollsense.stop')} onClick={sense.cancel} />
            ) : (
              facets &&
              facets.analysed < facets.total && (
                <Button variant="outline" onClick={() => void sense.analyze()}>
                  <ScanSearch data-icon="inline-start" /> {t('rollsense.analyze')}
                </Button>
              )
            )}
            {picks.length > 0 && count > 0 && (
              <>
                <Button onClick={onEdit}>
                  <PencilLine data-icon="inline-start" /> {t('rollsense.edit', { count })}
                </Button>
                <Button variant="secondary" onClick={() => void sense.toBasket(shown)}>
                  <ShoppingBasket data-icon="inline-start" /> {t('rollsense.toBasket', { count })}
                </Button>
                <Button variant="secondary" onClick={() => void sense.setAside(shown)}>
                  <Archive data-icon="inline-start" /> {t('rollsense.setAside', { count })}
                </Button>
                <RemoveButton kind="delete" wide label={t('rollsense.delete', { count })} onClick={() => void sense.markDelete(shown)} />
              </>
            )}
          </Stack>
          <PanelFooter />
        </>
      }
    >
      {sense.error && <PanelNote danger>{sense.error}</PanelNote>}
      {!facets || (facets.analysed === 0 && sense.running) ? <PanelNote>{t('rollsense.reading')}</PanelNote> : null}
      <Hint>{t('rollsense.hint')}</Hint>
      <ListRow icon={<Images />} chosen={picks.length === 0} onClick={() => onPicks([])}>
        {t('rollsense.all')}
      </ListRow>
      {props && (
        <>
          <SourcesCard {...props} />
          <SubjectsCard {...props} />
          <TimeCard {...props} />
          <TripsCard {...props} />
          <VisitsCard {...props} />
          <WhereCard {...props} />
          <DuplicatesCard {...props} />
        </>
      )}
    </SidePanel>
    </div>
  )
}
