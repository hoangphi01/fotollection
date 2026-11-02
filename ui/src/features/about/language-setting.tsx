// SPDX-License-Identifier: GPL-3.0-only
import { Languages } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useLanguage } from '@/features/settings/use-language'
import { LANGUAGES } from '@/i18n'
import type { Language } from '@/ipc/bindings'

/**
 * The interface language (FR-020), in the version popover so every mode reaches it without a
 * card of its own in the panel.
 */
export function LanguageSetting() {
  const { t } = useTranslation()
  const { language, change } = useLanguage()
  return (
    <div className="flex items-center gap-2 border-t px-4 py-3 text-xs">
      <Languages className="size-3.5 text-muted-foreground" />
      <Label htmlFor="ui-language" className="text-xs font-normal">
        {t('settings.language')}
      </Label>
      <Select value={language} onValueChange={(v) => void change(v as Language)}>
        <SelectTrigger id="ui-language" size="sm" className="ml-auto w-auto gap-1 px-2 text-xs">
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
    </div>
  )
}
