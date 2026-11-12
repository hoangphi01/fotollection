// SPDX-License-Identifier: GPL-3.0-only
import { ArrowLeft, ArrowRight, FolderOpen, Languages, Layers, Printer, SlidersHorizontal, Sparkles } from 'lucide-react'
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useDarkTheme } from '@/lib/theme'
import { ListRow } from '@/components/forms/list-row'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { languageInfo } from '@/i18n'
import { LanguagePage } from './language-page'
import { closeWelcome, decideWelcome, useWelcomeOpen } from './welcome-store'

/** The five pages, in the order of the work (owner, 2026-11-07); each a picture of the app. */
const PAGES = [
  { key: 'language', icon: Languages },
  { key: 'welcome', icon: FolderOpen },
  { key: 'modes', icon: Layers },
  { key: 'choose', icon: Sparkles },
  { key: 'develop', icon: SlidersHorizontal },
  { key: 'print', icon: Printer },
] as const

/**
 * The welcome screen (FR-109): on the first launch after the install or an update, and at
 * every start when its last page's tick asks for it, the language (FR-124) then five pages of the basics in the Settings
 * window's frame (owner, 2026-11-07: "the size of Settings and Export"), the pages listed on
 * the left as the window's sections are. *Start* or Esc closes it and remembers it was seen.
 */
export function WelcomeDialog() {
  const { t, i18n } = useTranslation()
  const { ui, update, loaded } = useUiSettings()
  if (loaded) decideWelcome(ui.welcomeSeen, ui.welcomeEveryStart)
  const open = useWelcomeOpen()
  const [page, setPage] = useState(0)
  // The tick follows the saved choice until the user changes it, then is theirs until Start.
  const [tick, setTick] = useState<boolean | null>(null)
  const everyStart = tick ?? ui.welcomeEveryStart
  const setEveryStart = (on: boolean) => setTick(on)
  const tickId = useId()

  const close = () => {
    closeWelcome()
    setPage(0)
    setTick(null)
    void update({ welcomeSeen: true, welcomeEveryStart: everyStart })
  }
  const current = PAGES[page] ?? PAGES[0]
  const last = page === PAGES.length - 1
  const lang = languageInfo(i18n.language).assets
  const theme = useDarkTheme() ? 'dark' : 'light'

  return (
    <SettingsFrame
      open={open}
      onClose={close}
      title={t('welcome.title')}
      full
      closable={false}
      nav={PAGES.map((p, i) => (
        <ListRow key={p.key} icon={<p.icon />} chosen={i === page} current="step" onClick={() => setPage(i)}>
          {t(`welcome.${p.key}.title`)}
        </ListRow>
      ))}
      heading={t(`welcome.${current.key}.title`)}
      description={t('welcome.step', { n: page + 1, count: PAGES.length })}
      actions={
        <>
          {last && (
            <>
              <Checkbox id={tickId} checked={everyStart} onCheckedChange={(v) => setEveryStart(v === true)} />
              <Label htmlFor={tickId} className="text-xs font-normal text-muted-foreground">
                {t('welcome.everyStart')}
              </Label>
            </>
          )}
          <div className="ml-auto flex gap-2">
            {page > 0 && (
              <Button variant="outline" onClick={() => setPage(page - 1)}>
                <ArrowLeft data-icon="inline-start" /> {t('welcome.back')}
              </Button>
            )}
            {last ? (
              <Button onClick={close}>{t('welcome.start')}</Button>
            ) : (
              <Button onClick={() => setPage(page + 1)}>
                {t('welcome.next')} <ArrowRight data-icon="inline-end" />
              </Button>
            )}
          </div>
        </>
      }
    >
      {current.key === 'language' ? (
        <LanguagePage />
      ) : (
      /* The picture fills the page; the words sit on it in frosted glass, the picture blurred
         behind them (owner, 2026-11-07). */
      <div className="relative min-h-0 flex-1 overflow-hidden rounded-lg border lifted">
        <img
          src={`/welcome/${current.key}-${lang}-${theme}.webp`}
          alt={t(`welcome.${current.key}.alt`)}
          width={1600}
          height={1000}
          className="absolute inset-0 size-full object-cover object-left-top"
        />
        {/* A welcome reads large (owner, 2026-11-07): the page's name as a title, its words at
            the size of a lead. */}
        <div className="absolute inset-x-6 bottom-6 flex max-w-2xl flex-col gap-2 rounded-2xl border bg-background/60 p-6 backdrop-blur-xl lifted">
          <h2 className="text-2xl font-semibold">{t(`welcome.${current.key}.title`)}</h2>
          <p className="text-lg leading-relaxed">{t(`welcome.${current.key}.text`)}</p>
        </div>
      </div>
      )}
    </SettingsFrame>
  )
}
