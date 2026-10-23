// SPDX-License-Identifier: GPL-3.0-only
// Interface translations (FR-020, ADR-0015). Every locale is bundled: nothing is fetched.
import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import type { Language } from '@/ipc/bindings'
import en from '@/locales/en.json'
import vi from '@/locales/vi.json'

export const LANGUAGES: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'vi', label: 'Tiếng Việt' },
]

export const resources = {
  en: { translation: en },
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

export default i18n
