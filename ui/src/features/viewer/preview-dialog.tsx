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
} from 'lucide-react'
import { useEffect, useState, type KeyboardEvent, type ReactNode } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { CameraProfileEditor } from '@/features/frame/camera-profile-editor'
import type { FrameConfigApi } from '@/features/frame/use-frame-config'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import {
  commands,
  type DateFormat,
  type DpiBand,
  type PhotoDetails,
  type PhotoSummary,
  type RenderSettings,
} from '@/ipc/bindings'
import { previewUrl } from '@/lib/photo-url'
import { cn } from '@/lib/utils'

type PreviewDialogProps = {
  photos: PhotoSummary[]
  index: number | null
  revision: number
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  frame: FrameConfigApi
  onIndexChange: (index: number) => void
  onClose: () => void
}

const DPI_STYLE: Record<DpiBand, { label: string; className: string }> = {
  excellent: { label: 'Excellent', className: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400' },
  good: { label: 'Good', className: 'bg-sky-500/15 text-sky-600 dark:text-sky-400' },
  fair: { label: 'Fair', className: 'bg-amber-500/15 text-amber-600 dark:text-amber-400' },
  poor: { label: 'Too low', className: 'bg-red-500/15 text-red-600 dark:text-red-400' },
}

const FORMATS: { value: DateFormat; label: string }[] = [
  { value: 'iso', label: '2026-07-14' },
  { value: 'dmy', label: '14.07.2026' },
  { value: 'retro', label: "'26 07 14" },
]

function formatCaptureLong(capture: string): string {
  const date = new Date(capture)
  if (Number.isNaN(date.getTime())) return capture
  return date.toLocaleString(undefined, {
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
  return bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : `${Math.round(bytes / 1e3)} KB`
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
  onIndexChange,
  onClose,
}: PreviewDialogProps) {
  const photo = index === null ? undefined : photos[index]
  const details = usePhotoDetails(photo?.id, revision)
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const src = photo ? previewUrl(photo.id, revision) : ''
  const [loadedSrc, setLoadedSrc] = useState('')
  const stamp = settings?.stamp

  const go = (delta: number) => {
    if (index === null) return
    onIndexChange(Math.min(photos.length - 1, Math.max(0, index + delta)))
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
  }

  return (
    <Dialog open={photo !== undefined} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        onKeyDown={onKeyDown}
        className="flex h-[90vh] w-[95vw] max-w-none gap-0 overflow-hidden p-0 sm:max-w-[min(1400px,95vw)]"
      >
        {/* Stage */}
        <div className="relative flex min-w-0 flex-1 items-center justify-center bg-neutral-950 p-8">
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
              <span className="text-xs">Rendering print preview…</span>
            </div>
          )}
          <Button
            variant="secondary"
            size="icon"
            aria-label="Previous photo"
            className="absolute left-3 opacity-80 hover:opacity-100"
            disabled={index === 0}
            onClick={() => go(-1)}
          >
            <ChevronLeft />
          </Button>
          <Button
            variant="secondary"
            size="icon"
            aria-label="Next photo"
            className="absolute right-3 opacity-80 hover:opacity-100"
            disabled={index === null || index >= photos.length - 1}
            onClick={() => go(1)}
          >
            <ChevronRight />
          </Button>
          <span className="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full bg-black/60 px-3 py-1 text-xs text-white/80 tabular-nums">
            {(index ?? 0) + 1} / {photos.length}
          </span>
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
            <Section icon={<CalendarDays className="size-4" />} title="Captured">
              <p>{photo ? formatCaptureLong(photo.capture) : '—'}</p>
              {photo?.dateFromFile ? (
                <Badge variant="outline" className="text-amber-600 dark:text-amber-400">
                  No EXIF date · using file time
                </Badge>
              ) : (
                <Badge variant="outline">From EXIF</Badge>
              )}
            </Section>

            <Section icon={<Camera className="size-4" />} title="Camera">
              <p>{photo?.camera ?? 'Unknown camera'}</p>
            </Section>

            <Section icon={<PanelBottom className="size-4" />} title="Frame & camera profile">
              {!frame.config?.settings.enabled && (
                <p className="text-xs text-muted-foreground">
                  Turn on <span className="font-medium">Info frame</span> in Print settings to see
                  it on the print.
                </p>
              )}
              <CameraProfileEditor frame={frame} details={details} />
            </Section>

            <Section icon={<ImageIcon className="size-4" />} title="Image">
              {details ? (
                <p className="tabular-nums">
                  {details.width} × {details.height} px ·{' '}
                  {((details.width * details.height) / 1e6).toFixed(1)} MP ·{' '}
                  {formatBytes(details.fileBytes ?? 0)}
                </p>
              ) : (
                <p className="text-muted-foreground">Loading…</p>
              )}
            </Section>

            <Section icon={<MapPin className="size-4" />} title="Location">
              {details?.gps && details.gps.lat !== null && details.gps.lon !== null ? (
                <p className="font-mono text-xs">
                  {details.gps.lat.toFixed(5)}, {details.gps.lon.toFixed(5)}
                </p>
              ) : (
                <p className="text-muted-foreground">No GPS in this photo</p>
              )}
              <p className="text-xs text-muted-foreground">Place names arrive with FR-005</p>
            </Section>

            <Separator />

            <Section icon={<Printer className="size-4" />} title="Print">
              <Select
                value={settings?.printSize ?? ''}
                onValueChange={(printSize) => onSettingsChange((s) => ({ ...s, printSize }))}
              >
                <SelectTrigger className="w-full" aria-label="Print size">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {printSizes?.map((size) => (
                    <SelectItem key={size.id} value={size.id}>
                      {size.shortMm / 10} × {size.longMm / 10} cm
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {details && (
                <div className="flex items-center justify-between pt-1">
                  <span className="text-xs text-muted-foreground">Resolution</span>
                  <span
                    className={cn(
                      'rounded-md px-2 py-0.5 text-xs font-medium tabular-nums',
                      DPI_STYLE[details.printQuality].className,
                    )}
                  >
                    {details.printDpi} DPI · {DPI_STYLE[details.printQuality].label}
                  </span>
                </div>
              )}
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Crop className="size-3.5" /> Crop
                </span>
                <span className="text-xs">
                  Centred <Badge variant="outline">Adjust soon</Badge>
                </span>
              </div>
            </Section>

            <Section icon={<CalendarDays className="size-4" />} title="Date stamp">
              <div className="flex items-center justify-between">
                <Label htmlFor="dialog-stamp" className="text-sm font-normal">
                  Show on print
                </Label>
                <Switch
                  id="dialog-stamp"
                  checked={stamp?.enabled ?? false}
                  onCheckedChange={(enabled) =>
                    onSettingsChange((s) => ({ ...s, stamp: { ...s.stamp, enabled } }))
                  }
                />
              </div>
              <ToggleGroup
                type="single"
                variant="outline"
                size="sm"
                className="w-full"
                value={stamp?.format ?? ''}
                disabled={!stamp?.enabled}
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
              <p className="text-xs text-muted-foreground">Print settings apply to all photos for now.</p>
            </Section>
          </div>

          <Separator />
          <p className="px-5 py-3 text-[11px] text-muted-foreground">
            <kbd className="font-mono">←</kbd> <kbd className="font-mono">→</kbd> browse ·{' '}
            <kbd className="font-mono">Esc</kbd> close
          </p>
        </aside>
      </DialogContent>
    </Dialog>
  )
}
