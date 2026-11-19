// SPDX-License-Identifier: GPL-3.0-only
// Interface translations (FR-020, ADR-0015). Every locale is bundled: nothing is fetched.
import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import type { Language } from '@/ipc/bindings'
import en from '@/locales/en.json'
import de from '@/locales/de.json'
import hu from '@/locales/hu.json'
import ja from '@/locales/ja.json'
import ko from '@/locales/ko.json'
import vi from '@/locales/vi.json'
import { pseudo } from './pseudo'
import { MOD } from '@/lib/platform'
import flagDe from '@/assets/flags/de.svg'
import flagGb from '@/assets/flags/gb.svg'
import flagHu from '@/assets/flags/hu.svg'
import flagJp from '@/assets/flags/jp.svg'
import flagKr from '@/assets/flags/kr.svg'
import flagVn from '@/assets/flags/vn.svg'

/**
 * Each language described once (docs/LOCALIZATION.md §2): its own name, its manual (the full
 * user guide, or the website's short one) and whose pictures it shows. Nothing else tests a
 * language code; what a language lacks is shown in English.
 */
export type LanguageInfo = {
  value: Language
  label: string
  /** Its own greeting, on the welcome's first page (FR-124). */
  hello: string
  guide: 'full' | 'short'
  assets: 'en' | 'vi'
  /** Its country's round flag beside its name on the welcome (circle-flags, MIT;
   * `assets/flags/README.md`), English the British one as on the website. */
  flag: string
  /** The photo behind the welcome's first page in this language (owner, 2026-11-15): Japan's
   * and Korea's own, else the Prague tram (`public/welcome/language-*.webp`). */
  backdrop: 'en' | 'ja' | 'ko'
}

/** English first, then by each language's own name, as the website (LOCALIZATION §2). */
export const LANGUAGES: LanguageInfo[] = [
  { value: 'en', label: 'English', hello: 'Hello', guide: 'full', assets: 'en', flag: flagGb, backdrop: 'en' },
  { value: 'de', label: 'Deutsch', hello: 'Hallo', guide: 'short', assets: 'en', flag: flagDe, backdrop: 'en' },
  { value: 'hu', label: 'Magyar', hello: 'Szia', guide: 'short', assets: 'en', flag: flagHu, backdrop: 'en' },
  { value: 'vi', label: 'Tiếng Việt', hello: 'Xin chào', guide: 'full', assets: 'vi', flag: flagVn, backdrop: 'en' },
  { value: 'ko', label: '한국어', hello: '안녕하세요', guide: 'short', assets: 'en', flag: flagKr, backdrop: 'ko' },
  { value: 'ja', label: '日本語', hello: 'こんにちは', guide: 'short', assets: 'en', flag: flagJp, backdrop: 'ja' },
]

/** The entry of an i18next language (`vi`, `hu-HU`…), English when it is none of ours. */
export function languageInfo(language: string): LanguageInfo {
  const code = language.split('-')[0]
  return LANGUAGES.find((l) => l.value === code) ?? (LANGUAGES[0] as LanguageInfo)
}

export const resources = {
  en: { translation: en },
  de: { translation: de },
  hu: { translation: hu },
  ja: { translation: ja },
  ko: { translation: ko },
  vi: { translation: vi },
} as const

void i18n.use(initReactI18next).init({
  resources,
  lng: 'en',
  fallbackLng: 'en',
  // React escapes every interpolated value; nothing is ever rendered as HTML (SECURITY.md).
  // {{mod}} is this system's modifier (⌘ or Ctrl) in every string, so no shortcut is written for one system only.
  interpolation: { escapeValue: false, defaultVariables: { mod: MOD } },
  returnNull: false,
})

// The pseudo-locale `en-XA` (the usual code for one), English 40 % longer (FR-125), only in a `VITE_PSEUDO=1` build for the
// checks (scripts/ui-check.sh); a release build drops this branch and the generator with it.
if (import.meta.env.VITE_PSEUDO === '1') i18n.addResourceBundle('en-XA', 'translation', pseudo(en))

export default i18n
