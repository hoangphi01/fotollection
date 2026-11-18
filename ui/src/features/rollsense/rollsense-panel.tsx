// SPDX-License-Identifier: GPL-3.0-only
import { Archive, Images, ScanSearch } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint, Stack } from '@/components/forms/field'
import { ListRow } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SidePanel } from '@/components/layout/side-panel'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
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
  pick: SortPick | null
  onPick(pick: SortPick | null): void
  /** The photos the pick shows in the grid, what the two actions take. */
  shown: PhotoId[]
}

/**
 * RollSense's side panel (FR-141, owner 2026-11-18: "sort, then delete or set aside; back in
 * Photos only the kept ones remain for colour and print"): the facets as cards, and under
 * them the pass's state and the two ends of a sort for the photos the pick shows.
 */
export function RollSensePanel({ sense, pick, onPick, shown }: RollSensePanelProps) {
  const { t } = useTranslation()
  const { facets } = sense
  const [asking, setAsking] = useState<PhotoId[] | null>(null)
  const [forGood, setForGood] = useState<PhotoId[] | null>(null)
  const props = facets ? { facets, pick, onPick } : null
  const count = shown.length

  const trash = async (ids: PhotoId[]) => {
    setAsking(null)
    const report = await sense.remove(ids, false)
    if (report && report.notRemoved.length > 0) setForGood(report.notRemoved)
    onPick(null)
  }
  const deleteForGood = async (ids: PhotoId[]) => {
    setForGood(null)
    await sense.remove(ids, true)
  }

  return (
    <SidePanel
      title={t('rollsense.title')}
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
            {pick && count > 0 && (
              <>
                <Button variant="secondary" onClick={() => void sense.setAside(shown)}>
                  <Archive data-icon="inline-start" /> {t('rollsense.setAside', { count })}
                </Button>
                <RemoveButton kind="delete" wide label={t('rollsense.delete', { count })} onClick={() => setAsking(shown)} />
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
      <ListRow icon={<Images />} chosen={pick === null} onClick={() => onPick(null)}>
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
      <AlertDialog open={asking !== null} onOpenChange={(open) => !open && setAsking(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('rollsense.deleteTitle', { count: asking?.length ?? 0 })}</AlertDialogTitle>
            <AlertDialogDescription>{t('rollsense.deleteBody')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <RemoveButton kind="delete" label={t('rollsense.deleteConfirm')} onClick={() => asking && void trash(asking)} />
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <AlertDialog open={forGood !== null} onOpenChange={(open) => !open && setForGood(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('rollsense.forGoodTitle', { count: forGood?.length ?? 0 })}</AlertDialogTitle>
            <AlertDialogDescription>{t('rollsense.forGoodBody')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('rollsense.keep')}</AlertDialogCancel>
            <RemoveButton kind="delete" label={t('rollsense.forGoodConfirm')} onClick={() => forGood && void deleteForGood(forGood)} />
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SidePanel>
  )
}
