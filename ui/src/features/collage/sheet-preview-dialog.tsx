// SPDX-License-Identifier: GPL-3.0-only
import { ChevronLeft, ChevronRight, Crop, Images, LoaderCircle, RotateCw } from 'lucide-react'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { SectionCard } from '@/components/layout/section-card'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { commands, type CullFilter, type PhotoId, type PhotoSummary, type RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { CropEditor } from '@/features/viewer/crop-editor'
import { useShownSrc } from '@/features/viewer/use-shown-src'
import { useZoom } from '@/features/viewer/use-zoom'
import { ZoomControls } from '@/features/viewer/zoom-controls'
import { formatCapture } from '@/lib/format'
import { ownsArrowKeys } from '@/lib/keyboard'
import { sheetUrl, sourceUrl, thumbUrl } from '@/lib/photo-url'
import { MOD } from '@/lib/platform'
import { cn } from '@/lib/utils'
import { CollageCellsCard } from './collage-cells-card'
import { useCellGeometry } from './use-cell-geometry'
import type { CollageApi } from './use-collage'

type SheetPreviewDialogProps = {
  collage: CollageApi
  /** The preview is open on the chosen sheet. */
  open: boolean
  photos: PhotoSummary[]
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  revision: number
  generation: number
  onClose: () => void
}

/**
 * A collage sheet as it will print (FR-035), in the same frame as the photo preview: zoom,
 * the sheet's two photos with their crop, its settings, and the export of this sheet only.
 */
export function SheetPreviewDialog({
  collage,
  open,
  photos,
  settings,
  onSettingsChange,
  exporter,
  filter,
  revision,
  generation,
  onClose,
}: SheetPreviewDialogProps) {
  const { t } = useTranslation()
  const { sheets, index, sheet, cellIndex, cell } = collage
  const key = sheet ? `${sheet.first}:${sheet.second}` : undefined
  const shownSrc = useShownSrc(key, sheet ? sheetUrl(sheet.first, sheet.second, revision) : '')
  const [stage, setStage] = useState<HTMLDivElement | null>(null)
  const [cropping, setCropping] = useState(false)
  const zoom = useZoom(stage, `${key}:${cropping}`)
  const [sideBySide, setSideBySide] = useState(true)
  const contentRef = useRef<HTMLDivElement>(null)
  const photo = sheet ? (cellIndex === 0 ? sheet.first : sheet.second) : null
  const geometry = useCellGeometry(cropping ? photo : null, sheet, revision)
  const summary = (id: PhotoId | null) => (id === null ? undefined : photos.find((p) => p.id === id))

  useEffect(() => {
    if (!cropping) contentRef.current?.focus()
  }, [cropping])

  const go = (delta: number) => {
    setCropping(false)
    collage.select(Math.min(sheets.length - 1, Math.max(0, index + delta)), 0)
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (cropping || ownsArrowKeys(event.target)) return
    if (zoom.onKey(event)) return
    const key = event.key.toLowerCase()
    if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (key === '1' || key === '2') collage.select(index, key === '1' ? 0 : 1)
    else if (key === 'c' && photo !== null) setCropping(true)
    else if (key === 'r' && sheet && !event.metaKey && !event.ctrlKey) collage.turn(sheet.first)
    else return
    event.preventDefault()
  }

  const cells: [PhotoId | null, 0 | 1][] = sheet ? [[sheet.first, 0], [sheet.second, 1]] : []

  return (
    <ViewerFrame
      open={open && sheet !== undefined}
      onClose={onClose}
      contentRef={contentRef}
      onKeyDown={onKeyDown}
      onEscapeKeyDown={(e) => {
        // Esc leaves the crop editor first, then closes the preview.
        if (cropping) {
          e.preventDefault()
          setCropping(false)
        }
      }}
      stageRef={setStage}
      title={t('collage.sheetOf', { index: index + 1, count: sheets.length })}
      description={[summary(sheet?.first ?? null)?.fileName, summary(sheet?.second ?? null)?.fileName].filter(Boolean).join(' + ')}
      footer={
        cropping ? (
          <>
            <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> {t('viewer.level')} · <kbd className="font-mono">Enter</kbd>{' '}
            {t('common.done')} · <kbd className="font-mono">Esc</kbd> {t('viewer.leaveCrop')}
          </>
        ) : (
          <>
            <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> {t('collage.browse')} · <kbd className="font-mono">1</kbd>{' '}
            <kbd className="font-mono">2</kbd> {t('collage.choose')} · <kbd className="font-mono">C</kbd> {t('viewer.crop')} ·{' '}
            <kbd className="font-mono">R</kbd> {t('collage.turn')} · <kbd className="font-mono">Esc</kbd> {t('viewer.close')}
          </>
        )
      }
      stage={
        cropping && geometry && photo !== null ? (
          <CropEditor
            key={`${photo}:${revision}`}
            geometry={geometry}
            sourceSrc={sourceUrl(photo, revision)}
            onCancel={() => setCropping(false)}
            onCommit={(crop, straighten) => {
              setCropping(false)
              void commands.setPhotoCrop(photo, crop, straighten, false)
            }}
          />
        ) : (
          <>
            {shownSrc ? (
              <div className="relative max-h-full max-w-full" style={{ transform: zoom.transform }}>
                <img
                  src={shownSrc}
                  alt={t('collage.sheetLabel')}
                  draggable={false}
                  onLoad={(e) => setSideBySide(e.currentTarget.naturalWidth >= e.currentTarget.naturalHeight)}
                  onPointerDown={zoom.pointer.onPointerDown}
                  onPointerMove={zoom.pointer.onPointerMove}
                  onPointerUp={zoom.pointer.onPointerUp}
                  className="block max-h-[calc(90vh-4rem)] max-w-full animate-in rounded-sm object-contain shadow-2xl fade-in-0"
                />
                {!zoom.zoomed && (
                  <div className={cn('absolute inset-0 flex', !sideBySide && 'flex-col')}>
                    {cells.map(([id, c]) => (
                      <button
                        key={c}
                        type="button"
                        disabled={id === null}
                        aria-label={summary(id)?.fileName ?? t('collage.emptyCell')}
                        className={cn('flex-1 rounded-sm', cellIndex === c && id !== null && 'ring-2 ring-primary ring-inset')}
                        onClick={() => collage.select(index, c)}
                        onDoubleClick={() => id !== null && setCropping(true)}
                      />
                    ))}
                  </div>
                )}
              </div>
            ) : (
              <div className="absolute flex flex-col items-center gap-2 text-white/60">
                <LoaderCircle className="size-7 animate-spin" />
                <span className="text-xs">{t('viewer.rendering')}</span>
              </div>
            )}
            <div className="absolute top-3 left-3 flex gap-1.5">
              <Button variant="secondary" size="sm" disabled={photo === null} onClick={() => setCropping(true)}>
                <Crop data-icon="inline-start" /> {t('viewer.crop')}
              </Button>
              <Button variant="secondary" size="sm" disabled={!sheet} onClick={() => sheet && collage.turn(sheet.first)}>
                <RotateCw data-icon="inline-start" /> {t('collage.turn')}
              </Button>
            </div>
            <Button
              variant="secondary"
              size="icon"
              aria-label={t('collage.previous')}
              data-hint={t('collage.previous')}
              className="absolute left-3 opacity-80 hover:opacity-100"
              disabled={index === 0}
              onClick={() => go(-1)}
            >
              <ChevronLeft />
            </Button>
            <Button
              variant="secondary"
              size="icon"
              aria-label={t('collage.next')}
              data-hint={t('collage.next')}
              className="absolute right-3 opacity-80 hover:opacity-100"
              disabled={index >= sheets.length - 1}
              onClick={() => go(1)}
            >
              <ChevronRight />
            </Button>
            <span className="absolute bottom-4 left-1/2 -translate-x-1/2 rounded-full bg-black/60 px-3 py-1 text-xs text-white/80 tabular-nums">
              {index + 1} / {sheets.length}
            </span>
            <ZoomControls
              percent={zoom.percent}
              zoomed={zoom.zoomed}
              canZoomIn={zoom.canZoomIn}
              loading={false}
              mod={MOD}
              onZoomIn={zoom.zoomIn}
              onZoomOut={zoom.zoomOut}
              onFit={zoom.reset}
            />
          </>
        )
      }
    >
      <SectionCard icon={<Images />} title={<span className="text-sm font-semibold">{t('collage.photosOnSheet')}</span>}>
        {cells.map(([id, c]) => {
          const p = summary(id)
          return (
            <div
              key={c}
              className={cn('flex items-center gap-2.5 rounded-md p-1.5', cellIndex === c && p && 'bg-primary/10')}
            >
              {p ? (
                <>
                  <button type="button" className="shrink-0" onClick={() => collage.select(index, c)} aria-label={p.fileName}>
                    <img src={thumbUrl(p.id, 0, generation)} alt="" className="size-10 rounded object-cover" />
                  </button>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm">{p.fileName}</p>
                    <p className="text-xs text-muted-foreground">{formatCapture(p.capture)}</p>
                  </div>
                  <Button
                    variant="outline"
                    size="icon-sm"
                    aria-label={t('viewer.crop')}
                    data-hint={t('viewer.crop')}
                    onClick={() => {
                      collage.select(index, c)
                      setCropping(true)
                    }}
                  >
                    <Crop />
                  </Button>
                </>
              ) : (
                <p className="text-xs text-muted-foreground">{t('collage.emptyCell')}</p>
              )}
            </div>
          )
        })}
      </SectionCard>
      <CollageCellsCard settings={settings} onChange={onSettingsChange} cell={cell} />
      {sheet && (
        <ExportSection
          exporter={exporter}
          photoCount={sheet.second === null ? 1 : 2}
          filter={filter}
          only={sheet.first}
          revision={revision}
          sheets={1}
        />
      )}
    </ViewerFrame>
  )
}
