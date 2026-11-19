// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type CullFilter, type GallerySection, type PhotoSummary } from '@/ipc/bindings'

/** A run of the grid under one header: its photos by position, and the header's words. */
export type GridSection = { start: number; count: number; label: string; trailing: string }

const TIMED = new Set(['day', 'event', 'month', 'year'])
const NONE: GallerySection[] = []

/**
 * The grid's photos with their headers under RollSense's time picks (FR-141 C2, owner
 * 2026-11-19): a day ticked splits it by its events (*07:48–08:16 · Seoul*), a month or a year
 * by its days, in the place's time (FR-145); the runs come from the core. The photos and their
 * headers are shown together: the last pair stays until the new photos' headers have come
 * (STYLE_RULE §4, the last answer stays), so the grid never draws twice and moves.
 */
export function useGallerySections(filter: CullFilter, photos: PhotoSummary[]): { photos: PhotoSummary[]; sections: GridSection[] } {
  const { t, i18n } = useTranslation()
  // The answer with the filter it was asked for; `null` sections when the core refused.
  const [answer, setAnswer] = useState<{ key: string; sections: GallerySection[] | null }>({ key: '', sections: [] })
  const key = JSON.stringify(filter)
  const timed = (filter.sort ?? []).some((p) => TIMED.has(p.kind))
  const count = photos.length
  // The filter by value: a new object with the same picks asks nothing again.
  useEffect(() => {
    if (!timed) return
    let current = true
    void commands.gallerySections(JSON.parse(key) as CullFilter).then((r) => {
      if (current) setAnswer({ key, sections: r.status === 'ok' ? r.data : null })
    })
    return () => {
      current = false
    }
  }, [key, count, timed])
  const ready = answer.key === key && (answer.sections === null || answer.sections.reduce((n, s) => n + s.count, 0) === count)
  // The pair shown, replaced only once both halves are new (state set while rendering, as React
  // allows for a value derived from props).
  const [shown, setShown] = useState<{ photos: PhotoSummary[]; sections: GallerySection[] }>({ photos, sections: NONE })
  const next = timed ? (answer.sections ?? NONE) : NONE
  if ((!timed || ready) && (shown.photos !== photos || shown.sections !== next)) setShown({ photos, sections: next })

  const clock = new Intl.DateTimeFormat(i18n.language, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: 'UTC' })
  const time = (min: number) => clock.format(new Date(min * 60_000))
  const date = new Intl.DateTimeFormat(i18n.language, { weekday: 'long', day: 'numeric', month: 'long', timeZone: 'UTC' })
  return {
    photos: shown.photos,
    sections: shown.sections.map((s) => {
      const when = s.kind === 'day' ? date.format(new Date(s.day * 86_400_000)) : s.startMin === s.endMin ? time(s.startMin) : `${time(s.startMin)}–${time(s.endMin)}`
      return { start: s.start, count: s.count, label: s.place ? `${when} · ${s.place}` : when, trailing: t('app.photos', { count: s.count }) }
    }),
  }
}
