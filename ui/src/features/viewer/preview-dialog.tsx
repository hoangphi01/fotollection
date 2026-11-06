// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, ChevronLeft, ChevronRight, Clock, Crop, Download, MapPin, Star } from 'lucide-react'
import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { ClockMatchDialog } from '@/features/cameras/clock-match-dialog'
import { DatePlaceDialog } from '@/features/places/date-place-dialog'
import { CullControls } from '@/features/culling/cull-controls'
import { handleCullKey } from '@/features/culling/cull-keys'
import { CameraProfileEditor } from '@/features/frame/camera-profile-editor'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { MOD } from '@/lib/platform'
import { SectionCard } from '@/components/layout/section-card'
import { InspectorSection } from '@/components/layout/inspector-section'
import { KeyHints } from '@/components/layout/key-hints'
import { StageSpinner } from '@/components/layout/stage-spinner'
import { sendTo } from './send-to'
import { useShownSrc } from './use-shown-src'
import { usePrefetch } from './use-prefetch'
import { PhotoFilmstrip } from './photo-filmstrip'
import { PreviewToolbar } from './preview-toolbar'
import { oneToOneScale } from './one-to-one'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { useGpuLook } from './use-gpu-look'
import { FrameSection } from '@/features/frame/frame-section'
import type { FontsApi } from '@/features/fonts/use-fonts'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import { useIpcQuery, useLiveQuery } from '@/features/settings/use-ipc-query'
import {
  commands,
  type CullFilter,
  type Queue,
  type PhotoSummary,
  type RenderSettings,
} from '@/ipc/bindings'
import { formatCaptureLong, formatNumber } from '@/lib/format'
import { previewUrl, sourceUrl } from '@/lib/photo-url'
import { CropEditor } from './crop-editor'
import { PhotoStampControls } from './photo-stamp-controls'
import { PhotoLookCards } from '@/features/look/photo-look-cards'
import { PhotoSummaryCard } from './photo-summary-card'
import { useCropDrag } from './use-crop-drag'
import { useSharpRegion } from './use-sharp-region'
import { useZoom } from './use-zoom'
import { ZoomControls } from './zoom-controls'
import { ownsArrowKeys } from '@/lib/keyboard'
import { cn } from '@/lib/utils'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { usePrintAdvice } from '@/features/print-advice/use-print-advice'
import { verdictOf } from '@/features/print-advice/verdict'
import { VerdictBadge } from '@/features/print-advice/verdict-badge'
import { PrintSizeCard } from '@/features/settings/print-size-card'
import { ValueRow } from '@/components/forms/field'

type PreviewDialogProps = {
  photos: PhotoSummary[]
  index: number | null
  revision: number
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The shared settings changed in Rust ("Apply to all"); fetch them again. */
  onSettingsReload: () => void
  frame: FrameConfigApi
  fonts: FontsApi
  /** The same export job and destination as the settings panel (FR-055). */
  exporter: ExportApi
  filter: CullFilter
  onIndexChange: (index: number) => void
  onClose: () => void
  /** For the filmstrip's thumbnails (FR-090), as the gallery has them. */
  thumbVersions: { [id: number]: number }
  generation: number
  /** Whether ⌘Z and ⇧⌘Z can act, for the toolbar's buttons (FR-095). */
  canUndo: boolean
  canRedo: boolean
}


const loadDetails = async (subject: string) => {
  const result = await commands.photoDetails(Number(subject))
  return result.status === 'ok' ? result.data : null
}

const loadGeometry = async (subject: string) => {
  const result = await commands.getCropGeometry(Number(subject), null)
  return result.status === 'ok' ? result.data : null
}

/**
 * Print preview in a dialog (FR-007): the Rust-rendered image on the left (exact crop and
 * stamp of the export) and an inspector on the right with capture details, print quality
 * (FR-026) and quick print settings.
 */
