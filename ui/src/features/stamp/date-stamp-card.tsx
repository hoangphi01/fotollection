// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, RotateCcw } from 'lucide-react'
import { useId, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { SectionCard } from '@/components/layout/section-card'
import { commands, type RenderSettings, type StampSettings } from '@/ipc/bindings'
import { StampSizeField } from './stamp-size-field'
import { StampStyleFields } from './stamp-style-fields'

type DateStampCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** Every style field, or the size only (Simple mode, FR-048). */
  fields?: 'all' | 'size'
  /** Photos with their own stamp from the preview (FR-044): they offer a reset. */
  customStamps: number
  /** More fields under the style ones (Pro's place and caption). */
  children?: ReactNode
}

/**
 * The date stamp card of every mode (FR-004): its switch, its fields and, when some photos
 * have their own stamp, the reset that brings them back to it, so a change here reaches
 * every photo (STYLE_RULE §4, same tools, same order).
 */
export function DateStampCard({ settings, onChange, fields = 'all', customStamps, children }: DateStampCardProps) {
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
      {fields === 'all' ? (
        <StampStyleFields stamp={stamp} onChange={setStamp} />
      ) : (
        <StampSizeField stamp={stamp} onChange={setStamp} />
      )}
      {children}
    </SectionCard>
  )
}
