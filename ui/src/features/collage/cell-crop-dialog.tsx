// SPDX-License-Identifier: GPL-3.0-only
import { Ruler } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { commands, type CropGeometry, type PhotoId, type SheetPair } from '@/ipc/bindings'
import { CropEditor } from '@/features/viewer/crop-editor'
import { formatNumber } from '@/lib/format'
import { sourceUrl } from '@/lib/photo-url'

type CellCropDialogProps = {
  photo: PhotoId
  sheet: SheetPair
  fileName: string
  revision: number
  onClose: () => void
}

/**
 * The crop editor for one photo of a collage sheet (FR-035, FR-006), in the preview's frame:
 * the crop fills the photo's area in its cell, and the sizes on its edges are the print's.
 */
export function CellCropDialog({ photo, sheet, fileName, revision, onClose }: CellCropDialogProps) {
  const { t } = useTranslation()
  const [geometry, setGeometry] = useState<CropGeometry | null>(null)
  const { first, second } = sheet
  useEffect(() => {
    let live = true
    void commands.getCropGeometry(photo, { first, second }).then((result) => {
      if (live && result.status === 'ok') setGeometry(result.data)
    })
    return () => {
      live = false
    }
  }, [photo, first, second])
  const cm = (v: number | null) => formatNumber(v ?? 0, 1)

  return (
    <ViewerFrame
      open
      onClose={onClose}
      title={fileName}
      description={t('collage.cropTitle')}
      footer={
        <>
          <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> {t('viewer.level')} ·{' '}
          <kbd className="font-mono">Enter</kbd> {t('common.done')} · <kbd className="font-mono">Esc</kbd> {t('viewer.leaveCrop')}
        </>
      }
      stage={
        geometry && (
          <CropEditor
            geometry={geometry}
            sourceSrc={sourceUrl(photo, revision)}
            onCancel={onClose}
            onCommit={(crop, straighten) => {
              void commands.setPhotoCrop(photo, crop, straighten, false)
              onClose()
            }}
          />
        )
      }
    >
      {geometry && (
        <p className="flex gap-1.5 rounded-md bg-muted/60 px-2.5 py-2 text-xs text-muted-foreground">
          <Ruler className="mt-0.5 size-3.5 shrink-0 text-primary" />
          {t('collage.cropSize', {
            photo: `${cm(geometry.printWidthCm)} × ${cm(geometry.printHeightCm)}`,
            area: `${cm(geometry.outputWidthCm)} × ${cm(geometry.outputHeightCm)}`,
          })}
        </p>
      )}
    </ViewerFrame>
  )
}
