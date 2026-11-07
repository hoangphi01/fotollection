// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { ChartArea, Pipette, TriangleAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SectionCard } from '@/components/layout/section-card'
import { Toggle } from '@/components/ui/toggle'
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
  picking,
  onPicking,
}: {
  photoId: PhotoId | null
  revision: number
  /** The clipped pixels marked on the photo (FR-098, `J`); absent where it cannot be shown. */
  clipping?: boolean
  onClipping?: (on: boolean) => void
  /** The white-balance picker armed (FR-098, `W`); absent where it cannot be used. */
  picking?: boolean
  onPicking?: (on: boolean) => void
}) {
  const { t } = useTranslation()
  const histogram = useLiveQuery(photoId === null ? null : String(photoId), revision, loadHistogram).data
  return (
    <SectionCard
      icon={<ChartArea />}
      title={t('histogram.title')}
      // Choices in a card's header: flat, the one on pressed in (STYLE_RULE §4), as the
      // Navigator's; each is on or off by itself.
      action={
        onClipping && (
          <div className="flex gap-1">
            {onPicking && (
              <Toggle size="sm" pressed={picking} onPressedChange={onPicking} aria-label={t('histogram.pick')} data-hint={t('histogram.pickHint')}>
                <Pipette />
              </Toggle>
            )}
            <Toggle size="sm" pressed={clipping} onPressedChange={onClipping} aria-label={t('histogram.clipping')} data-hint={t('histogram.clippingHint')}>
              <TriangleAlert />
            </Toggle>
          </div>
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
            {/* The channels in their own colours, from the theme's tokens (`--hist-*`). */}
            <path d={area(histogram.r)} fill="var(--hist-r)" />
            <path d={area(histogram.g)} fill="var(--hist-g)" />
            <path d={area(histogram.b)} fill="var(--hist-b)" />
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
      {/* One line of a fixed room under the curves (nothing jumps): W's words while it is
          armed, else how much is blown and blocked, else empty. */}
      <p className="flex h-8 items-start gap-1.5 text-xs leading-4 text-muted-foreground [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-primary">
        {picking ? (
          <>
            <Pipette aria-hidden />
            <span className="line-clamp-2">{t('histogram.pickNow')}</span>
          </>
        ) : (
          histogram &&
          (histogram.highPermille > 0 || histogram.lowPermille > 0) && (
            <span className="line-clamp-2">
              {t('histogram.clipped', { high: percent(histogram.highPermille), low: percent(histogram.lowPermille) })}
            </span>
          )
        )}
      </p>
    </SectionCard>
  )
}
