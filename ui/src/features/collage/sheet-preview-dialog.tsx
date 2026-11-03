// SPDX-License-Identifier: GPL-3.0-only
import { ChevronLeft, ChevronRight, Crop, Download, Images, LoaderCircle, Mail, RotateCw, RotateCwSquare, ShoppingBasket } from 'lucide-react'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { InspectorSection } from '@/components/layout/inspector-section'
import { KeyHints } from '@/components/layout/key-hints'
import { StageSpinner } from '@/components/layout/stage-spinner'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { commands, type CullFilter, type PhotoId, type PhotoSummary, type Queue, type RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { CropEditor } from '@/features/viewer/crop-editor'
import { sendTo } from '@/features/viewer/send-to'
import { useShownSrc } from '@/features/viewer/use-shown-src'
import { useZoom } from '@/features/viewer/use-zoom'
import { ZoomControls } from '@/features/viewer/zoom-controls'
import { formatCapture, formatNumber } from '@/lib/format'
import { ownsArrowKeys } from '@/lib/keyboard'
import { sheetUrl, sourceUrl, thumbUrl } from '@/lib/photo-url'
import { MOD } from '@/lib/platform'
import { cn } from '@/lib/utils'
import { CollageCellsCard } from './collage-cells-card'
import { PostcardCard } from './postcard-card'
import { PostcardTextField } from './postcard-text-field'
import { sheetCells } from './sheet-cells'
import { SheetCellsOverlay } from './sheet-cells-overlay'
import { SheetSizeCard } from './sheet-size-card'
import { useCellGeometry } from './use-cell-geometry'
import type { CollageApi } from './use-collage'
import { useSheetInfo } from './use-sheet-info'
import { useSheetShape } from './use-sheet-shape'

type SheetPreviewDialogProps = {
  collage: CollageApi
  /** The preview is open on the chosen sheet. */
  open: boolean
  photos: PhotoSummary[]
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  exporter: ExportApi
  filter: CullFilter
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
  generation: number
  onClose: () => void
}

/**
 * A collage sheet as it will print (FR-035), in the same frame as the photo preview: zoom,
 * the same cards as the Collage panel in the same order, the export of this sheet only, then
 * the sheet's two photos with their basket and crop. A photo's crop here is its crop on
 * sheets, apart from its print crop.
 */
export function SheetPreviewDialog({
  collage,
  open,
  photos,
  settings,
  onSettingsChange,
  exporter,
  filter,
  customStamps,
  revision,
  generation,
  onClose,
}: SheetPreviewDialogProps) {
  const { t } = useTranslation()
  const { sheets, index, sheet } = collage
  const key = sheet ? `${sheet.first}:${sheet.second}` : undefined
  const src = sheet ? sheetUrl(sheet.first, sheet.second, revision) : ''
  const { src: shownSrc, error: shownError } = useShownSrc(key, src)
  const shape = useSheetShape(shownSrc ?? '')
  const [stage, setStage] = useState<HTMLDivElement | null>(null)
  const [cropping, setCropping] = useState(false)
  const zoom = useZoom(stage, `${key}:${cropping}`)
  const contentRef = useRef<HTMLDivElement>(null)
  // No photo is chosen until the user picks one, so the sheet shows as it prints.
  const [picked, setPicked] = useState<{ key: string | undefined; cell: 0 | 1 } | null>(null)
  const chosen = picked && picked.key === key ? picked.cell : null
  const choose = (cell: 0 | 1 | null) => setPicked(cell === null ? null : { key, cell })
  const photo = sheet && chosen !== null ? (chosen === 0 ? sheet.first : sheet.second) : null
  const info = useSheetInfo(open ? sheet : undefined, revision)
  const cell = info?.cells.find((c) => c.id === photo) ?? info?.cells[0]
  const geometry = useCellGeometry(cropping ? photo : null, sheet, revision)
  const summary = (id: PhotoId | null) => (id === null ? undefined : photos.find((p) => p.id === id))
  const cells = sheet ? sheetCells(sheet) : []

  useEffect(() => {
    if (!cropping) contentRef.current?.focus()
  }, [cropping])

  // ↑ to the print basket, ↓ set aside, for the chosen photo (FR-050); the choice moves on
  // to the next photo of the sheets, as in the photo preview.
  const nextPhoto = () => {
    if (chosen === 0 && sheet?.second != null) choose(1)
    else go(1)
  }
  const send = (queue: Queue) => {
    if (photo !== null) sendTo(photo, queue, filter.queue ?? 'gallery', nextPhoto)
  }
  const go = (delta: number) => {
    setCropping(false)
    choose(null)
    collage.select(Math.min(sheets.length - 1, Math.max(0, index + delta)), 0)
  }
  // A quarter turn of the photo in its cell only (FR-035): its print keeps its orientation.
  const postcard = settings?.collage?.postcard?.enabled ?? false
  const rotatePhoto = (id: PhotoId) => void commands.rotateCollagePhoto(id)
  const onKeyDown = (event: KeyboardEvent) => {
    if (cropping || ownsArrowKeys(event.target)) return
    if (zoom.onKey(event)) return
    const pressed = event.key.toLowerCase()
    if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (event.key === 'ArrowUp') send('print')
    else if (event.key === 'ArrowDown') send('aside')
    else if (pressed === '1' || pressed === '2') choose(pressed === '1' ? 0 : 1)
    else if (pressed === 'c' && photo !== null) setCropping(true)
    // ⇧R turns the chosen photo in its cell; R alone turns the sheet.
    else if (pressed === 'r' && event.shiftKey && photo !== null) rotatePhoto(photo)
    else if (pressed === 'r' && sheet && !postcard && !event.metaKey && !event.ctrlKey) collage.turn(sheet.first)
    else return
    event.preventDefault()
  }
  const names = cells.flatMap(([id]) => summary(id)?.fileName ?? [])

  return (
    <ViewerFrame
      open={open && sheet !== undefined}
      onClose={onClose}
      contentRef={contentRef}
      onKeyDown={onKeyDown}
      onEscapeKeyDown={(e) => {
        // Esc leaves the crop editor first, then lets go of the chosen photo, then closes.
        if (cropping) {
          e.preventDefault()
          setCropping(false)
        } else if (chosen !== null) {
          e.preventDefault()
          choose(null)
        }
      }}
      stageRef={setStage}
      // A click on the ground around the sheet lets go of the chosen photo, like Esc.
      onStageGroundClick={() => !cropping && choose(null)}
      title={t('collage.sheetOf', { index: formatNumber(index + 1), count: formatNumber(sheets.length) })}
      description={names.length === 2 ? t('collage.pairNames', { first: names[0], second: names[1] }) : names[0]}
      footer={
        <KeyHints
          hints={
            cropping
              ? [
                  { keys: ['←', '→'], label: t('viewer.level') },
                  { keys: ['Enter'], label: t('common.done') },
                  { keys: ['Esc'], label: t('viewer.leaveCrop') },
                ]
              : [
                  { keys: ['←', '→'], label: t('collage.browse') },
                  { keys: [`${MOD} + −`], label: t('viewer.zoom') },
                  { keys: ['1', '2'], label: t('collage.choose') },
                  { keys: ['↑'], label: t('viewer.upBasket') },
                  { keys: ['↓'], label: t('viewer.downAside') },
                  { keys: ['C'], label: t('viewer.crop') },
                  ...(postcard ? [] : [{ keys: ['R'], label: t('collage.turn') }]),
                  { keys: ['⇧R'], label: t('collage.rotatePhoto') },
                  { keys: ['Esc'], label: t('viewer.close') },
                ]
          }
        />
      }
      stage={
        cropping && geometry && photo !== null ? (
          <CropEditor
            key={photo}
            geometry={geometry}
            sourceSrc={sourceUrl(photo, revision, true)}
            onCancel={() => setCropping(false)}
            onCommit={(crop, straighten) => {
              setCropping(false)
              // The crop on sheets, apart from the print crop (FR-035).
              void commands.setCollageCrop(photo, crop, straighten)
            }}
          />
        ) : (
          <>
            {shownSrc && sheet ? (
              <div className="relative max-h-full max-w-full" style={{ transform: zoom.transform }}>
                <img
                  src={shownSrc}
                  alt={t('collage.sheetLabel')}
                  draggable={false}
                  onLoad={shape.onLoad}
                  onPointerDown={zoom.pointer.onPointerDown}
                  onPointerMove={zoom.pointer.onPointerMove}
                  onPointerUp={zoom.pointer.onPointerUp}
                  className="block max-h-[calc(90vh-4rem)] max-w-full animate-in object-contain stage-photo fade-in-0"
                />
                {!zoom.zoomed && (
                  <SheetCellsOverlay sheet={sheet} sideBySide={shape.sideBySide} postcard={postcard}>
                    {(id, c) => (
                      <button
                        key={c}
                        type="button"
                        disabled={id === null}
                        aria-label={summary(id)?.fileName ?? t('collage.emptyCell')}
                        className={cn('flex-1 rounded-sm', chosen === c && id !== null && 'ring-2 ring-primary ring-inset')}
                        onClick={() => choose(chosen === c ? null : c)}
                        onDoubleClick={() => {
                          if (id === null) return
                          choose(c)
                          setCropping(true)
                        }}
                      />
                    )}
                  </SheetCellsOverlay>
                )}
              </div>
            ) : (
              <StageSpinner error={shownError} />
            )}
            <div className="absolute top-3 left-3 flex gap-1.5">
              {/* A disabled button gets no pointer events: its hint sits on a wrapper. */}
              <span data-hint={photo === null ? t('collage.pickToCrop') : undefined}>
                <Button variant="secondary" size="sm" disabled={photo === null} onClick={() => setCropping(true)}>
                  <Crop data-icon="inline-start" /> {t('viewer.crop')}
                </Button>
              </span>
              {/* A postcard is held the way the panel says: only the photo turns. */}
              {!postcard && (
                <Button variant="secondary" size="sm" disabled={!sheet} onClick={() => sheet && collage.turn(sheet.first)}>
                  <RotateCw data-icon="inline-start" /> {t('collage.turn')}
                </Button>
              )}
              {/* Turns the chosen photo in its cell, so a landscape photo fills a standing cell. */}
              <span data-hint={photo === null ? t('collage.pickToRotate') : t('collage.rotatePhotoHint')}>
                <Button variant="secondary" size="sm" disabled={photo === null} onClick={() => photo !== null && rotatePhoto(photo)}>
                  <RotateCwSquare data-icon="inline-start" /> {t('collage.rotatePhoto')}
                </Button>
              </span>
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
            <span className="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-black/60 px-3 py-1 text-xs text-white/80 tabular-nums">
              {/* The new render after a change of setting is on its way; the spinner keeps
                  its place when hidden, so the text never shifts. */}
              <LoaderCircle
                className={cn('size-3 animate-spin', (shownSrc === null || shownSrc === src) && 'invisible')}
                aria-label={shownSrc !== null && shownSrc !== src ? t('viewer.rendering') : undefined}
                aria-hidden={shownSrc === null || shownSrc === src}
              />
              {t('collage.position', { index: formatNumber(index + 1), count: formatNumber(sheets.length) })}
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
      {/* The same cards in the same order as the Collage panel (STYLE_RULE §4); the sheet's
          own photos follow, as a photo's facts follow in its preview. */}
      <SheetSizeCard settings={settings} onChange={onSettingsChange} />
      <PostcardCard settings={settings} onChange={onSettingsChange} />
      {postcard && sheet && (
        <InspectorSection icon={<Mail className="size-4" />} title={t('postcard.text')}>
          <PostcardTextField photo={sheet.first} revision={revision} />
        </InspectorSection>
      )}
      <CollageCellsCard settings={settings} onChange={onSettingsChange} cell={cell} />
      <DateStampCard
        settings={settings}
        onChange={onSettingsChange}
        customStamps={customStamps}
        photos={cells.flatMap(([id]) => (id === null ? [] : [{ id, name: summary(id)?.fileName ?? '' }]))}
        revision={revision}
      />
      {sheet && (
        <InspectorSection icon={<Download className="size-4" />} title={t('export.title')}>
          <ExportSection
            exporter={exporter}
            photoCount={sheet.second === null ? 1 : 2}
            filter={filter}
            only={sheet.first}
            revision={revision}
            sheets={1}
          />
        </InspectorSection>
      )}
      <InspectorSection icon={<Images className="size-4" />} title={t('collage.photosOnSheet')}>
        {cells.map(([id, c]) => {
          const p = summary(id)
          return (
            <div key={c} className={cn('flex items-center gap-2.5 rounded-md p-1.5', chosen === c && p && 'bg-primary/10')}>
              {p ? (
                <>
                  <button type="button" className="shrink-0" onClick={() => choose(chosen === c ? null : c)} aria-label={p.fileName}>
                    <img src={thumbUrl(p.id, 0, generation)} alt="" className="size-10 rounded object-cover" />
                  </button>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm">{p.fileName}</p>
                    <p className="text-xs text-muted-foreground">{formatCapture(p.capture)}</p>
                  </div>
                  <Button
                    variant={p.queue === 'print' ? 'default' : 'outline'}
                    size="icon-sm"
                    aria-pressed={p.queue === 'print'}
                    aria-label={t(p.queue === 'print' ? 'preview.inBasket' : 'preview.toBasket')}
                    data-hint={t(p.queue === 'print' ? 'preview.inBasket' : 'preview.toBasket')}
                    onClick={() => void commands.setPhotoQueue([p.id], p.queue === 'print' ? null : 'print')}
                  >
                    <ShoppingBasket />
                  </Button>
                  <Button
                    variant="outline"
                    size="icon-sm"
                    aria-label={t('viewer.crop')}
                    data-hint={t('viewer.crop')}
                    onClick={() => {
                      choose(c)
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
      </InspectorSection>
    </ViewerFrame>
  )
}
