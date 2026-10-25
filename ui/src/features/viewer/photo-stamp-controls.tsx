// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, CopyCheck } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { SectionCard } from '@/components/layout/section-card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
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
  const [current, setCurrent] = useState<PhotoStamp | null>(null)

  useEffect(() => {
    let live = true
    void commands.getPhotoStamp(photoId).then((result) => {
      if (live && result.status === 'ok') setCurrent(result.data)
    })
    return () => {
      live = false
    }
  }, [photoId, revision])

  if (!current) return null
  const stamp = current.stamp
  const set = (patch: Partial<StampSettings>) => {
    const next = { ...stamp, ...patch }
    setCurrent({ stamp: next, custom: true }) // optimistic; Rust validates
    void commands.setPhotoStamp(photoId, next)
  }

  return (
    <SectionCard
      icon={<CalendarDays />}
      title={
        <Label htmlFor="photo-stamp" className="text-sm font-semibold">
          {t('viewer.dateStamp')}
        </Label>
      }
      action={
        <>
          {current.custom && <Badge variant="outline">{t('viewer.thisPhoto')}</Badge>}
          <Switch id="photo-stamp" checked={stamp.enabled} onCheckedChange={(enabled) => set({ enabled })} />
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
      <div className="flex items-center justify-between">
        <Label htmlFor="photo-stamp-place" className="text-sm font-normal">
          {t('settings.location')}
        </Label>
        <Switch id="photo-stamp-place" checked={stamp.place ?? false} onCheckedChange={(place) => set({ place })} />
      </div>
      <CommitInput
        key={photoId}
        value={stamp.caption ?? ''}
        placeholder={t('settings.captionPlaceholder')}
        maxLength={80}
        liveMs={300}
        ariaLabel={t('settings.caption')}
        onCommit={(caption) => set({ caption })}
      />
    </SectionCard>
  )
}
