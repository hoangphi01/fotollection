// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect } from 'react'
import i18n from '@/i18n'
import type { Language } from '@/ipc/bindings'
import { useUiSettings } from './use-ui-settings'

/** The interface language, stored per machine in `settings.json` (FR-020). */
export function useLanguage() {
  const { ui, update } = useUiSettings()
  const language = ui.language

  useEffect(() => {
    void i18n.changeLanguage(language)
    document.documentElement.lang = language
  }, [language])

  const change = useCallback((next: Language) => update({ language: next }), [update])

  return { language, change }
}
