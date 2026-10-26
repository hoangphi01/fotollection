// SPDX-License-Identifier: GPL-3.0-only
import { Check } from 'lucide-react'
import { useEffect, useState } from 'react'
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
import { commands, type PhotoSummary } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { formatCapture } from '@/lib/format'
import { thumbUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'
import { formatOffset } from './clock-offset'
import { useCameraClocks } from './use-camera-clocks'

type ClockMatchDialogProps = {
  /** The photo whose camera clock is corrected. */
  photoId: number
  clockId: number
  open: boolean
  onClose: () => void
}

/**
 * The FR-030 helper: pick, among the other devices' photos closest in time, the one taken at
 * the same moment as this photo; the camera's clock offset follows from the two times.
 */
export function ClockMatchDialog({ photoId, clockId, open, onClose }: ClockMatchDialogProps) {
  const { t } = useTranslation()
  const { clocks, setOffset } = useCameraClocks()
  const [candidates, setCandidates] = useState<PhotoSummary[]>([])
  const [picked, setPicked] = useState<number | null>(null)
  const [offset, setOffsetValue] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const clock = clocks.find((c) => c.id === clockId)

  // Mounted only while open (see the preview's Captured card), so state starts fresh.
  useEffect(() => {
    void commands.clockMatchCandidates(photoId, 24).then((r) => {
      if (r.status === 'ok') setCandidates(r.data)
    })
  }, [photoId])

  const pick = async (reference: number) => {
    setPicked(reference)
    const r = await commands.clockOffsetBetween(photoId, reference)
    if (r.status === 'ok') {
      setOffsetValue(r.data ?? 0)
      setError(null)
    } else {
      setOffsetValue(null)
      setError(errorText(r.error))
    }
  }

  const apply = async () => {
    if (!clock || offset === null) return
    if (await setOffset(clock, offset)) onClose()
  }

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t('cameras.matchTitle', { camera: clock?.label ?? '' })}</DialogTitle>
          <DialogDescription>{t('cameras.matchBody')}</DialogDescription>
        </DialogHeader>
        {candidates.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('cameras.matchNone')}</p>
        ) : (
          <ScrollArea className="max-h-[50vh]">
            <ul className="grid grid-cols-4 gap-2 pr-3">
              {candidates.map((c) => (
                <li key={c.id}>
                  <button
                    type="button"
                    onClick={() => void pick(c.id)}
                    aria-pressed={picked === c.id}
                    data-hint={c.fileName}
                    className={cn(
                      'relative w-full overflow-hidden rounded-md border text-left outline-none focus-visible:ring-2 focus-visible:ring-primary',
                      picked === c.id && 'ring-2 ring-primary',
                    )}
                  >
                    <img src={thumbUrl(c.id)} alt="" className="aspect-[4/3] w-full bg-muted object-cover" />
                    {picked === c.id && (
                      <Check className="absolute top-1 right-1 size-4 rounded-full bg-primary p-0.5 text-primary-foreground" />
                    )}
                    <span className="block truncate px-1.5 py-1 text-xs">
                      {formatCapture(c.capture)}
                      <span className="block truncate text-muted-foreground">{c.camera ?? t('viewer.unknownCamera')}</span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
        {offset !== null && (
          <p className="text-sm">
            {t('cameras.matchResult', { camera: clock?.label ?? '', offset: formatOffset(offset), count: clock?.photos ?? 0 })}
          </p>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button disabled={offset === null || !clock} onClick={() => void apply()}>
            {t('cameras.matchApply')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
