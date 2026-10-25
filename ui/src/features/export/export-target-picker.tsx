// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { ExportTarget, RenderSettings, SharePreset } from '@/ipc/bindings'

/** Matches `share::CUSTOM_EDGE_RANGE`; Rust clamps whatever is sent. */
const CUSTOM_DEFAULT = 1600
const PRESETS: SharePreset['kind'][] = ['original', 'facebook', 'instagram', 'custom']

type ExportTargetPickerProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
  disabled?: boolean
}

/** Prints, or a lighter file for sharing at the size a platform shows best (FR-053). */
export function ExportTargetPicker({ settings, onChange, disabled = false }: ExportTargetPickerProps) {
  const { t } = useTranslation()
  const target: ExportTarget = settings?.exportTarget ?? { kind: 'print' }
  const preset: SharePreset = target.kind === 'share' ? target.preset : { kind: 'facebook' }
  const set = (next: ExportTarget) => onChange((s) => ({ ...s, exportTarget: next }))
  const setPreset = (kind: SharePreset['kind']) =>
    set({
      kind: 'share',
      preset: kind === 'custom' ? { kind: 'custom', longEdge: CUSTOM_DEFAULT } : { kind },
    } as ExportTarget)

  return (
    <div className="space-y-2">
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        className="w-full"
        disabled={disabled}
        value={target.kind}
        onValueChange={(kind) => kind && set(kind === 'print' ? { kind: 'print' } : { kind: 'share', preset })}
      >
        <ToggleGroupItem value="print" className="flex-1 text-xs">
          {t('export.target.print')}
        </ToggleGroupItem>
        <ToggleGroupItem value="share" className="flex-1 text-xs">
          {t('export.target.share')}
        </ToggleGroupItem>
      </ToggleGroup>

      {target.kind === 'share' && (
        <>
          <div className="flex gap-2">
            <Select value={preset.kind} onValueChange={(v) => setPreset(v as SharePreset['kind'])} disabled={disabled}>
              <SelectTrigger size="sm" className="min-w-0 flex-1 text-xs" aria-label={t('export.target.preset')}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PRESETS.map((kind) => (
                  <SelectItem key={kind} value={kind}>
                    {t(`export.preset.${kind}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {preset.kind === 'custom' && (
              <div className="w-24">
                <CommitInput
                  value={String(preset.longEdge)}
                  placeholder={String(CUSTOM_DEFAULT)}
                  numeric
                  ariaLabel={t('export.target.longEdge')}
                  onCommit={(v) => {
                    const edge = Number.parseInt(v, 10)
                    if (Number.isFinite(edge)) set({ kind: 'share', preset: { kind: 'custom', longEdge: edge } })
                  }}
                />
              </div>
            )}
          </div>
          <p className="text-xs text-muted-foreground">{t(`export.presetHint.${preset.kind}`)}</p>
        </>
      )}
    </div>
  )
}
