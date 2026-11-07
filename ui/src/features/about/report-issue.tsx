// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { Check, ClipboardCopy } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { ReportButton } from '@/components/forms/danger-button'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { commands } from '@/ipc/bindings'

/**
 * "Report a problem" (FR-069), in the Settings window (FR-086): opens the
 * project's issue forms in the browser. The address is fixed in Rust; the app sends nothing.
 * "Copy diagnostics" puts the version, the system and the last error codes on the clipboard
 * for the issue (ERROR_CONTROL.md §3); where the clipboard is refused, the text is shown to
 * select by hand.
 */
export function ReportIssue() {
  const { t } = useTranslation()
  const [copied, setCopied] = useState(false)
  const [shown, setShown] = useState<string | null>(null)

  const copy = async () => {
    const text = await commands.diagnostics()
    try {
      await navigator.clipboard.writeText(text)
      setShown(null)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      setShown(text)
    }
  }

  return (
    <Field>
      <ReportButton label={t('about.report')} onClick={() => void commands.openIssuePage()} />
      <Button size="sm" variant="outline" className="w-full" onClick={() => void copy()}>
        {copied ? <Check data-icon="inline-start" /> : <ClipboardCopy data-icon="inline-start" />}
        {copied ? t('about.copied') : t('about.diagnostics')}
      </Button>
      {shown !== null && (
        <Textarea
          readOnly
          value={shown}
          aria-label={t('about.diagnostics')}
          className="h-28 font-mono text-xs"
          onFocus={(e) => e.currentTarget.select()}
          autoFocus
        />
      )}
      <Hint>{t('about.reportHint')}</Hint>
    </Field>
  )
}
