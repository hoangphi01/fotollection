// SPDX-License-Identifier: GPL-3.0-only
import { Ruler } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type CropGeometry } from '@/ipc/bindings'
import { formatNumber } from '@/lib/format'

/**
 * Under the print size, while the info frame is on *Fit whole photo* (FR-040): the print
 * comes out larger than the chosen size, because the full-size photo gets a 5 mm border
 * and the bar. Shown for the photo at hand, as the size depends on its ratio.
 */
export function FitSizeNote({ photoId, revision }: { photoId: number | undefined; revision: number }) {
  const { t } = useTranslation()
  const [geometry, setGeometry] = useState<{ id: number; value: CropGeometry } | null>(null)
  useEffect(() => {
    if (photoId === undefined) return
    let live = true
    void commands.getCropGeometry(photoId).then((result) => {
      if (live && result.status === 'ok') setGeometry({ id: photoId, value: result.data })
    })
    return () => {
      live = false
    }
  }, [photoId, revision])
  const g = geometry?.id === photoId ? geometry?.value : null
  if (!g?.cropDisabled || g.outputWidthCm === null || g.outputHeightCm === null) return null
  const inches = g.unit === 'in'
  const size = (cm: number) => formatNumber(inches ? cm / 2.54 : cm, 1, 1)
  return (
    <p className="flex gap-1.5 rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground">
      <Ruler className="mt-0.5 size-3.5 shrink-0 text-primary" />
      <span>
        {t('frame.fitSize', {
          size: `${size(g.outputWidthCm)} × ${size(g.outputHeightCm)} ${inches ? 'in' : 'cm'}`,
        })}
      </span>
    </p>
  )
}
