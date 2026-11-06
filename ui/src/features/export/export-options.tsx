// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the export dialog and the Settings window (STYLE_RULE §4, checked by check-lego).
import { CircleAlert, CircleCheck, FileImage, FolderOutput, Maximize2, Tags, TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { FactLines, Field, Hint, ValueRow } from '@/components/forms/field'
import { CommitInput } from '@/components/forms/commit-input'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SliderField } from '@/components/forms/slider-field'
import { SectionCard } from '@/components/layout/section-card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { ExportFolders, RenderSettings } from '@/ipc/bindings'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { ShareFillFields } from './share-fill-fields'

/** Matches `share::CUSTOM_EDGE_RANGE`; Rust clamps whatever is sent. */
const CUSTOM_DEFAULT = 1600

/**
 * What a quality is good for on paper, from the owner's photos at print size (RESULT §30,
 * FR-092): 90 and above prints beautifully, 75 to 89 prints well, below 75 is for screens.
 */
function qualityVerdict(quality: number): 'beautiful' | 'good' | 'screen' {
  return quality >= 90 ? 'beautiful' : quality >= 75 ? 'good' : 'screen'
}

const VERDICT = {
  beautiful: { variant: 'success', Icon: CircleCheck },
  good: { variant: 'warning', Icon: CircleAlert },
  screen: { variant: 'destructive', Icon: TriangleAlert },
} as const

type ExportOptionsCardsProps = {
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** The quality under the slider, before it is saved (the dialog's live estimate). */
  quality: number
  onQuality: (quality: number) => void
  /** The estimate's line, under the slider (the dialog only). */
  estimate?: ReactNode
  /** Source folders in this export, for *together / apart* (FR-056); 1 hides it. */
  folderCount?: number
  /** Choose where exports go. */
  onChooseDestination: () => void
  /** Only what this computer keeps (the Settings window): the size and names belong to the
   * project's print settings, so they stay in the dialog. */
  machineOnly?: boolean
}

/**
 * Every export option, as Lightroom's Export dialog groups them (FR-093): where, the file (the
 * quality slider with what it is good for, FR-092), the size and the names. The same cards in
 * the export dialog and in the Settings window, so a choice made in one shows in the other.
 */
export function ExportOptionsCards({
  settings,
  onSettingsChange,
  quality,
  onQuality,
  estimate,
  folderCount = 1,
  onChooseDestination,
  machineOnly = false,
}: ExportOptionsCardsProps) {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  const target = settings?.exportTarget ?? { kind: 'print' as const }
  const share = target.kind === 'share'
  const verdict = VERDICT[qualityVerdict(quality)]
  const destination = ui.exportDestination
  return (
    <>
      <SectionCard icon={<FolderOutput />} title={t('exportDialog.where')}>
        <ValueRow label={t('exportDialog.folder')}>
          <span className="max-w-48 truncate" data-hint={destination ?? undefined}>
            {destination ? destination.split(/[\\/]/).filter(Boolean).pop() : t('exportDialog.noFolder')}
          </span>
          <Button variant="outline" size="xs" onClick={onChooseDestination}>
            {t('exportDialog.choose')}
          </Button>
        </ValueRow>
        <Field
          label={t('export.days.label')}
          hint={t(ui.exportFolders === 'byDay' ? 'export.days.byDayHint' : 'export.days.oneFolderHint')}
        >
          <SegmentedToggle<ExportFolders>
            value={ui.exportFolders}
            options={[
              { value: 'oneFolder', label: t('export.days.oneFolder') },
              { value: 'byDay', label: t('export.days.byDay') },
            ]}
            onChange={(exportFolders) => void update({ exportFolders })}
            ariaLabel={t('export.days.label')}
          />
        </Field>
        {folderCount > 1 && (
          <Field
            label={t('export.folders.label')}
            hint={t(ui.exportSeparate ? 'export.folders.separateHint' : 'export.folders.togetherHint', { count: folderCount })}
          >
            <SegmentedToggle
              value={ui.exportSeparate ? 'separate' : 'together'}
              options={[
                { value: 'together', label: t('export.folders.together') },
                { value: 'separate', label: t('export.folders.separate') },
              ]}
              onChange={(v) => void update({ exportSeparate: v === 'separate' })}
              ariaLabel={t('export.folders.label')}
            />
          </Field>
        )}
      </SectionCard>

      <SectionCard icon={<FileImage />} title={t('exportDialog.file')}>
        <SliderField
          label={t('exportDialog.quality')}
          value={quality}
          min={60}
          max={100}
          reset={share ? 88 : 95}
          gpu={false}
          format={(v) => String(v)}
          hint={t('exportDialog.qualityHint')}
          onChange={(v) => onQuality(v)}
        />
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={verdict.variant}>
            <verdict.Icon data-icon="inline-start" aria-hidden />
            {t(`exportDialog.verdict.${qualityVerdict(quality)}`)}
          </Badge>
          {estimate}
        </div>
        <Hint>{t(share ? 'exportDialog.fileShare' : 'exportDialog.filePrint')}</Hint>
      </SectionCard>

      {!machineOnly && (
        <>
          <SectionCard icon={<Maximize2 />} title={t('exportDialog.size')}>
            {share ? (
              <>
                {target.preset.kind === 'custom' && (
                  <Field label={t('export.target.longEdge')}>
                    <CommitInput
                      value={String(target.preset.longEdge)}
                      placeholder={String(CUSTOM_DEFAULT)}
                      numeric
                      ariaLabel={t('export.target.longEdge')}
                      onCommit={(v) => {
                        const edge = Number.parseInt(v, 10)
                        if (Number.isFinite(edge))
                          onSettingsChange((s) => ({
                            ...s,
                            exportTarget: {
                              kind: 'share',
                              preset: { kind: 'custom', longEdge: edge },
                            },
                          }))
                      }}
                    />
                  </Field>
                )}
                <Hint>{t(`export.presetHint.${target.preset.kind}`)}</Hint>
                <ShareFillFields settings={settings} onChange={onSettingsChange} />
              </>
            ) : (
              <Hint>{t('exportDialog.sizePrint')}</Hint>
            )}
          </SectionCard>

          <SectionCard icon={<Tags />} title={t('exportDialog.names')}>
            <FactLines>
              <p className="text-xs">{t('exportDialog.naming')}</p>
              <p className="text-xs">{t('exportDialog.metadata')}</p>
            </FactLines>
          </SectionCard>
        </>
      )}
    </>
  )
}
