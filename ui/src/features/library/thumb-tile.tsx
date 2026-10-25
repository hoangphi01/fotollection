// SPDX-License-Identifier: GPL-3.0-only
import { CalendarX2, Check, LoaderCircle, MapPinOff, ShoppingBasket } from 'lucide-react'
import { memo, useState, type MouseEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { PhotoSummary } from '@/ipc/bindings'
import { CullBadges } from '@/features/culling/cull-badges'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

type ThumbTileProps = {
  photo: PhotoSummary
  version: number
  /** Bumps when another project replaces the library (photo ids are reused). */
  generation: number
  size: number
  selected: boolean
  /** Select mode (FR-050): a tick circle on every photo; `checked` when ticked. */
  selectMode?: boolean
  checked?: boolean
  onSelect: (event: MouseEvent) => void
  onOpen: () => void
  /** The pointer is over this photo (for Quick-preview, FR-059). */
  onHover?: (over: boolean) => void
}

function formatCapture(capture: string): string {
  return capture.replace('T', ' ').slice(0, 16)
}

/** One grid cell: skeleton + spinner until the thumbnail arrives, then a fade-in. */
export const ThumbTile = memo(function ThumbTile({
  photo,
  version,
  generation,
  size,
  selected,
  selectMode = false,
  checked = false,
  onSelect,
  onOpen,
  onHover,
}: ThumbTileProps) {
  const { t } = useTranslation()
  // Stays true across T0 → T1 upgrades, so the sharper image swaps in without flashing.
  const [loaded, setLoaded] = useState(false)

  return (
    <button
      type="button"
      data-hint={`${photo.fileName} · ${formatCapture(photo.capture)}`}
      onClick={onSelect}
      onDoubleClick={selectMode ? undefined : onOpen}
      onMouseEnter={() => onHover?.(true)}
      onMouseLeave={() => onHover?.(false)}
      aria-pressed={selectMode ? checked : undefined}
      className={cn(
        'group relative shrink-0 overflow-hidden rounded-md bg-muted outline-none',
        // The selected photo is framed in the theme colour (teal in light, wood in dark);
        // in select mode, the ticked ones are.
        (selectMode ? checked : selected) && 'ring-[3px] ring-primary ring-offset-2 ring-offset-background',
      )}
      style={{ width: size, height: size }}
    >
      {!loaded && (
        <div className="absolute inset-0 flex animate-pulse items-center justify-center bg-muted">
          <LoaderCircle className="size-5 animate-spin text-muted-foreground/60" />
        </div>
      )}
      <img
        src={thumbUrl(photo.id, version, generation)}
        alt=""
        loading="lazy"
        decoding="async"
        draggable={false}
        onLoad={() => setLoaded(true)}
        style={photo.rotation ? { transform: `rotate(${photo.rotation * 90}deg)` } : undefined}
        className={cn(
          'size-full object-cover transition-opacity duration-300',
          !loaded ? 'opacity-0' : photo.flag === 'reject' ? 'opacity-35' : 'opacity-100',
        )}
      />
      <CullBadges rating={photo.rating} flag={photo.flag} className="absolute top-1 left-1" />
      <div className="absolute inset-x-0 bottom-0 flex items-center gap-1 bg-gradient-to-t from-black/60 to-transparent px-1.5 pt-4 pb-1 text-[10px] text-white opacity-0 transition-opacity group-hover:opacity-100">
        <span className="truncate">{formatCapture(photo.capture)}</span>
      </div>
      {selectMode && (
        <span
          className={cn(
            'absolute right-1.5 bottom-1.5 flex size-5 items-center justify-center rounded-full border-2 shadow',
            checked ? 'border-primary bg-primary text-primary-foreground' : 'border-white bg-black/30',
          )}
          aria-hidden
        >
          {checked && <Check className="size-3" strokeWidth={3} />}
        </span>
      )}
      <div className="absolute top-1 right-1 flex gap-0.5">
        {photo.queue === 'print' && (
          <span className="rounded bg-primary p-0.5 text-primary-foreground" aria-label={t('library.inBasket')}>
            <ShoppingBasket className="size-3" />
          </span>
        )}
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
