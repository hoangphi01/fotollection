// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CalendarDays, MapPin, RotateCcw } from 'lucide-react'
import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Hint } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type PhotoId, type PlaceFormat, type RenderSettings, type StampSettings } from '@/ipc/bindings'
import { CaptionField } from './caption-field'
import { StampStyleFields } from './stamp-style-fields'

const PLACE_FORMATS: PlaceFormat[] = ['city', 'cityCountryCode', 'cityCountry']

type DateStampCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** Photos with their own stamp from the preview (FR-044): they offer a reset. */
  customStamps: number
  /** The photos of an open collage sheet: each may have its own line under the date. */
  photos?: { id: PhotoId; name: string }[]
  /** Bumps on every edit and undo, for those photos' own lines. */
  revision?: number
}

/**
 * The date stamp card of every mode (FR-004): its switch, its style fields, the place
 * (FR-005) and the caption (FR-036) and, when some photos have their own stamp, the reset
 * that brings them back to it. One card, so every mode offers the same stamp (STYLE_RULE §4,
 * same tools, same order).
 */
export function DateStampCard({ settings, onChange, customStamps, photos, revision }: DateStampCardProps) {
  const { t } = useTranslation()
  const id = useId()
  const stamp = settings?.stamp
  const setStamp = (patch: Partial<StampSettings>) => onChange((s) => ({ ...s, stamp: { ...s.stamp, ...patch } }))
  return (
    <SectionCard
      icon={<CalendarDays />}
      title={
        <Label htmlFor={id} className="text-sm font-semibold">
          {t('settings.dateStamp')}
        </Label>
      }
      action={<Switch id={id} checked={stamp?.enabled ?? false} onCheckedChange={(enabled) => setStamp({ enabled })} />}
      open={stamp?.enabled ?? false}
      footer={
        customStamps > 0 && (
          <Button variant="outline" size="sm" className="w-full" onClick={() => void commands.resetPhotoStamps()}>
            <RotateCcw data-icon="inline-start" />
            {t('settings.resetCustomStamps', { count: customStamps })}
          </Button>
        )
      }
    >
      <StampStyleFields stamp={stamp} onChange={setStamp} />

      {/* The place on the date line (FR-005), from the photo's GPS. */}
      <SwitchField
        icon={<MapPin />}
        label={t('settings.location')}
        checked={stamp?.place ?? false}
        onChange={(place) => setStamp({ place })}
      >
        <SegmentedToggle
          value={settings?.placeFormat}
          options={PLACE_FORMATS.map((f) => ({ value: f, label: t(`settings.placeFormat.${f}`) }))}
          onChange={(placeFormat) => onChange((s) => ({ ...s, placeFormat }))}
          ariaLabel={t('settings.location')}
        />
        <Hint>{t('settings.locationHint')}</Hint>
      </SwitchField>

      {/* A line of the user's own text under the date (FR-036). */}
      <CaptionField
        caption={stamp?.caption ?? ''}
        onCommit={(caption) => setStamp({ caption })}
        photos={photos}
        revision={revision}
      />
    </SectionCard>
  )
}
