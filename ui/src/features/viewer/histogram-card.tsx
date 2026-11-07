// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { ChartArea, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { commands, type HistogramView, type PhotoId } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { formatNumber } from '@/lib/format'

const loadHistogram = async (subject: string): Promise<HistogramView | null> => {
  const result = await commands.photoHistogram(Number(subject))
  return result.status === 'ok' ? result.data : null
}

/** One channel as a filled area over the 256 levels, 0–1000 high (Rust's scale). */
function area(levels: readonly number[]): string {
  const points = levels.map((v, i) => `${i},${1000 - v}`).join(' ')
  return `M0,1000 L${points} L255,1000 Z`
}

/** Per mille as a percentage with one decimal, "0.4 %". */
const percent = (perMille: number) => `${formatNumber(perMille / 10, 1)} %`

/**
 * The photo's histogram (FR-098), as Lightroom's above its Navigator: red, green and blue
 * blended where they overlap, the light as a line, and how much is blown or blocked. Made in
 * Rust from the preview's own photo and look, never the stamp or frame; it follows a slider as
 * the preview does, and the last one stays while the next loads, so nothing blinks.
 */
export function HistogramCard({
  photoId,
  revision,
  clipping,
  onClipping,
}: {
  photoId: PhotoId | null
  revision: number
  /** The clipped pixels marked on the photo (FR-098, `J`); absent where it cannot be shown. */
  clipping?: boolean
  onClipping?: (on: boolean) => void
}) {
  const { t } = useTranslation()
  const histogram = useLiveQuery(photoId === null ? null : String(photoId), revision, loadHistogram).data
  return (
    <SectionCard
      icon={<ChartArea />}
      title={t('histogram.title')}
      action={
        onClipping && (
          <Button
            variant={clipping ? 'default' : 'outline'}
            size="icon-sm"
            aria-pressed={clipping}
            aria-label={t('histogram.clipping')}
            data-hint={t('histogram.clippingHint')}
            onClick={() => onClipping(!clipping)}
          >
            <TriangleAlert />
          </Button>
        )
      }
    >
      <svg
        viewBox="0 0 255 1000"
        preserveAspectRatio="none"
        role="img"
        aria-label={t('histogram.label')}
        className="block h-16 w-full rounded-md bg-muted/60 [&_path]:mix-blend-multiply dark:[&_path]:mix-blend-screen"
      >
        {histogram && (
          <>
            {/* The channels in their own colours, as a swatch of what prints is (STYLE_RULE §4). */}
            <path d={area(histogram.r)} fill="oklch(0.63 0.22 27 / 0.7)" />
            <path d={area(histogram.g)} fill="oklch(0.7 0.18 145 / 0.7)" />
            <path d={area(histogram.b)} fill="oklch(0.6 0.18 255 / 0.7)" />
            <polyline
              points={histogram.l.map((v, i) => `${i},${1000 - v}`).join(' ')}
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              vectorEffect="non-scaling-stroke"
              className="text-foreground/70 mix-blend-normal"
            />
          </>
        )}
      </svg>
      {histogram && (histogram.highPermille > 0 || histogram.lowPermille > 0) && (
        <Hint>
          {t('histogram.clipped', { high: percent(histogram.highPermille), low: percent(histogram.lowPermille) })}
        </Hint>
      )}
    </SectionCard>
  )
}
