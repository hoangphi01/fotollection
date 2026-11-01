// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands } from '@/ipc/bindings'
import { VersionHistory } from './version-history'

/**
 * The foot of every mode's panel: the privacy line and the version, whose popover holds the
 * versions, "Check for updates" and "Report a problem" (FR-061, FR-069).
 */
export function PanelFooter() {
  const { t } = useTranslation()
  const appInfo = useIpcQuery(commands.appInfo)
  return (
    <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
      <span>{t('settings.footer')}</span>
      {appInfo && <VersionHistory version={appInfo.appVersion} />}
    </div>
  )
}
