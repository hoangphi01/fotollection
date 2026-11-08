// SPDX-License-Identifier: GPL-3.0-only
import { RotateCcw, TriangleAlert } from 'lucide-react'
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Field } from '@/components/forms/field'
import { PanelNote } from '@/components/layout/panel-note'
import { commands, type EffectivePlace } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { PlacePicker } from './place-picker'
import { RemoveButton } from '@/components/forms/danger-button'

/** What one photo has now, when the dialog is opened from the preview. */
export type DatePlaceCurrent = {
  /** `YYYY-MM-DDTHH:MM:SS`, the time it is stamped with. */
  capture: string
  dateManual: boolean
  place: EffectivePlace | null
}

type DatePlaceDialogProps = {
  ids: number[]
  /** Only for a single photo: fills the fields with what it has. */
  current?: DatePlaceCurrent
  onClose: () => void
}

/** A change to one field: leave it, set it, or go back to what the photo says. */
type Change<T> = { kind: 'keep' } | { kind: 'set'; value: T } | { kind: 'clear' }

/**
 * Set the date and place of one photo or a selection by hand (FR-032), for scans and photos
 * without EXIF or GPS. Only the fields the user touched change; each change is undoable.
 */
export function DatePlaceDialog({ ids, current, onClose }: DatePlaceDialogProps) {
  const { t } = useTranslation()
  const dateId = useId()
  const [date, setDate] = useState<Change<string>>({ kind: 'keep' })
  const [place, setPlace] = useState<Change<{ id: number; label: string }>>({ kind: 'keep' })
  const [error, setError] = useState<string | null>(null)

  const dateText = date.kind === 'set' ? date.value : date.kind === 'keep' ? (current?.capture.slice(0, 16) ?? '') : ''
  const placeLabel =
    place.kind === 'set' ? place.value.label : place.kind === 'keep' ? (current?.place?.label ?? null) : null
  const manualPlace = current?.place?.source === 'manual'

  const apply = async () => {
    if (date.kind !== 'keep') {
      const r = await commands.setPhotoDate(ids, date.kind === 'set' ? date.value : null)
      if (r.status === 'error') return setError(errorText(r.error))
    }
    if (place.kind !== 'keep') {
      const r = await commands.setPhotoPlace(ids, place.kind === 'set' ? place.value.id : null)
      if (r.status === 'error') return setError(errorText(r.error))
    }
    onClose()
  }

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{ids.length === 1 ? t('places.dialogTitleOne') : t('places.dialogTitle', { count: ids.length })}</DialogTitle>
          <DialogDescription>{t('places.dialogBody')}</DialogDescription>
        </DialogHeader>
        <Field label={t('places.date')} htmlFor={dateId} hint={date.kind === 'clear' ? t('places.cameraDateNote') : undefined}>
          <div className="flex gap-2">
            <Input
              id={dateId}
              type="datetime-local"
              min="1826-01-01T00:00"
              max="2100-12-31T23:59"
              value={dateText}
              onChange={(e) => setDate(e.target.value ? { kind: 'set', value: e.target.value } : { kind: 'keep' })}
            />
            {(current?.dateManual ?? ids.length > 1) && (
              <Button
                size="icon"
                variant="ghost"
                aria-label={t('places.cameraDate')}
                data-hint={t('places.cameraDate')}
                aria-pressed={date.kind === 'clear'}
                onClick={() => setDate({ kind: 'clear' })}
              >
                <RotateCcw />
              </Button>
            )}
          </div>
        </Field>
        <Field label={t('places.place')} hint={place.kind === 'clear' ? t('places.ownPlaceNote') : undefined}>
          <div className="flex gap-2">
            <PlacePicker
              label={placeLabel}
              onPick={(p) => setPlace({ kind: 'set', value: { id: p.geonamesId, label: p.name } })}
            />
            {(manualPlace || ids.length > 1) && (
              <Button
                size="icon"
                variant="ghost"
                aria-label={t('places.ownPlace')}
                data-hint={t('places.ownPlace')}
                aria-pressed={place.kind === 'clear'}
                onClick={() => setPlace({ kind: 'clear' })}
              >
                <RotateCcw />
              </Button>
            )}
          </div>
        </Field>
        {error && (
          <PanelNote danger icon={<TriangleAlert />}>
            {error}
          </PanelNote>
        )}
        <DialogFooter>
          <RemoveButton kind="stop" label={t('common.cancel')} onClick={onClose} />
          <Button disabled={date.kind === 'keep' && place.kind === 'keep'} onClick={() => void apply()}>
            {t('places.apply')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
