// SPDX-License-Identifier: GPL-3.0-only
import { FolderSearch } from 'lucide-react'
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
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { MissingFolder } from '@/ipc/bindings'
import type { UnsavedChoice } from './use-project'

type UnsavedDialogProps = {
  name: string
  onChoose: ((choice: UnsavedChoice) => void) | null
}

/** "Save changes to Trip?" before New or Open replaces the library (FR-023). */
export function UnsavedDialog({ name, onChoose }: UnsavedDialogProps) {
  const { t } = useTranslation()
  return (
    <AlertDialog open={onChoose !== null} onOpenChange={(open) => !open && onChoose?.('cancel')}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('project.unsavedTitle', { name })}</AlertDialogTitle>
          <AlertDialogDescription>{t('project.unsavedBody')}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <Button variant="ghost" className="sm:mr-auto" onClick={() => onChoose?.('discard')}>
            {t('project.dontSave')}
          </Button>
          <AlertDialogCancel onClick={() => onChoose?.('cancel')}>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={() => onChoose?.('save')}>{t('project.save')}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

type RelinkDialogProps = {
  missing: MissingFolder[]
  open: boolean
  onLocate: (folderId: number) => void
  onClose: () => void
}

/** Folders of an opened project that moved: point each one at its new place (FR-023). */
export function RelinkDialog({ missing, open, onLocate, onClose }: RelinkDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open && missing.length > 0} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('project.relinkTitle', { count: missing.length })}</DialogTitle>
          <DialogDescription>{t('project.relinkBody')}</DialogDescription>
        </DialogHeader>
        <ul className="space-y-2">
          {missing.map((m) => (
            <li key={m.id} className="flex items-center gap-3 rounded-md border px-3 py-2">
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{m.name}</p>
                <p className="truncate text-xs text-muted-foreground" title={m.path}>
                  {m.path}
                </p>
              </div>
              <Button size="sm" variant="outline" onClick={() => onLocate(m.id)}>
                <FolderSearch data-icon="inline-start" /> {t('project.locate')}
              </Button>
            </li>
          ))}
        </ul>
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t('project.later')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
