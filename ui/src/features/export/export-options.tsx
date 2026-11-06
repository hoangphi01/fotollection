// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the export dialog and the Settings window (STYLE_RULE §4, checked by check-lego).
import { CircleAlert, CircleCheck, FileImage, FileText, FolderOutput, Maximize2, ShieldCheck, TriangleAlert } from 'lucide-react'
import { useEffect, useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Field, Hint, ValueRow } from '@/components/forms/field'
import { Collapse } from '@/components/layout/collapse'
import { PanelNote } from '@/components/layout/panel-note'
import { SwitchField } from '@/components/forms/switch-field'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { CommitInput } from '@/components/forms/commit-input'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SliderField } from '@/components/forms/slider-field'
import { SectionCard } from '@/components/layout/section-card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { commands, type ExistingFiles, type ExportFolders, type ExportMetadata, type ExportNaming, type NameTemplate, type RenderSettings } from '@/ipc/bindings'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { ShareFillFields } from './share-fill-fields'

/** Matches `share::CUSTOM_EDGE_RANGE`; Rust clamps whatever is sent. */
const CUSTOM_DEFAULT = 1600
/** Lightroom's naming templates, in its order (FR-104); Rust writes the names. */
const TEMPLATES: readonly NameTemplate[] = ['original', 'dateOriginal', 'textSequence', 'textDateSequence', 'dateSequence']
const EXISTING: readonly ExistingFiles[] = ['addNumber', 'skip', 'replace']
/** A size limit when first turned on, in KB (FR-105). */
const LIMIT_DEFAULT_KB = 2000

/**
 * The first name the export will write (FR-104), from Rust's own naming, so the example is
 * the real name; the last answer stays while the next one comes, so the row never blinks.
 */
function useNameExample(naming: ExportNaming, count: number): string | null {
  const [example, setExample] = useState<string | null>(null)
  const key = JSON.stringify(naming)
  useEffect(() => {
    let live = true
    void commands.exportNameExample(naming, Math.max(1, count)).then((r) => {
      if (live && r.status === 'ok') setExample(r.data)
    })
    return () => {
      live = false
    }
  }, [key, count]) // eslint-disable-line react-hooks/exhaustive-deps
  return example
}

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
  /** Files this export writes, for the sequence's width in the name example (FR-104). */
  count?: number
  /** Choose where exports go. */
  onChooseDestination: () => void
  /** Only what this computer keeps (the Settings window): the size belongs to the project's
   * print settings, so it stays in the dialog. */
  machineOnly?: boolean
}

/**
 * Every export option, as Lightroom's Export dialog groups them (FR-093): where (a subfolder,
 * existing files, FR-009), the file's name (FR-104), the file (the quality slider with what it
 * is good for, FR-092, and a size limit, FR-105), the size, and the metadata (FR-010). The
 * same cards in the export dialog and in the Settings window, so a choice made in one shows
 * in the other; the size belongs to the project's print settings and shows in the dialog only.
 */
