// SPDX-License-Identifier: GPL-3.0-only
import { Proportions } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { RenderSettings, ShareFill } from '@/ipc/bindings'

/** The ratios people post at; anything else is Custom. Rust keeps a ratio to 1–100 a side. */
const RATIOS = [
  { value: '1:1', key: '1x1' },
  { value: '4:5', key: '4x5' },
  { value: '3:4', key: '3x4' },
  { value: '2:3', key: '2x3' },
  { value: '3:2', key: '3x2' },
  { value: '16:9', key: '16x9' },
  { value: '9:16', key: '9x16' },
] as const
const WHITE = '#ffffff'
const BLACK = '#000000'
/** A warm paper tone, the first custom colour offered. */
const CUSTOM_DEFAULT = '#efe9dd'

const DEFAULT_FILL: Required<ShareFill> = { enabled: false, ratio: [1, 1], color: [255, 255, 255] }

const toHex = ([r, g, b]: readonly number[]) =>
  `#${[r, g, b].map((v) => Math.max(0, Math.min(255, v ?? 0)).toString(16).padStart(2, '0')).join('')}`

const fromHex = (hex: string): [number, number, number] | null => {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim())
  if (!m?.[1]) return null
  const n = Number.parseInt(m[1], 16)
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}

type ShareFillFieldsProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  disabled?: boolean
}

/**
 * Each shared photo whole, centred in a frame of the chosen ratio, the rest filled with white,
 * black or a colour of the user's (FR-075): a portrait on a square feed without cropping it.
 * Under the share size, so the panel, Simple mode and the preview show it alike.
 */
export function ShareFillFields({ settings, onChange, disabled = false }: ShareFillFieldsProps) {
  const { t } = useTranslation()
  const fill = { ...DEFAULT_FILL, ...settings?.shareFill }
  const set = (change: Partial<ShareFill>) =>
    onChange((s) => ({ ...s, shareFill: { ...DEFAULT_FILL, ...s.shareFill, ...change } }))

  const ratio = `${fill.ratio[0]}:${fill.ratio[1]}`
  const preset = RATIOS.some((r) => r.value === ratio)
  const [customRatio, setCustomRatio] = useState(!preset)
  const ratioValue = customRatio || !preset ? 'custom' : ratio
  const setSide = (side: 0 | 1, v: string) => {
    const n = Number.parseInt(v, 10)
    if (!Number.isFinite(n)) return
    const next: [number, number] = [...fill.ratio]
    next[side] = n
    set({ ratio: next })
  }

  const hex = toHex(fill.color)
  const colorMode = hex === WHITE ? 'white' : hex === BLACK ? 'black' : 'custom'
  const [customColor, setCustomColor] = useState(colorMode === 'custom' ? hex : CUSTOM_DEFAULT)
  const setColor = (next: string) => {
    const rgb = fromHex(next)
    if (!rgb) return
    if (next !== WHITE && next !== BLACK) setCustomColor(next)
    set({ color: rgb })
  }

  return (
    <SwitchField
      icon={<Proportions />}
      label={t('export.fill.label')}
      checked={fill.enabled}
      disabled={disabled}
      onChange={(enabled) => set({ enabled })}
      hint={t('export.fill.hint')}
    >
        <div className="flex gap-2">
          <Select
            value={ratioValue}
            disabled={disabled}
            onValueChange={(v) => {
              if (v === 'custom') {
                setCustomRatio(true)
                return
              }
              setCustomRatio(false)
              const [w, h] = v.split(':').map(Number)
              set({ ratio: [w ?? 1, h ?? 1] })
            }}
          >
            <SelectTrigger className="min-w-0 flex-1" aria-label={t('export.fill.ratio')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {RATIOS.map((r) => (
                <SelectItem key={r.value} value={r.value}>
                  {t(`export.fill.ratios.${r.key}`)}
                </SelectItem>
              ))}
              <SelectItem value="custom">{t('export.fill.ratios.custom')}</SelectItem>
            </SelectContent>
          </Select>
          {ratioValue === 'custom' && (
            <div className="flex w-28 items-center gap-2">
              <CommitInput
                value={String(fill.ratio[0])}
                placeholder="4"
                numeric
                ariaLabel={t('export.fill.width')}
                onCommit={(v) => setSide(0, v)}
              />
              <span className="text-xs text-muted-foreground">:</span>
              <CommitInput
                value={String(fill.ratio[1])}
                placeholder="5"
                numeric
                ariaLabel={t('export.fill.height')}
                onCommit={(v) => setSide(1, v)}
              />
            </div>
          )}
        </div>
        <SwatchToggle
          ariaLabel={t('export.fill.color')}
          disabled={disabled}
          value={colorMode}
          options={[
            { value: 'white', swatch: 'bg-white', label: t('export.fill.colors.white') },
            { value: 'black', swatch: 'bg-black', label: t('export.fill.colors.black') },
            { value: 'custom', swatch: customColor, label: t('export.fill.colors.custom') },
          ]}
          onChange={(mode) => setColor(mode === 'white' ? WHITE : mode === 'black' ? BLACK : customColor)}
        />
        {colorMode === 'custom' && (
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={hex}
              disabled={disabled}
              aria-label={t('export.fill.color')}
              onChange={(e) => setColor(e.target.value)}
              className="h-8 w-10 shrink-0 cursor-pointer rounded-md border bg-transparent p-0.5"
            />
            <CommitInput
              value={hex}
              placeholder={CUSTOM_DEFAULT}
              maxLength={7}
              ariaLabel={t('export.fill.hex')}
              onCommit={setColor}
            />
          </div>
        )}
    </SwitchField>
  )
}
