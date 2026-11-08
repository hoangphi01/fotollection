// SPDX-License-Identifier: GPL-3.0-only
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

type SaveLookDialogProps = {
  open: boolean
  /** The dialog's own words, for a rename or a move (the default: *Save as look*). */
  title?: string
  body?: string
  label?: string
  /** The name typed already: the current one, for a rename. */
  initial?: string
  okLabel?: string
  /** The name typed, or `null` when cancelled. */
  onAnswer(name: string | null): void
}

/** *Save as look* (FR-079): the name the photo's look is kept under on this machine. */
export function SaveLookDialog({ open, onAnswer, title, body, label, initial = '', okLabel }: SaveLookDialogProps) {
  const { t } = useTranslation()
  const [name, setName] = useState(initial)
  // Opened again for another look: its name, not the last one typed.
  const [shownFor, setShownFor] = useState<string | null>(null)
  if (open && shownFor !== initial) {
    setShownFor(initial)
    setName(initial)
  }
  const id = useId()
  const answer = (value: string | null) => {
    setName('')
    setShownFor(null)
    onAnswer(value)
  }
  return (
    <Dialog open={open} onOpenChange={(next) => !next && answer(null)}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{title ?? t('look.saveAs')}</DialogTitle>
          <DialogDescription>{body ?? t('look.saveAsBody')}</DialogDescription>
        </DialogHeader>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            if (name.trim()) answer(name)
          }}
        >
          <Field label={label ?? t('look.name')} htmlFor={id}>
            <Input id={id} autoFocus value={name} maxLength={60} placeholder={t('look.namePlaceholder')} onChange={(e) => setName(e.target.value)} />
          </Field>
          <DialogFooter className="pt-3">
            <Button type="button" variant="outline" onClick={() => answer(null)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={!name.trim()}>
              {okLabel ?? t('look.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
