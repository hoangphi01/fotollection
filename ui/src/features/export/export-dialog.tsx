// SPDX-License-Identifier: GPL-3.0-only
import { Download, Image, MonitorSmartphone, Printer, Ruler, Smartphone, type LucideIcon } from 'lucide-react'
import { useId, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import type { ExportTarget, RenderSettings, SharePreset } from '@/ipc/bindings'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { ExportOptionsCards } from './export-options'
import { SizeChange } from './export-section'
import { useExportDialog, showExportDialog } from './use-export-dialog'
import { useExportEstimate } from './use-export-estimate'
import type { ExportApi } from './use-export'
import { ListRow } from '@/components/forms/list-row'
import { Button } from '@/components/ui/button'
import { RemoveButton } from '@/components/forms/danger-button'

/** Lightroom's export presets, ours: what each sets and its quality (share defaults from
 * RESULT §10, prints at 90, RESULT §30). */
const PRESETS = [
  { key: 'print', icon: Printer, target: { kind: 'print' }, quality: 90 },
  {
    key: 'facebook',
    icon: MonitorSmartphone,
    target: { kind: 'share', preset: { kind: 'facebook' } },
    quality: 88,
  },
  {
    key: 'instagram',
    icon: Smartphone,
    target: { kind: 'share', preset: { kind: 'instagram' } },
    quality: 88,
  },
  {
    key: 'original',
    icon: Image,
    target: { kind: 'share', preset: { kind: 'original' } },
    quality: 80,
  },
  {
    key: 'custom',
    icon: Ruler,
    target: { kind: 'share', preset: { kind: 'custom', longEdge: 1600 } },
    quality: 88,
  },
] as const satisfies readonly {
  key: string
  icon: LucideIcon
  target: ExportTarget
  quality: number
}[]
type PresetKey = (typeof PRESETS)[number]['key']

function presetOf(target: ExportTarget | undefined): PresetKey {
  if (!target || target.kind === 'print') return 'print'
  return (target.preset as SharePreset).kind
}

type ExportDialogProps = {
  exporter: ExportApi
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/** Settle time before a slider's quality is saved: one write per gesture, not per frame. */
const SAVE_MS = 400

/**
 * The export dialog (FR-093), as Lightroom's: presets on the left, every option in cards on the
 * right, *Export* at the foot. One dialog for the gallery, Simple, Collage and both previews;
 * the panel that asked keeps the progress and the result. The Settings window's size.
 */
export function ExportDialog({ exporter, settings, onSettingsChange }: ExportDialogProps) {
  const { t } = useTranslation()
  const request = useExportDialog()
  const { ui, update } = useUiSettings()
  const tickId = useId()
  // The slider's value before it is saved; null shows what is saved.
  const [draft, setDraft] = useState<number | null>(null)
  const quality = draft ?? ui.exportQuality
  const [dontAsk, setDontAsk] = useState(false)
  const saving = useRef<number | null>(null)
  const { estimate, stale } = useExportEstimate(
    request?.filter ?? { flag: 'all', minRating: 0 },
    request?.only ?? null,
    0,
    request !== null,
    request?.collage ?? false,
    request?.ticked ?? [],
    quality,
  )
  // Closing forgets the unsaved state, so each opening starts from what is saved.
  const close = () => {
    setDraft(null)
    setDontAsk(false)
    showExportDialog(null)
  }
  const setQualityLive = (q: number) => {
    setDraft(q)
    if (saving.current !== null) window.clearTimeout(saving.current)
    saving.current = window.setTimeout(() => void update({ exportQuality: q }), SAVE_MS)
  }
  const choosePreset = (p: (typeof PRESETS)[number]) => {
    onSettingsChange((s) => ({ ...s, exportTarget: p.target as ExportTarget }))
    setQualityLive(p.quality)
  }
  const current = presetOf(settings?.exportTarget)
  const run = async () => {
    if (!request) return
    await update({
      exportQuality: quality,
      ...(dontAsk ? { exportAsk: false } : {}),
    })
    close()
    const separate = request.folderCount > 1 && !request.collage && request.only === null && ui.exportSeparate
    void exporter.exportPhotos(request.filter, request.count, request.only, separate, request.collage, request.ticked)
  }
  const label = request
    ? request.only !== null
      ? t(request.collage ? 'export.buttonThisSheet' : 'export.buttonThis')
      : t(request.collage ? 'export.buttonSheets' : 'export.buttonCount', {
          count: request.count,
        })
    : ''

  return (
    <SettingsFrame
      open={request !== null}
      onClose={close}
      title={t('exportDialog.title')}
      nav={PRESETS.map((p) => (
        <ListRow key={p.key} icon={<p.icon />} chosen={p.key === current} onClick={() => choosePreset(p)}>
          {t(`exportDialog.preset.${p.key}`)}
        </ListRow>
      ))}
      heading={t(`exportDialog.preset.${current}`)}
      description={label}
      actions={
        <>
          <Checkbox id={tickId} checked={dontAsk} onCheckedChange={(v) => setDontAsk(v === true)} />
          <Label htmlFor={tickId} className="text-xs font-normal text-muted-foreground" data-hint={t('exportDialog.dontAskHint')}>
            {t('exportDialog.dontAsk')}
          </Label>
          <span className="flex-1" />
          <RemoveButton kind="stop" label={t('common.cancel')} onClick={close} />
          <Button onClick={() => void run()} disabled={!request}>
            <Download data-icon="inline-start" />
            {label}
          </Button>
        </>
      }
    >
      <ExportOptionsCards
        settings={settings}
        onSettingsChange={onSettingsChange}
        quality={quality}
        onQuality={setQualityLive}
        folderCount={request && request.only === null && !request.collage ? request.folderCount : 1}
        count={request?.count ?? 1}
        onChooseDestination={() => void exporter.chooseDestination()}
        estimate={
          estimate && estimate.sourceBytes && estimate.estimatedBytes ? (
            <SizeChange
              from={estimate.sourceBytes}
              to={estimate.estimatedBytes}
              estimate={estimate.sampled !== estimate.photos}
              before
              stale={stale}
            />
          ) : null
        }
      />
    </SettingsFrame>
  )
}
