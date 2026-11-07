// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Bookmark, BookmarkPlus, Palette } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton, ResetButton } from '@/components/forms/danger-button'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { Field } from '@/components/forms/field'
import { SliderField } from '@/components/forms/slider-field'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { commands, type LookSettings } from '@/ipc/bindings'
import { NO_LOOK, fullLook, type LookChange } from './look-fields'
import { SaveLookDialog } from './save-look-dialog'
import { useLooks } from './use-looks'

type LooksCardProps = {
  /** Simple mode: the amount and the reset only (owner, 2026-11-07). */
  simple?: boolean
  look: LookSettings | undefined
  onChange: LookChange
}

const percent = (v: number) => `${v} %`

/**
 * The first level of the colour cards (FR-079, "drive first"), beside the strip of looks under
 * the photo: the amount of the whole look; *Save as look* keeps the photo's look on this
 * machine, listed here to pick or forget; ↺ gives the original back. A pick copies the look's values into the
 * photo, so the sliders below show them and a later change of the library changes no photo.
 */
export function LooksCard({ look, onChange, simple = false }: LooksCardProps) {
  const { t } = useTranslation()
  const { looks, setLooks, reload } = useLooks()
  const [saving, setSaving] = useState(false)
  const current = fullLook(look)
  // Coloured: a look or a slider changes the photo (Original changes nothing). Only then is
  // there an amount to set and a reset to show, in every mode (owner, 2026-11-07); the reset
  // gives the photo (or the whole sheet, none picked) the original back, one undo step.
  const { name: _name, base: _base, amount: _amount, ...colours } = current
  const { name: _n, base: _b, amount: _a, ...none } = NO_LOOK
  const coloured = JSON.stringify(colours) !== JSON.stringify(none)
  return (
    <SectionCard
      icon={<Palette />}
      title={t('look.looks')}
      // Simple mode has nothing but the amount: with the original, the card is its title alone.
      open={coloured || !simple}
      action={coloured && <ResetButton iconOnly label={t('look.resetLook')} onClick={() => onChange(() => NO_LOOK, true)} />}
    >
      {/* The user's own looks, to pick or forget (the strip under the photo lists them all),
          as the recent addresses are listed (owner, 2026-11-07). */}
      {!simple && looks.some((e) => e.kind === 'user') && (
        <Field label={t('look.yours')}>
          <ListRows>
            {looks
              .filter((e) => e.kind === 'user')
              .map((entry) => (
                <div key={entry.id} className="flex items-center gap-2">
                  <div className="min-w-0 flex-1" data-hint={entry.name}>
                    <ListRow
                      icon={<Bookmark />}
                      chosen={current.base === entry.id}
                      onClick={() => onChange(() => fullLook({ ...entry.look, name: entry.name }), true)}
                    >
                      {entry.name}
                    </ListRow>
                  </div>
                  <RemoveButton
                    iconOnly
                    label={t('look.remove')}
                    onClick={() => void commands.deleteLook(entry.id).then((r) => r.status === 'ok' && setLooks(r.data))}
                  />
                </div>
              ))}
          </ListRows>
        </Field>
      )}
      <Collapse open={coloured} stack="card">
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
      </Collapse>
      {!simple && (
        <>
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
        </>
      )}
    </SectionCard>
  )
}