export function ExportOptionsCards({
  settings,
  onSettingsChange,
  quality,
  onQuality,
  estimate,
  folderCount = 1,
  count = 1,
  onChooseDestination,
  machineOnly = false,
}: ExportOptionsCardsProps) {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  const target = settings?.exportTarget ?? { kind: 'print' as const }
  const share = target.kind === 'share'
  const verdict = VERDICT[qualityVerdict(quality)]
  const destination = ui.exportDestination
  // Every field filled, as Rust's defaults: the generated type marks them optional.
  const naming: Required<ExportNaming> = { template: 'original', text: '', start: 1, ...ui.exportNaming }
  const example = useNameExample(naming, count)
  const setNaming = (change: Partial<ExportNaming>) => void update({ exportNaming: { ...naming, ...change } })
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
        <SwitchField
          label={t('exportDialog.subfolder')}
          checked={ui.exportSubfolder !== null}
          onChange={(on) => void update({ exportSubfolder: on ? t('exportDialog.subfolderDefault') : null })}
          hint={t('exportDialog.subfolderHint')}
        >
          <CommitInput
            value={ui.exportSubfolder ?? ''}
            placeholder={t('exportDialog.subfolderDefault')}
            maxLength={100}
            ariaLabel={t('exportDialog.subfolder')}
            onCommit={(v) => void update({ exportSubfolder: v.trim() || t('exportDialog.subfolderDefault') })}
          />
        </SwitchField>
        <Field label={t('exportDialog.existing')}>
          <Select value={ui.exportExisting} onValueChange={(v) => void update({ exportExisting: v as ExistingFiles })}>
            <SelectTrigger className="w-full" aria-label={t('exportDialog.existing')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {EXISTING.map((e) => (
                <SelectItem key={e} value={e}>
                  {t(`exportDialog.existingRule.${e}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        {ui.exportExisting === 'replace' && <PanelNote danger icon={<TriangleAlert />}>{t('exportDialog.replaceWarning')}</PanelNote>}
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

      <SectionCard icon={<FileText />} title={t('exportDialog.fileName')}>
        <Field label={t('exportDialog.template')}>
          <Select value={naming.template} onValueChange={(v) => setNaming({ template: v as NameTemplate })}>
            <SelectTrigger className="w-full" aria-label={t('exportDialog.template')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TEMPLATES.map((tpl) => (
                <SelectItem key={tpl} value={tpl}>
                  {t(`exportDialog.templates.${tpl}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Collapse open={naming.template === 'textSequence' || naming.template === 'textDateSequence'}>
          <Field label={t('exportDialog.customText')}>
            <CommitInput
              value={naming.text}
              placeholder={t('exportDialog.customTextPlaceholder')}
              maxLength={100}
              liveMs={400}
              ariaLabel={t('exportDialog.customText')}
              onCommit={(text) => setNaming({ text })}
            />
          </Field>
        </Collapse>
        <Collapse open={naming.template !== 'original' && naming.template !== 'dateOriginal'}>
          <Field label={t('exportDialog.startAt')}>
            <CommitInput
              value={String(naming.start)}
              placeholder="1"
              numeric
              ariaLabel={t('exportDialog.startAt')}
              onCommit={(v) => {
                const start = Number.parseInt(v, 10)
                if (Number.isFinite(start)) setNaming({ start })
              }}
            />
          </Field>
        </Collapse>
        <ValueRow label={t('exportDialog.example')}>
          <span className="max-w-48 truncate" data-hint={example ?? undefined}>
            {example ?? '…'}
          </span>
        </ValueRow>
      </SectionCard>

      <SectionCard icon={<FileImage />} title={t('exportDialog.file')}>
        <SliderField
          label={t('exportDialog.quality')}
          value={quality}
          min={60}
          max={100}
          reset={share ? 88 : 90}
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
        <SwitchField
          label={t('exportDialog.limit')}
          checked={ui.exportLimitKb !== null}
          onChange={(on) => void update({ exportLimitKb: on ? LIMIT_DEFAULT_KB : null })}
          hint={t('exportDialog.limitHint')}
        >
          <Field label={t('exportDialog.limitKb')}>
            <CommitInput
              value={String(ui.exportLimitKb ?? LIMIT_DEFAULT_KB)}
              placeholder={String(LIMIT_DEFAULT_KB)}
              numeric
              ariaLabel={t('exportDialog.limitKb')}
              onCommit={(v) => {
                const kb = Number.parseInt(v, 10)
                if (Number.isFinite(kb)) void update({ exportLimitKb: kb })
              }}
            />
          </Field>
        </SwitchField>
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
        </>
      )}

      <SectionCard icon={<ShieldCheck />} title={t('exportDialog.metadataTitle')}>
        <Field label={t('exportDialog.metadataKeep')} hint={t(`exportDialog.metadataHint.${ui.exportMetadata}`)}>
          <SegmentedToggle<ExportMetadata>
            value={ui.exportMetadata}
            options={[
              { value: 'camera', label: t('exportDialog.metadataChoice.camera') },
              { value: 'dateOnly', label: t('exportDialog.metadataChoice.dateOnly') },
              { value: 'nothing', label: t('exportDialog.metadataChoice.nothing') },
            ]}
            onChange={(exportMetadata) => void update({ exportMetadata })}
            ariaLabel={t('exportDialog.metadataKeep')}
          />
        </Field>
      </SectionCard>
    </>
  )
}
