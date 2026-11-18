// SPDX-License-Identifier: GPL-3.0-only
import { Eraser, ShoppingBasket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PanelFooter } from '@/features/about/panel-footer'
import { SidePanel } from '@/components/layout/side-panel'
import { commands, type CleanupRow, type CullFilter, type PhotoId, type RenderSettings } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
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
  /** RollSense's clean-up rows for the folder shown (FR-141): a phone's roll is offered a tidy-up. */
  cleanup?: CleanupRow[]
  onCleanup(): void
}

/** The print basket is what Simple mode exports (FR-048, FR-050). */
const BASKET: CullFilter = { ...ALL_PHOTOS, queue: 'print' }

/**
 * Simple mode's few choices (FR-048): prints or a share size, the print size and the 3 mm
 * border, the date stamp, and Export, with the same controls as Pro so both
 * stay in step.
 */
export function SimplePanel({ settings, onChange, exporter, basketCount, customStamps, revision, photoId, onSettingsReload, cleanup, onCleanup }: SimplePanelProps) {
  const { t } = useTranslation()
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const countOf = (kind: CleanupRow['kind']) => cleanup?.find((r) => r.kind === kind)?.count ?? 0
  const tidy =
    countOf('screenshots') + countOf('duplicates') > 0 ? { screenshots: countOf('screenshots'), copies: countOf('duplicates') } : null
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
      {/* A roll with screenshots or extra copies is offered a tidy-up first (FR-141, owner 2026-11-18). */}
      {tidy && (
        <PanelNote
          icon={<Eraser />}
          action={
            <Button variant="outline" size="sm" onClick={onCleanup}>
              {t('simple.cleanupOpen')}
            </Button>
          }
        >
          {t('simple.cleanup', { screenshots: tidy.screenshots, copies: tidy.copies })}
        </PanelNote>
      )}
    </SidePanel>
  )
}
