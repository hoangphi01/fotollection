// SPDX-License-Identifier: GPL-3.0-only
import { HomeZoneSetting } from '@/features/settings/home-zone-setting'
import { useLanguage } from '@/features/settings/use-language'
import { LANGUAGES } from '@/i18n'

/**
 * The home time's page of the welcome after an update (FR-146, FR-145): the language page's
 * picture under frosted glass, the one setting on it, as the whole welcome's first page holds it.
 */
export function HomeZonePage() {
  const { language } = useLanguage()
  const backdrop = (LANGUAGES.find((l) => l.value === language) ?? LANGUAGES[0])?.backdrop ?? 'en'
  return (
    <div className="relative flex min-h-0 flex-1 flex-col items-center justify-center overflow-hidden rounded-lg border lifted">
      <img
        src={`/welcome/language-${backdrop}.webp`}
        alt=""
        aria-hidden
        className="absolute inset-0 size-full scale-105 object-cover blur-sm brightness-75"
      />
      <div className="relative w-80 rounded-2xl border bg-background/40 p-3 backdrop-blur-lg backdrop-saturate-150 lifted">
        <HomeZoneSetting />
      </div>
    </div>
  )
}
