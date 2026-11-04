// SPDX-License-Identifier: GPL-3.0-only
import { Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { InspectorSection } from '@/components/layout/inspector-section'
import type { PhotoDetails, PhotoSummary } from '@/ipc/bindings'
import { formatBytes, formatCaptureLong, formatMegapixels } from '@/lib/format'

type PhotoSummaryCardProps = {
  photo: PhotoSummary
  /** `null` while the first answer loads; the last photo's holds its place after that. */
  details: PhotoDetails | null
}

/**
 * The photo at a glance, at the top of the preview (FR-078): the camera and lens and the
 * exposure as the info frame prints them (the camera profile's names first), where and when
 * it was taken, and its size. The cards further down change the date and the place.
 */
export function PhotoSummaryCard({ photo, details }: PhotoSummaryCardProps) {
  const { t } = useTranslation()
  const camera = details?.cameraName ?? photo.camera
  const lines = [
    [camera, details?.lensName].filter(Boolean).join(' · '),
    (details?.exposure ?? []).join(' · '),
    [details?.place?.label, formatCaptureLong(photo.capture)].filter(Boolean).join(' · '),
    details
      ? `${details.width} × ${details.height} · ${formatMegapixels(details.width, details.height)} MP · ${formatBytes(details.fileBytes ?? 0)}`
      : '',
  ].filter(Boolean)
  return (
    <InspectorSection icon={<Info className="size-4" />} title={t('viewer.summary')}>
      {lines.map((line, i) => (
        <p key={line} className={i === 0 ? 'font-medium' : 'text-xs text-muted-foreground tabular-nums'}>
          {line}
        </p>
      ))}
    </InspectorSection>
  )
}
