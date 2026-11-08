// SPDX-License-Identifier: GPL-3.0-only
import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Repeat, ShoppingBasket } from 'lucide-react'
import { useEffect, useEffectEvent, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { KeyHints } from '@/components/layout/key-hints'
import { StageSpinner } from '@/components/layout/stage-spinner'
import type { PhotoSummary, Queue, QueueView } from '@/ipc/bindings'
import { handleCullKey } from '@/features/culling/cull-keys'
import { LookStrip } from '@/features/look/look-strip'
import { LOOK_KEYS, useLookPicker } from '@/features/look/use-looks'
import { sendTo } from '@/features/viewer/send-to'
import { useShownSrc } from '@/features/viewer/use-shown-src'
import { usePrefetch } from '@/features/viewer/use-prefetch'
import { isTyping, ownsArrowKeys } from '@/lib/keyboard'
import { formatNumber } from '@/lib/format'
import { previewUrl } from '@/lib/photo-url'

type SimpleViewProps = {
  photos: PhotoSummary[]
  index: number
  onIndexChange: (index: number) => void
  view: QueueView
  revision: number
  queueCounts: { print: number; aside: number }
}

/**
 * Simple mode (FR-048): one photo at a time, as it will print. ↑ keeps it in the print
 * basket, ↓ drops it (set aside), and the next photo comes at once (FR-050, FR-046).
 */
export function SimpleView({ photos, index, onIndexChange, view, revision, queueCounts }: SimpleViewProps) {
  const { t } = useTranslation()
  const photo = photos[index]
  const { src: shownSrc, error: shownError } = useShownSrc(photo?.id, photo ? previewUrl(photo.id, revision) : '')
  // The next photos load while this one is looked at (FR-091).
  usePrefetch(photos, index, !!photo && shownSrc === previewUrl(photo.id, revision))
  // Simple mode's number keys pick a look (FR-079): 0 Original, 1–9 the next; Simple mode
  // keeps or drops, it shows no stars, so its digits are not ratings.
  const looks = useLookPicker(photo?.id, revision)
  const go = (delta: number) => onIndexChange(Math.min(photos.length - 1, Math.max(0, index + delta)))
  const send = (queue: Queue) => {
    if (photo) sendTo(photo.id, queue, view, () => go(1))
  }

  // The keys of the preview (FR-050) and its culling keys (FR-022): 0–5, P / X / U, [ ].
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (ownsArrowKeys(event.target) || document.querySelector('[role="dialog"]')) return
    const digit = /^[0-9]$/.test(event.key) && !event.metaKey && !event.ctrlKey && !event.altKey && !isTyping(event.target)
    if (digit && Number(event.key) < LOOK_KEYS) {
      event.preventDefault()
      looks.pick(Number(event.key))
    } else if (handleCullKey(event, photo?.id)) event.preventDefault()
    else if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
      event.preventDefault()
      go(event.key === 'ArrowRight' ? 1 : -1)
    } else if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault()
      send(event.key === 'ArrowUp' ? 'print' : 'aside')
    }
  })
  useEffect(() => {
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  // Every photo in the gallery is decided: kept photos stay there, dropped ones leave it.
  // (The basket view lists only kept photos: that is a review, not a finished sort.)
  const [reviewing, setReviewing] = useState(false)
  const sorted = view === 'gallery' && photos.every((p) => p.queue === 'print')
  // A photo not decided yet (a new folder, an undo) ends the review.
  if (reviewing && !sorted) setReviewing(false)
  if (!photo || (sorted && !reviewing)) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 p-3 text-center">
        <p className="font-medium">{t('simple.done')}</p>
        <p className="max-w-sm text-sm text-muted-foreground">
          {t('simple.doneHint', { count: queueCounts.print, kept: formatNumber(queueCounts.print) })}
        </p>
        {photo && (
          <Button variant="outline" size="sm" onClick={() => setReviewing(true)}>
            <Repeat data-icon="inline-start" /> {t('simple.review')}
          </Button>
        )}
      </div>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="box-content flex h-header shrink-0 items-center gap-2 border-b px-4 text-sm">
        <span className="font-medium tabular-nums">
          {t('simple.progress', { index: formatNumber(index + 1), count: formatNumber(photos.length) })}
        </span>
        <span className="text-xs text-muted-foreground tabular-nums">
          {t('simple.counts', { kept: formatNumber(queueCounts.print), dropped: formatNumber(queueCounts.aside) })}
        </span>
        <span className="min-w-0 flex-1 truncate text-right text-xs text-muted-foreground" data-hint={photo.fileName}>{photo.fileName}</span>
      </div>
      <div className="relative flex min-h-0 flex-1 items-center justify-center bg-stage p-6">
        {shownSrc ? (
          <img
            src={shownSrc}
            alt={photo.fileName}
            draggable={false}
            className="max-h-full max-w-full animate-in object-contain stage-photo fade-in-0 motion-in"
          />
        ) : (
          <StageSpinner error={shownError} />
        )}
        {photo.queue === 'print' && (
          <Badge className="absolute top-3 left-3 gap-1">
            <ShoppingBasket /> {t('simple.inBasket')}
          </Badge>
        )}
      </div>
      <LookStrip photoId={photo.id} picker={looks} />
      <div className="flex items-center justify-center gap-2 border-t p-3">
        {/* The hints on wrappers: a disabled button gets no pointer events (STYLE_RULE §4). */}
        <span data-hint={t('simple.previous')}>
          <Button variant="outline" size="icon" onClick={() => go(-1)} disabled={index === 0} aria-label={t('simple.previous')}>
            <ChevronLeft />
          </Button>
        </span>
        <Button variant="outline" size="lg" className="min-w-36" onClick={() => send('aside')}>
          <ArrowDown data-icon="inline-start" /> {t('simple.drop')}
        </Button>
        <Button size="lg" className="min-w-36" onClick={() => send('print')}>
          <ArrowUp data-icon="inline-start" /> {t('simple.keep')}
        </Button>
        <span data-hint={t('simple.next')}>
          <Button variant="outline" size="icon" onClick={() => go(1)} disabled={index >= photos.length - 1} aria-label={t('simple.next')}>
            <ChevronRight />
          </Button>
        </span>
      </div>
      <p className="pb-3 text-center text-xs text-muted-foreground">
        <KeyHints
          hints={[
            { keys: ['0–9'], label: t('simple.look') },
            { keys: ['↑'], label: t('simple.keep') },
            { keys: ['↓'], label: t('simple.drop') },
            { keys: ['←', '→'], label: t('viewer.browse') },
          ]}
        />
      </p>
    </div>
  )
}
