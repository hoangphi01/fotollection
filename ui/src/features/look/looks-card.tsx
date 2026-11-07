// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { BookmarkPlus, Palette } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { LookGrid, LookTile } from '@/components/forms/look-tile'
import { SliderField } from '@/components/forms/slider-field'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { commands, type LookSettings, type PhotoId } from '@/ipc/bindings'
import { lookTileUrl } from '@/lib/photo-url'
import { fullLook, type LookChange } from './look-fields'
import { SaveLookDialog } from './save-look-dialog'
import { lookName, useLooks } from './use-looks'

type LooksCardProps = {
  photoId: PhotoId
  look: LookSettings | undefined
  onChange: LookChange
  /** `false` where a strip of looks under the stage lists them (a collage sheet's preview). */
  grid?: boolean
}

const percent = (v: number) => `${v} %`

/**
 * The first level of the colour cards (FR-079, "drive first"): the looks of the library as
 * tiles of the photo through each, the chosen one marked; the amount of the whole look; *Save
 * as look* keeps the photo's look on this machine. A pick copies the look's values into the
 * photo, so the sliders below show them and a later change of the library changes no photo.
 */
export function LooksCard({ photoId, look, onChange, grid = true }: LooksCardProps) {
  const { t } = useTranslation()
  const { looks, setLooks, reload } = useLooks()
  const [saving, setSaving] = useState(false)
  const current = fullLook(look)
  return (
    <SectionCard icon={<Palette />} title={t('look.looks')}>
      {grid && (
      <LookGrid>
        {looks.map((entry) => (
          <LookTile
            key={entry.id}
            src={lookTileUrl(photoId, entry.id)}
            name={lookName(entry)}
            chosen={current.base === entry.id}
            onPick={() => onChange(() => fullLook({ ...entry.look, name: lookName(entry) }), true)}
            corner={
              entry.kind === 'user' && (
                <RemoveButton
                  iconOnly
                  small
                  label={t('look.remove')}
                  onClick={() => void commands.deleteLook(entry.id).then((r) => r.status === 'ok' && setLooks(r.data))}
                />
              )
            }
          />
        ))}
      </LookGrid>
      )}
      <SliderField
        label={t('look.amount')}
        hint={t('look.hints.amount')}
        value={current.amount}
        min={0}
        max={100}
        reset={100}
        format={percent}
        onChange={(v, first) => onChange((l) => ({ ...l, amount: v }), first)}
      />
      <Button variant="outline" size="sm" className="w-full" onClick={() => setSaving(true)}>
        <BookmarkPlus data-icon="inline-start" /> {t('look.saveAs')}
      </Button>
      <SaveLookDialog
        open={saving}
        onAnswer={(name) => {
          setSaving(false)
          if (name === null) return
          void commands.saveLook(name, look ?? current).then((r) => {
            if (r.status !== 'ok') return
            reload()
            onChange((l) => ({ ...l, name: r.data.name, base: r.data.id }), true)
          })
        }}
      />
    </SectionCard>
  )
}
