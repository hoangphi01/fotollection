// SPDX-License-Identifier: GPL-3.0-only
import { ArrowLeft, ArrowRight, BookOpen, Bookmark, ChevronsUpDown, ListOrdered, type LucideIcon } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ManualText } from '@/components/layout/manual-text'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
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

/** Every chapter, found by its title or its words, case and accents aside (FR-110). */
function ChapterPicker({ list, chapter }: { list: Chapter[]; chapter: Chapter }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const found = useMemo(() => findChapters(list, query), [list, query])
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" role="combobox" aria-expanded={open} className="w-full justify-start font-normal">
          <BookOpen data-icon="inline-start" />
          <span className="min-w-0 flex-1 truncate text-left">{`${chapter.number}. ${chapter.title}`}</span>
          <ChevronsUpDown data-icon="inline-end" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-(--radix-popover-trigger-width) p-0" align="start">
        {/* The search reads the chapters' words too, so cmdk's own filtering is off. */}
        <Command shouldFilter={false}>
          <CommandInput value={query} onValueChange={setQuery} placeholder={t('manual.search')} />
          <CommandList>
            <CommandEmpty>{t('manual.none')}</CommandEmpty>
            {found.map((c) => (
              <CommandItem
                key={c.id}
                value={c.id}
                onSelect={() => {
                  showManual(c.id)
                  setOpen(false)
                  setQuery('')
                }}
              >
                <span className="min-w-0 flex-1 truncate">{`${c.number}. ${c.title}`}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
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
  if (!chapter) return null
  const lang = i18n.language.startsWith('vi') ? 'vi' : 'en'
  const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light'
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
