// SPDX-License-Identifier: GPL-3.0-only
import { Camera, FolderInput, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

type CameraSessionBannerProps = {
  count: number
  onSave: () => void
  onDiscard: () => void
}

/** Over a camera session's tab (ADR-0028): its shots are in a temporary folder until saved. */
export function CameraSessionBanner({ count, onSave, onDiscard }: CameraSessionBannerProps) {
  const { t } = useTranslation()
  return (
    <Alert className="m-3 mb-0 w-auto">
      <Camera />
      <AlertDescription className="flex flex-wrap items-center gap-3">
        <span className="min-w-0 flex-1">{t('capture.sessionBanner', { count })}</span>
        <span className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={onDiscard}>
            <Trash2 data-icon="inline-start" /> {t('capture.discard')}
          </Button>
          <Button size="sm" onClick={onSave}>
            <FolderInput data-icon="inline-start" /> {t('capture.saveAll')}
          </Button>
        </span>
      </AlertDescription>
    </Alert>
  )
}
