// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in the gallery panels only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { MapPin, MapPinCheck } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Field, Hint, RowList } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { BorrowDialog } from './borrow-dialog'
import { usePlaces } from './use-places'
import { ResetButton } from '@/components/forms/danger-button'

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
      <Hint>{t('places.hint')}</Hint>
      <RowList>
        {places.map((place) => (
          <li key={place.geonamesId} className="flex items-center gap-2">
            <div className="min-w-0 flex-1">
              <Field>
              <CommitInput
                value={place.name}
                placeholder={place.original}
                maxLength={80}
                ariaLabel={t('places.renameLabel', { place: place.original })}
                onCommit={(text) => void rename(place, text.trim() === '' ? null : text)}
              />
              <Hint className="truncate">
                {place.renamed ? `${place.original}, ${place.country}` : place.country} ·{' '}
                {t('cameras.photos', { count: place.photos })}
              </Hint>
              </Field>
            </div>
            <ResetButton iconOnly disabled={!place.renamed} label={t('places.resetName')} onClick={() => void rename(place, null)} />
          </li>
        ))}
      </RowList>
      {error && <p className="text-xs text-destructive">{error}</p>}
    </SectionCard>
  )
}
