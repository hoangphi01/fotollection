// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in RollSense's side panel (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CalendarDays, Copy, House, MapPin, Plane, Smartphone, Tags } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ResetButton } from '@/components/forms/danger-button'
import { GroupRows } from '@/components/forms/group-rows'
import { CheckRow, ListRows } from '@/components/forms/list-row'
import { SectionCard } from '@/components/layout/section-card'
import type { Facets, SortPick } from '@/ipc/bindings'

type PickProps = {
  facets: Facets
  picks: SortPick[]
  onPicks(picks: SortPick[]): void
}

/** The card a pick belongs to, as the core's `SortPick::card`: one card's picks widen the grid,
 * different cards' narrow it (owner, 2026-11-18). */
function cardOf(p: SortPick): number {
  switch (p.kind) {
    case 'source':
      return 0
    case 'subject':
    case 'noSubject':
      return 1
    case 'year':
    case 'month':
    case 'event':
      return 2
    case 'trip':
      return 3
    case 'visit':
      return 4
    case 'place':
      return 5
    default:
      return 6
  }
}

const same = (a: SortPick, b: SortPick) => JSON.stringify(a) === JSON.stringify(b)

/** A count after the words, as every list's count (GroupRows). */
function Count({ n }: { n: number }) {
  const { i18n } = useTranslation()
  return <span className="text-xs text-muted-foreground tabular-nums">{n.toLocaleString(i18n.language)}</span>
}

/** One choice of a facet, ticked with others: its words and its count. */
function PickRow({ value, count, picks, onPicks, children }: { value: SortPick; count: number; children: ReactNode } & Omit<PickProps, 'facets'>) {
  const checked = picks.some((p) => same(p, value))
  return (
    <CheckRow checked={checked} trailing={<Count n={count} />} onToggle={() => onPicks(checked ? picks.filter((p) => !same(p, value)) : [...picks, value])}>
      {children}
    </CheckRow>
  )
}

/** A card of one facet, its red square that clears the card's ticks (owner, 2026-11-18), shown
 * only once the card has one: a disabled red button read as pink. */
function FacetCard({ card, icon, title, picks, onPicks, children }: { card: number; icon: ReactNode; title: string; children: ReactNode } & Omit<PickProps, 'facets'>) {
  const { t } = useTranslation()
  const ticked = picks.some((p) => cardOf(p) === card)
  return (
    <SectionCard
      icon={icon}
      title={title}
      action={ticked && <ResetButton iconOnly label={t('rollsense.reset')} onClick={() => onPicks(picks.filter((p) => cardOf(p) !== card))} />}
    >
      <ListRows>{children}</ListRows>
    </SectionCard>
  )
}

/** A day (days since 1970) as the reader writes a short date. */
function useDay() {
  const { i18n } = useTranslation()
  const format = new Intl.DateTimeFormat(i18n.language, { day: 'numeric', month: 'short', timeZone: 'UTC' })
  return (day: number) => format.format(new Date(day * 86_400_000))
}

/** Where each photo came from (FR-141 S2). */
export function SourcesCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  if (facets.sources.length === 0) return null
  return (
    <FacetCard card={0} icon={<Smartphone />} title={t('rollsense.sources')} picks={picks} onPicks={onPicks}>
      {facets.sources.map((s) => (
        <PickRow key={s.value} value={{ kind: 'source', class: s.value }} count={s.count} picks={picks} onPicks={onPicks}>
          {t(`rollsense.source.${s.value}`)}
        </PickRow>
      ))}
    </FacetCard>
  )
}

/** What the photos show, from the pixels (FR-141 S6); shown once some are read. */
export function SubjectsCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  if (facets.analysed === 0) return null
  return (
    <FacetCard card={1} icon={<Tags />} title={t('rollsense.subjects')} picks={picks} onPicks={onPicks}>
      {facets.subjects.map((s) => (
        <PickRow key={s.value} value={{ kind: 'subject', key: s.value }} count={s.count} picks={picks} onPicks={onPicks}>
          {t(`rollsense.subject.${s.value}`, { defaultValue: s.value })}
        </PickRow>
      ))}
      {facets.noSubject > 0 && (
        <PickRow value={{ kind: 'noSubject' }} count={facets.noSubject} picks={picks} onPicks={onPicks}>
          {t('rollsense.noSubject')}
        </PickRow>
      )}
    </FacetCard>
  )
}

