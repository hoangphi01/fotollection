// SPDX-License-Identifier: GPL-3.0-only
import { Ruler } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PanelNote } from '@/components/layout/panel-note'
import { commands } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'
import { formatNumber } from '@/lib/format'

/**
 * Under the print size, while the info frame is on *Fit whole photo* (FR-040): the print
 * comes out larger than the chosen size, because the full-size photo gets a 5 mm border
 * and the bar. Shown for the photo at hand, as the size depends on its ratio.
 */
const load = async (subject: string) => {
  const result = await commands.getCropGeometry(Number(subject), null)
  return result.status === 'ok' ? result.data : null
}

export function FitSizeNote({ photoId, revision }: { photoId: number | undefined; revision: number }) {
  const { t } = useTranslation()
  // The last answer holds its place while the next loads (LESSONS).
  const g = useLiveQuery(photoId === undefined ? null : String(photoId), revision, load).data
  if (!g?.printGrows || g.outputWidthCm === null || g.outputHeightCm === null) return null
  const inches = g.unit === 'in'
  const size = (cm: number) => formatNumber(inches ? cm / 2.54 : cm, 1, 1)
  return (
    <PanelNote icon={<Ruler />}>
      {t('frame.fitSize', {
        size: `${size(g.outputWidthCm)} × ${size(g.outputHeightCm)} ${inches ? 'in' : 'cm'}`,
      })}
    </PanelNote>
  )
}
