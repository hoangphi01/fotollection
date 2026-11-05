// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Crop } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Hint } from '@/components/forms/field'
import { SectionCard } from '@/components/layout/section-card'
import type { PhotoSummary, PrintAdvice, PrintSize, RenderSettings } from '@/ipc/bindings'
import { ExportTargetPicker } from '@/features/export/export-target-picker'
import { FitSizeNote } from '@/features/frame/fit-size-note'
import { PrintAdviceNote } from '@/features/print-advice/print-advice-note'
import { BorderSwitch } from './border-switch'
import { PrintSizePicker } from './print-size-picker'

type PrintSizeCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  printSizes: PrintSize[] | undefined
  /** The photo the advice and the grown-print note speak of, when there is one. */
  photo?: PhotoSummary
  advice?: PrintAdvice | null
  revision: number
  /** The preview's own rows about this photo (its DPI, its crop), at the foot. */
  children?: ReactNode
}

/**
 * *Print / export size*, one card in Pro, Simple and the preview (STYLE_RULE §4, the lego
 * rule): prints or a share size, then the print size, the advice for the photo, the note when
 * the print grows, the hint and the 3 mm border, always in that order.
 */
export function PrintSizeCard({ settings, onChange, printSizes, photo, advice, revision, children }: PrintSizeCardProps) {
  const { t } = useTranslation()
  const share = settings?.exportTarget?.kind === 'share'
  return (
    <SectionCard icon={<Crop />} title={t('settings.printSize')}>
      {/* Print or share is chosen with the size, so the export card only exports (FR-053). */}
      <ExportTargetPicker settings={settings} onChange={onChange} />
      {!share && (
        <>
          <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onChange} advice={advice} />
          {advice && photo && <PrintAdviceNote advice={advice} fileName={photo.fileName} />}
          <FitSizeNote photoId={photo?.id} revision={revision} />
          <Hint>{t('settings.printSizeHint')}</Hint>
          <BorderSwitch settings={settings} onChange={onChange} />
        </>
      )}
      {children}
    </SectionCard>
  )
}
