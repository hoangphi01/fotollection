// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in RollSense's side panel (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CalendarDays, Copy, House, Layers, MapPin, Plane, Smartphone, Tags } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { GroupRows } from '@/components/forms/group-rows'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { SectionCard } from '@/components/layout/section-card'
import type { Facets, SortPick } from '@/ipc/bindings'

type PickProps = {
  facets: Facets
  pick: SortPick | null
  onPick(pick: SortPick | null): void
}

const same = (a: SortPick | null, b: SortPick | null) => JSON.stringify(a) === JSON.stringify(b)

/** A count after the words, as every list's count (GroupRows). */
function Count({ n }: { n: number }) {
  const { i18n } = useTranslation()
  return <span className="text-xs text-muted-foreground tabular-nums">{n.toLocaleString(i18n.language)}</span>
}

/** One choice of a facet: its words, its count, chosen when it is the grid's pick. */
function PickRow({ value, count, pick, onPick, icon, children }: { value: SortPick; count: number; icon?: ReactNode; children: ReactNode } & Omit<PickProps, 'facets'>) {
  const chosen = same(value, pick)
  return (
    <ListRow icon={icon} chosen={chosen} trailing={<Count n={count} />} onClick={() => onPick(chosen ? null : value)}>
      <span className="truncate">{children}</span>
    </ListRow>
  )
}

/** A day (days since 1970) as the reader writes a short date. */
function useDay() {
  const { i18n } = useTranslation()
  const format = new Intl.DateTimeFormat(i18n.language, { day: 'numeric', month: 'short', timeZone: 'UTC' })
  return (day: number) => format.format(new Date(day * 86_400_000))
}

/** Where each photo came from (FR-141 S2). */
export function SourcesCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  if (facets.sources.length === 0) return null
  return (
    <SectionCard icon={<Smartphone />} title={t('rollsense.sources')}>
      <ListRows>
        {facets.sources.map((s) => (
          <PickRow key={s.value} value={{ kind: 'source', class: s.value }} count={s.count} pick={pick} onPick={onPick}>
            {t(`rollsense.source.${s.value}`)}
          </PickRow>
        ))}
      </ListRows>
    </SectionCard>
  )
}

/** What the photos show, from the pixels (FR-141 S6); shown once some are read. */
export function SubjectsCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  if (facets.analysed === 0) return null
  return (
    <SectionCard icon={<Tags />} title={t('rollsense.subjects')}>
      <ListRows>
        {facets.subjects.map((s) => (
          <PickRow key={s.value} value={{ kind: 'subject', key: s.value }} count={s.count} pick={pick} onPick={onPick}>
            {t(`rollsense.subject.${s.value}`, { defaultValue: s.value })}
          </PickRow>
        ))}
        {facets.noSubject > 0 && (
          <PickRow value={{ kind: 'noSubject' }} count={facets.noSubject} pick={pick} onPick={onPick}>
            {t('rollsense.noSubject')}
          </PickRow>
        )}
      </ListRows>
    </SectionCard>
  )
}

