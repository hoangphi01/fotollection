// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { RemoveButton } from '@/components/forms/danger-button'

export type SessionChoice = 'save' | 'discard' | 'cancel'

type DiscardSessionDialogProps = {
  /** The shots not saved yet; the dialog is open while this is not null. */
  count: number | null
  /** Closing one session tab, or the whole app with session tabs left (ADR-0028). */
  reason?: 'tab' | 'quit'
  onChoose: (choice: SessionChoice) => void
}

/** Closing a camera session's tab, or the app, with unsaved shots asks first (ADR-0028). */
export function DiscardSessionDialog({ count, reason = 'tab', onChoose }: DiscardSessionDialogProps) {
  const { t } = useTranslation()
  const quit = reason === 'quit'
  return (
    <AlertDialog open={count !== null} onOpenChange={(open) => !open && onChoose('cancel')}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t(quit ? 'capture.quitTitle' : 'capture.discardTitle')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(quit ? 'capture.quitBody' : 'capture.discardBody', { count: count ?? 0 })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <RemoveButton kind="delete" label={t('capture.discard')} onClick={() => onChoose('discard')} className="sm:mr-auto" />
          <AlertDialogCancel onClick={() => onChoose('cancel')}>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={() => onChoose('save')}>{t('capture.saveAll')}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
