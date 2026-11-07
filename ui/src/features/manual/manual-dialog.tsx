// SPDX-License-Identifier: GPL-3.0-only
import { ArrowLeft, ArrowRight, BookOpen, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ListRow } from '@/components/forms/list-row'
import { ManualText } from '@/components/layout/manual-text'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { Button } from '@/components/ui/button'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { ScrollArea } from '@/components/ui/scroll-area'
import { findChapters } from '@/lib/manual'
import { useManualChapters } from './manual-content'
import { showManual, useManualChapter } from './manual-store'

/**
 * The manual inside the app (FR-110): the whole user guide, bundled at build time with its
 * screenshots, so it reads with no internet. The Settings window's frame (owner, 2026-11-07),
 * the chapters listed on the left with a search over their words, the chapter on the right,
 * *Back* and *Next* at its foot. Opened from Settings → *Manual* and the Help menu; loaded
 * then, as its own chunk (`app.tsx`), so the guide never weighs on the start.
 */
export function ManualDialog() {
  const { t, i18n } = useTranslation()
  const list = useManualChapters()
  const asked = useManualChapter()
  const [query, setQuery] = useState('')
  const shown = useMemo(() => findChapters(list, query), [list, query])
  const index = Math.max(0, list.findIndex((c) => c.id === asked))
  const chapter = list[index]
  const lang = i18n.language.startsWith('vi') ? 'vi' : 'en'
  const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light'

  if (!chapter) return null
  const go = (to: number) => {
    const next = list[to]
    if (next) showManual(next.id)
  }
  return (
    <SettingsFrame
      open={asked !== null}
      onClose={() => {
        showManual(null)
        setQuery('')
      }}
      title={t('manual.title')}
      full
      nav={
        <>
          <InputGroup>
            <InputGroupAddon>
              <Search />
            </InputGroupAddon>
            <InputGroupInput
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('manual.search')}
              aria-label={t('manual.search')}
            />
          </InputGroup>
          {shown.map((c) => (
            <ListRow key={c.id} chosen={c.id === chapter.id} current="page" onClick={() => showManual(c.id)}>
              <span className="truncate">{`${c.number}. ${c.title}`}</span>
            </ListRow>
          ))}
          {shown.length === 0 && <p className="px-2 text-xs text-muted-foreground">{t('manual.none')}</p>}
        </>
      }
      navFooter={t('manual.offline')}
      heading={`${chapter.number}. ${chapter.title}`}
      description={t('manual.chapterOf', { n: index + 1, count: list.length })}
      actions={
        <div className="ml-auto flex gap-2">
          <Button variant="outline" disabled={index === 0} onClick={() => go(index - 1)}>
            <ArrowLeft data-icon="inline-start" /> {t('manual.back')}
          </Button>
          <Button disabled={index === list.length - 1} onClick={() => go(index + 1)}>
            {t('manual.next')} <ArrowRight data-icon="inline-end" />
          </Button>
        </div>
      }
    >
      {/* A new chapter starts at its top: the scroll is keyed by it. */}
      <ScrollArea key={chapter.id} className="min-h-0 flex-1">
        <article className="mx-auto max-w-2xl p-3" aria-label={chapter.title}>
          <ManualText markdown={chapter.body} picture={(name) => `/manual/${name}-${lang}-${theme}.webp`} />
        </article>
      </ScrollArea>
    </SettingsFrame>
  )
}

/** Settings → *Manual*: the chapters, each opening the manual there (FR-110). */
export function ManualChapterList({ onOpen }: { onOpen: () => void }) {
  const list = useManualChapters()
  return (
    <>
      {list.map((c) => (
        <ListRow
          key={c.id}
          icon={<BookOpen />}
          onClick={() => {
            onOpen()
            showManual(c.id)
          }}
        >
          <span className="truncate">{`${c.number}. ${c.title}`}</span>
        </ListRow>
      ))}
    </>
  )
}
