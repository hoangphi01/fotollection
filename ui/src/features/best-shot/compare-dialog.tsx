// SPDX-License-Identifier: GPL-3.0-only
import { Check, EyeOff } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import type { Group, PhotoScore, PhotoSummary } from '@/ipc/bindings'
import { CullBadges } from '@/features/culling/cull-badges'
import { sourceUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'
import { reasonText } from './reason-text'

const METRICS = ['sharpness', 'eyes', 'steadiness', 'exposure', 'highlights', 'noise'] as const

type CompareDialogProps = {
  group: Group | null
  photosById: Map<number, PhotoSummary>
  revision: number
  onKeep: (group: number, photo: number) => void
  onClose: () => void
}

/** Bar centred on zero: right for better than the group, left for worse. */
function MetricBar({ value, scale }: { value: number; scale: number }) {
  const width = scale > 0 ? (Math.min(Math.abs(value), scale) / scale) * 50 : 0
  return (
    <div className="relative h-1.5 flex-1 rounded-full bg-muted">
      <div className="absolute inset-y-0 left-1/2 w-px bg-border" />
      <div
        className={cn(
          'absolute inset-y-0 rounded-full',
          value >= 0 ? 'bg-primary' : 'bg-destructive/70',
        )}
        style={
          value >= 0 ? { left: '50%', width: `${width}%` } : { right: '50%', width: `${width}%` }
        }
      />
    </div>
  )
}

/** The group's photos side by side, with a bar per measure (FR-041). */
export function CompareDialog({
  group,
  photosById,
  revision,
  onKeep,
  onClose,
}: CompareDialogProps) {
  const { t } = useTranslation()
  const photos = group?.result.photos ?? []
  // One scale per measure across the group, so bars compare photos, not measures.
  const scales = Object.fromEntries(
    METRICS.map((m) => [m, Math.max(0, ...photos.map((p) => Math.abs(p[m] ?? 0)))]),
  ) as Record<(typeof METRICS)[number], number>

  return (
    <Dialog open={group !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex max-h-[92vh] flex-col gap-3 sm:max-w-[min(96vw,1400px)]">
        <DialogTitle>{t('bestShot.compareTitle', { count: photos.length })}</DialogTitle>
        <DialogDescription>
          {group?.result.tooClose
            ? t('bestShot.tooCloseHint')
            : group?.result.reasons.map(reasonText).join(' · ')}
        </DialogDescription>
        <div
          className="grid min-h-0 flex-1 gap-3 overflow-y-auto"
          style={{ gridTemplateColumns: `repeat(${Math.min(photos.length, 4)}, minmax(0, 1fr))` }}
        >
          {photos.map((score: PhotoScore, rank) => {
            const photo = photosById.get(score.id)
            return (
              <figure key={score.id} className="flex min-w-0 flex-col gap-2">
                <div
                  className={cn(
                    'relative overflow-hidden rounded-md bg-muted',
                    rank === 0 && 'ring-2 ring-primary',
                  )}
                >
                  <img
                    src={sourceUrl(score.id, revision)}
                    alt=""
                    draggable={false}
                    className="aspect-[4/3] w-full object-contain"
                  />
                  {photo && (
                    <CullBadges
                      rating={photo.rating}
                      flag={photo.flag}
                      className="absolute top-1.5 left-1.5"
                    />
                  )}
                </div>
                <figcaption className="flex items-center gap-2 text-xs">
                  <span className="truncate font-medium">{photo?.fileName ?? `#${score.id}`}</span>
                  {rank === 0 && !group?.result.tooClose && <Badge>{t('bestShot.best')}</Badge>}
                  {score.closedEyes > 0 && (
                    <span className="flex items-center gap-1 text-amber-600 dark:text-amber-400">
                      <EyeOff className="size-3.5" />
                      {t('bestShot.closedEyes', { count: score.closedEyes })}
                    </span>
                  )}
                </figcaption>
                <dl className="grid grid-cols-[5.5rem_1fr] items-center gap-x-2 gap-y-1 text-[11px] text-muted-foreground">
                  {METRICS.filter((m) => m !== 'eyes' || score.faces > 0).map((m) => (
                    <div key={m} className="contents">
                      <dt>{t(`bestShot.metric.${m}`)}</dt>
                      <dd className="flex items-center">
                        <MetricBar value={score[m] ?? 0} scale={scales[m]} />
                      </dd>
                    </div>
                  ))}
                </dl>
                <Button
                  variant={rank === 0 ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => group && onKeep(group.id, score.id)}
                >
                  <Check data-icon="inline-start" /> {t('bestShot.keepThis')}
                </Button>
              </figure>
            )
          })}
        </div>
      </DialogContent>
    </Dialog>
  )
}
