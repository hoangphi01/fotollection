// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the Settings window only (STYLE_RULE §4, checked by check-lego).
import { MessageSquareMore } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { ExportOptionsCards } from '@/features/export/export-options'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { commands } from '@/ipc/bindings'

/** Settle time before the slider's quality is saved: one write per gesture. */
const SAVE_MS = 400

/**
 * The Settings window's *Export* section (FR-093): what this computer keeps for exports, the
 * same cards as the export dialog, and *Ask every time* to bring the dialog back after
 * *Don't ask again*.
 */
export function ExportSettingsSection() {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  // The slider's value until it is saved; null shows what is saved.
  const [draft, setDraft] = useState<number | null>(null)
  const quality = draft ?? ui.exportQuality
  const saving = useRef<number | null>(null)
  useEffect(
    () => () => {
      if (saving.current !== null) window.clearTimeout(saving.current)
    },
    [],
  )
  const onQuality = (q: number) => {
    setDraft(q)
    if (saving.current !== null) window.clearTimeout(saving.current)
    saving.current = window.setTimeout(() => void update({ exportQuality: q }).then(() => setDraft(null)), SAVE_MS)
  }
  const chooseDestination = async () => {
    const result = await commands.pickExportFolder()
    if (result.status === 'ok' && result.data) void update({ exportDestination: result.data })
  }
  return (
    <>
      <SectionCard icon={<MessageSquareMore />} title={t('exportDialog.askTitle')}>
        <SwitchField
          label={t('exportDialog.askEveryTime')}
          hint={t('exportDialog.askEveryTimeHint')}
          checked={ui.exportAsk}
          onChange={(exportAsk) => void update({ exportAsk })}
        />
      </SectionCard>
      <ExportOptionsCards
        settings={null}
        onSettingsChange={() => {}}
        quality={quality}
        onQuality={onQuality}
        onChooseDestination={() => void chooseDestination()}
        machineOnly
      />
    </>
  )
}
