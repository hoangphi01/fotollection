// SPDX-License-Identifier: GPL-3.0-only
import { Download, Sparkles, TriangleAlert } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { languageInfo } from '@/i18n'
import { commands, type UpdateCheck } from '@/ipc/bindings'

/**
 * The check at start (FR-103): once per launch, unless turned off in Settings → Updates, a
 * small window says a newer version is out, with *What's new* and *Download*; below the
 * minimum safe version it carries a red warning with the reason the release gives. It only
 * reads; nothing is downloaded until the user asks. *Later* closes it until the next launch.
 */
export function UpdatePopup() {
  const { t, i18n } = useTranslation()
  const [update, setUpdate] = useState<UpdateCheck | null>(null)

  useEffect(() => {
    let live = true
    void commands.checkForUpdateAtStart().then((result) => {
      if (live && result.status === 'ok' && result.data) setUpdate(result.data)
    })
    return () => {
      live = false
    }
  }, [])

  const reason = update ? (languageInfo(i18n.language).value === 'vi' ? update.reasonVi : update.reasonEn) : ''
  return (
    <Dialog open={update !== null} onOpenChange={(open) => !open && setUpdate(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('update.popupTitle', { version: update?.latest ?? '' })}</DialogTitle>
          <DialogDescription>{t('update.popupBody', { current: update?.current ?? '' })}</DialogDescription>
        </DialogHeader>
        {update?.belowMinSafe && (
          <PanelNote danger icon={<TriangleAlert />}>
            {t('update.belowMinSafe', { version: update.minSafe ?? '' })} {reason}
          </PanelNote>
        )}
        <Hint>{t('update.replaces')}</Hint>
        <DialogFooter>
          <Button variant="outline" onClick={() => setUpdate(null)}>
            {t('update.later')}
          </Button>
          <Button variant="outline" onClick={() => void commands.openUpdatePage()}>
            <Sparkles data-icon="inline-start" /> {t('update.whatsNew')}
          </Button>
          <Button onClick={() => void commands.openUpdateDownload()}>
            <Download data-icon="inline-start" /> {t('update.download')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
