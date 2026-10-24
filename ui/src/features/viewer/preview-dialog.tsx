// SPDX-License-Identifier: GPL-3.0-only
import {
  Camera,
  CalendarDays,
  ChevronLeft,
  ChevronRight,
  Crop,
  ImageIcon,
  LoaderCircle,
  MapPin,
  PanelBottom,
  Printer,
  Star,
} from 'lucide-react'
import { useEffect, useState, type KeyboardEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Collapse } from '@/components/layout/collapse'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { CullControls } from '@/features/culling/cull-controls'
import { handleCullKey } from '@/features/culling/cull-keys'
import { CameraProfileEditor } from '@/features/frame/camera-profile-editor'
import { FrameSection } from '@/features/frame/frame-section'
import type { FontsApi } from '@/features/fonts/use-fonts'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import { PrintSizePicker } from '@/features/settings/print-size-picker'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import {
  commands,
  type DateFormat,
  type CropGeometry,
  type PhotoDetails,
  type PhotoSummary,
  type RenderSettings,
} from '@/ipc/bindings'
import i18n from '@/i18n'
import { DPI_STYLE } from '@/lib/crop-geometry'
import { formatNumber } from '@/lib/format'
import { previewUrl, sourceUrl } from '@/lib/photo-url'
import { CropEditor } from './crop-editor'
import { cn } from '@/lib/utils'

type PreviewDialogProps = {
  photos: PhotoSummary[]
  index: number | null
  revision: number
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  frame: FrameConfigApi
  fonts: FontsApi
  onIndexChange: (index: number) => void
  onClose: () => void
}

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

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
    <section className="space-y-1.5">
      <h3 className="flex items-center gap-2 text-xs font-medium text-muted-foreground uppercase">
        {icon}
        {title}
      </h3>
      <div className="space-y-1 pl-6 text-sm">{children}</div>
    </section>
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
  frame,
  fonts,
  onIndexChange,
  onClose,
}: PreviewDialogProps) {
  const { t } = useTranslation()
  const photo = index === null ? undefined : photos[index]
  const details = usePhotoDetails(photo?.id, revision)
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const src = photo ? previewUrl(photo.id, revision) : ''
  const [loadedSrc, setLoadedSrc] = useState('')
  const stamp = settings?.stamp
  const geometry = useCropGeometry(photo?.id, revision)
  const [cropping, setCropping] = useState(false)

  const go = (delta: number) => {
    setCropping(false)
    if (index === null) return
    onIndexChange(Math.min(photos.length - 1, Math.max(0, index + delta)))
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (cropping || event.target instanceof HTMLInputElement) return
    if (handleCullKey(event, photo?.id)) event.preventDefault()
    else if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (event.key.toLowerCase() === 'c' && geometry && !geometry.cropDisabled) setCropping(true)
  }

  return (
    <Dialog open={photo !== undefined} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
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
        <div className="relative flex min-w-0 flex-1 items-center justify-center bg-neutral-950 p-8">
          {cropping && geometry && photo ? (
            <CropEditor
              key={`${photo.id}:${revision}`}
              geometry={geometry}
              sourceSrc={sourceUrl(photo.id, revision)}
              onCancel={() => setCropping(false)}
              onCommit={(crop) => {
                setCropping(false)
                void commands.setPhotoCrop(photo.id, crop)
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
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="absolute top-3 left-3">
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
            <DialogDescription className="truncate text-xs" title={details?.folderPath}>
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

            <Section icon={<PanelBottom className="size-4" />} title={t('viewer.frameProfile')}>
              {/* Same controls as the print settings panel, so the frame can be tuned here (FR-040). */}
              <FrameSection frame={frame} fonts={fonts} compact />
              <CameraProfileEditor frame={frame} details={details} />
            </Section>

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
              <PrintSizePicker settings={settings} builtIn={printSizes} onChange={onSettingsChange} />
              {details && (
                <div className="flex items-center justify-between pt-1">
                  <span className="text-xs text-muted-foreground">{t('viewer.resolution')}</span>
                  <span
                    className={cn(
                      'rounded-md px-2 py-0.5 text-xs font-medium tabular-nums',
                      DPI_STYLE[details.printQuality].className,
                    )}
                  >
                    {details.printDpi} DPI · {t(`dpi.${details.printQuality}`)}
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
                    <Button variant="ghost" size="xs" onClick={() => void commands.setPhotoCrop(photo.id, null)}>
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

            <Section icon={<CalendarDays className="size-4" />} title={t('viewer.dateStamp')}>
              <div className="flex items-center justify-between">
                <Label htmlFor="dialog-stamp" className="text-sm font-normal">
                  {t('viewer.showOnPrint')}
                </Label>
                <Switch
                  id="dialog-stamp"
                  checked={stamp?.enabled ?? false}
                  onCheckedChange={(enabled) =>
                    onSettingsChange((s) => ({ ...s, stamp: { ...s.stamp, enabled } }))
                  }
                />
              </div>
              <Collapse open={stamp?.enabled ?? false}>
              <ToggleGroup
                type="single"
                variant="outline"
                size="sm"
                className="w-full"
                value={stamp?.format ?? ''}
                onValueChange={(format) =>
                  format &&
                  onSettingsChange((s) => ({
                    ...s,
                    stamp: { ...s.stamp, format: format as DateFormat },
                  }))
                }
              >
                {FORMATS.map((f) => (
                  <ToggleGroupItem key={f.value} value={f.value} className="flex-1 font-mono text-[11px]">
                    {f.label}
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
              </Collapse>
              <p className="text-xs text-muted-foreground">{t('viewer.allPhotos')}</p>
            </Section>
          </div>

          <Separator />
          <p className="px-5 py-3 text-[11px] text-muted-foreground">
            <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> {t('viewer.browse')} ·{' '}
            <kbd className="font-mono">Esc</kbd> {t('viewer.close')}
          </p>
        </aside>
      </DialogContent>
    </Dialog>
  )
}
