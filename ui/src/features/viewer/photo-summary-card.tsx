// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDown, Info } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FactLines } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import { Badge } from '@/components/ui/badge'
import type { PhotoDetails, PhotoSummary } from '@/ipc/bindings'
import { factsParts } from '@/lib/facts'
import { formatBytes, formatCaptureLong, formatMegapixels } from '@/lib/format'
import { cn } from '@/lib/utils'

/** Whether the card is unfolded, kept while the app runs, so ← → and the next preview keep it. */
let remembered = true

type PhotoSummaryCardProps = {
  photo: PhotoSummary
  /** `null` while the first answer loads; the last photo's holds its place after that. */
  details: PhotoDetails | null
}

/**
 * The photo at a glance, at the top of the preview (FR-078): the camera and lens and the
 * exposure as the info frame prints them (the camera profile's names first), where and when
 * it was taken, its size, and what the file is (FR-071). Its title folds it away. The cards
 * further down change the date and the place.
 */
export function PhotoSummaryCard({ photo, details }: PhotoSummaryCardProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(remembered)
  const toggle = () => {
    remembered = !open
    setOpen(!open)
  }
  const camera = details?.cameraName ?? photo.camera ?? t('viewer.unknownCamera')
  const lines = [
    [camera, details?.lensName].filter(Boolean).join(' · '),
    (details?.exposure ?? []).join(' · '),
    [details?.place?.label, formatCaptureLong(photo.capture)].filter(Boolean).join(' · '),
    details
      ? `${details.width} × ${details.height} px · ${formatMegapixels(details.width, details.height)} MP · ${formatBytes(details.fileBytes ?? 0)}`
      : t('common.loading'),
  ].filter(Boolean)
  // RAW or not at a glance (owner, 2026-11-05), as the grid's tile says it.
  const raw = photo.raw
  const ext = photo.fileName.split('.').pop()?.toUpperCase() ?? ''
  const kind = raw ? 'RAW' : ext === 'JPG' ? 'JPEG' : ext
  return (
    <SectionCard
      icon={<Info className="size-4" />}
      open={open}
      action={
        kind && (
          // Capitals have no descenders, so centring the line box sets them 1.6 px high
          // (measured): a one-line-high box, 1 px lower, centres the letters themselves.
          <Badge variant={raw ? 'default' : 'outline'} className="pt-px pb-0 leading-none" data-hint={raw ? ext : undefined}>
            {kind}
          </Badge>
        )
      }
      title={
        <button
          type="button"
          className="flex w-full items-center gap-1.5 text-left"
          aria-expanded={open}
          onClick={toggle}
        >
          {t('viewer.summary')}
          <ChevronDown
            className={cn('size-4 text-muted-foreground transition-transform', !open && '-rotate-90')}
          />
        </button>
      }
    >
      <FactLines>
        {lines.map((line, i) => (
          <p key={line} className={i === 0 ? 'font-medium' : 'text-xs text-muted-foreground tabular-nums'}>
            {line}
          </p>
        ))}
        {/* Recorded screenshot data from before FR-071 has no facts. */}
        {details?.facts && (
          <p className="text-xs text-muted-foreground">
            {/* A line breaks between facts, never inside one ("zune-jpeg"). */}
            {factsParts(details.facts, photo.fileName).map((part, i) => (
              <span key={part}>
                {i > 0 && ' · '}
                <span className="whitespace-nowrap">{part}</span>
              </span>
            ))}
          </p>
        )}
      </FactLines>
    </SectionCard>
  )
}
