// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { PanelBottom } from 'lucide-react'
import { useId, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
import { SwatchToggle } from '@/components/forms/swatch-toggle'
import { Field, Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import type {
  FrameColor,
  FrameDateFormat,
  FrameFields,
  FrameFit,
  LogoStyle,
} from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import type { FontsApi } from '@/features/fonts/use-fonts'
import { LogoScaleSlider } from './logo-scale-slider'
import type { FrameConfigApi } from './use-frame-config'

const LOGO_STYLES: LogoStyle[] = ['classic', 'original', 'mono']

const FIELDS: { key: keyof FrameFields }[] = [
  { key: 'logo' },
  { key: 'camera' },
  { key: 'lens' },
  { key: 'focal' },
  { key: 'aperture' },
  { key: 'shutter' },
  { key: 'iso' },
  { key: 'photoStyle' },
  { key: 'date' },
  { key: 'place' },
]

type FrameSectionProps = {
  frame: FrameConfigApi
  fonts: FontsApi
  /** In the preview: no "edit in the preview" hint. */
  compact?: boolean
  /** Extra controls at the end of the card, such as the preview's camera profile. */
  children?: ReactNode
}

/**
 * "Shot on …" info frame settings, LUMIX Lab style (FR-040). Rendered both in the print
 * settings panel and in the preview inspector; both edit the same Rust state.
 */
export function FrameSection({ frame, fonts, compact = false, children }: FrameSectionProps) {
  const { t } = useTranslation()
  const settings = frame.config?.settings
  const enabled = settings?.enabled ?? false
  // Two instances can be on screen at once, so DOM ids must be unique.
  const switchId = useId()

  return (
    <SectionCard
      icon={<PanelBottom />}
      title={
        <Label htmlFor={switchId} className="text-sm font-semibold">
          {t('frame.title')}
        </Label>
      }
      action={
        <Switch
          id={switchId}
          checked={enabled}
          onCheckedChange={(on) => void frame.setSettings((s) => ({ ...s, enabled: on }))}
        />
      }
      open={enabled}
    >
      <Field hint={t(settings?.fit === 'fit' ? 'frame.fitHint' : 'frame.cropHint')}>
        <SegmentedToggle<FrameFit>
          value={settings?.fit}
          options={[
            { value: 'crop', label: t('frame.cropToFill') },
            { value: 'fit', label: t('frame.fitWhole') },
          ]}
          onChange={(fit) => void frame.setSettings((s) => ({ ...s, fit }))}
        />
      </Field>

      <SwatchToggle<FrameColor>
        value={settings?.color}
        options={[
          { value: 'white', swatch: 'bg-white', label: t('frame.white') },
          { value: 'black', swatch: 'bg-black', label: t('frame.black') },
        ]}
        onChange={(color) => void frame.setSettings((s) => ({ ...s, color }))}
      />

      <SegmentedToggle<FrameDateFormat>
        mono
        value={settings?.dateFormat}
        options={[
          { value: 'slash', label: '2026/10/18' },
          { value: 'iso', label: '2026-10-18' },
          { value: 'dmy', label: '18.10.2026' },
        ]}
        onChange={(dateFormat) => void frame.setSettings((s) => ({ ...s, dateFormat }))}
        ariaLabel={t('settings.dateFormat')}
      />

        <FontSelect
          fonts={fonts}
          label={t('frame.font')}
          value={settings?.font}
          onChange={(font) => void frame.setSettings((s) => ({ ...s, font }))}
        />

      <Field label={t('frame.show')}>
        <div className="grid grid-cols-2 gap-x-3 gap-y-3">
          {FIELDS.map(({ key }) => (
            <label key={key} className="flex items-center gap-2 text-sm">
              <Checkbox
                checked={settings?.fields[key] ?? false}
                onCheckedChange={(checked) =>
                  void frame.setSettings((s) => ({
                    ...s,
                    fields: { ...s.fields, [key]: checked === true },
                  }))
                }
              />
              <span>{t(`frame.field.${key}`)}</span>
            </label>
          ))}
        </div>
      </Field>
      {/* Colours of the bundled brand logos (ADR-0020), while the logo is shown. */}
      <Collapse open={settings?.fields.logo ?? false}>
        <Field label={t('frame.logoStyle')}>
          <SegmentedToggle<LogoStyle>
            value={settings?.logoStyle}
            options={LOGO_STYLES.map((style) => ({ value: style, label: t(`frame.logoStyles.${style}`) }))}
            onChange={(logoStyle) => void frame.setSettings((s) => ({ ...s, logoStyle }))}
            ariaLabel={t('frame.logoStyle')}
          />
          <LogoScaleSlider
            value={settings?.logoScale ?? 1}
            onCommit={(logoScale) => void frame.setSettings((s) => ({ ...s, logoScale }))}
          />
        </Field>
      </Collapse>
      <Hint>
        {t('frame.keepsSize')}
        {!compact && ` ${t('frame.editInPreview')}`}
      </Hint>
      {children}
    </SectionCard>
  )
}
