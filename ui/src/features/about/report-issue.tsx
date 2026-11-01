// SPDX-License-Identifier: GPL-3.0-only
import { MessageSquareWarning } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands } from '@/ipc/bindings'

/**
 * "Report a problem" (FR-069), in the versions popover next to "Check for updates": opens the
 * project's issue forms in the browser. The address is fixed in Rust; the app sends nothing.
 */
export function ReportIssue() {
  const { t } = useTranslation()
  return (
    <div className="space-y-2 border-t px-4 py-3">
      <Button
        size="sm"
        className="w-full bg-destructive text-white hover:bg-destructive/90"
        onClick={() => void commands.openIssuePage()}
      >
        <MessageSquareWarning data-icon="inline-start" />
        {t('about.report')}
      </Button>
      <p className="text-xs text-muted-foreground">{t('about.reportHint')}</p>
    </div>
  )
}
