// SPDX-License-Identifier: GPL-3.0-only
import { Archive, ChevronLeft, Images, PencilLine, ScanSearch, ShoppingBasket, Trash } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint, Stack } from '@/components/forms/field'
import { ListRow } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SidePanel } from '@/components/layout/side-panel'
import { Button } from '@/components/ui/button'
import type { PhotoId, SortPick, TripFacet } from '@/ipc/bindings'
import { TripPage } from './trip-page'
import { useTripName } from './pick-label'
import { PanelFooter } from '@/features/about/panel-footer'
import { CleanupCard } from './cleanup-card'
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
  /** What the actions take: the photos ticked in select mode, else those the pick shows. */
  shown: PhotoId[]
  /** Photos are ticked in select mode: the actions show even with no pick. */
  ticked: boolean
  /** To Photos with the pick kept, to colour, print or export it (ADR-0057). */
  onEdit(): void
  /** Photos' star or flag filter is on and narrows RollSense too: named here, with its clear
   * (two UX reviews, 2026-11-18: a filter is never hidden). */
  onClearFilter?: () => void
}

/**
 * RollSense's side panel (FR-141): the facets as cards, and under them the pass's state and
 * what a pick leads to (owner 2026-11-18, ADR-0057): edited in Photos with the pick kept, into
 * the print basket, set aside, or deleted.
 */
export function RollSensePanel({ sense, picks, onPicks, shown, ticked, onEdit, onClearFilter }: RollSensePanelProps) {
  const { t } = useTranslation()
  const { facets } = sense
  // A trip's page in place of the cards (S7c): its days, a split, a merge.
  const [tripOpen, setTripOpen] = useState<TripFacet | null>(null)
  const tripName = useTripName()
  const trips = facets?.trips ?? []
  const at = tripOpen ? trips.findIndex((x) => x.firstDay === tripOpen.firstDay) : -1
  const props = facets ? { facets, picks, onPicks } : null
  const count = shown.length

  return (
    // RollSense's column: violet on its small solid things, from `.theme-sense`'s tokens.
    <div className="contents theme-sense">
    <SidePanel
      title={
        tripOpen ? (
          <span className="flex items-center gap-2">
            <Button variant="outline" size="icon-sm" aria-label={t('rollsense.trip.back')} data-hint={t('rollsense.trip.back')} onClick={() => setTripOpen(null)}>
              <ChevronLeft />
            </Button>
            {tripName(tripOpen)}
          </span>
        ) : (
          // The column's name beside RollSense's violet key, the mark each card repeats.
          <span className="flex items-center gap-2">
            <span data-slot="card-icon">
              <ScanSearch />
            </span>
            {t('rollsense.title')}
          </span>
        )
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
            {(picks.length > 0 || ticked) && count > 0 && (
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
                {/* Marking can be undone, as setting aside: neutral; red is only the move to the Trash. */}
                <Button variant="secondary" onClick={() => void sense.markDelete(shown)}>
                  <Trash data-icon="inline-start" /> {t('rollsense.delete', { count })}
                </Button>
              </>
            )}
          </Stack>
          <PanelFooter />
        </>
      }
    >
      {sense.error && <PanelNote danger>{sense.error}</PanelNote>}
      {tripOpen && at >= 0 ? (
        <TripPage key={tripOpen.firstDay} trip={trips[at] ?? tripOpen} before={trips[at - 1]} after={trips[at + 1]} onDone={() => setTripOpen(null)} />
      ) : (
      <>
      {!facets || (facets.analysed === 0 && sense.running) ? <PanelNote>{t('rollsense.reading')}</PanelNote> : null}
      {onClearFilter && (
        <PanelNote action={<RemoveButton kind="clear" label={t('rollsense.clearFilter')} onClick={onClearFilter} />}>
          {t('rollsense.alsoFiltered')}
        </PanelNote>
      )}
      <Hint>{t('rollsense.hint')}</Hint>
      <ListRow icon={<Images />} chosen={picks.length === 0} onClick={() => onPicks([])}>
        {t('rollsense.all')}
      </ListRow>
      {props && (
        <>
          <CleanupCard rows={props.facets.cleanup} picks={picks} onPicks={onPicks} />
          <SourcesCard {...props} />
          <SubjectsCard {...props} />
          <TimeCard {...props} />
          <TripsCard {...props} onOpenTrip={setTripOpen} />
          <VisitsCard {...props} />
          <WhereCard {...props} folders={sense.folders} />
          <DuplicatesCard {...props} />
        </>
      )}
      </>
      )}
    </SidePanel>
    </div>
  )
}
