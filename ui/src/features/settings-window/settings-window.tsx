// SPDX-License-Identifier: GPL-3.0-only
import { Cpu, FolderOutput, MessageSquareWarning, RefreshCw, SlidersHorizontal, Sparkles, type LucideIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { SettingsFrame } from '@/components/layout/settings-frame'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { ReportIssue } from '@/features/about/report-issue'
import { UpdateCheck } from '@/features/about/update-check'
import { ReleaseList } from '@/features/about/version-history'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands } from '@/ipc/bindings'
import { cn } from '@/lib/utils'
import { ExportSettingsSection } from './export-section'
import { GeneralSection } from './general-section'
import { MemorySection } from './memory-section'
import { SECTIONS, showSettings, useSettingsWindow, type Section } from './use-settings-window'

const ICONS: Record<Section, LucideIcon> = {
  general: SlidersHorizontal,
  export: FolderOutput,
  memory: Cpu,
  updates: RefreshCw,
  report: MessageSquareWarning,
  whatsNew: Sparkles,
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
          <UpdateCheck />
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
          <Button
            key={s}
            variant="ghost"
            size="sm"
            aria-current={s === shown ? 'page' : undefined}
            className={cn('justify-start [&_svg]:text-primary', s === shown && 'bg-accent text-accent-foreground row-chosen')}
            onClick={() => showSettings(s)}
          >
            <Icon />
            {t(`settingsWindow.sections.${s}`)}
          </Button>
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
