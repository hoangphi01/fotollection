// SPDX-License-Identifier: GPL-3.0-only
import { CopyCheck } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Collapse } from '@/components/layout/collapse'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { commands, type DateFormat, type PhotoStamp, type StampSettings } from '@/ipc/bindings'

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

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
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <Label htmlFor="photo-stamp" className="text-sm font-normal">
          {t('viewer.showOnPrint')}
        </Label>
        <div className="flex items-center gap-2">
          {current.custom && <Badge variant="outline">{t('viewer.thisPhoto')}</Badge>}
          <Switch
            id="photo-stamp"
            checked={stamp.enabled}
            onCheckedChange={(enabled) => set({ enabled })}
          />
        </div>
      </div>
      <Collapse open={stamp.enabled} className="space-y-3">
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={stamp.format}
          onValueChange={(format) => format && set({ format: format as DateFormat })}
        >
          {FORMATS.map((f) => (
            <ToggleGroupItem key={f.value} value={f.value} className="flex-1 font-mono text-[11px]">
              {f.label}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
        <div className="flex items-center justify-between">
          <Label htmlFor="photo-stamp-place" className="text-sm font-normal">
            {t('settings.location')}
          </Label>
          <Switch
            id="photo-stamp-place"
            checked={stamp.place ?? false}
            onCheckedChange={(place) => set({ place })}
          />
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
      </Collapse>
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
    </div>
  )
}
