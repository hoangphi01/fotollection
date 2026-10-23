// SPDX-License-Identifier: GPL-3.0-only
// Typed translation keys: a key missing from en.json is a compile error (FR-020).
import 'i18next'
import type en from '@/locales/en.json'

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation'
    resources: { translation: typeof en }
  }
}
