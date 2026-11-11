// SPDX-License-Identifier: GPL-3.0-only
import { BookOpen, Cpu, FolderOutput, MessageSquareWarning, Package, RefreshCw, SlidersHorizontal, Sparkles, type LucideIcon, Database } from 'lucide-react'
import { lazy, Suspense, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { PanelNote } from '@/components/layout/panel-note'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { ReportIssue } from '@/features/about/report-issue'
import { UpdateCheck } from '@/features/about/update-check'
import { ReleaseList } from '@/features/about/version-history'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands } from '@/ipc/bindings'
import { ExportSettingsSection } from './export-section'
import { GeneralSection } from './general-section'
import { DataSection } from './data-section'
import { MemorySection } from './memory-section'
import { useManualChapter } from '@/features/manual/manual-store'
import { SECTIONS, showSettings, useSettingsWindow, type Section } from './use-settings-window'
import { ListRow } from '@/components/forms/list-row'

// Settings → Manual, the guide inside the window: it loads with the section, never at start
// (FR-110, NFR-010).
const ManualSection = lazy(() => import('@/features/manual/manual-section').then((m) => ({ default: m.ManualSection })))
const ManualSteps = lazy(() => import('@/features/manual/manual-section').then((m) => ({ default: m.ManualSteps })))

const ICONS: Record<Section, LucideIcon> = {
  general: SlidersHorizontal,
  export: FolderOutput,
  memory: Cpu,
  data: Database,
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

/** *Updates*: the check at start and the button, or, in a Flatpak, a note that its software
 * centre brings each version (Flathub forbids an app updating itself; Rust never asks GitHub
 * there either, `updates.rs` `UpdateSource`). */
function UpdatesCard({ icon }: { icon: ReactNode }) {
  const { t } = useTranslation()
  const source = useIpcQuery(commands.updateSource)
  return (
    <SectionCard icon={icon} title={t('settingsWindow.sections.updates')}>
      {source === 'softwareCentre' ? (
        <PanelNote icon={<Package />}>{t('update.softwareCentre')}</PanelNote>
      ) : (
        <>
          <UpdatesAtStart />
          <UpdateCheck />
        </>
      )}
    </SectionCard>
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
    case 'data':
      return <DataSection />
    case 'updates':
      return <UpdatesCard icon={<Icon />} />
    case 'manual':
      // The manual inside this window (owner, 2026-11-07: "no other window").
      return (
        <Suspense fallback={null}>
          <ManualSection />
        </Suspense>
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
  const chapter = useManualChapter()
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
      actions={
        shown === 'manual' ? (
          <Suspense fallback={null}>
            <ManualSteps />
          </Suspense>
        ) : undefined
      }
      scrollKey={shown === 'manual' ? `manual:${chapter ?? ''}` : shown}
    >
      <Body section={shown} version={appInfo?.appVersion} />
    </SettingsFrame>
  )
}
