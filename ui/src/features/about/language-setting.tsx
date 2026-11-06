// SPDX-License-Identifier: GPL-3.0-only
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useLanguage } from '@/features/settings/use-language'
import { LANGUAGES } from '@/i18n'
import type { Language } from '@/ipc/bindings'

/** The interface language (FR-020), in the Settings window's *General* section (FR-086). */
export function LanguageSetting() {
  const { t } = useTranslation()
  const { language, change } = useLanguage()
  const id = useId()
  return (
    <Field label={t('settings.language')} htmlFor={id}>
      <Select value={language} onValueChange={(v) => void change(v as Language)}>
        <SelectTrigger id={id} className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {LANGUAGES.map((l) => (
            <SelectItem key={l.value} value={l.value}>
              {l.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </Field>
  )
}
