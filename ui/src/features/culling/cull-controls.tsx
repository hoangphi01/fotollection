// SPDX-License-Identifier: GPL-3.0-only
import { Flag as FlagIcon, FlagOff, Star } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type PhotoSummary } from '@/ipc/bindings'
import { cn } from '@/lib/utils'

/** Clickable stars and pick/reject buttons for one photo (FR-022), same as the shortcuts. */
export function CullControls({ photo }: { photo: PhotoSummary }) {
  const { t } = useTranslation()
  const rate = (n: number) => void commands.setPhotoRating([photo.id], photo.rating === n ? 0 : n)
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-3">
        <div className="flex" role="radiogroup" aria-label={t('culling.rating')}>
          {[1, 2, 3, 4, 5].map((n) => (
            <button
              key={n}
              type="button"
              role="radio"
              aria-checked={photo.rating === n}
              aria-label={t('culling.stars', { count: n })}
              title={t(n === photo.rating ? 'culling.starClear' : 'culling.starKey', { count: n })}
              onClick={() => rate(n)}
              className="rounded p-0.5 text-muted-foreground hover:text-amber-500"
            >
              <Star className={cn('size-4', n <= photo.rating && 'fill-amber-400 text-amber-500')} />
            </button>
          ))}
        </div>
        <div className="ml-auto flex gap-1">
          <Button
            variant={photo.flag === 'pick' ? 'default' : 'outline'}
            size="icon-sm"
            aria-label={t('culling.pickKey')}
            title={t('culling.pickKey')}
            onClick={() => void commands.setPhotoFlag([photo.id], photo.flag === 'pick' ? null : 'pick')}
          >
            <FlagIcon />
          </Button>
          <Button
            variant={photo.flag === 'reject' ? 'destructive' : 'outline'}
            size="icon-sm"
            aria-label={t('culling.rejectKey')}
            title={t('culling.rejectKey')}
            onClick={() =>
              void commands.setPhotoFlag([photo.id], photo.flag === 'reject' ? null : 'reject')
            }
          >
            <FlagOff />
          </Button>
        </div>
      </div>
      <p className="text-[11px] text-muted-foreground">{t('culling.keysHint')}</p>
    </div>
  )
}
