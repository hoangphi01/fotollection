// SPDX-License-Identifier: GPL-3.0-only
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useEffect, useState, type KeyboardEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Toggle } from '@/components/ui/toggle'
import { commands, type CalDay } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

/** The week's first day of the language (`Intl.Locale` week info), with the system's region
 * when it speaks that language (English on a British system: Monday), as the core counts it:
 * 0 Monday … 6 Sunday; Monday where the engine does not say. */
function firstWeekday(language: string): number {
  try {
    const base = language.split('-')[0]
    const regional = navigator.languages.find((l) => l.split('-')[0] === base) ?? language
    const locale = new Intl.Locale(regional) as Intl.Locale & { getWeekInfo?: () => { firstDay: number }; weekInfo?: { firstDay: number } }
    const first = locale.getWeekInfo?.().firstDay ?? locale.weekInfo?.firstDay ?? 1
    return (first + 6) % 7
  } catch {
    return 0
  }
}

/** The weeks of a month, laid out by the core's calendar (FR-141 C1), the last answer kept
 * while the next loads. */
function useMonthGrid(year: number, month: number, first: number) {
  const [weeks, setWeeks] = useState<CalDay[][]>([])
  useEffect(() => {
    let current = true
    void commands.monthGrid(year, month, first).then((r) => {
      if (current && r.status === 'ok') setWeeks(r.data)
    })
    return () => {
      current = false
    }
  }, [year, month, first])
  return weeks
}

/** How dark a day with photos is shaded, in three neutral steps of the month's busiest day, a
 * step darker under the pointer: only a ticked day wears the accent (owner, 2026-11-19: the
 * busiest days read as ticked). */
function shade(count: number, most: number): string {
  if (count >= most * 0.5) return 'bg-foreground/28 hover:bg-foreground/36'
  if (count >= most * 0.15) return 'bg-foreground/16 hover:bg-foreground/28'
  return 'bg-foreground/8 hover:bg-foreground/16'
}

type CalendarMonthProps = {
  year: number
  /** 1…12. */
  month: number
  /** The month's name, its year, or controls for them, between the arrows. */
  title: ReactNode
  /** The photos of each day (days since 1970); a day without is not a choice. */
  counts: Map<number, number>
  /** The days switched on, pressed in as every switched-on control. */
  pressed(day: number): boolean
  /** A day with photos switched on or off. */
  onDay(day: number): void
  /** The month before or after; none when there is no other. */
  onPrev?: () => void
  onNext?: () => void
}

/**
 * One month as a grid of days (FR-141 C1–C2; STYLE_RULE §4, the calendar block): the arrows and
 * a title, the weekdays in the interface language from its first day of the week, then the
 * days laid out by the core, each a `Toggle` 32 px tall, 4 px apart as the parts of one control,
 * shaded in a neutral grey by how many photos it holds,
 * pressed in once switched on as every toggle. Arrow keys move between the days; Enter or Space
 * switches one. Every calendar is this block.
 */
export function CalendarMonth({ year, month, title, counts, pressed, onDay, onPrev, onNext }: CalendarMonthProps) {
  const { t, i18n } = useTranslation()
  const first = firstWeekday(i18n.language)
  const weeks = useMonthGrid(year, month, first)
  const weekday = new Intl.DateTimeFormat(i18n.language, { weekday: 'narrow', timeZone: 'UTC' })
  // 1 January 2024 was a Monday.
  const names = Array.from({ length: 7 }, (_, i) => weekday.format(new Date(Date.UTC(2024, 0, 1 + ((first + i) % 7)))))
  const days = weeks.flat()
  // Today on this computer's calendar, named to assistive technology.
  const [today] = useState(() => Math.floor((Date.now() - new Date().getTimezoneOffset() * 60_000) / 86_400_000))
  const most = Math.max(1, ...days.filter((d) => d.inMonth).map((d) => counts.get(d.day) ?? 0))

  // Arrows move to the next day with photos that way; up and down by a week.
  const move = (e: KeyboardEvent<HTMLDivElement>) => {
    const step = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7 }[e.key]
    const at = days.findIndex((d) => String(d.day) === (e.target as HTMLElement).dataset.day)
    if (!step || at < 0) return
    e.preventDefault()
    for (let i = at + step; i >= 0 && i < days.length; i += step) {
      const button = e.currentTarget.querySelector<HTMLButtonElement>(`[data-day="${days[i]?.day}"]:not(:disabled)`)
      if (button) return button.focus()
    }
  }

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-1">
        {/* The hint on a wrapper, so a disabled arrow still names itself (STYLE_RULE §4). */}
        <span data-hint={t('calendar.prev')}>
          <Button variant="outline" size="icon-sm" aria-label={t('calendar.prev')} disabled={!onPrev} onClick={onPrev}>
            <ChevronLeft />
          </Button>
        </span>
        <div className="flex min-w-0 flex-1 items-center justify-center gap-1">{title}</div>
        {/* The hint on a wrapper, so a disabled arrow still names itself (STYLE_RULE §4). */}
        <span data-hint={t('calendar.next')}>
          <Button variant="outline" size="icon-sm" aria-label={t('calendar.next')} disabled={!onNext} onClick={onNext}>
            <ChevronRight />
          </Button>
        </span>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center text-2xs text-muted-foreground" aria-hidden>
        {names.map((n, i) => (
          <span key={i}>{n}</span>
        ))}
      </div>
      <div role="group" aria-label={new Intl.DateTimeFormat(i18n.language, { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(new Date(Date.UTC(year, month - 1, 1)))} className="grid grid-cols-7 gap-1" onKeyDown={move}>
        {weeks.map((week) => (
          <div key={week[0]?.day} className="contents">
            {week.map((d) => {
              const count = d.inMonth ? (counts.get(d.day) ?? 0) : 0
              const on = count > 0 && pressed(d.day)
              return (
                <Toggle
                  key={d.day}
                  size="sm"
                  data-day={d.day}
                  disabled={count === 0}
                  pressed={on}
                  aria-current={d.day === today ? 'date' : undefined}
                  onPressedChange={() => onDay(d.day)}
                  aria-label={count > 0 ? `${d.date} · ${count.toLocaleString(i18n.language)}` : undefined}
                  className={cn('w-full min-w-0 px-0 tabular-nums', !d.inMonth && 'invisible', count > 0 && !on && shade(count, most))}
                >
                  {d.date}
                </Toggle>
              )
            })}
          </div>
        ))}
      </div>
    </div>
  )
}
