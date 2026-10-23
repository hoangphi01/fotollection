// SPDX-License-Identifier: GPL-3.0-only
import { Plus, Trash2, Type } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { FontsApi } from './use-fonts'

const BUILT_IN = 'builtin'
const IMPORT = 'import'

type FontSelectProps = {
  fonts: FontsApi
  value: number | null | undefined
  onChange: (fontId: number | null) => void
  label: string
}

/** Font picker shared by the date stamp and the info frame (FR-042). */
export function FontSelect({ fonts, value, onChange, label }: FontSelectProps) {
  const { t } = useTranslation()
  const selected = value ? String(value) : BUILT_IN
  const current = fonts.fonts.find((f) => f.id === value)

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        <Select
          value={selected}
          onValueChange={(v) => {
            if (v === IMPORT) {
              void fonts.importFont().then((font) => font && onChange(font.id))
            } else {
              onChange(v === BUILT_IN ? null : Number(v))
            }
          }}
        >
          <SelectTrigger className="w-full" aria-label={label}>
            <Type className="size-3.5 text-muted-foreground" />
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={BUILT_IN}>{t('fonts.builtIn')}</SelectItem>
            {fonts.fonts.length > 0 && (
              <SelectGroup>
                <SelectLabel>{t('fonts.yours')}</SelectLabel>
                {fonts.fonts.map((f) => (
                  <SelectItem key={f.id} value={String(f.id)}>
                    {f.name} <span className="text-muted-foreground uppercase">· {f.format}</span>
                  </SelectItem>
                ))}
              </SelectGroup>
            )}
            <SelectSeparator />
            <SelectItem value={IMPORT}>
              <Plus className="size-3.5" /> {t('fonts.import')}
            </SelectItem>
          </SelectContent>
        </Select>
        {current && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t('fonts.removeNamed', { name: current.name })}
            title={t('fonts.removeTitle')}
            onClick={() => void fonts.removeFont(current.id)}
          >
            <Trash2 />
          </Button>
        )}
      </div>
      {fonts.error && <p className="text-xs text-destructive">{fonts.error}</p>}
    </div>
  )
}
