// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, CopyCheck, MapPin } from 'lucide-react'
import { useEffect, useId, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { CaptionField } from '@/features/stamp/caption-field'
import { StampStyleFields } from '@/features/stamp/stamp-style-fields'
import { commands, type PhotoStamp, type StampSettings } from '@/ipc/bindings'

type PhotoStampControlsProps = {
  photoId: number
  /** Bumps on every edit and undo, so the controls follow the photo's stamp. */
  revision: number
  /** After "Apply to all" changed the shared settings. */
  onAppliedToAll(): void
}

/**
 * The open photo's own stamp (FR-044): changes here apply to this photo only; "Apply to
 * all" makes them the setting for every photo.
 */
export function PhotoStampControls({ photoId, revision, onAppliedToAll }: PhotoStampControlsProps) {
  const { t } = useTranslation()
  const id = useId()
  const [current, setCurrent] = useState<PhotoStamp | null>(null)
  // Local changes so far: an answer asked for before the latest one would put an older
  // stamp back on screen (STYLE_RULE §4).
  const changes = useRef(0)

  useEffect(() => {
    let live = true
    const asked = changes.current
    void commands.getPhotoStamp(photoId).then((result) => {
      if (live && result.status === 'ok' && asked === changes.current) setCurrent(result.data)
    })
    return () => {
      live = false
    }
  }, [photoId, revision])

  if (!current) return null
  const stamp = current.stamp
  const set = (patch: Partial<StampSettings>) => {
    const next = { ...stamp, ...patch }
    // Optimistic; Rust validates. An own stamp carries its line, so the own line goes.
    changes.current += 1
    setCurrent({ stamp: next, custom: true, caption: null })
    void commands.setPhotoStamp(photoId, next)
  }

  return (
    <SectionCard
      icon={<CalendarDays />}
      title={
        <Label htmlFor={id} className="text-sm font-semibold">
          {t('viewer.dateStamp')}
        </Label>
      }
      action={
        <>
          {current.custom && <Badge variant="outline">{t('viewer.thisPhoto')}</Badge>}
          <Switch id={id} checked={stamp.enabled} onCheckedChange={(enabled) => set({ enabled })} />
        </>
      }
      open={stamp.enabled}
      footer={
        <Button
          variant="outline"
          size="sm"
          className="w-full"
          onClick={() =>
            void commands.applyStampToAll(photoId).then((r) => r.status === 'ok' && onAppliedToAll())
          }
        >
          <CopyCheck data-icon="inline-start" /> {t('viewer.applyToAll')}
        </Button>
      }
    >
      <StampStyleFields stamp={stamp} onChange={set} />
      <SwitchField
        icon={<MapPin />}
        label={t('settings.location')}
        checked={stamp.place ?? false}
        onChange={(place) => set({ place })}
        hint={t('settings.locationHint')}
      />
      <CaptionField key={photoId} caption={stamp.caption ?? ''} onCommit={(caption) => set({ caption })} />
    </SectionCard>
  )
}
