// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Bookmark, BookmarkPlus, FileInput, Layers, TriangleAlert } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint } from '@/components/forms/field'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { commands, type PhotoId } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { SaveLookDialog } from './save-look-dialog'
import { useLookPicker } from './use-looks'

type PresetsCardProps = {
  photoId: PhotoId
  revision: number
  /** A collage sheet's photos when none is picked: a pick gives both the look. */
  sheet?: PhotoId[]
}

/**
 * *Presets & LUTs*, as Lightroom's Presets panel on the left of its Develop view (owner,
 * 2026-11-07): the looks of one's own (imported LUTs, saved looks, Lightroom presets later),
 * to apply or forget; *Save as look* and *Import LUT…*. The built-in looks stay in the strip
 * under the photo; the Looks card on the right sets the amount.
 */
export function PresetsCard({ photoId, revision, sheet }: PresetsCardProps) {
  const { t } = useTranslation()
  const picker = useLookPicker(photoId, revision, sheet)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const own = picker.looks.filter((e) => e.kind === 'user')
  const chosen = picker.current === null ? undefined : picker.looks[picker.current]
  return (
    <SectionCard icon={<Layers />} title={t('look.presets')}>
      {own.length === 0 ? (
        <Hint>{t('look.presetsEmpty')}</Hint>
      ) : (
        <ListRows>
          {own.map((entry) => (
            <div key={entry.id} className="flex items-center gap-2">
              <div className="min-w-0 flex-1" data-hint={entry.name}>
                <ListRow icon={<Bookmark />} chosen={chosen?.id === entry.id} onClick={() => picker.apply(entry)}>
                  {entry.name}
                </ListRow>
              </div>
              <RemoveButton iconOnly label={t('look.remove')} onClick={() => picker.remove(entry.id)} />
            </div>
          ))}
        </ListRows>
      )}
      {error && (
        <PanelNote danger icon={<TriangleAlert />}>
          {error}
        </PanelNote>
      )}
      <Button variant="outline" size="sm" className="w-full" onClick={() => setSaving(true)}>
        <BookmarkPlus data-icon="inline-start" /> {t('look.saveAs')}
      </Button>
      <Button
        variant="outline"
        size="sm"
        className="w-full"
        onClick={() =>
          void commands.importLut().then((r) => {
            if (r.status !== 'ok') {
              setError(errorText(r.error))
              return
            }
            setError(null)
            if (r.data === null) return
            picker.reload()
            picker.apply(r.data)
          })
        }
      >
        <FileInput data-icon="inline-start" /> {t('look.importLut')}
      </Button>
      <SaveLookDialog
        open={saving}
        onAnswer={(name) => {
          setSaving(false)
          if (name === null) return
          void commands.getPhotoLook(photoId).then((current) => {
            if (current.status !== 'ok') return
            void commands.saveLook(name, current.data.look).then((r) => {
              if (r.status !== 'ok') {
                setError(errorText(r.error))
                return
              }
              picker.reload()
              picker.apply(r.data)
            })
          })
        }}
      />
    </SectionCard>
  )
}
