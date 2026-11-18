// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in RollSense's side panel, in place of its cards (STYLE_RULE §4).
import { CalendarDays, Merge, Scissors } from 'lucide-react'
import { Fragment, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, Stack } from '@/components/forms/field'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { Toggle } from '@/components/ui/toggle'
import { commands, type IpcError, type TripDay, type TripFacet } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { useDay, useTripName } from './pick-label'

type TripPageProps = {
  trip: TripFacet
  /** The trips just before and after it in time, to merge with; none at the ends. */
  before?: TripFacet
  after?: TripFacet
  /** Back to the cards, after a change or without one. */
  onDone(): void
}

/**
 * A trip's page (FR-012 S7c, owner 2026-11-18, option C): its name, its days with their photos
 * and places, a split chosen between two days, and a merge with the trip before or after.
 */
export function TripPage({ trip, before, after, onDone }: TripPageProps) {
  const { t } = useTranslation()
  const day = useDay()
  const name = useTripName()
  const [days, setDays] = useState<TripDay[] | null>(null)
  const [cut, setCut] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    void commands.tripDays(trip.firstDay).then((r) => {
      if (r.status === 'ok') setDays(r.data)
      else setError(errorText(r.error))
    })
  }, [trip.firstDay])

  const run = async (change: Promise<{ status: 'ok'; data: null } | { status: 'error'; error: IpcError }>) => {
    const r = await change
    if (r.status === 'ok') onDone()
    else setError(errorText(r.error))
  }

  return (
    <Stack>
      {error && <PanelNote danger>{error}</PanelNote>}
      <Field label={t('rollsense.trip.name')}>
        <CommitInput
          value={trip.name ?? ''}
          placeholder={name({ ...trip, name: null })}
          maxLength={80}
          ariaLabel={t('rollsense.trip.name')}
          onCommit={(value) => void commands.renameTrip(trip.firstDay, value.trim() || null)}
        />
      </Field>
      <SectionCard icon={<CalendarDays />} title={t('rollsense.trip.days')}>
        <ListRows>
          {(days ?? []).map((d, i) => (
            <Fragment key={d.day}>
              <ListRow trailing={<span className="text-xs text-muted-foreground tabular-nums">{d.count}</span>} onClick={() => setCut(cut === d.day ? null : d.day)}>
                {day(d.day)}
                {d.place ? ` · ${d.place}` : ''}
              </ListRow>
              {/* Between two days: where the trip may be split. */}
              {i < (days?.length ?? 0) - 1 && (
                <Toggle variant="outline" size="sm" pressed={cut === d.day} onPressedChange={(on) => setCut(on ? d.day : null)}>
                  <Scissors data-icon="inline-start" /> {t(cut === d.day ? 'rollsense.trip.splitHere' : 'rollsense.trip.splitAfter')}
                </Toggle>
              )}
            </Fragment>
          ))}
        </ListRows>
      </SectionCard>
      {cut !== null && (
        <Button onClick={() => void run(commands.splitTrip(trip.firstDay, cut))}>
          <Scissors data-icon="inline-start" /> {t('rollsense.trip.splitDo')}
        </Button>
      )}
      {before && (
        <Button variant="secondary" onClick={() => void run(commands.mergeTrip(trip.firstDay))}>
          <Merge data-icon="inline-start" /> {t('rollsense.trip.mergeBefore', { name: name(before) })}
        </Button>
      )}
      {after && (
        <Button variant="secondary" onClick={() => void run(commands.mergeTrip(after.firstDay))}>
          <Merge data-icon="inline-start" /> {t('rollsense.trip.mergeAfter', { name: name(after) })}
        </Button>
      )}
    </Stack>
  )
}
