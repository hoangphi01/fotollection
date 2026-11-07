// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { ClipboardPaste, Copy, Image, Images } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ResetButton } from '@/components/forms/danger-button'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { Kbd } from '@/components/ui/kbd'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { PhotoId } from '@/ipc/bindings'
import { MOD, isMac } from '@/lib/platform'
import { openCopySettings, pasteSettings, resetSettings, useSettingsCopy } from './settings-copy'

const SHIFT = isMac ? '⇧' : 'Shift'
const ALT = isMac ? '⌥' : 'Alt'

type SettingsButtonsProps = {
  /** The photo shown; none while nothing is. */
  photo: PhotoId | undefined
  /** Photos ticked in the gallery or Collage: Paste offers them too (Lightroom's Sync). */
  ticked: readonly PhotoId[]
  /** On a collage sheet the crop is the photo's collage crop (FR-035). */
  collage?: boolean
}

/**
 * Copy, Paste and Reset of a photo's develop settings (FR-097), after undo and redo in the
 * develop view's toolbar and the collage sheet's (owner, 2026-11-07). With photos ticked,
 * Paste asks: this photo, or the ticked ones.
 */
export function SettingsButtons({ photo, ticked, collage = false }: SettingsButtonsProps) {
  const { t } = useTranslation()
  const { copied } = useSettingsCopy()
  const others = ticked.filter((id) => id !== photo)
  const pasteHint = t('develop.pasteHint', { keys: `${SHIFT}${MOD}V` })
  return (
    <>
      <ToolbarButton
        icon={<Copy />}
        label={t('develop.copy')}
        data-hint={t('develop.copyKeyHint', { keys: `${SHIFT}${MOD}C` })}
        iconOnly
        disabled={photo === undefined}
        onClick={() => photo !== undefined && openCopySettings({ photo, collage })}
      />
      {others.length > 0 && copied ? (
        <Popover>
          <PopoverTrigger asChild>
            <ToolbarButton icon={<ClipboardPaste />} label={t('develop.paste')} data-hint={pasteHint} iconOnly />
          </PopoverTrigger>
          <PopoverContent align="start" className="w-72 p-1">
            <ListRows>
              <ListRow
                icon={<Image />}
                trailing={[SHIFT, MOD, 'V'].map((k) => <Kbd key={k}>{k}</Kbd>)}
                onClick={() => photo !== undefined && pasteSettings([photo], collage)}
              >
                {t('develop.pasteHere')}
              </ListRow>
              <ListRow
                icon={<Images />}
                trailing={[ALT, SHIFT, MOD, 'V'].map((k) => <Kbd key={k}>{k}</Kbd>)}
                onClick={() => pasteSettings(ticked, collage)}
              >
                {t('develop.pasteTicked', { count: ticked.length })}
              </ListRow>
            </ListRows>
          </PopoverContent>
        </Popover>
      ) : (
        <ToolbarButton
          icon={<ClipboardPaste />}
          label={t('develop.paste')}
          data-hint={copied ? pasteHint : t('develop.pasteNothing')}
          iconOnly
          disabled={photo === undefined || !copied}
          onClick={() => photo !== undefined && pasteSettings([photo], collage)}
        />
      )}
      <ResetButton
        iconOnly
        label={t('develop.reset')}
        hint={t('develop.resetHint', { keys: `${SHIFT}${MOD}R` })}
        disabled={photo === undefined}
        onClick={() => photo !== undefined && resetSettings([photo], collage)}
      />
    </>
  )
}
