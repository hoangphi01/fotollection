// SPDX-License-Identifier: GPL-3.0-only
import { Crop, Palette, Stamp, SunMedium } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogForm, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import type { SettingsParts } from '@/ipc/bindings'
import { anyPart, closeCopySettings, copySettings, useSettingsCopy } from './settings-copy'

const PARTS: { key: keyof SettingsParts; icon: ReactNode }[] = [
  { key: 'look', icon: <Palette /> },
  { key: 'rawLook', icon: <SunMedium /> },
  { key: 'crop', icon: <Crop /> },
  { key: 'stamp', icon: <Stamp /> },
]

/**
 * *Copy settings* (FR-097), as Lightroom's dialog: which of the photo's develop settings to
 * take, the choice kept for the session. One dialog for every surface, mounted once.
 */
export function CopySettingsDialog() {
  const { dialog, lastParts } = useSettingsCopy()
  return dialog ? <CopySettingsForm key={dialog.photo} initial={lastParts} /> : null
}

function CopySettingsForm({ initial }: { initial: SettingsParts }) {
  const { t } = useTranslation()
  const { dialog } = useSettingsCopy()
  const [parts, setParts] = useState(initial)
  return (
    <Dialog open onOpenChange={(open) => !open && closeCopySettings()}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t('develop.copyTitle')}</DialogTitle>
          <DialogDescription>{t('develop.copyBody')}</DialogDescription>
        </DialogHeader>
        <DialogForm
          onSubmit={(e) => {
            e.preventDefault()
            if (dialog && anyPart(parts)) void copySettings(dialog, parts)
          }}
        >
          {PARTS.map(({ key, icon }) => (
            <label key={key} className="flex items-center gap-2 text-sm [&>svg]:size-4 [&>svg]:text-muted-foreground">
              <Checkbox checked={parts[key]} onCheckedChange={(on) => setParts((p) => ({ ...p, [key]: on === true }))} />
              {icon}
              <span>{t(`develop.part.${key}`)}</span>
            </label>
          ))}
          <Hint>{t(dialog?.collage ? 'develop.copyCollageHint' : 'develop.copyHint')}</Hint>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={closeCopySettings}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={!anyPart(parts)}>
              {t('develop.copy')}
            </Button>
          </DialogFooter>
        </DialogForm>
      </DialogContent>
    </Dialog>
  )
}
