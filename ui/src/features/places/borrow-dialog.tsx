// SPDX-License-Identifier: GPL-3.0-only
import { Check, LoaderCircle, MapPinCheck } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { commands, type Borrow, type BorrowReport } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

/** Minutes either side of a photo that the other devices' photos are looked for in. */
const WINDOWS = [5, 10, 30] as const

type BorrowDialogProps = { onClose: () => void }

/**
 * Borrow places (FR-031): a camera without GPS gets the place of a phone photo of the same
 * scene taken close in time. Sure matches are applied at once (one Undo step); the others
 * are shown side by side to accept or skip.
 */
export function BorrowDialog({ onClose }: BorrowDialogProps) {
  const { t } = useTranslation()
  const [minutes, setMinutes] = useState<number>(10)
  const [busy, setBusy] = useState(false)
  const [report, setReport] = useState<BorrowReport | null>(null)
  const [accepted, setAccepted] = useState<ReadonlySet<number>>(new Set())
  const [error, setError] = useState<string | null>(null)

  const search = async () => {
    setBusy(true)
    setError(null)
    const r = await commands.findBorrowedPlaces(minutes)
    setBusy(false)
    if (r.status === 'ok') {
      setReport(r.data)
      setAccepted(new Set(r.data.suggestions.map((s) => s.photo)))
    } else {
      setError(errorText(r.error))
    }
  }

  const toggle = (s: Borrow) =>
    setAccepted((prev) => {
      const next = new Set(prev)
      if (next.has(s.photo)) next.delete(s.photo)
      else next.add(s.photo)
      return next
    })

  const apply = async () => {
    const chosen = (report?.suggestions ?? []).filter((s) => accepted.has(s.photo))
    if (chosen.length > 0) {
      const r = await commands.acceptBorrowedPlaces(chosen.map((s) => ({ photo: s.photo, geonamesId: s.geonamesId })))
      if (r.status === 'error') return setError(errorText(r.error))
    }
    onClose()
  }

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t('places.borrowTitle')}</DialogTitle>
          <DialogDescription>{t('places.borrowBody')}</DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <span className="text-muted-foreground">{t('places.window')}</span>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={String(minutes)}
            onValueChange={(v) => v && setMinutes(Number(v))}
          >
            {WINDOWS.map((w) => (
              <ToggleGroupItem key={w} value={String(w)}>
                {t('places.minutes', { count: w })}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <Button size="sm" className="ml-auto" disabled={busy} onClick={() => void search()}>
            {busy ? <LoaderCircle className="animate-spin" data-icon="inline-start" /> : <MapPinCheck data-icon="inline-start" />}
            {t('places.borrowSearch')}
          </Button>
        </div>
        {report && (
          <p className="text-sm">
            {t('places.borrowApplied', { count: report.applied })}{' '}
            {report.suggestions.length === 0 && report.applied === 0 && t('places.borrowNone')}
          </p>
        )}
        {report && report.suggestions.length > 0 && (
          <>
            <p className="text-sm text-muted-foreground">{t('places.borrowReview')}</p>
            <ScrollArea className="max-h-[50vh]">
              <ul className="grid grid-cols-2 gap-2 pr-3">
                {report.suggestions.map((s) => {
                  const on = accepted.has(s.photo)
                  return (
                    <li key={s.photo}>
                      <button
                        type="button"
                        onClick={() => toggle(s)}
                        aria-pressed={on}
                        className={cn(
                          'relative w-full overflow-hidden rounded-md border text-left outline-none focus-visible:ring-2 focus-visible:ring-primary',
                          on && 'ring-2 ring-primary',
                        )}
                      >
                        <span className="grid grid-cols-2 gap-px bg-border">
                          <img src={thumbUrl(s.photo)} alt="" className="aspect-[4/3] w-full bg-muted object-cover" />
                          <img src={thumbUrl(s.donor)} alt="" className="aspect-[4/3] w-full bg-muted object-cover" />
                        </span>
                        {on && (
                          <Check className="absolute top-1 right-1 size-4 rounded-full bg-primary p-0.5 text-primary-foreground" />
                        )}
                        <span className="block truncate px-1.5 py-1 text-xs">
                          {s.name}
                          <span className="block text-muted-foreground">
                            {t('places.minutesApart', { count: Math.max(1, Math.round((s.gapS ?? 0) / 60)) })}
                          </span>
                        </span>
                      </button>
                    </li>
                  )
                })}
              </ul>
            </ScrollArea>
          </>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t('common.done')}
          </Button>
          <Button disabled={!report || report.suggestions.length === 0 || accepted.size === 0} onClick={() => void apply()}>
            {t('places.borrowAccept', { count: accepted.size })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
