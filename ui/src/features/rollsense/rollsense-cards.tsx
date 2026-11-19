// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in RollSense's side panel (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CalendarDays, Copy, House, MapPin, Merge, Pencil, Plane, Scissors, Smartphone, Tags } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { CalendarMonth } from '@/components/forms/calendar-month'
import { ResetButton } from '@/components/forms/danger-button'
import { GroupRows } from '@/components/forms/group-rows'
import { CommitInput } from '@/components/forms/commit-input'
import { CheckRow, ListRow, ListRows } from '@/components/forms/list-row'
import { MenuSeparator } from '@/components/forms/menu-content'
import { RowMenu } from '@/components/forms/row-menu'
import { SectionCard } from '@/components/layout/section-card'
import { Toggle } from '@/components/ui/toggle'
import { commands, type Facets, type SortPick, type TripFacet } from '@/ipc/bindings'
import { CountryMapView } from './country-map'
import { useDay, useTripName } from './pick-label'

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
    case 'day':
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

/** The months with photos, oldest first. */
function monthsOf(facets: Facets) {
  return facets.years.flatMap((y) => y.months.map((m) => ({ year: y.year, month: m.month }))).sort((a, b) => a.year * 12 + a.month - (b.year * 12 + b.month))
}

/** A month as a calendar (FR-141 C1–C2, owner 2026-11-19): days shaded by their photos, the
 * month and the year picked from the title; a day chosen shows its events under it with their
 * local times (FR-145), and the whole day, each a pick of this card. */
export function TimeCard({ facets, picks, onPicks }: PickProps) {
  const { t, i18n } = useTranslation()
  const day = useDay()
  const [shown, setShown] = useState<{ year: number; month: number } | null>(null)
  const [chosen, setChosen] = useState<number | null>(null)
  const months = monthsOf(facets)
  if (months.length === 0) return null
  const found = shown ? months.findIndex((m) => m.year === shown.year && m.month === shown.month) : -1
  const index = found >= 0 ? found : months.length - 1
  const { year, month } = months[index] ?? { year: 1970, month: 1 }
  const go = (i: number) => {
    setShown(months[i] ?? null)
    setChosen(null)
  }
  const counts = new Map(facets.days.map((d) => [d.day, d.count]))
  const events = facets.years.flatMap((y) => y.months.flatMap((m) => m.events))
  const ticked = (p: SortPick) => picks.some((x) => same(x, p))
  const pressed = (d: number) => {
    const date = new Date(d * 86_400_000)
    return (
      ticked({ kind: 'day', day: d }) ||
      ticked({ kind: 'month', year: date.getUTCFullYear(), month: date.getUTCMonth() + 1 }) ||
      ticked({ kind: 'year', year: date.getUTCFullYear() }) ||
      events.some((e) => d >= e.firstDay && d <= e.lastDay && ticked({ kind: 'event', first: e.first }))
    )
  }
  const flip = (p: SortPick) => onPicks(ticked(p) ? picks.filter((x) => !same(x, p)) : [...picks, p])
  const monthPick: SortPick = { kind: 'month', year, month }
  const yearPick: SortPick = { kind: 'year', year }
  const clock = new Intl.DateTimeFormat(i18n.language, { hour: '2-digit', minute: '2-digit', timeZone: 'UTC' })
  const time = (min: number) => clock.format(new Date(min * 60_000))
  const ofDay = chosen === null ? [] : events.filter((e) => chosen >= e.firstDay && chosen <= e.lastDay)
  const row = { picks, onPicks }
  return (
    <FacetCard card={2} icon={<CalendarDays />} title={t('rollsense.time')} {...row}>
      <CalendarMonth
        year={year}
        month={month}
        title={
          <>
            <Toggle size="sm" pressed={ticked(monthPick)} onPressedChange={() => flip(monthPick)} data-hint={t('rollsense.wholeMonth')}>
              {new Intl.DateTimeFormat(i18n.language, { month: 'long', timeZone: 'UTC' }).format(new Date(Date.UTC(year, month - 1, 1)))}
            </Toggle>
            <Toggle size="sm" pressed={ticked(yearPick)} onPressedChange={() => flip(yearPick)} data-hint={t('rollsense.wholeYear')}>
              {year}
            </Toggle>
          </>
        }
        counts={counts}
        pressed={pressed}
        chosen={chosen}
        onDay={(d) => setChosen(chosen === d ? null : d)}
        onPrev={index > 0 ? () => go(index - 1) : undefined}
        onNext={index < months.length - 1 ? () => go(index + 1) : undefined}
      />
      {chosen !== null && (
        <>
          <PickRow value={{ kind: 'day', day: chosen }} count={counts.get(chosen) ?? 0} {...row}>
            {t('rollsense.wholeDay', { day: day(chosen) })}
          </PickRow>
          {ofDay.map((e) => (
              <PickRow key={e.first} value={{ kind: 'event', first: e.first }} count={e.count} {...row}>
                {e.firstDay === e.lastDay ? (e.startMin === e.endMin ? time(e.startMin) : `${time(e.startMin)}–${time(e.endMin)}`) : `${day(e.firstDay)} ${time(e.startMin)} – ${day(e.lastDay)} ${time(e.endMin)}`}
                {e.place ? ` · ${e.place}` : ''}
              </PickRow>
            ))}
        </>
      )}
    </FacetCard>
  )
}

