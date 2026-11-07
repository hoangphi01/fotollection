// SPDX-License-Identifier: GPL-3.0-only
import { Flag as FlagIcon, FlagOff, Star } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { Toggle } from '@/components/ui/toggle'
import { commands, type PhotoSummary } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

/**
 * Clickable stars and pick/reject for one photo (FR-022), the same as the keys. Every one is
 * a 32 px toggle, pressed in the theme's colour when on (STYLE_RULE §4; owner, 2026-11-07:
 * a reject is a state, never the red kept for removing): the stars up to the rating, then
 * Pick and Reject on a row of their own, so both rows fit an 18 rem panel.
 */
export function CullControls({ photo }: { photo: PhotoSummary }) {
  const { t } = useTranslation()
  const rate = (n: number) => void commands.setPhotoRating([photo.id], photo.rating === n ? 0 : n)
  const flag = (f: 'pick' | 'reject') => void commands.setPhotoFlag([photo.id], photo.flag === f ? null : f)
  return (
    <Field hint={t('culling.keysHint')}>
      {/* Two rows of controls, 12 px apart (STYLE_RULE §4). */}
      <div className="flex flex-col gap-3">
      <div className="grid grid-cols-5 gap-1" role="group" aria-label={t('culling.rating')}>
        {[1, 2, 3, 4, 5].map((n) => (
          <span key={n} className="flex" data-hint={t(n === photo.rating ? 'culling.starClear' : 'culling.starKey', { count: n })}>
            <Toggle
              variant="outline"
              size="sm"
              className="w-full"
              pressed={n <= photo.rating}
              aria-label={t('culling.stars', { count: n })}
              onPressedChange={() => rate(n)}
            >
              <Star className={cn(n <= photo.rating && 'fill-current')} />
            </Toggle>
          </span>
        ))}
      </div>
      <div className="grid grid-cols-2 gap-2">
        <Toggle variant="outline" size="sm" pressed={photo.flag === 'pick'} onPressedChange={() => flag('pick')}>
          <FlagIcon data-icon="inline-start" />
          {t('culling.pickKey')}
        </Toggle>
        <Toggle variant="outline" size="sm" pressed={photo.flag === 'reject'} onPressedChange={() => flag('reject')}>
          <FlagOff data-icon="inline-start" />
          {t('culling.rejectKey')}
        </Toggle>
      </div>
      </div>
    </Field>
  )
}
