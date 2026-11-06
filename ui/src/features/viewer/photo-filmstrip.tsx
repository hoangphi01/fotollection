// SPDX-License-Identifier: GPL-3.0-only
import { Filmstrip } from '@/components/layout/filmstrip'
import { CullBadges } from '@/features/culling/cull-badges'
import { BasketBadge } from '@/features/library/basket-badge'
import { RawBadge } from '@/features/library/raw-badge'
import type { PhotoSummary } from '@/ipc/bindings'
import { formatCapture } from '@/lib/format'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

type PhotoFilmstripProps = {
  /** The view's photos, in the gallery's order and filter. */
  photos: PhotoSummary[]
  index: number | null
  thumbVersions: Record<number, number>
  generation: number
  onPick: (index: number) => void
}

/**
 * Pro mode's filmstrip (FR-090): the view's photos under the one shown, from their gallery
 * thumbnails (no new decode), with the tiles' rating, basket and RAW marks. A click goes the
 * way ← → go, so the photo picked develops first and the one left stops (FR-091).
 */
export function PhotoFilmstrip({ photos, index, thumbVersions, generation, onPick }: PhotoFilmstripProps) {
  return (
    <Filmstrip
      items={photos}
      current={index}
      onPick={(_, i) => onPick(i)}
      hint={(photo) => `${photo.fileName} · ${formatCapture(photo.capture)}`}
      cell={(photo) => (
        <>
          <img
            src={thumbUrl(photo.id, thumbVersions[photo.id] ?? 0, generation)}
            alt=""
            loading="lazy"
            decoding="async"
            draggable={false}
            style={photo.rotation ? { transform: `rotate(${photo.rotation * 90}deg)` } : undefined}
            className={cn('size-full object-cover', photo.flag === 'reject' && 'opacity-35')}
          />
          <CullBadges rating={photo.rating} flag={photo.flag} className="absolute top-1 left-1" />
          <span className="absolute top-1 right-1 flex gap-1">
            {photo.queue === 'print' && <BasketBadge />}
            {photo.raw && <RawBadge />}
          </span>
        </>
      )}
    />
  )
}
