// SPDX-License-Identifier: GPL-3.0-only
import { PanelBottom } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Collapse } from '@/components/layout/collapse'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
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

const FIELDS: { key: keyof FrameFields; disabled?: boolean }[] = [
  { key: 'logo' },
  { key: 'camera' },
  { key: 'lens' },
  { key: 'focal' },
  { key: 'aperture' },
  { key: 'shutter' },
  { key: 'iso' },
  { key: 'photoStyle' },
  { key: 'date' },
  // Needs offline place names (FR-005).
  { key: 'place', disabled: true },
]

type FrameSectionProps = {
  frame: FrameConfigApi
  fonts: FontsApi
  /** The preview inspector hides the heading: its section title already says "Frame". */
  compact?: boolean
}

/**
 * "Shot on …" info frame settings, LUMIX Lab style (FR-040). Rendered both in the print
 * settings panel and in the preview inspector; both edit the same Rust state.
 */
export function FrameSection({ frame, fonts, compact = false }: FrameSectionProps) {
  const { t } = useTranslation()
  const settings = frame.config?.settings
  const enabled = settings?.enabled ?? false
  // Two instances can be on screen at once, so DOM ids must be unique.
  const switchId = useId()

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <Label htmlFor={switchId} className="flex items-center gap-2">
          {!compact && <PanelBottom className="size-4 text-muted-foreground" />}
          {t('frame.title')}
        </Label>
        <Switch
          id={switchId}
          checked={enabled}
          onCheckedChange={(on) => void frame.setSettings((s) => ({ ...s, enabled: on }))}
        />
      </div>

      <Collapse open={enabled} className="space-y-3">
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={settings?.fit ?? ''}
          onValueChange={(fit) =>
            fit && void frame.setSettings((s) => ({ ...s, fit: fit as FrameFit }))
          }
        >
          <ToggleGroupItem value="crop" className="flex-1 text-xs">
            {t('frame.cropToFill')}
          </ToggleGroupItem>
          <ToggleGroupItem value="fit" className="flex-1 text-xs">
            {t('frame.fitWhole')}
          </ToggleGroupItem>
        </ToggleGroup>

        <div className="flex gap-2">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            className="flex-1"
            value={settings?.color ?? ''}
            onValueChange={(color) =>
              color && void frame.setSettings((s) => ({ ...s, color: color as FrameColor }))
            }
          >
            <ToggleGroupItem value="white" className="flex-1 gap-1.5 text-xs">
              <span className="size-3 rounded-full border bg-white" /> {t('frame.white')}
            </ToggleGroupItem>
            <ToggleGroupItem value="black" className="flex-1 gap-1.5 text-xs">
              <span className="size-3 rounded-full border bg-neutral-900" /> {t('frame.black')}
            </ToggleGroupItem>
          </ToggleGroup>
        </div>

        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={settings?.dateFormat ?? ''}
          onValueChange={(f) =>
            f && void frame.setSettings((s) => ({ ...s, dateFormat: f as FrameDateFormat }))
          }
        >
          <ToggleGroupItem value="slash" className="flex-1 font-mono text-[11px]">
            2026/10/18
          </ToggleGroupItem>
          <ToggleGroupItem value="iso" className="flex-1 font-mono text-[11px]">
            2026-10-18
          </ToggleGroupItem>
          <ToggleGroupItem value="dmy" className="flex-1 font-mono text-[11px]">
            18.10.2026
          </ToggleGroupItem>
        </ToggleGroup>

        <FontSelect
          fonts={fonts}
          label={t('frame.font')}
          value={settings?.font}
          onChange={(font) => void frame.setSettings((s) => ({ ...s, font }))}
        />

        <div className="space-y-1.5">
          <span className="text-xs font-medium text-muted-foreground uppercase">
            {t('frame.show')}
          </span>
          <div className="grid grid-cols-2 gap-x-3 gap-y-2">
            {FIELDS.map(({ key, disabled = false }) => (
              <label
                key={key}
                data-hint={disabled ? t('frame.placeHint') : undefined}
                className="flex items-center gap-2 text-sm"
                aria-disabled={disabled}
              >
                <Checkbox
                  checked={settings?.fields[key] ?? false}
                  disabled={disabled}
                  onCheckedChange={(checked) =>
                    void frame.setSettings((s) => ({
                      ...s,
                      fields: { ...s.fields, [key]: checked === true },
                    }))
                  }
                />
                <span className={disabled ? 'text-muted-foreground' : ''}>
                  {t(`frame.field.${key}`)}
                </span>
              </label>
            ))}
          </div>
        </div>
        {/* Colours of the bundled brand logos (ADR-0020), while the logo is shown. */}
        <Collapse open={settings?.fields.logo ?? false} className="space-y-1.5">
          <span className="text-xs font-medium text-muted-foreground uppercase">
            {t('frame.logoStyle')}
          </span>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            className="w-full"
            value={settings?.logoStyle ?? ''}
            onValueChange={(v) =>
              v && void frame.setSettings((s) => ({ ...s, logoStyle: v as LogoStyle }))
            }
          >
            {LOGO_STYLES.map((style) => (
              <ToggleGroupItem key={style} value={style} className="flex-1 text-xs">
                {t(`frame.logoStyles.${style}`)}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <LogoScaleSlider
            value={settings?.logoScale ?? 1}
            onCommit={(logoScale) => void frame.setSettings((s) => ({ ...s, logoScale }))}
          />
        </Collapse>
        <p className="text-xs text-muted-foreground">
          {t('frame.keepsSize')}
          {!compact && ` ${t('frame.editInPreview')}`}
        </p>
      </Collapse>
    </section>
  )
}
