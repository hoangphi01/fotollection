// SPDX-License-Identifier: GPL-3.0-only
import { CircleCheck } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoSummary } from '@/ipc/bindings'
import { previewUrl, thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

type QuickPreviewProps = {
  photo: PhotoSummary
  revision: number
  thumbVersion: number
  generation: number
  checked: boolean
  onClose: () => void
}

/**
 * Quick-preview (FR-059): the photo as it will be exported, with the gallery's print size,
 * stamp and frame, and nothing to edit. The thumbnail shows at once; the render replaces it
 * when it arrives.
 */
export function QuickPreview({ photo, revision, thumbVersion, generation, checked, onClose }: QuickPreviewProps) {
  const { t } = useTranslation()
  const src = previewUrl(photo.id, revision)
  const [ready, setReady] = useState<string | null>(null)
  useEffect(() => {
    let live = true
    const img = new Image()
    img.onload = () => live && setReady(src)
    img.src = src
    return () => {
      live = false
    }
  }, [src])

  return (
    <div
      className="fixed inset-0 z-50 flex animate-in flex-col items-center justify-center gap-3 bg-black/85 p-10 fade-in-0"
      onClick={onClose}
      role="presentation"
    >
      <div className="relative flex min-h-0 flex-1 items-center justify-center">
        {ready === src ? (
          <img src={src} alt={photo.fileName} className="max-h-full max-w-full rounded-sm object-contain shadow-2xl" />
        ) : (
          <img
            src={thumbUrl(photo.id, thumbVersion, generation)}
            alt={photo.fileName}
            style={photo.rotation ? { transform: `rotate(${photo.rotation * 90}deg)` } : undefined}
            className="max-h-full max-w-full rounded-sm object-contain opacity-80 shadow-2xl blur-[1px]"
          />
        )}
      </div>
      <p className="flex items-center gap-2 text-sm text-white/80">
        <span className="truncate">{photo.fileName}</span>
        <span
          className={cn(
            'flex items-center gap-1 rounded-full px-2 py-0.5 text-xs',
            checked ? 'bg-primary text-primary-foreground' : 'bg-white/15',
          )}
        >
          <CircleCheck className="size-3.5" /> {t(checked ? 'select.checked' : 'select.notChecked')}
        </span>
        <span className="text-xs text-white/50">{t('select.quickHint')}</span>
      </p>
    </div>
  )
}
