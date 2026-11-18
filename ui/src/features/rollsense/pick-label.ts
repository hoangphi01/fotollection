// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import type { Facets, SortPick, TripFacet } from '@/ipc/bindings'

/** A day (days since 1970) as the reader writes a short date. */
export function useDay() {
  const { i18n } = useTranslation()
  const format = new Intl.DateTimeFormat(i18n.language, { day: 'numeric', month: 'short', timeZone: 'UTC' })
  return (day: number) => format.format(new Date(day * 86_400_000))
}

/** A pick's words, as its row in the cards writes them: what the toolbar's chip names. */
export function usePickLabel(facets: Facets | null) {
  const { t, i18n } = useTranslation()
  const day = useDay()
  const monthName = new Intl.DateTimeFormat(i18n.language, { month: 'long', year: 'numeric', timeZone: 'UTC' })
  return (p: SortPick): string => {
    switch (p.kind) {
      case 'source':
        return t(`rollsense.source.${p.class}`)
      case 'subject':
        return t(`rollsense.subject.${p.key}`, { defaultValue: p.key })
      case 'noSubject':
        return t('rollsense.noSubject')
      case 'year':
        return String(p.year)
      case 'month':
        return monthName.format(new Date(Date.UTC(p.year, p.month - 1, 1)))
      case 'event': {
        const e = facets?.years.flatMap((y) => y.months.flatMap((m) => m.events)).find((x) => x.first === p.first)
        return e ? day(e.firstDay) : t('rollsense.time')
      }
      case 'trip': {
        // The trip's own name when the user gave it one, as its row in the card says (S7c).
        const trip = facets?.trips.find((x) => x.firstDay === p.firstDay)
        return trip?.name ?? trip?.place ?? t('rollsense.trips')
      }
      case 'visit':
        return facets?.visits.find((x) => x.home === p.home && x.year === p.year)?.place ?? t('rollsense.visits')
      case 'place':
        return facets?.countries.flatMap((c) => c.places).find((x) => x.id === p.id)?.name ?? t('rollsense.places')
      case 'duplicates':
        return t('rollsense.duplicates')
      default:
        return t('rollsense.stacks')
    }
  }
}

/** A trip's name: the user's, else its place and first day (`Lisbon · 12 Jul`). */
export function useTripName() {
  const { t } = useTranslation()
  const day = useDay()
  return (trip: TripFacet) => trip.name ?? `${trip.place ?? t('rollsense.unknownPlace')} · ${day(trip.firstDay)}`
}