/** Trips away from home (FR-012), named after their place. */
export function TripsCard({ facets, picks, onPicks, onOpenTrip }: PickProps & { onOpenTrip(trip: TripFacet): void }) {
  const { t } = useTranslation()
  const name = useTripName()
  const [renaming, setRenaming] = useState<number | null>(null)
  if (facets.trips.length === 0) return null
  const trips = facets.trips
  return (
    <FacetCard card={3} icon={<Plane />} title={t('rollsense.trips')} picks={picks} onPicks={onPicks}>
      {[...trips].reverse().map((trip) => {
        // The trip after it in time, the one "Merge with the next" takes in.
        const next = trips[trips.indexOf(trip) + 1]
        return renaming === trip.firstDay ? (
          <CommitInput
            key={trip.firstDay}
            value={trip.name ?? ''}
            placeholder={name({ ...trip, name: null })}
            maxLength={80}
            ariaLabel={t('rollsense.trip.rename')}
            onCommit={(value) => {
              setRenaming(null)
              void commands.renameTrip(trip.firstDay, value.trim() || null)
            }}
          />
        ) : (
          // Its actions on a ⋯ and a right click (FR-117), the split on the trip's page.
          <RowMenu
            key={trip.firstDay}
            label={t('rollsense.trip.actions')}
            menu={(close) => (
              <>
                <ListRow icon={<Pencil />} onClick={() => (close(), setRenaming(trip.firstDay))}>
                  {t('rollsense.trip.rename')}
                </ListRow>
                <ListRow icon={<Scissors />} onClick={() => (close(), onOpenTrip(trip))}>
                  {t('rollsense.trip.split')}
                </ListRow>
                {next && (
                  <ListRow icon={<Merge />} onClick={() => (close(), void commands.mergeTrip(next.firstDay))}>
                    {t('rollsense.trip.mergeNext')}
                  </ListRow>
                )}
                {trip.name && (
                  <>
                    <MenuSeparator />
                    <ResetButton wide label={t('rollsense.trip.suggested')} onClick={() => (close(), void commands.renameTrip(trip.firstDay, null))} />
                  </>
                )}
              </>
            )}
          >
            <PickRow value={{ kind: 'trip', firstDay: trip.firstDay }} count={trip.count} picks={picks} onPicks={onPicks}>
              {name(trip)}
            </PickRow>
          </RowMenu>
        )
      })}
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

/** Countries › places (FR-141 S4), one country's map at the top (owner, 2026-11-18). */
export function WhereCard({ facets, picks, onPicks, folders }: PickProps & { folders: number[] }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<string | null>(null)
  if (facets.countries.length === 0) return null
  // The map shows the country open in the list, else the one with the most photos (owner,
  // 2026-11-18: one country at a time).
  const shown = facets.countries.find((c) => c.country === open) ?? [...facets.countries].sort((a, b) => b.count - a.count)[0]
  return (
    <FacetCard card={5} icon={<MapPin />} title={t('rollsense.places')} picks={picks} onPicks={onPicks}>
      {shown && <CountryMapView country={shown} folders={folders} picks={picks} onPicks={onPicks} />}
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