export function PreviewDialog({
  photos,
  index,
  revision,
  settings,
  onSettingsChange,
  onSettingsReload,
  frame,
  fonts,
  exporter,
  filter,
  onIndexChange,
  onClose,
  thumbVersions,
  generation,
  canUndo,
  canRedo,
}: PreviewDialogProps) {
  const { t } = useTranslation()
  // The filmstrip under the photo (FR-090), shown unless the user hid it.
  const { ui, update: updateUi } = useUiSettings()
  const photo = index === null ? undefined : photos[index]
  // The last photo's facts hold their place while the next one's load, so ← → never make
  // the inspector jump (STYLE_RULE §4); actions use the photo itself.
  const details = useLiveQuery(photo ? String(photo.id) : null, revision, loadDetails).data
  const [clockMatch, setClockMatch] = useState(false)
  const [datePlace, setDatePlace] = useState(false)
  const printSizes = useIpcQuery(commands.listPrintSizes)
  // The same sharp / acceptable / pixelated advice as the settings panel (FR-026).
  const advice = usePrintAdvice(photo?.id, settings)
  // The photo without its colour look (FR-079, FR-096): the button or the \ key turn it on and
  // off, as in Lightroom; another photo starts with its look.
  const [beforeOf, setBeforeOf] = useState<number | null>(null)
  const before = photo !== undefined && beforeOf === photo.id
  const toggleBefore = () => setBeforeOf(before || !photo ? null : photo.id)
  const src = photo ? `${previewUrl(photo.id, revision)}${before ? '&plain=1' : ''}` : ''
  const { src: shownSrc, error: shownError } = useShownSrc(photo?.id, src)
  // The next photos load while this one is looked at (FR-091).
  usePrefetch(photos, index, !!photo && shownSrc === src)
  // While a colour slider is dragged, the look is drawn on the GPU (ADR-0039).
  const { attach: attachGpu, active: onGpu } = useGpuLook(photo?.id, revision, shownSrc === src)
  // The crop editor and the drag need this photo's geometry; the crop row only shows it.
  const geometryQuery = useLiveQuery(photo ? String(photo.id) : null, revision, loadGeometry)
  const geometry = geometryQuery.current
  const shownGeometry = geometryQuery.data
  const [cropping, setCropping] = useState(false)
  // The stage mounts only when the preview opens, so it is kept in state, not a ref.
  const [stage, setStage] = useState<HTMLDivElement | null>(null)
  // Zoomed past the preview's pixels, the visible part comes again at full resolution.
  const [previewImg, setPreviewImg] = useState<HTMLImageElement | null>(null)
  // Percentages of the photo's own pixels, 100 % being 1:1 (FR-095).
  const zoom = useZoom(stage, `${photo?.id}:${cropping}`, oneToOneScale(geometry, previewImg))
  const sharp = useSharpRegion(previewImg, stage, zoom.view, photo?.id, revision, shownSrc === src && !before ? shownSrc : null)
  // Without zoom, dragging the photo moves the crop (Crop to fill, or no frame).
  const cropDrag = useCropDrag(photo?.id, geometry, !cropping && !zoom.zoomed && !!geometry && !geometry.cropDisabled)
  const canDrag = !zoom.zoomed && !!geometry && !geometry.cropDisabled

  const go = (delta: number) => {
    setCropping(false)
    if (index === null) return
    onIndexChange(Math.min(photos.length - 1, Math.max(0, index + delta)))
  }
  // A filmstrip click goes the way ← → go.
  const pick = (next: number) => {
    setCropping(false)
    onIndexChange(next)
  }
  // Leaving the crop editor hands the keyboard back, so ← → move between photos again.
  const contentRef = useRef<HTMLDivElement>(null)
  const exportRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!cropping) contentRef.current?.focus()
  }, [cropping])
  const canCrop = !!geometry && !geometry.cropDisabled
  const toExport = () => exportRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' })
  const onKeyDown = (event: KeyboardEvent) => {
    // A focused slider or field keeps its own arrow keys (the logo size, the caption…).
    if (cropping || ownsArrowKeys(event.target)) return
    const mod = event.metaKey || event.ctrlKey
    const key = event.key.toLowerCase()
    // Lightroom's ⌘[ ⌘] rotate and ⇧⌘E export (FR-095).
    if (mod && (event.key === '[' || event.key === ']')) {
      event.preventDefault()
      if (photo) void commands.rotatePhotos([photo.id], event.key === ']')
      return
    }
    if (mod && event.shiftKey && key === 'e') {
      event.preventDefault()
      toExport()
      return
    }
    if (zoom.onKey(event)) return
    if (handleCullKey(event, photo?.id)) event.preventDefault()
    else if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault()
      send(event.key === 'ArrowUp' ? 'print' : 'aside')
    } else if (event.key === '\\' && !mod) toggleBefore()
    else if ((key === 'r' || key === 'c') && !mod && !event.altKey && canCrop) setCropping(true)
  }

  const view = filter.queue ?? 'gallery'
  const send = (queue: Queue | null) => {
    if (photo) sendTo(photo.id, queue, view, () => go(1))
  }

  return (
    <ViewerFrame
      open={photo !== undefined}
      onClose={onClose}
      contentRef={contentRef}
      onKeyDown={onKeyDown}
      onEscapeKeyDown={(e) => {
        // Esc leaves crop mode first, then closes the dialog.
        if (cropping) {
          e.preventDefault()
          setCropping(false)
        }
      }}
      stageRef={setStage}
      // Clicking the photo hands the keyboard back to browsing, away from a focused control.
      onStagePointerDown={(e) => {
        if (!cropping && ownsArrowKeys(document.activeElement)) contentRef.current?.focus()
        cropDrag.onPointerDown(e)
      }}
      onStagePointerMove={cropDrag.onPointerMove}
      onStagePointerUp={cropDrag.onPointerUp}
      below={
        ui.filmstrip && !cropping ? (
          <PhotoFilmstrip photos={photos} index={index} thumbVersions={thumbVersions} generation={generation} onPick={pick} />
        ) : undefined
      }
      toolbar={
        <PreviewToolbar
          photo={photo}
          cropBlocked={shownGeometry?.cropDisabled ? t('viewer.cropFitMode') : null}
          onCrop={() => setCropping(true)}
          before={before}
          onBefore={toggleBefore}
          canUndo={canUndo}
          canRedo={canRedo}
          onExport={toExport}
          zoom={
            <ZoomControls
              inline
              percent={zoom.percent}
              zoomed={zoom.zoomed}
              canZoomIn={zoom.canZoomIn && !cropping}
              loading={sharp.loading}
              mod={MOD}
              onZoomIn={zoom.zoomIn}
              onZoomOut={zoom.zoomOut}
              onFit={zoom.reset}
            />
          }
          index={index ?? 0}
          count={photos.length}
          filmstrip={ui.filmstrip}
          onFilmstrip={() => void updateUi({ filmstrip: !ui.filmstrip })}
        />
      }
      title={photo?.fileName}
      description={details?.folderPath ?? photo?.folderName}
      descriptionHint={details?.folderPath}
      footer={
        <KeyHints
          hints={
            cropping
              ? [
                  { keys: ['←', '→'], label: t('viewer.level') },
                  { keys: ['Esc'], label: t('viewer.leaveCrop') },
                ]
              : [
                  { keys: ['←', '→'], label: t('viewer.browse') },
                  { keys: ['Z'], label: t('viewer.oneToOne') },
                  { keys: ['\\'], label: t('viewer.beforeKey') },
                  ...(canDrag ? [{ label: t('viewer.dragToFrame') }] : []),
                  { keys: ['Esc'], label: t('viewer.close') },
                  { keys: ['ⓘ'], label: t('viewer.everyKey') },
                ]
          }
        />
      }
      stage={
        <>
          {cropping && geometry && photo ? (
            <CropEditor
              key={photo.id}
              geometry={geometry}
              sourceSrc={sourceUrl(photo.id, revision)}
              onCancel={() => setCropping(false)}
              onCommit={(crop, straighten) => {
                setCropping(false)
                void commands.setPhotoCrop(photo.id, crop, straighten, false)
              }}
            />
          ) : (
            <>
              {photo && shownSrc && (
                <img
                  ref={setPreviewImg}
                  src={shownSrc}
                  alt={photo.fileName}
                  draggable={false}
                  onPointerDown={zoom.pointer.onPointerDown}
                  onPointerMove={zoom.pointer.onPointerMove}
                  onPointerUp={zoom.pointer.onPointerUp}
                  onClick={zoom.pointer.onClick}
                  style={{ transform: zoom.transform }}
                  className={cn(
                    'max-h-full max-w-full animate-in object-contain stage-photo fade-in-0',
                    zoom.zoomed || canDrag ? 'cursor-grab active:cursor-grabbing' : 'cursor-zoom-in',
                    onGpu && 'hidden',
                  )}
                />
              )}
              {/* The same place and size as the image: it stands in for it while drawing. */}
              <canvas
                ref={attachGpu}
                aria-hidden
                style={{ transform: zoom.transform }}
                className={cn('max-h-full max-w-full stage-photo', !onGpu && 'hidden')}
              />
              {sharp.layer && !onGpu && (
                <img
                  src={sharp.layer.src}
                  alt=""
                  aria-hidden
                  draggable={false}
                  style={sharp.layer.style}
                  className="pointer-events-none absolute top-1/2 left-1/2 max-w-none"
                />
              )}
              {!shownSrc && <StageSpinner error={shownError} />}
              {/* Browse buttons for the mouse; they show while the pointer is over the stage. */}
              <Button
                variant="secondary"
                size="icon"
                aria-label={t('viewer.previous')}
                data-hint={t('viewer.previous')}
                className="absolute left-3 opacity-0 transition-opacity group-hover/stage:opacity-80 hover:opacity-100 focus-visible:opacity-100"
                disabled={index === 0}
                onClick={() => go(-1)}
              >
                <ChevronLeft />
              </Button>
              <Button
                variant="secondary"
                size="icon"
                aria-label={t('viewer.next')}
                data-hint={t('viewer.next')}
                className="absolute right-3 opacity-0 transition-opacity group-hover/stage:opacity-80 hover:opacity-100 focus-visible:opacity-100"
                disabled={index === null || index >= photos.length - 1}
                onClick={() => go(1)}
              >
                <ChevronRight />
              </Button>
            </>
          )}
        </>
      }
    >
            {/* What the photo is at a glance (FR-078), then its colour, then the print tools in the
                settings panel's order: print size, date stamp, info frame, export (STYLE_RULE §4).
                The photo's own facts follow, to change them. */}
            {photo && <PhotoSummaryCard photo={photo} details={details} />}
            {/* Develop first, as Lightroom's right panel (FR-095): RAW, Basic, Tools. */}
            {photo && (
              <PhotoLookCards photoId={photo.id} revision={revision} onAppliedToAll={onSettingsReload} />
            )}
            {/* A card of controls, spaced as in the settings panel; InspectorSection's tighter
                rows are for the photo's facts below (STYLE_RULE §4). */}
            <PrintSizeCard
              settings={settings}
              onChange={onSettingsChange}
              printSizes={printSizes}
              photo={photo}
              advice={advice}
              revision={revision}
            >
              {settings?.exportTarget?.kind !== 'share' && details && (
                <ValueRow label={t('viewer.resolution')}>
                  {details.printDpi} DPI
                  <VerdictBadge verdict={verdictOf(details.printQuality)} dpi={details.printDpi} />
                </ValueRow>
              )}
              <ValueRow icon={<Crop />} label={t('viewer.crop')}>
                {shownGeometry?.crop
                  ? t('viewer.cropCustom', { zoom: formatNumber(shownGeometry.crop.zoom ?? 1, 1, 1) })
                  : t('viewer.cropCentred')}
                {shownGeometry?.crop && photo && (
                  <Button variant="ghost" size="xs" onClick={() => void commands.setPhotoCrop(photo.id, null, 0, false)}>
                    {t('common.reset')}
                  </Button>
                )}
                {!shownGeometry?.crop && (
                  <Button variant="ghost" size="xs" disabled={!shownGeometry || shownGeometry.cropDisabled} onClick={() => setCropping(true)}>
                    {t('viewer.adjust')}
                  </Button>
                )}
              </ValueRow>
            </PrintSizeCard>

            {photo && (
              <PhotoStampControls photoId={photo.id} revision={revision} onAppliedToAll={onSettingsReload} />
            )}

            {/* Same card as the print settings panel, plus this camera's profile (FR-040). */}
            <FrameSection frame={frame} fonts={fonts} compact>
              <CameraProfileEditor frame={frame} details={details} />
            </FrameSection>

            {/* Like it? Export just this one, with the same controls as the panel (FR-055). */}
            {photo && (
              <div ref={exportRef}>
              <SectionCard icon={<Download />} title={t('export.title')}>
                <ExportSection
                  exporter={exporter}
                  photoCount={1}
                  filter={filter}
                  only={photo.id}
                  revision={revision}
                />
              </SectionCard>
              </div>
            )}

            <Separator />

            {photo && (
              <InspectorSection icon={<Star className="size-4" />} title={t('viewer.ratingFlag')}>
                <CullControls photo={photo} />
              </InspectorSection>
            )}

            <InspectorSection icon={<CalendarDays className="size-4" />} title={t('viewer.captured')}>
              <p>{photo ? formatCaptureLong(photo.capture) : '—'}</p>
              {photo?.timeShifted && details && (
                <p className="text-xs text-muted-foreground">
                  {t('viewer.cameraTime', { time: formatCaptureLong(details.cameraTime) })}
                </p>
              )}
              {details?.dateManual ? (
                <Badge variant="outline">{t('viewer.dateManual')}</Badge>
              ) : photo?.dateFromFile ? (
                <Badge variant="outline" className="text-amber-600 dark:text-amber-400">
                  {t('viewer.noExifDate')}
                </Badge>
              ) : photo?.timeShifted ? (
                <Badge variant="outline">{t('viewer.clockShifted')}</Badge>
              ) : (
                <Badge variant="outline">{t('viewer.fromExif')}</Badge>
              )}
              {photo && details && (
                <div className="flex flex-wrap gap-2 pt-1">
                  <Button size="sm" variant="outline" onClick={() => setDatePlace(true)}>
                    <CalendarDays data-icon="inline-start" /> {t('places.setDate')}
                  </Button>
                  {!photo.dateFromFile && !details.dateManual && details.clockId != null && (
                    <Button size="sm" variant="outline" onClick={() => setClockMatch(true)}>
                      <Clock data-icon="inline-start" /> {t('viewer.syncClock')}
                    </Button>
                  )}
                </div>
              )}
              {photo && details?.clockId != null && clockMatch && (
                <ClockMatchDialog photoId={photo.id} clockId={details.clockId} open onClose={() => setClockMatch(false)} />
              )}
            </InspectorSection>

            <InspectorSection icon={<MapPin className="size-4" />} title={t('viewer.location')}>
              <p>{details?.place?.label ?? t('places.none')}</p>
              {details?.place && <Badge variant="outline">{t(`places.${details.place.source}`)}</Badge>}
              {details?.gps && details.gps.lat !== null && details.gps.lon !== null ? (
                <p className="text-xs text-muted-foreground tabular-nums">
                  {formatNumber(details.gps.lat, 5, 5)} · {formatNumber(details.gps.lon, 5, 5)}
                </p>
              ) : (
                <p className="text-xs text-muted-foreground">{t('viewer.noGps')}</p>
              )}
              {photo && details && (
                <div className="flex flex-wrap gap-2 pt-1">
                  <Button size="sm" variant="outline" onClick={() => setDatePlace(true)}>
                    <MapPin data-icon="inline-start" /> {t('places.setPlace')}
                  </Button>
                </div>
              )}
              {datePlace && photo && details && (
                <DatePlaceDialog
                  ids={[photo.id]}
                  current={{ capture: photo.capture, dateManual: details.dateManual, place: details.place }}
                  onClose={() => setDatePlace(false)}
                />
              )}
            </InspectorSection>
    </ViewerFrame>
  )
}
