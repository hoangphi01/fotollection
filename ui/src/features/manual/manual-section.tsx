// SPDX-License-Identifier: GPL-3.0-only
import { ArrowLeft, ArrowRight, BookOpen, Bookmark, ChevronDown, ListOrdered, type LucideIcon } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useDarkTheme } from '@/lib/theme'
import { ManualText } from '@/components/layout/manual-text'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { findChapters, type Chapter, type ManualCard } from '@/lib/manual'
import { useManualChapters } from './manual-content'
import { showManual, useManualChapter } from './manual-store'

const CARD_ICON: Record<ManualCard['kind'], LucideIcon> = { overview: BookOpen, topic: Bookmark, step: ListOrdered }

/** The chapter shown: the one asked for, else the first. */
function useShownChapter(): { list: Chapter[]; index: number; chapter: Chapter | undefined } {
  const list = useManualChapters()
  const asked = useManualChapter()
  const index = Math.max(0, list.findIndex((c) => c.id === asked))
  return { list, index, chapter: list[index] }
}

/**
 * Every chapter, found by its title or its words, case and accents aside (FR-110): a popover
 * as the compare views' menu, the search on top (an `Input`, as every field), the chapters as
 * `ListRow`s 4 px apart, the one open marked as every chosen row is (STYLE_RULE §4).
 */
function ChapterPicker({ list, chapter }: { list: Chapter[]; chapter: Chapter }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const found = useMemo(() => findChapters(list, query), [list, query])
  const pick = (id: string) => {
    showManual(id)
    setOpen(false)
    setQuery('')
  }
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" aria-expanded={open} className="w-full justify-start font-normal">
          <BookOpen data-icon="inline-start" />
          <span className="min-w-0 flex-1 truncate text-left">{`${chapter.number}. ${chapter.title}`}</span>
          <ChevronDown data-icon="inline-end" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="flex w-(--radix-popover-trigger-width) flex-col gap-1 p-1">
        {/* The same text field as every other in the app (STYLE_RULE §4: 32 px, set in). */}
        <Input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            const first = found[0]
            if (e.key === 'Enter' && first) pick(first.id)
          }}
          placeholder={t('manual.search')}
          aria-label={t('manual.search')}
        />
        {/* A native scroll, in the theme's thin bar (index.css): Radix's needs a fixed height. */}
        <div className="max-h-80 overflow-y-auto overscroll-contain">
          <ListRows>
            {found.map((c) => (
              <ListRow key={c.id} chosen={c.id === chapter.id} current="page" onClick={() => pick(c.id)}>
                <span className="min-w-0 flex-1 truncate text-left">{`${c.number}. ${c.title}`}</span>
              </ListRow>
            ))}
          </ListRows>
          {found.length === 0 && <p className="px-2 py-1.5 text-xs text-muted-foreground">{t('manual.none')}</p>}
        </div>
      </PopoverContent>
    </Popover>
  )
}

/**
 * Settings → *Manual* (FR-110): the whole user guide inside the Settings window, offline, its
 * words and pictures in the app's own cards (owner, 2026-11-07: in Settings itself, no other
 * window; "take only the words and the pictures"). A chapter picker with a search heads the
 * column, then the chapter's opening, its topics and its steps, 12 px apart as every card.
 * Loaded on first open, as its own chunk, so the guide never weighs on the start.
 */
export function ManualSection() {
  const { t, i18n } = useTranslation()
  const { list, chapter } = useShownChapter()
  const theme = useDarkTheme() ? 'dark' : 'light'
  if (!chapter) return null
  const lang = i18n.language.startsWith('vi') ? 'vi' : 'en'
  return (
    <>
      <ChapterPicker list={list} chapter={chapter} />
      {chapter.cards.map((card, i) => {
        const Icon = CARD_ICON[card.kind]
        return (
          <SectionCard key={`${chapter.id}:${i}`} icon={<Icon />} title={card.kind === 'overview' ? t('manual.overview') : card.title}>
            <ManualText markdown={card.body} picture={(name) => `/manual/${name}-${lang}-${theme}.webp`} />
          </SectionCard>
        )
      })}
    </>
  )
}

/** The Settings window's foot while the manual is open: where in it, *Back* and *Next*. */
export function ManualSteps() {
  const { t } = useTranslation()
  const { list, index } = useShownChapter()
  const go = (to: number) => {
    const next = list[to]
    if (next) showManual(next.id)
  }
  return (
    <>
      <p className="text-xs text-muted-foreground">{t('manual.chapterOf', { n: index + 1, count: list.length })}</p>
      <div className="ml-auto flex gap-2">
        <Button variant="outline" disabled={index === 0} onClick={() => go(index - 1)}>
          <ArrowLeft data-icon="inline-start" /> {t('manual.back')}
        </Button>
        <Button disabled={index >= list.length - 1} onClick={() => go(index + 1)}>
          {t('manual.next')} <ArrowRight data-icon="inline-end" />
        </Button>
      </div>
    </>
  )
}
