// SPDX-License-Identifier: GPL-3.0-only
import { ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { SidePanel } from '@/components/layout/side-panel'
import { commands, type CullFilter, type PhotoId, type RenderSettings } from '@/ipc/bindings'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { PhotoLookCards } from '@/features/look/photo-look-cards'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { PrintSizeCard } from '@/features/settings/print-size-card'
import { PanelNote } from '@/components/layout/panel-note'

type SimplePanelProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  /** Photos in the print basket: what Export sends out. */
  basketCount: number
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
  /** The photo on Simple mode's stage, whose colour the panel's cards change. */
  photoId: PhotoId | undefined
  /** After "Apply to all" changed the shared settings. */
  onSettingsReload: () => void
}

/** The print basket is what Simple mode exports (FR-048, FR-050). */
const BASKET: CullFilter = { ...ALL_PHOTOS, queue: 'print' }

/**
 * Simple mode's few choices (FR-048): prints or a share size, the print size and the 3 mm
 * border, the date stamp, and Export, with the same controls as Pro so both
 * stay in step.
 */
export function SimplePanel({ settings, onChange, exporter, basketCount, customStamps, revision, photoId, onSettingsReload }: SimplePanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  return (
    <SidePanel
      title={t('simple.title')}
      footer={
        <>
          <ExportSection share={settings?.exportTarget?.kind === 'share'} exporter={exporter} photoCount={basketCount} filter={BASKET} revision={revision} />
          <PanelFooter />
        </>
      }
    >
      <PrintSizeCard settings={settings} onChange={onChange} printSizes={printSizes} revision={revision} orientationIds={photoId !== undefined ? [photoId] : undefined} />
      {/* The photo's colour, after its size as in the previews (STYLE_RULE §4): the strip's look,
          its amount, and Faves (owner, 2026-11-07). */}
      {photoId !== undefined && <PhotoLookCards photoId={photoId} simple revision={revision} onAppliedToAll={onSettingsReload} />}
      <DateStampCard settings={settings} onChange={onChange} customStamps={customStamps} />
      <PanelNote icon={<ShoppingBasket />}>{t('simple.exportHint', { count: basketCount })}</PanelNote>
    </SidePanel>
  )
}
