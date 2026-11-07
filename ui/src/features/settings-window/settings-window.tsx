// SPDX-License-Identifier: GPL-3.0-only
import { BookOpen, Cpu, FolderOutput, MessageSquareWarning, RefreshCw, SlidersHorizontal, Sparkles, type LucideIcon } from 'lucide-react'
import { lazy, Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { ReportIssue } from '@/features/about/report-issue'
import { UpdateCheck } from '@/features/about/update-check'
import { ReleaseList } from '@/features/about/version-history'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands } from '@/ipc/bindings'
import { ExportSettingsSection } from './export-section'
import { GeneralSection } from './general-section'
import { MemorySection } from './memory-section'
import { SECTIONS, showSettings, useSettingsWindow, type Section } from './use-settings-window'
import { ListRow } from '@/components/forms/list-row'

// Settings → Manual lists the guide's chapters: the guide loads with it, never at start (FR-110).
const ManualChapterList = lazy(() =>
  import('@/features/manual/manual-dialog').then((m) => ({ default: m.ManualChapterList })),
)

const ICONS: Record<Section, LucideIcon> = {
  general: SlidersHorizontal,
  export: FolderOutput,
  memory: Cpu,
  updates: RefreshCw,
  manual: BookOpen,
  report: MessageSquareWarning,
  whatsNew: Sparkles,
}

/** *Check for updates when the app starts* (FR-103): on from the install, off only when the
 * user turns it off here. */
function UpdatesAtStart() {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  return (
    <SwitchField
      label={t('update.atStart')}
      hint={t('update.atStartHint')}
      checked={ui.checkUpdatesAtStart}
      onChange={(checkUpdatesAtStart) => void update({ checkUpdatesAtStart })}
    />
  )
}

/** The cards of `section`. */
function Body({ section, version }: { section: Section; version: string | undefined }) {
  const { t } = useTranslation()
  const Icon = ICONS[section]
  switch (section) {
    case 'general':
      return <GeneralSection />
    case 'export':
      return <ExportSettingsSection />
    case 'memory':
      return <MemorySection />
    case 'updates':
      return (
        <SectionCard icon={<Icon />} title={t('settingsWindow.sections.updates')}>
          <UpdatesAtStart />
          <UpdateCheck />
        </SectionCard>
      )
    case 'manual':
      // Each chapter opens the manual's own window there (FR-110), in place of this one.
      return (
        <SectionCard icon={<Icon />} title={t('manual.chapters')}>
          <div className="flex flex-col gap-1">
            <Suspense fallback={null}>
              <ManualChapterList onOpen={() => showSettings(null)} />
            </Suspense>
          </div>
        </SectionCard>
      )
    case 'report':
      return (
        <SectionCard icon={<Icon />} title={t('settingsWindow.sections.report')}>
          <ReportIssue />
        </SectionCard>
      )
    case 'whatsNew':
      return (
        <SectionCard icon={<Icon />} title={t('about.versions')}>
          <ReleaseList version={version} />
        </SectionCard>
      )
  }
}

/**
 * One window for every setting of this computer (FR-086), opened by the gear beside the
 * byline, the version badge or Cmd+, from any mode: the sections listed on the left as on an
 * iPad, the open one's cards on the right. Projects never carry these settings.
 */
export function SettingsWindow() {
  const { t } = useTranslation()
  const section = useSettingsWindow()
  const appInfo = useIpcQuery(commands.appInfo)
  const shown = section ?? 'general'
  return (
    <SettingsFrame
      open={section !== null}
      onClose={() => showSettings(null)}
      title={t('settingsWindow.title')}
      nav={SECTIONS.map((s) => {
        const Icon = ICONS[s]
        return (
          <ListRow key={s} icon={<Icon />} chosen={s === shown} current="page" onClick={() => showSettings(s)}>
            {t(`settingsWindow.sections.${s}`)}
          </ListRow>
        )
      })}
      navFooter={appInfo ? t('settingsWindow.version', { version: appInfo.appVersion }) : undefined}
      heading={t(`settingsWindow.sections.${shown}`)}
      description={t(`settingsWindow.about.${shown}`)}
      footer={t('settingsWindow.footer')}
    >
      <Body section={shown} version={appInfo?.appVersion} />
    </SettingsFrame>
  )
}