/** Years › months › events (FR-141 S4), the newest first, each folded until opened. */
export function TimeCard({ facets, pick, onPick }: PickProps) {
  const { t, i18n } = useTranslation()
  const day = useDay()
  const [open, setOpen] = useState<Set<string>>(new Set())
  if (facets.years.length === 0) return null
  const monthName = new Intl.DateTimeFormat(i18n.language, { month: 'long', timeZone: 'UTC' })
  const toggle = (key: string) =>
    setOpen((o) => {
      const next = new Set(o)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  return (
    <SectionCard icon={<CalendarDays />} title={t('rollsense.time')}>
      <ListRows>
        {facets.years.map((y) => (
          <GroupRows key={y.year} title={String(y.year)} count={y.count} open={open.has(`${y.year}`)} onToggle={() => toggle(`${y.year}`)}>
            <PickRow value={{ kind: 'year', year: y.year }} count={y.count} pick={pick} onPick={onPick}>
              {t('rollsense.all')}
            </PickRow>
            {y.months.map((m) => (
              <GroupRows
                key={m.month}
                title={monthName.format(new Date(Date.UTC(y.year, m.month - 1, 1)))}
                count={m.count}
                open={open.has(`${y.year}-${m.month}`)}
                onToggle={() => toggle(`${y.year}-${m.month}`)}
              >
                <PickRow value={{ kind: 'month', year: y.year, month: m.month }} count={m.count} pick={pick} onPick={onPick}>
                  {t('rollsense.all')}
                </PickRow>
                {m.events.map((e) => (
                  <PickRow key={e.first} value={{ kind: 'event', first: e.first }} count={e.count} pick={pick} onPick={onPick}>
                    {e.firstDay === e.lastDay ? day(e.firstDay) : `${day(e.firstDay)} – ${day(e.lastDay)}`}
                    {e.place ? ` · ${e.place}` : ''}
                  </PickRow>
                ))}
              </GroupRows>
            ))}
          </GroupRows>
        ))}
      </ListRows>
    </SectionCard>
  )
}

/** Trips away from home (FR-012), named after their place. */
export function TripsCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  const day = useDay()
  if (facets.trips.length === 0) return null
  return (
    <SectionCard icon={<Plane />} title={t('rollsense.trips')}>
      <ListRows>
        {[...facets.trips].reverse().map((trip) => (
          <PickRow key={trip.firstDay} value={{ kind: 'trip', firstDay: trip.firstDay }} count={trip.count} pick={pick} onPick={onPick}>
            {trip.place ?? t('rollsense.unknownPlace')} · {day(trip.firstDay)}
          </PickRow>
        ))}
      </ListRows>
    </SectionCard>
  )
}

/** Visits to the other homes, per year (FR-012). */
export function VisitsCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  if (facets.visits.length === 0) return null
  return (
    <SectionCard icon={<House />} title={t('rollsense.visits')}>
      <ListRows>
        {facets.visits.map((v) => (
          <PickRow key={`${v.home}-${v.year}`} value={{ kind: 'visit', home: v.home, year: v.year }} count={v.count} pick={pick} onPick={onPick}>
            {t('rollsense.visit', { year: v.year, place: v.place ?? t('rollsense.unknownPlace'), count: v.visits })}
          </PickRow>
        ))}
      </ListRows>
    </SectionCard>
  )
}

/** Countries › places (FR-141 S4); the map comes later. */
export function WhereCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<string | null>(null)
  if (facets.countries.length === 0) return null
  return (
    <SectionCard icon={<MapPin />} title={t('rollsense.places')}>
      <ListRows>
        {facets.countries.map((c) => (
          <GroupRows key={c.country} title={c.country} count={c.count} open={open === c.country} onToggle={() => setOpen(open === c.country ? null : c.country)}>
            {c.places.map((p) => (
              <PickRow key={p.id} value={{ kind: 'place', id: p.id }} count={p.count} pick={pick} onPick={onPick}>
                {p.name}
              </PickRow>
            ))}
          </GroupRows>
        ))}
      </ListRows>
    </SectionCard>
  )
}

/** Near-duplicates (FR-015) and stacks of one capture (FR-141 S3). */
export function DuplicatesCard({ facets, pick, onPick }: PickProps) {
  const { t } = useTranslation()
  if (facets.duplicates === 0 && facets.stacks === 0) return null
  return (
    <SectionCard icon={<Copy />} title={t('rollsense.more')}>
      <ListRows>
        {facets.duplicates > 0 && (
          <PickRow icon={<Copy />} value={{ kind: 'duplicates' }} count={facets.duplicates} pick={pick} onPick={onPick}>
            {t('rollsense.duplicates')}
          </PickRow>
        )}
        {facets.stacks > 0 && (
          <PickRow icon={<Layers />} value={{ kind: 'stacks' }} count={facets.stacks} pick={pick} onPick={onPick}>
            {t('rollsense.stacks')}
          </PickRow>
        )}
      </ListRows>
    </SectionCard>
  )
}
