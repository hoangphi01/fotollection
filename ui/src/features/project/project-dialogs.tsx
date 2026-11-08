// SPDX-License-Identifier: GPL-3.0-only
import { FolderSearch } from 'lucide-react'
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Field, RowList } from '@/components/forms/field'
import { Input } from '@/components/ui/input'
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
          {/* Discards the changes: the red block (STYLE_RULE §4), as the capture's discard. */}
          <RemoveButton kind="delete" label={t('project.dontSave')} onClick={() => onChoose?.('discard')} className="sm:mr-auto" />
          <AlertDialogCancel onClick={() => onChoose?.('cancel')}>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={() => onChoose?.('save')}>{t('project.save')}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

type NewProjectDialogProps = {
  onAnswer: ((name: string | null) => void) | null
}

/** *New project*: its name, then *Create* or *Cancel* (owner, 2026-11-06). Rust checks the
 * name (`rename_document`); an empty one keeps *Untitled* until the first Save. */
export function NewProjectDialog({ onAnswer }: NewProjectDialogProps) {
  const { t } = useTranslation()
  const [name, setName] = useState('')
  const id = useId()
  const answer = (value: string | null) => {
    setName('')
    onAnswer?.(value)
  }
  return (
    <Dialog open={onAnswer !== null} onOpenChange={(open) => !open && answer(null)}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t('project.new')}</DialogTitle>
          <DialogDescription>{t('project.newBody')}</DialogDescription>
        </DialogHeader>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            answer(name)
          }}
        >
          <Field label={t('project.rename')} htmlFor={id}>
            <Input id={id} autoFocus value={name} maxLength={120} placeholder={t('project.newPlaceholder')} onChange={(e) => setName(e.target.value)} />
          </Field>
          <DialogFooter className="pt-3">
            <RemoveButton kind="stop" label={t('common.cancel')} onClick={() => answer(null)} />
            <Button type="submit">{t('project.create')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
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
        {/* Rows that each hold a button, 12 px apart (STYLE_RULE §4, RowList). */}
        <RowList>
          {missing.map((m) => (
            <li key={m.id} className="flex items-center gap-2 rounded-md border px-3 py-2">
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{m.name}</p>
                <p className="truncate text-xs text-muted-foreground" data-hint={m.path}>
                  {m.path}
                </p>
              </div>
              <Button size="sm" variant="outline" onClick={() => onLocate(m.id)}>
                <FolderSearch data-icon="inline-start" /> {t('project.locate')}
              </Button>
            </li>
          ))}
        </RowList>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t('project.later')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
