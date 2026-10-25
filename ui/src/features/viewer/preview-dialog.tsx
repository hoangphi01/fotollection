// SPDX-License-Identifier: GPL-3.0-only
import {
  Camera,
  CalendarDays,
  ChevronLeft,
  Download,
  ChevronRight,
  Crop,
  ImageIcon,
  LoaderCircle,
  MapPin,
  Printer,
  RotateCcw,
  RotateCw,
  Star,
} from 'lucide-react'
import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { CullControls } from '@/features/culling/cull-controls'
import { handleCullKey } from '@/features/culling/cull-keys'
import { CameraProfileEditor } from '@/features/frame/camera-profile-editor'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { FrameSection } from '@/features/frame/frame-section'
import type { FontsApi } from '@/features/fonts/use-fonts'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import {
  commands,
  type CropGeometry,
  type CullFilter,
  type PhotoDetails,
  type PhotoSummary,
  type RenderSettings,
} from '@/ipc/bindings'
import i18n from '@/i18n'
import { formatNumber } from '@/lib/format'
import { previewUrl, sourceUrl } from '@/lib/photo-url'
import { CropEditor } from './crop-editor'
import { PhotoStampControls } from './photo-stamp-controls'
import { ownsArrowKeys } from '@/lib/keyboard'
import { cn } from '@/lib/utils'
import { SectionCard } from '@/components/layout/section-card'
import { PrintAdviceNote } from '@/features/print-advice/print-advice-note'
import { usePrintAdvice } from '@/features/print-advice/use-print-advice'
import { verdictOf } from '@/features/print-advice/verdict'
import { VerdictBadge } from '@/features/print-advice/verdict-badge'

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
}


