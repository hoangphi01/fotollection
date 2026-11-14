// SPDX-License-Identifier: GPL-3.0-only
// Interface translations (FR-020, ADR-0015). Every locale is bundled: nothing is fetched.
import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import type { Language } from '@/ipc/bindings'
import en from '@/locales/en.json'
import de from '@/locales/de.json'
import hu from '@/locales/hu.json'
import ja from '@/locales/ja.json'
import vi from '@/locales/vi.json'
import { pseudo } from './pseudo'

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
}

/** English first, then by each language's own name, as the website (LOCALIZATION §2). */
export const LANGUAGES: LanguageInfo[] = [
  { value: 'en', label: 'English', hello: 'Hello', guide: 'full', assets: 'en' },
  { value: 'de', label: 'Deutsch', hello: 'Hallo', guide: 'short', assets: 'en' },
  { value: 'hu', label: 'Magyar', hello: 'Szia', guide: 'short', assets: 'en' },
  { value: 'vi', label: 'Tiếng Việt', hello: 'Xin chào', guide: 'full', assets: 'vi' },
  { value: 'ja', label: '日本語', hello: 'こんにちは', guide: 'short', assets: 'en' },
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
  vi: { translation: vi },
} as const

void i18n.use(initReactI18next).init({
  resources,
  lng: 'en',
  fallbackLng: 'en',
  // React escapes every interpolated value; nothing is ever rendered as HTML (SECURITY.md).
  interpolation: { escapeValue: false },
  returnNull: false,
})

// The pseudo-locale `en-XA` (the usual code for one), English 40 % longer (FR-125), only in a `VITE_PSEUDO=1` build for the
// checks (scripts/ui-check.sh); a release build drops this branch and the generator with it.
if (import.meta.env.VITE_PSEUDO === '1') i18n.addResourceBundle('en-XA', 'translation', pseudo(en))

export default i18n
