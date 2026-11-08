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
  /** Only the header line: the strip hidden (FR-090). */
  collapsed?: boolean
}

/**
 * Pro mode's filmstrip (FR-090): the view's photos under the one shown, from their gallery
 * thumbnails (no new decode), with the tiles' rating, basket and RAW marks. A click goes the
 * way ← → go, so the photo picked develops first and the one left stops (FR-091).
 */
export function PhotoFilmstrip({ photos, index, thumbVersions, generation, onPick, collapsed = false }: PhotoFilmstripProps) {
  const shown = index === null ? undefined : photos[index]
  return (
    <Filmstrip
      collapsed={collapsed}
      // Where the photo is in the set, and its name, as Lightroom's strip says it (FR-095).
      header={
        <>
          <span>
            {(index ?? 0) + 1} / {photos.length}
          </span>
          {shown && <span className="truncate text-stage-ink" data-hint={shown.fileName}>· {shown.fileName}</span>}
        </>
      }
      items={photos}
      current={index}
      onPick={(_, i) => onPick(i)}
      hint={(photo) => `${photo.fileName} · ${formatCapture(photo.capture)}`}
      cell={(photo) => (
        <>
          <img
            src={thumbUrl(photo.id, thumbVersions[photo.id] ?? 0, generation, photo.look)}
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
