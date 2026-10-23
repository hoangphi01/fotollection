// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import i18n from '@/i18n'
import { commands, type Language } from '@/ipc/bindings'

function apply(language: Language) {
  void i18n.changeLanguage(language)
  document.documentElement.lang = language
}

/** The interface language, stored per machine in `settings.json` (FR-020). */
export function useLanguage() {
  const [language, setLanguage] = useState<Language>('en')

  useEffect(() => {
    let live = true
    void commands.getUiSettings().then((result) => {
      if (live && result.status === 'ok') {
        setLanguage(result.data.language)
        apply(result.data.language)
      }
    })
    return () => {
      live = false
    }
  }, [])

  const change = useCallback(async (next: Language) => {
    setLanguage(next)
    apply(next)
    await commands.setUiSettings({ language: next })
  }, [])

  return { language, change }
}
