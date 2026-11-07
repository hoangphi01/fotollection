// SPDX-License-Identifier: GPL-3.0-only
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

type SaveLookDialogProps = {
  open: boolean
  /** The name typed, or `null` when cancelled. */
  onAnswer(name: string | null): void
}

/** *Save as look* (FR-079): the name the photo's look is kept under on this machine. */
export function SaveLookDialog({ open, onAnswer }: SaveLookDialogProps) {
  const { t } = useTranslation()
  const [name, setName] = useState('')
  const id = useId()
  const answer = (value: string | null) => {
    setName('')
    onAnswer(value)
  }
  return (
    <Dialog open={open} onOpenChange={(next) => !next && answer(null)}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t('look.saveAs')}</DialogTitle>
          <DialogDescription>{t('look.saveAsBody')}</DialogDescription>
        </DialogHeader>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            if (name.trim()) answer(name)
          }}
        >
          <Field label={t('look.name')} htmlFor={id}>
            <Input id={id} autoFocus value={name} maxLength={60} placeholder={t('look.namePlaceholder')} onChange={(e) => setName(e.target.value)} />
          </Field>
          <DialogFooter className="pt-3">
            <Button type="button" variant="outline" onClick={() => answer(null)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={!name.trim()}>
              {t('look.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