function formatCaptureLong(capture: string): string {
  const date = new Date(capture)
  if (Number.isNaN(date.getTime())) return capture
  return date.toLocaleString(i18n.language, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function formatBytes(bytes: number): string {
  return bytes >= 1e6 ? `${formatNumber(bytes / 1e6, 1, 1)} MB` : `${Math.round(bytes / 1e3)} KB`
}

function usePhotoDetails(id: number | undefined, revision: number): PhotoDetails | null {
  const [details, setDetails] = useState<PhotoDetails | null>(null)
  useEffect(() => {
    if (id === undefined) return
    let live = true
    void commands.photoDetails(id).then((result) => {
      if (live && result.status === 'ok') setDetails(result.data)
    })
    return () => {
      live = false
    }
  }, [id, revision])
  return details && details.id === id ? details : null
}

function useCropGeometry(id: number | undefined, revision: number): CropGeometry | null {
  const [geometry, setGeometry] = useState<{ id: number; value: CropGeometry } | null>(null)
  useEffect(() => {
    if (id === undefined) return
    let live = true
    void commands.getCropGeometry(id).then((result) => {
      if (live && result.status === 'ok') setGeometry({ id, value: result.data })
    })
    return () => {
      live = false
    }
  }, [id, revision])
  return geometry && geometry.id === id ? geometry.value : null
}

function Section({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) {
  return (
    <SectionCard icon={icon} title={title}>
      <div className="space-y-1">{children}</div>
    </SectionCard>
  )
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
}: PreviewDialogProps) {
  const { t } = useTranslation()
  const photo = index === null ? undefined : photos[index]
  const details = usePhotoDetails(photo?.id, revision)
  const printSizes = useIpcQuery(commands.listPrintSizes)
  // The same sharp / acceptable / pixelated advice as the settings panel (FR-026).
  const sizesKey = (settings?.customSizes ?? []).map((c) => `${c.id}:${c.shortMm}x${c.longMm}`).join()
  const advice = usePrintAdvice(photo?.id, sizesKey)
  const src = photo ? previewUrl(photo.id, revision) : ''
  const [loadedSrc, setLoadedSrc] = useState('')
  const geometry = useCropGeometry(photo?.id, revision)
  const [cropping, setCropping] = useState(false)

  const go = (delta: number) => {
    setCropping(false)
    if (index === null) return
    onIndexChange(Math.min(photos.length - 1, Math.max(0, index + delta)))
  }
  // Leaving the crop editor hands the keyboard back, so ← → move between photos again.
  const contentRef = useRef<HTMLDivElement>(null)
  const exportRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!cropping) contentRef.current?.focus()
  }, [cropping])
  const onKeyDown = (event: KeyboardEvent) => {
    // A focused slider or field keeps its own arrow keys (the logo size, the caption…).
    if (cropping || ownsArrowKeys(event.target)) return
    if (handleCullKey(event, photo?.id)) event.preventDefault()
    else if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (event.key.toLowerCase() === 'c' && geometry && !geometry.cropDisabled) setCropping(true)
  }

  return (
    <Dialog open={photo !== undefined} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        ref={contentRef}
        onKeyDown={onKeyDown}
        onEscapeKeyDown={(e) => {
          // Esc leaves crop mode first, then closes the dialog.
          if (cropping) {
            e.preventDefault()
            setCropping(false)
          }
        }}
        className="flex h-[90vh] w-[95vw] max-w-none gap-0 overflow-hidden p-0 sm:max-w-[min(1400px,95vw)]"
      >
        {/* Stage */}
        <div
          className="relative flex min-w-0 flex-1 items-center justify-center bg-neutral-950 p-8"
          // Clicking the photo hands the keyboard back to browsing, away from a focused control.
          onPointerDown={() => {
            if (!cropping && ownsArrowKeys(document.activeElement)) contentRef.current?.focus()
          }}
        >
          {cropping && geometry && photo ? (
            <CropEditor
              key={`${photo.id}:${revision}`}
              geometry={geometry}
              sourceSrc={sourceUrl(photo.id, revision)}
              onCancel={() => setCropping(false)}
              onCommit={(crop, straighten) => {
                setCropping(false)
                void commands.setPhotoCrop(photo.id, crop, straighten)
              }}
            />
          ) : (
            <>
              {photo && (
                <img
                  key={src}
                  src={src}
                  alt={photo.fileName}
                  onLoad={() => setLoadedSrc(src)}
                  className={cn(
                    'max-h-full max-w-full rounded-sm object-contain shadow-2xl transition-opacity duration-200',
                    loadedSrc === src ? 'opacity-100' : 'opacity-0',
                  )}
                />
              )}
              {loadedSrc !== src && (
                <div className="absolute flex flex-col items-center gap-2 text-white/60">
                  <LoaderCircle className="size-7 animate-spin" />
                  <span className="text-xs">{t('viewer.rendering')}</span>
                </div>
              )}
              <div className="absolute top-3 left-3 flex gap-1.5">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span>
                      <Button
                        variant="secondary"
                        size="sm"
                        disabled={!geometry || geometry.cropDisabled}
                        onClick={() => setCropping(true)}
                      >
                        <Crop data-icon="inline-start" /> {t('viewer.crop')}
                      </Button>
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>
                    {geometry?.cropDisabled ? t('viewer.cropFitMode') : t('viewer.adjustCrop')}
                  </TooltipContent>
                </Tooltip>
                {/* Lossless quarter turns (FR-006); the crop turns with the photo. */}
                {(
                  [
                    [false, RotateCcw, 'viewer.rotateLeft'],
                    [true, RotateCw, 'viewer.rotateRight'],
                  ] as const
                ).map(([clockwise, Icon, label]) => (
                  <Tooltip key={label}>
                    <TooltipTrigger asChild>
                      <Button
                        variant="secondary"
                        size="icon-sm"
                        aria-label={t(label)}
                        disabled={!photo}
                        onClick={() => photo && void commands.rotatePhotos([photo.id], clockwise)}
                      >
                        <Icon />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>{t(label)}</TooltipContent>
                  </Tooltip>
                ))}
                {/* Like what you see? Jump to the export card for this photo (FR-055). */}
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={!photo}
                  onClick={() => exportRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' })}
                >
                  <Download data-icon="inline-start" /> {t('export.title')}
                </Button>
              </div>
              <Button
                variant="secondary"
                size="icon"
                aria-label={t('viewer.previous')}
                className="absolute left-3 opacity-80 hover:opacity-100"
                disabled={index === 0}
                onClick={() => go(-1)}
              >
                <ChevronLeft />
              </Button>
              <Button
                variant="secondary"
                size="icon"
                aria-label={t('viewer.next')}
                className="absolute right-3 opacity-80 hover:opacity-100"
                disabled={index === null || index >= photos.length - 1}
                onClick={() => go(1)}
              >
                <ChevronRight />
              </Button>
              <span className="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full bg-black/60 px-3 py-1 text-xs text-white/80 tabular-nums">
                {(index ?? 0) + 1} / {photos.length}
              </span>
            </>
          )}
        </div>

        {/* Inspector */}
        <aside className="flex w-96 shrink-0 flex-col border-l bg-background">
          <div className="space-y-0.5 px-5 pt-5 pb-4 pr-12">
            <DialogTitle className="truncate text-base">{photo?.fileName}</DialogTitle>
            <DialogDescription className="truncate text-xs" data-hint={details?.folderPath}>
              {details?.folderPath ?? photo?.folderName}
            </DialogDescription>
          </div>
          <Separator />

          <div className="flex-1 space-y-5 overflow-y-auto p-5">
            {photo && (
              <Section icon={<Star className="size-4" />} title={t('viewer.ratingFlag')}>
                <CullControls photo={photo} />
              </Section>
            )}

            <Section icon={<CalendarDays className="size-4" />} title={t('viewer.captured')}>
              <p>{photo ? formatCaptureLong(photo.capture) : '—'}</p>
              {photo?.dateFromFile ? (
                <Badge variant="outline" className="text-amber-600 dark:text-amber-400">
                  {t('viewer.noExifDate')}
                </Badge>
              ) : (
                <Badge variant="outline">{t('viewer.fromExif')}</Badge>
              )}
            </Section>

            <Section icon={<Camera className="size-4" />} title={t('viewer.camera')}>
              <p>{photo?.camera ?? t('viewer.unknownCamera')}</p>
            </Section>

            {photo && (
              <PhotoStampControls photoId={photo.id} revision={revision} onAppliedToAll={onSettingsReload} />
            )}

            {/* Same card as the print settings panel, plus this camera's profile (FR-040). */}
            <FrameSection frame={frame} fonts={fonts} compact>
              <CameraProfileEditor frame={frame} details={details} />
            </FrameSection>

            <Section icon={<ImageIcon className="size-4" />} title={t('viewer.image')}>
              {details ? (
                <p className="tabular-nums">
                  {details.width} × {details.height} px ·{' '}
                  {formatNumber((details.width * details.height) / 1e6, 1, 1)} MP ·{' '}
                  {formatBytes(details.fileBytes ?? 0)}
                </p>
              ) : (
                <p className="text-muted-foreground">{t('common.loading')}</p>
              )}
            </Section>

            <Section icon={<MapPin className="size-4" />} title={t('viewer.location')}>
              {details?.gps && details.gps.lat !== null && details.gps.lon !== null ? (
                <p className="font-mono text-xs">
                  {details.gps.lat.toFixed(5)}, {details.gps.lon.toFixed(5)}
                </p>
              ) : (
                <p className="text-muted-foreground">{t('viewer.noGps')}</p>
              )}
              <p className="text-xs text-muted-foreground">{t('viewer.placesSoon')}</p>
            </Section>

            <Separator />

            <Section icon={<Printer className="size-4" />} title={t('viewer.print')}>
              <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onSettingsChange} advice={advice} />
              {advice && photo && <PrintAdviceNote advice={advice} fileName={photo.fileName} />}
              {details && (
                <div className="flex items-center justify-between pt-1">
                  <span className="text-xs text-muted-foreground">{t('viewer.resolution')}</span>
                  <span className="flex items-center gap-1.5 text-xs tabular-nums">
                    {details.printDpi} DPI
                    <VerdictBadge verdict={verdictOf(details.printQuality)} dpi={details.printDpi} />
                  </span>
                </div>
              )}
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Crop className="size-3.5" /> {t('viewer.crop')}
                </span>
                <span className="flex items-center gap-1.5 text-xs">
                  {geometry?.crop
                    ? t('viewer.cropCustom', { zoom: formatNumber(geometry.crop.zoom ?? 1, 1, 1) })
                    : t('viewer.cropCentred')}
                  {geometry?.crop && photo && (
                    <Button variant="ghost" size="xs" onClick={() => void commands.setPhotoCrop(photo.id, null, 0)}>
                      {t('common.reset')}
                    </Button>
                  )}
                  {!geometry?.crop && (
                    <Button variant="ghost" size="xs" disabled={!geometry || geometry.cropDisabled} onClick={() => setCropping(true)}>
                      {t('viewer.adjust')}
                    </Button>
                  )}
                </span>
              </div>
            </Section>

            {/* Like it? Export just this one, with the same controls as the panel (FR-055). */}
            {photo && (
              <div ref={exportRef}>
              <Section icon={<Download className="size-4" />} title={t('export.title')}>
                <ExportSection
                  exporter={exporter}
                  photoCount={1}
                  filter={filter}
                  only={photo.id}
                  settings={settings}
                  onChange={onSettingsChange}
                />
              </Section>
              </div>
            )}
          </div>

          <Separator />
          <p className="px-5 py-3 text-[11px] text-muted-foreground">
            <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> {t(cropping ? 'viewer.level' : 'viewer.browse')} ·{' '}
            <kbd className="font-mono">Esc</kbd> {t(cropping ? 'viewer.leaveCrop' : 'viewer.close')}
          </p>
        </aside>
      </DialogContent>
    </Dialog>
  )
}
