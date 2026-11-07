// SPDX-License-Identifier: GPL-3.0-only
import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
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
import { RemoveButton } from '@/components/forms/danger-button'

const BUILT_IN = 'builtin'
const IMPORT = 'import'

/** A face bundled with the app, by its own key (a postcard's typewriter or handwriting). */
/** `hint`: the typeface's own name (Xanh Mono), shown on hover. */
export type BuiltInFace = { key: string; label: string; hint?: string }

type FontSelectProps = {
  fonts: FontsApi
  value: number | null | undefined
  /** A user font's id, or `null` and the bundled face's key when one of `builtIns` is chosen. */
  onChange: (fontId: number | null, builtIn?: string) => void
  label: string
  /** The bundled faces, when they are not just Inter (a postcard's typewriter and handwriting). */
  builtIns?: readonly BuiltInFace[]
  /** Which of `builtIns` is in use while no user font is. */
  builtIn?: string
}

/** Font picker shared by the info frame and the postcard (FR-042, FR-073). */
export function FontSelect({ fonts, value, onChange, label, builtIns, builtIn }: FontSelectProps) {
  const { t } = useTranslation()
  const faces = builtIns ?? [{ key: BUILT_IN, label: t('fonts.builtIn') }]
  const builtInValue = (key: string) => `${BUILT_IN}:${key}`
  const selected = value ? String(value) : builtInValue(builtIn ?? faces[0]?.key ?? BUILT_IN)
  const current = fonts.fonts.find((f) => f.id === value)

  return (
    <Field hint={fonts.error && <span className="text-destructive">{fonts.error}</span>}>
      <div className="flex items-center gap-2">
        <Select
          value={selected}
          onValueChange={(v) => {
            if (v === IMPORT) {
              void fonts.importFont().then((font) => font && onChange(font.id))
            } else if (v.startsWith(`${BUILT_IN}:`)) {
              onChange(null, v.slice(BUILT_IN.length + 1))
            } else {
              onChange(Number(v))
            }
          }}
        >
          {/* The face's short name alone, so it fits an 18 rem panel (STYLE_RULE §4: icons
              first, a label never cut); the typeface's own name is the hint. */}
          <SelectTrigger className="w-full" aria-label={label} data-hint={faces.find((f) => builtInValue(f.key) === selected)?.hint}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {faces.map((face) => (
              <SelectItem key={face.key} value={builtInValue(face.key)} data-hint={face.hint}>
                {face.label}
              </SelectItem>
            ))}
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
              <Plus /> {t('fonts.import')}
            </SelectItem>
          </SelectContent>
        </Select>
        {current && (
          <RemoveButton
            kind="delete"
            iconOnly
            label={t('fonts.removeNamed', { name: current.name })}
            hint={t('fonts.removeTitle')}
            onClick={() => void fonts.removeFont(current.id)}
          />
        )}
      </div>
    </Field>
  )
}
