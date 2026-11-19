// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: in the gallery panels only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { MapPin, MapPinCheck, Pencil } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Hint } from '@/components/forms/field'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { MenuSeparator } from '@/components/forms/menu-content'
import { RowMenu } from '@/components/forms/row-menu'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { BorrowDialog } from './borrow-dialog'
import { usePlaces } from './use-places'
import { ResetButton } from '@/components/forms/danger-button'

/**
 * Places (FR-005): every place on the photos of the folders shown, with the name printed for
 * it, its country and its photos, one list row each (owner, 2026-11-19: "a text field and a red
 * reset on every row does not make sense"). Renamed in place ("Praha 1" → "Prague", or
 * "Grandma's"), every stamp and frame follows.
 */
export function PlacesCard({ folders }: { folders: number[] }) {
  const { t, i18n } = useTranslation()
  const { places, error, rename } = usePlaces(folders)
  const [borrowing, setBorrowing] = useState(false)
  const [renaming, setRenaming] = useState<number | null>(null)
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
      <ListRows>
        {places.map((place) =>
          renaming === place.geonamesId ? (
            <CommitInput
              key={place.geonamesId}
              value={place.name}
              placeholder={place.original}
              maxLength={80}
              ariaLabel={t('places.renameLabel', { place: place.original })}
              onCommit={(text) => {
                setRenaming(null)
                void rename(place, text.trim() === '' ? null : text)
              }}
            />
          ) : (
            // Renamed from a click, its ⋯ or a right click (FR-117), as RollSense's trips; the
            // red way back only once renamed (owner, 2026-11-19).
            <RowMenu
              key={place.geonamesId}
              label={t('places.actions')}
              menu={(close) => (
                <>
                  <ListRow icon={<Pencil />} onClick={() => (close(), setRenaming(place.geonamesId))}>
                    {t('places.rename')}
                  </ListRow>
                  {place.renamed && (
                    <>
                      <MenuSeparator />
                      <ResetButton wide label={t('places.resetName')} onClick={() => (close(), void rename(place, null))} />
                    </>
                  )}
                </>
              )}
            >
              <ListRow
                trailing={<span className="text-xs text-muted-foreground tabular-nums">{place.photos.toLocaleString(i18n.language)}</span>}
                onClick={() => setRenaming(place.geonamesId)}
              >
                <span className="truncate">
                  {place.name}
                  <span className="text-muted-foreground"> · {place.renamed ? `${place.original}, ${place.country}` : place.country}</span>
                </span>
              </ListRow>
            </RowMenu>
          ),
        )}
      </ListRows>
      {error && <p className="text-xs text-destructive">{error}</p>}
    </SectionCard>
  )
}
