// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import type { PrintAdvice, PrintMm } from '@/ipc/bindings'
import { formatNumber } from '@/lib/format'
import { sizeLabel } from '@/lib/print-size'

/** Whole centimetres read better than `35.2 × 52.8 cm` for "up to" sizes. */
const roughly = ({ shortMm, longMm }: PrintMm) => sizeLabel(Math.round(shortMm / 10) * 10, Math.round(longMm / 10) * 10)

/** One sentence under the print-size picker: how large the selected photo prints well (FR-026). */
export function PrintAdviceNote({ advice, fileName }: { advice: PrintAdvice; fileName: string }) {
  const { t } = useTranslation()
  const mp = advice.megapixels ?? (advice.width * advice.height) / 1e6
  return (
    <p className="text-xs text-muted-foreground">
      <span className="font-medium text-foreground">{fileName}</span>
      {': '}
      {t('advice.summary', {
        width: advice.width,
        height: advice.height,
        mp: formatNumber(mp, 1),
        sharp: roughly(advice.sharpUpTo),
        acceptable: roughly(advice.acceptableUpTo),
      })}
    </p>
  )
}
