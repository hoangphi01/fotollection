// SPDX-License-Identifier: GPL-3.0-only
import { Check, Columns2, EyeOff, Scale, Sparkles } from 'lucide-react'
import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { Group, PhotoSummary } from '@/ipc/bindings'
import { CullBadges } from '@/features/culling/cull-badges'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'
import { reasonText } from './reason-text'

const THUMB = 132

type GroupStripProps = {
  group: Group
  photosById: Map<number, PhotoSummary>
  thumbVersions: { [id: number]: number }
  generation: number
  onCompare: () => void
  onKeepBest: () => void
}

/** One group of similar shots: "Best of N" with its reasons, best photo first (FR-041). */
export const GroupStrip = memo(function GroupStrip({
  group,
  photosById,
  thumbVersions,
  generation,
  onCompare,
  onKeepBest,
}: GroupStripProps) {
  const { t } = useTranslation()
  const { photos, reasons, tooClose } = group.result
  return (
    <section className="rounded-lg border bg-card p-3">
      <div className="mb-2 flex flex-wrap items-center gap-2">
        <Badge className="gap-1">
          <Sparkles data-icon="inline-start" />
          {t('bestShot.bestOf', { count: photos.length })}
        </Badge>
        {tooClose ? (
          <Badge variant="outline" className="gap-1">
            <Scale data-icon="inline-start" />
            {t('bestShot.tooClose')}
          </Badge>
        ) : (
          <span className="text-xs text-muted-foreground">
            {reasons.map(reasonText).join(' · ')}
          </span>
        )}
        <div className="ml-auto flex gap-2">
          <Button variant="outline" size="sm" onClick={onCompare}>
            <Columns2 data-icon="inline-start" /> {t('bestShot.compare')}
          </Button>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button size="sm" onClick={tooClose ? onCompare : onKeepBest}>
                <Check data-icon="inline-start" /> {t('bestShot.keepBest')}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {tooClose ? t('bestShot.chooseHint') : t('bestShot.keepBestHint')}
            </TooltipContent>
          </Tooltip>
        </div>
      </div>
      <div className="flex gap-2 overflow-x-auto pb-1">
        {photos.map((score, rank) => {
          const photo = photosById.get(score.id)
          return (
            <button
              key={score.id}
              type="button"
              onClick={onCompare}
              title={photo?.fileName}
              className={cn(
                'relative shrink-0 overflow-hidden rounded-md bg-muted outline-none focus-visible:ring-2 focus-visible:ring-ring',
                rank === 0 && 'ring-2 ring-primary ring-offset-2 ring-offset-background',
              )}
              style={{ width: THUMB, height: THUMB }}
            >
              <img
                src={thumbUrl(score.id, thumbVersions[score.id] ?? 0, generation)}
                alt=""
                loading="lazy"
                decoding="async"
                draggable={false}
                className={cn('size-full object-cover', photo?.flag === 'reject' && 'opacity-35')}
              />
              {photo && (
                <CullBadges
                  rating={photo.rating}
                  flag={photo.flag}
                  className="absolute top-1 left-1"
                />
              )}
              {rank === 0 && !tooClose && (
                <span className="absolute right-1 bottom-1 rounded bg-primary px-1.5 py-0.5 text-[10px] font-medium text-primary-foreground">
                  {t('bestShot.best')}
                </span>
              )}
              {score.closedEyes > 0 && (
                <span
                  className="absolute top-1 right-1 rounded bg-black/60 p-0.5 text-amber-300"
                  aria-label={t('bestShot.closedEyes', { count: score.closedEyes })}
                >
                  <EyeOff className="size-3" />
                </span>
              )}
            </button>
          )
        })}
      </div>
    </section>
  )
})
