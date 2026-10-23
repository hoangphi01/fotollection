// SPDX-License-Identifier: GPL-3.0-only
import { CalendarX2, LoaderCircle, MapPinOff } from 'lucide-react'
import { memo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoSummary } from '@/ipc/bindings'
import { CullBadges } from '@/features/culling/cull-badges'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

type ThumbTileProps = {
  photo: PhotoSummary
  version: number
  size: number
  selected: boolean
  onSelect: () => void
  onOpen: () => void
}

function formatCapture(capture: string): string {
  return capture.replace('T', ' ').slice(0, 16)
}

/** One grid cell: skeleton + spinner until the thumbnail arrives, then a fade-in. */
export const ThumbTile = memo(function ThumbTile({
  photo,
  version,
  size,
  selected,
  onSelect,
  onOpen,
}: ThumbTileProps) {
  const { t } = useTranslation()
  // Stays true across T0 → T1 upgrades, so the sharper image swaps in without flashing.
  const [loaded, setLoaded] = useState(false)

  return (
    <button
      type="button"
      title={`${photo.fileName} · ${formatCapture(photo.capture)}`}
      onClick={onSelect}
      onDoubleClick={onOpen}
      className={cn(
        'group relative shrink-0 overflow-hidden rounded-md bg-muted outline-none',
        selected && 'ring-2 ring-primary ring-offset-2 ring-offset-background',
      )}
      style={{ width: size, height: size }}
    >
      {!loaded && (
        <div className="absolute inset-0 flex animate-pulse items-center justify-center bg-muted">
          <LoaderCircle className="size-5 animate-spin text-muted-foreground/60" />
        </div>
      )}
      <img
        src={thumbUrl(photo.id, version)}
        alt=""
        loading="lazy"
        decoding="async"
        draggable={false}
        onLoad={() => setLoaded(true)}
        className={cn(
          'size-full object-cover transition-opacity duration-300',
          !loaded ? 'opacity-0' : photo.flag === 'reject' ? 'opacity-35' : 'opacity-100',
        )}
      />
      <CullBadges rating={photo.rating} flag={photo.flag} className="absolute top-1 left-1" />
      <div className="absolute inset-x-0 bottom-0 flex items-center gap-1 bg-gradient-to-t from-black/60 to-transparent px-1.5 pt-4 pb-1 text-[10px] text-white opacity-0 transition-opacity group-hover:opacity-100">
        <span className="truncate">{formatCapture(photo.capture)}</span>
      </div>
      <div className="absolute top-1 right-1 flex gap-0.5">
        {photo.dateFromFile && (
          <span className="rounded bg-black/60 p-0.5 text-amber-300" aria-label={t('library.noCaptureDate')}>
            <CalendarX2 className="size-3" />
          </span>
        )}
        {!photo.hasGps && (
          <span className="rounded bg-black/60 p-0.5 text-white/80" aria-label={t('library.noLocation')}>
            <MapPinOff className="size-3" />
          </span>
        )}
      </div>
    </button>
  )
})
