// SPDX-License-Identifier: GPL-3.0-only
import { MessageSquareWarning } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { commands } from '@/ipc/bindings'
import { VersionHistory } from './version-history'

/**
 * The foot of every mode's panel: the privacy line, "Report a problem" (FR-069) and the
 * version. The report button opens the project's issue page in the browser; the app sends
 * nothing itself.
 */
export function PanelFooter() {
  const { t } = useTranslation()
  const appInfo = useIpcQuery(commands.appInfo)
  return (
    <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
      <span className="min-w-0 leading-snug">{t('settings.footer')}</span>
      <div className="flex shrink-0 items-center gap-1.5">
        <Button
          size="xs"
          className="bg-destructive text-white hover:bg-destructive/90"
          data-hint={t('about.reportHint')}
          onClick={() => void commands.openIssuePage()}
        >
          <MessageSquareWarning data-icon="inline-start" />
          {t('about.report')}
        </Button>
        {appInfo && <VersionHistory version={appInfo.appVersion} />}
      </div>
    </div>
  )
}
