// SPDX-License-Identifier: GPL-3.0-only
import { MapPin, MapPinCheck, RotateCcw } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { BorrowDialog } from './borrow-dialog'
import { usePlaces } from './use-places'

/**
 * Places (FR-005): every place on the library's photos, with the name printed for it. Type
 * a new one ("Praha 1" → "Prague", or "Grandma's") and every stamp and frame follows.
 */
export function PlacesCard() {
  const { t } = useTranslation()
  const { places, error, rename } = usePlaces()
  const [borrowing, setBorrowing] = useState(false)
  if (places.length === 0) return null
  return (
    <SectionCard
      icon={<MapPin />}
      title={t('places.title')}
      action={
        <Button size="sm" variant="outline" onClick={() => setBorrowing(true)}>
          <MapPinCheck data-icon="inline-start" /> {t('places.borrowOpen')}
        </Button>
      }
    >
      {borrowing && <BorrowDialog onClose={() => setBorrowing(false)} />}
      <p className="text-xs text-muted-foreground">{t('places.hint')}</p>
      <ul className="space-y-2">
        {places.map((place) => (
          <li key={place.geonamesId} className="flex items-center gap-2">
            <div className="min-w-0 flex-1">
              <CommitInput
                value={place.name}
                placeholder={place.original}
                maxLength={80}
                ariaLabel={t('places.renameLabel', { place: place.original })}
                onCommit={(text) => void rename(place, text.trim() === '' ? null : text)}
              />
              <p className="mt-0.5 truncate text-xs text-muted-foreground">
                {place.renamed ? `${place.original}, ${place.country}` : place.country} ·{' '}
                {t('cameras.photos', { count: place.photos })}
              </p>
            </div>
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={!place.renamed}
              aria-label={t('places.resetName')}
              data-hint={t('places.resetName')}
              onClick={() => void rename(place, null)}
            >
              <RotateCcw />
            </Button>
          </li>
        ))}
      </ul>
      {error && <p className="text-xs text-destructive">{error}</p>}
    </SectionCard>
  )
}
