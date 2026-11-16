// SPDX-License-Identifier: GPL-3.0-only
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'

/** The toolbar's button that shows the left column a narrow window folded, first in the row. */
export function SideToggle({ onShow, label }: { onShow: () => void; label?: string }) {
  const { t } = useTranslation()
  const text = label ?? t('viewer.sideShow')
  return (
    <>
      <ToolbarButton icon={<PanelLeftOpen />} label={text} variant="outline" data-hint={text} iconOnly onClick={onShow} />
      <Separator orientation="vertical" className="mx-1 h-5!" />
    </>
  )
}

/** The left column's own button, in its head, that folds it again in a narrow window. */
export function SideHide({ onHide }: { onHide: () => void }) {
  const { t } = useTranslation()
  return <ToolbarButton icon={<PanelLeftClose />} label={t('viewer.sideHide')} data-hint={t('viewer.sideHide')} iconOnly onClick={onHide} />
}

/**
 * At the inspector's head while a narrow window folds the left column (owner, 2026-11-07, and
 * 2026-11-16 for the sheet's preview): what the column holds, and the button that shows it now.
 */
export function SideFoldedNote({ text, onOpen }: { text: string; onOpen: () => void }) {
  const { t } = useTranslation()
  return (
    <PanelNote
      icon={<PanelLeftOpen />}
      action={
        <Button size="sm" onClick={onOpen}>
          <PanelLeftOpen data-icon="inline-start" /> {t('viewer.sideOpen')}
        </Button>
      }
    >
      {text}
    </PanelNote>
  )
}