/** Years › months › events (FR-141 S4), the newest first, each folded until opened. */
export function TimeCard({ facets, picks, onPicks }: PickProps) {
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
  const row = { picks, onPicks }
  return (
    <FacetCard card={2} icon={<CalendarDays />} title={t('rollsense.time')} {...row}>
      {facets.years.map((y) => (
        <GroupRows key={y.year} title={String(y.year)} count={y.count} open={open.has(`${y.year}`)} onToggle={() => toggle(`${y.year}`)}>
          <PickRow value={{ kind: 'year', year: y.year }} count={y.count} {...row}>
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
              <PickRow value={{ kind: 'month', year: y.year, month: m.month }} count={m.count} {...row}>
                {t('rollsense.all')}
              </PickRow>
              {m.events.map((e) => (
                <PickRow key={e.first} value={{ kind: 'event', first: e.first }} count={e.count} {...row}>
                  {e.firstDay === e.lastDay ? day(e.firstDay) : `${day(e.firstDay)} – ${day(e.lastDay)}`}
                  {e.place ? ` · ${e.place}` : ''}
                </PickRow>
              ))}
            </GroupRows>
          ))}
        </GroupRows>
      ))}
    </FacetCard>
  )
}

/** Trips away from home (FR-012), named after their place. */
export function TripsCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  const day = useDay()
  if (facets.trips.length === 0) return null
  return (
    <FacetCard card={3} icon={<Plane />} title={t('rollsense.trips')} picks={picks} onPicks={onPicks}>
      {[...facets.trips].reverse().map((trip) => (
        <PickRow key={trip.firstDay} value={{ kind: 'trip', firstDay: trip.firstDay }} count={trip.count} picks={picks} onPicks={onPicks}>
          {trip.place ?? t('rollsense.unknownPlace')} · {day(trip.firstDay)}
        </PickRow>
      ))}
    </FacetCard>
  )
}

/** Visits to the other homes, per year (FR-012). */
export function VisitsCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  if (facets.visits.length === 0) return null
  return (
    <FacetCard card={4} icon={<House />} title={t('rollsense.visits')} picks={picks} onPicks={onPicks}>
      {facets.visits.map((v) => (
        <PickRow key={`${v.home}-${v.year}`} value={{ kind: 'visit', home: v.home, year: v.year }} count={v.count} picks={picks} onPicks={onPicks}>
          {t('rollsense.visit', { year: v.year, place: v.place ?? t('rollsense.unknownPlace'), count: v.visits })}
        </PickRow>
      ))}
    </FacetCard>
  )
}

/** Countries › places (FR-141 S4); the map comes later. */
export function WhereCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<string | null>(null)
  if (facets.countries.length === 0) return null
  return (
    <FacetCard card={5} icon={<MapPin />} title={t('rollsense.places')} picks={picks} onPicks={onPicks}>
      {facets.countries.map((c) => (
        <GroupRows key={c.country} title={c.country} count={c.count} open={open === c.country} onToggle={() => setOpen(open === c.country ? null : c.country)}>
          {c.places.map((p) => (
            <PickRow key={p.id} value={{ kind: 'place', id: p.id }} count={p.count} picks={picks} onPicks={onPicks}>
              {p.name}
            </PickRow>
          ))}
        </GroupRows>
      ))}
    </FacetCard>
  )
}

/** Near-duplicates (FR-015) and stacks of one capture (FR-141 S3). */
export function DuplicatesCard({ facets, picks, onPicks }: PickProps) {
  const { t } = useTranslation()
  if (facets.duplicates === 0 && facets.stacks === 0) return null
  return (
    <FacetCard card={6} icon={<Copy />} title={t('rollsense.more')} picks={picks} onPicks={onPicks}>
      {facets.duplicates > 0 && (
        <PickRow value={{ kind: 'duplicates' }} count={facets.duplicates} picks={picks} onPicks={onPicks}>
          {t('rollsense.duplicates')}
        </PickRow>
      )}
      {facets.stacks > 0 && (
        <PickRow value={{ kind: 'stacks' }} count={facets.stacks} picks={picks} onPicks={onPicks}>
          {t('rollsense.stacks')}
        </PickRow>
      )}
    </FacetCard>
  )
}
